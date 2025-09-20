//! OWL2 Tableaux Reasoning Engine
//!
//! Implements a tableaux-based reasoning algorithm for OWL2 ontologies
//! based on SROIQ(D) description logic.

use crate::axioms::*;
use crate::entities::*;
use crate::error::{OwlError, OwlResult};
use crate::iri::IRI;
use crate::ontology::Ontology;

use hashbrown::HashMap;
use smallvec::SmallVec;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

/// Optimized edge storage for tableaux graph
#[derive(Debug, Default)]
struct EdgeStorage {
    /// Optimized storage for edges using flat representation
    edges: Vec<(NodeId, IRI, NodeId)>,
    /// Index for fast lookups: (from_node, property) -> Vec<to_node>
    index: HashMap<(NodeId, IRI), SmallVec<[NodeId; 4]>>,
}

impl EdgeStorage {
    fn new() -> Self {
        Self {
            edges: Vec::new(),
            index: HashMap::default(),
        }
    }

    fn add_edge(&mut self, from: NodeId, property: &IRI, to: NodeId) {
        // Add to flat storage
        self.edges.push((from, property.clone(), to));

        // Update index
        let key = (from, property.clone());
        self.index.entry(key).or_insert_with(SmallVec::new).push(to);
    }

    fn get_targets(&self, from: NodeId, property: &IRI) -> Option<&[NodeId]> {
        let key = (from, property.clone());
        self.index.get(&key).map(|vec| vec.as_slice())
    }

    #[allow(dead_code)]
    fn clear(&mut self) {
        self.edges.clear();
        self.index.clear();
    }
}

/// Tableaux reasoning engine for OWL2 ontologies
pub struct TableauxReasoner {
    pub ontology: Arc<Ontology>,
    #[allow(dead_code)]
    rules: ReasoningRules,
    cache: ReasoningCache,
}

/// Reasoning configuration options
#[derive(Debug, Clone)]
pub struct ReasoningConfig {
    /// Maximum depth for tableaux expansion
    pub max_depth: usize,
    /// Enable debugging output
    pub debug: bool,
    /// Enable incremental reasoning
    pub incremental: bool,
    /// Timeout in milliseconds
    pub timeout: Option<u64>,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        ReasoningConfig {
            max_depth: 1000,
            debug: false,
            incremental: true,
            timeout: Some(30000), // 30 seconds default
        }
    }
}

/// Tableaux node with optimized concept storage
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableauxNode {
    id: NodeId,
    /// Optimized concept storage using SmallVec for small sets
    concepts: SmallVec<[ClassExpression; 8]>,
    /// Lazy hashset for large concept sets
    concepts_hashset: Option<HashSet<ClassExpression>>,
    labels: SmallVec<[String; 4]>,
    blocked_by: Option<NodeId>,
}

impl TableauxNode {
    fn new(id: NodeId) -> Self {
        Self {
            id,
            concepts: SmallVec::new(),
            concepts_hashset: None,
            labels: SmallVec::new(),
            blocked_by: None,
        }
    }

    fn add_concept(&mut self, concept: ClassExpression) {
        if self.concepts_hashset.is_some() {
            // Use hashset for large collections
            self.concepts_hashset.as_mut().unwrap().insert(concept);
        } else {
            // Use SmallVec for small collections
            if self.concepts.len() < 8 {
                if !self.concepts.contains(&concept) {
                    self.concepts.push(concept);
                }
            } else {
                // Convert to hashset when exceeding SmallVec capacity
                let mut hashset = HashSet::new();
                hashset.extend(self.concepts.drain(..));
                hashset.insert(concept);
                self.concepts_hashset = Some(hashset);
            }
        }
    }

    fn has_concept(&self, concept: &ClassExpression) -> bool {
        if let Some(ref hashset) = self.concepts_hashset {
            hashset.contains(concept)
        } else {
            self.concepts.contains(concept)
        }
    }

    fn concepts_iter(&self) -> Box<dyn Iterator<Item = &ClassExpression> + '_> {
        if let Some(ref hashset) = self.concepts_hashset {
            Box::new(hashset.iter())
        } else {
            Box::new(self.concepts.iter())
        }
    }

    #[allow(dead_code)]
    fn concepts_len(&self) -> usize {
        if let Some(ref hashset) = self.concepts_hashset {
            hashset.len()
        } else {
            self.concepts.len()
        }
    }
}

/// Node identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(usize);

/// Optimized tableaux graph structure
#[derive(Debug)]
pub struct TableauxGraph {
    nodes: HashMap<NodeId, TableauxNode>,
    edges: EdgeStorage,
    root: NodeId,
    next_id: usize,
    /// Cache for commonly accessed nodes
    #[allow(dead_code)]
    node_cache: HashMap<NodeId, *const TableauxNode>,
}

/// Reasoning result
#[derive(Debug, Clone)]
pub struct ReasoningResult {
    pub is_satisfiable: bool,
    pub explanation: Option<String>,
    pub model: Option<HashMap<IRI, HashSet<ClassExpression>>>,
    pub stats: ReasoningStats,
}

/// Reasoning statistics
#[derive(Debug, Clone)]
pub struct ReasoningStats {
    pub nodes_created: usize,
    pub rules_applied: usize,
    pub time_ms: u64,
    pub cache_hits: usize,
    pub backtracks: usize,
}

/// Optimized reasoning cache with size limits
#[derive(Debug)]
struct ReasoningCache {
    concept_satisfiability: HashMap<ClassExpression, bool>,
    class_hierarchy: HashMap<IRI, SmallVec<[IRI; 4]>>,
    property_hierarchy: HashMap<IRI, SmallVec<[IRI; 4]>>,
    /// Cache statistics for eviction
    stats: CacheStats,
    /// Maximum cache size
    max_size: usize,
}

#[derive(Debug, Default)]
struct CacheStats {
    #[allow(dead_code)]
    hits: usize,
    #[allow(dead_code)]
    misses: usize,
    evictions: usize,
}

impl ReasoningCache {
    fn with_capacity(max_size: usize) -> Self {
        Self {
            concept_satisfiability: HashMap::default(),
            class_hierarchy: HashMap::default(),
            property_hierarchy: HashMap::default(),
            stats: CacheStats::default(),
            max_size,
        }
    }

    fn get_concept_satisfiability(&mut self, concept: &ClassExpression) -> Option<bool> {
        self.concept_satisfiability.get(concept).copied()
    }

    fn set_concept_satisfiability(&mut self, concept: ClassExpression, satisfiable: bool) {
        // Check cache size and evict if necessary
        if self.concept_satisfiability.len() >= self.max_size {
            self.evict_lru();
        }
        self.concept_satisfiability.insert(concept, satisfiable);
    }

    #[allow(dead_code)]
    fn get_class_hierarchy(&self, class_iri: &IRI) -> Option<&SmallVec<[IRI; 4]>> {
        self.class_hierarchy.get(class_iri)
    }

    #[allow(dead_code)]
    fn set_class_hierarchy(&mut self, class_iri: IRI, parents: SmallVec<[IRI; 4]>) {
        self.class_hierarchy.insert(class_iri, parents);
    }

    fn evict_lru(&mut self) {
        // Simple eviction: remove oldest entries (first inserted)
        if let Some(key) = self.concept_satisfiability.keys().next().cloned() {
            self.concept_satisfiability.remove(&key);
            self.stats.evictions += 1;
        }
    }
}

/// Built-in reasoning rules
#[derive(Debug)]
struct ReasoningRules {
    // Rule implementations will be added here
}

/// Resolve nested inverse property expressions into a direction flag and base IRI
/// Returns (is_inverse, iri) where is_inverse indicates whether the effective
/// direction is inverse (odd number of inversions)
fn resolve_property_direction<'a>(expr: &'a ObjectPropertyExpression) -> (bool, &'a IRI) {
    fn flatten<'a>(e: &'a ObjectPropertyExpression, invert: bool) -> (bool, &'a IRI) {
        match e {
            ObjectPropertyExpression::ObjectProperty(prop) => (invert, prop.iri()),
            ObjectPropertyExpression::ObjectInverseOf(inner) => flatten(inner.as_ref(), !invert),
        }
    }
    flatten(expr, false)
}

impl TableauxReasoner {
    /// Create a new tableaux reasoner
    pub fn new(ontology: Ontology) -> Self {
        Self::with_config(&ontology, ReasoningConfig::default())
    }

    /// Create a new tableaux reasoner from an Arc reference (no cloning)
    pub fn from_arc(ontology: &Arc<Ontology>) -> Self {
        Self::with_config_from_arc(ontology, ReasoningConfig::default())
    }

    /// Create a new tableaux reasoner with custom configuration
    pub fn with_config(ontology: &Ontology, _config: ReasoningConfig) -> Self {
        let ontology = Arc::new(ontology.clone());
        let rules = ReasoningRules::new(&ontology);
        let cache = ReasoningCache::new(&ontology);

        TableauxReasoner {
            ontology,
            rules,
            cache,
        }
    }

    /// Create a new tableaux reasoner with custom configuration from Arc (no cloning)
    pub fn with_config_from_arc(ontology: &Arc<Ontology>, _config: ReasoningConfig) -> Self {
        let rules = ReasoningRules::new(ontology);
        let cache = ReasoningCache::new(ontology);

        TableauxReasoner {
            ontology: ontology.clone(),
            rules,
            cache,
        }
    }

    /// Check if a class expression is satisfiable
    pub fn is_satisfiable(&mut self, concept: &ClassExpression) -> OwlResult<bool> {
        // Check cache first
        if let Some(result) = self.cache.get_concept_satisfiability(concept) {
            return Ok(result);
        }

        // Create tableaux graph
        let mut graph = TableauxGraph::new();
        let root = graph.add_node();
        graph.add_concept(root, concept.clone());

        // Run tableaux algorithm
        let result = self.run_tableaux(&mut graph, ReasoningConfig::default())?;

        // Cache result
        self.cache.set_concept_satisfiability(concept.clone(), result.is_satisfiable);

        Ok(result.is_satisfiable)
    }

    /// Check if a class is satisfiable
    pub fn is_class_satisfiable(&mut self, class_iri: &IRI) -> OwlResult<bool> {
        let concept = ClassExpression::Class(Class::new(class_iri.clone()));
        self.is_satisfiable(&concept)
    }

    /// Check if one class is a subclass of another
    pub fn is_subclass_of(&mut self, sub: &IRI, sup: &IRI) -> OwlResult<bool> {
        let sub_concept = ClassExpression::Class(Class::new(sub.clone()));
        let sup_concept = ClassExpression::Class(Class::new(sup.clone()));

        // A ⊑ B iff A ⊓ ¬B is unsatisfiable
        let intersection = ClassExpression::ObjectIntersectionOf(
            vec![
                Box::new(sub_concept),
                Box::new(ClassExpression::ObjectComplementOf(Box::new(sup_concept))),
            ]
            .into()
        );

        Ok(!self.is_satisfiable(&intersection)?)
    }

    /// Check if two classes are equivalent
    pub fn are_equivalent_classes(&mut self, a: &IRI, b: &IRI) -> OwlResult<bool> {
        Ok(self.is_subclass_of(a, b)? && self.is_subclass_of(b, a)?)
    }

    /// Check if two classes are disjoint
    pub fn are_disjoint_classes(&mut self, a: &IRI, b: &IRI) -> OwlResult<bool> {
        let a_concept = ClassExpression::Class(Class::new(a.clone()));
        let b_concept = ClassExpression::Class(Class::new(b.clone()));

        // A and B are disjoint iff A ⊓ B is unsatisfiable
        let intersection = ClassExpression::ObjectIntersectionOf(
            vec![Box::new(a_concept), Box::new(b_concept)].into()
        );

        Ok(!self.is_satisfiable(&intersection)?)
    }

    /// Get all instances of a class
    pub fn get_instances(&mut self, class: &IRI) -> OwlResult<HashSet<IRI>> {
        let mut instances = HashSet::new();

        // Get named individuals from ontology
        let individuals: Vec<_> = self.ontology.named_individuals().iter().cloned().collect();
        for individual in individuals {
            if self.is_instance_of(&individual.iri(), class)? {
                instances.insert(individual.iri().clone());
            }
        }

        Ok(instances)
    }

    /// Check if an individual is an instance of a class
    pub fn is_instance_of(&mut self, individual: &IRI, class: &IRI) -> OwlResult<bool> {
        // For now, check direct assertions in the ontology
        // This will be enhanced with full reasoning later
        for axiom in self.ontology.class_assertions() {
            if axiom.individual() == individual && axiom.class_expr().contains_class(class) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Run the tableaux algorithm
    fn run_tableaux(
        &mut self,
        graph: &mut TableauxGraph,
        config: ReasoningConfig,
    ) -> OwlResult<ReasoningResult> {
        let start_time = std::time::Instant::now();
        let mut stats = ReasoningStats {
            nodes_created: 1, // root node
            rules_applied: 0,
            time_ms: 0,
            cache_hits: 0,
            backtracks: 0,
        };

        let mut queue = VecDeque::new();
        queue.push_back(graph.root);

        while let Some(node_id) = queue.pop_front() {
            if stats.nodes_created > config.max_depth {
                return Ok(ReasoningResult {
                    is_satisfiable: false,
                    explanation: Some("Maximum depth exceeded".to_string()),
                    model: None,
                    stats,
                });
            }

            // Apply reasoning rules
            let node = graph.nodes.get(&node_id)
                .ok_or_else(|| OwlError::ReasoningError(format!("Node {} not found in graph", node_id.0)))?;
            let concepts: Vec<_> = node.concepts.iter().cloned().collect();
            let _new_nodes: Vec<NodeId> = Vec::new();

            for concept in concepts {
                if let Some((new_concepts, new_nodes_created)) =
                    self.apply_rules(&concept, node_id, graph)?
                {
                    stats.rules_applied += 1;

                    // Add new concepts to current node
                    for new_concept in new_concepts {
                        graph.add_concept(node_id, new_concept);
                    }

                    // Add new nodes to queue
                    for new_node_id in new_nodes_created {
                        queue.push_back(new_node_id);
                        stats.nodes_created += 1;
                    }
                }
            }

            // Check for contradictions
            let node = graph.nodes.get(&node_id)
                .ok_or_else(|| OwlError::ReasoningError(format!("Node {} not found in graph", node_id.0)))?;
            if self.has_contradiction(node) {
                stats.backtracks += 1;
                continue; // Skip this branch and try other paths
            }

            // Check blocking conditions
            if self.is_blocked(node_id, graph) {
                continue;
            }

            // Check if we've found a complete model (satisfiable)
            if self.is_complete_model(graph) {
                stats.time_ms = start_time.elapsed().as_millis() as u64;
                return Ok(ReasoningResult {
                    is_satisfiable: true,
                    explanation: Some("Complete model found".to_string()),
                    model: Some(self.extract_model(graph)),
                    stats,
                });
            }
        }

        stats.time_ms = start_time.elapsed().as_millis() as u64;

        // If we exhausted all possibilities without finding a model, it's unsatisfiable
        Ok(ReasoningResult {
            is_satisfiable: false,
            explanation: Some("No complete model found".to_string()),
            model: None,
            stats,
        })
    }

    /// Apply reasoning rules to a concept
    fn apply_rules(
        &self,
        concept: &ClassExpression,
        node_id: NodeId,
        graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        match concept {
            ClassExpression::ObjectIntersectionOf(operands) => {
                // Decompose intersection: C ⊓ D → C, D
                let operands_vec: Vec<ClassExpression> = operands.iter().map(|op| (**op).clone()).collect();
                Ok(Some((operands_vec, Vec::new())))
            }

            ClassExpression::ObjectUnionOf(operands) => {
                // Non-deterministic choice for union: C ⊔ D → C or D
                // For now, choose the first operand
                if !operands.is_empty() {
                    Ok(Some((vec![(*operands[0]).clone()], Vec::new())))
                } else {
                    Ok(None)
                }
            }

            ClassExpression::ObjectSomeValuesFrom(property, filler) => {
                // ∃R.C → create new node with C and R-edge from the current node
                if let Some(new_node_id) =
                    self.create_successor_node(node_id, property, filler, graph)
                {
                    Ok(Some((Vec::new(), vec![new_node_id])))
                } else {
                    Ok(None)
                }
            }

            ClassExpression::ObjectAllValuesFrom(property, filler) => {
                // ∀R.C → check all R-successors have C
                self.apply_all_values_from_rule(property, filler, node_id, graph)
            }

            ClassExpression::ObjectComplementOf(concept) => {
                // ¬C → check for contradiction with C and trigger propagation
                self.apply_complement_rule(concept, node_id, graph)
            }

            ClassExpression::Class(_) => {
                // Atomic class - no decomposition needed
                Ok(None)
            }

            ClassExpression::ObjectOneOf(individuals) => {
                // {a₁, ..., aₙ} → create nominal nodes
                self.apply_one_of_rule(individuals, node_id, graph)
            }

            ClassExpression::ObjectMinCardinality(n, property) => {
                // ≥ n R → ensure at least n R-successors
                self.apply_min_cardinality_rule(
                    *n as usize,
                    property,
                    &ClassExpression::Class(Class::new(
                        IRI::new("http://www.w3.org/2002/07/owl#Thing")
                            .expect("Failed to create owl:Thing IRI"),
                    )),
                    node_id,
                    graph,
                )
            }

            ClassExpression::ObjectMaxCardinality(n, property) => {
                // ≤ n R → ensure at most n R-successors
                self.apply_max_cardinality_rule(
                    *n as usize,
                    property,
                    &ClassExpression::Class(Class::new(
                        IRI::new("http://www.w3.org/2002/07/owl#Thing")
                            .expect("Failed to create owl:Thing IRI"),
                    )),
                    node_id,
                    graph,
                )
            }

            ClassExpression::ObjectExactCardinality(n, property) => {
                // = n R → ensure exactly n R-successors
                self.apply_exact_cardinality_rule(
                    *n as usize,
                    property,
                    &ClassExpression::Class(Class::new(
                        IRI::new("http://www.w3.org/2002/07/owl#Thing")
                            .expect("Failed to create owl:Thing IRI"),
                    )),
                    node_id,
                    graph,
                )
            }

            ClassExpression::DataSomeValuesFrom(_, _) => {
                // ∃P.D → data property restrictions (to be implemented)
                Ok(None)
            }

            ClassExpression::DataAllValuesFrom(_, _) => {
                // ∀P.D → data property restrictions (to be implemented)
                Ok(None)
            }

            ClassExpression::DataMinCardinality(_, _) => {
                // ≥ n P → data property restrictions (to be implemented)
                Ok(None)
            }

            ClassExpression::DataMaxCardinality(_, _) => {
                // ≤ n P → data property restrictions (to be implemented)
                Ok(None)
            }

            ClassExpression::DataExactCardinality(_, _) => {
                // = n P → data property restrictions (to be implemented)
                Ok(None)
            }

            ClassExpression::DataHasValue(_, _) => {
                // P(v) → data property has value (to be implemented)
                Ok(None)
            }

            ClassExpression::ObjectHasValue(_, _) => {
                // R(a) → object property has value (to be implemented)
                Ok(None)
            }

            ClassExpression::ObjectHasSelf(_) => {
                // R(a,a) → object has self (to be implemented)
                Ok(None)
            }
        }
    }

    /// Check if a node contains a contradiction
    fn has_contradiction(&self, node: &TableauxNode) -> bool {
        // Check for direct contradictions: C and ¬C in the same node
        let concepts: Vec<_> = node.concepts.iter().collect();

        for i in 0..concepts.len() {
            for j in i + 1..concepts.len() {
                if self.are_contradictory(concepts[i], concepts[j]) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if two class expressions are contradictory
    fn are_contradictory(&self, expr1: &ClassExpression, expr2: &ClassExpression) -> bool {
        use ClassExpression::*;

        match (expr1, expr2) {
            (Class(class1), Class(class2)) => {
                // Check if classes are declared disjoint
                for disjoint_axiom in self.ontology.disjoint_classes_axioms() {
                    let classes = disjoint_axiom.classes();
                    if classes.contains(&class1.iri()) && classes.contains(&class2.iri()) {
                        return true;
                    }
                }
                false
            }
            (ObjectComplementOf(comp1), ObjectComplementOf(comp2)) => {
                // ¬¬C ≡ C, so check if the inner expressions are contradictory
                self.are_contradictory(comp1.as_ref(), comp2.as_ref())
            }
            (ObjectComplementOf(comp), other) | (other, ObjectComplementOf(comp)) => {
                // Check if C and ¬C are contradictory (this is the main case)
                if let (ClassExpression::Class(class1), ClassExpression::Class(class2)) =
                    (comp.as_ref(), other)
                {
                    class1.iri() == class2.iri()
                } else if let (ClassExpression::Class(class1), ClassExpression::Class(class2)) =
                    (other, comp.as_ref())
                {
                    class1.iri() == class2.iri()
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Check if a node is blocked (simplified blocking detection)
    fn is_blocked(&self, node_id: NodeId, graph: &TableauxGraph) -> bool {
        let node = &graph.nodes[&node_id];

        // Check if this node is blocked by any ancestor
        let mut current = node_id;
        while let Some(parent) = graph.get_parent(current) {
            if parent == node_id {
                continue; // Skip self-reference
            }

            let parent_node = &graph.nodes[&parent];

            // Simple blocking: if parent has the same concepts (subset)
            let mut is_subset = true;
            for c in node.concepts_iter() {
                if !parent_node.has_concept(c) {
                    is_subset = false;
                    break;
                }
            }
            if is_subset {
                return true;
            }

            current = parent;
        }

        false
    }

    /// Check if we have a complete model (simplified)
    fn is_complete_model(&self, graph: &TableauxGraph) -> bool {
        // For now, consider any contradiction-free model as complete
        // In a full implementation, we'd check all applicable rules have been applied
        !graph
            .nodes
            .values()
            .any(|node| self.has_contradiction(node))
    }

    /// Create a successor node for existential restrictions from a specific node
    fn create_successor_node(
        &self,
        from_node: NodeId,
        property: &ObjectPropertyExpression,
        filler: &ClassExpression,
        graph: &mut TableauxGraph,
    ) -> Option<NodeId> {
        // Create a new node
        let new_node_id = graph.add_node();

        // Resolve property direction and named IRI (handles nested inverses)
        let (is_inverse, property_iri) = resolve_property_direction(property);

        if is_inverse {
            // For inverse R^-, we need an incoming edge via R: new_node --R--> from_node
            graph.add_edge(new_node_id, property_iri.clone(), from_node);
            graph.add_concept(new_node_id, filler.clone());
            Some(new_node_id)
        } else {
            // Regular direction: from_node --R--> new_node
            graph.add_edge(from_node, property_iri.clone(), new_node_id);
            graph.add_concept(new_node_id, filler.clone());
            Some(new_node_id)
        }
    }

    /// Apply ∀R.C rule: check all R-successors have C
    fn apply_all_values_from_rule(
        &self,
        property: &ObjectPropertyExpression,
        filler: &ClassExpression,
        node_id: NodeId,
        graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        // Determine if we look at successors (R) or predecessors (R^-)
        let (is_inverse, property_iri) = resolve_property_direction(property);

        if !is_inverse {
            // Collect successors first to avoid holding an immutable borrow while mutating
            let to_visit: Vec<NodeId> = graph
                .get_successors(node_id, property_iri)
                .map(|s| s.iter().copied().collect())
                .unwrap_or_default();

            for successor_id in to_visit.into_iter() {
                let needs_add = graph
                    .nodes
                    .get(&successor_id)
                    .map(|n| !n.concepts.contains(filler))
                    .unwrap_or(false);
                if needs_add {
                    graph.add_concept(successor_id, filler.clone());
                }
            }
        } else {
            // For inverse properties, ensure all predecessors via R have the filler
            let predecessors = graph.get_predecessors(node_id, property_iri);
            for pred_id in predecessors.into_iter() {
                let needs_add = graph
                    .nodes
                    .get(&pred_id)
                    .map(|n| !n.concepts.contains(filler))
                    .unwrap_or(false);
                if needs_add {
                    graph.add_concept(pred_id, filler.clone());
                }
            }
        }

        // No new concepts for the current node; side-effects applied to neighbors
        Ok(None)
    }

    /// Apply ¬C rule: check for contradiction and propagate
    fn apply_complement_rule(
        &self,
        concept: &ClassExpression,
        node_id: NodeId,
        graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        // Check if the complement concept exists in the node (contradiction)
        let node = graph.nodes.get(&node_id)
            .ok_or_else(|| OwlError::ReasoningError(format!("Node {} not found in graph", node_id.0)))?;

        // Check for direct contradiction
        if node.concepts.contains(concept) {
            // Contradiction found - this will be handled by has_contradiction
            return Ok(None);
        }

        // For negated class expressions, propagate the negation
        match concept {
            ClassExpression::Class(class) => {
                // For ¬A, check if A exists in the node
                let a_concept = ClassExpression::Class(class.clone());
                if node.concepts.contains(&a_concept) {
                    return Ok(None); // Contradiction will be detected
                }
            }
            ClassExpression::ObjectIntersectionOf(operands) => {
                // De Morgan's law: ¬(C₁ ⊓ ... ⊓ Cₙ) ≡ ¬C₁ ⊔ ... ⊔ ¬Cₙ
                let new_concepts: SmallVec<[Box<ClassExpression>; 4]> = operands
                    .iter()
                    .map(|op| Box::new(ClassExpression::ObjectComplementOf((*op).clone())))
                    .collect();
                return Ok(Some((vec![ClassExpression::ObjectUnionOf(new_concepts)], Vec::new())));
            }
            ClassExpression::ObjectUnionOf(operands) => {
                // De Morgan's law: ¬(C₁ ⊔ ... ⊔ Cₙ) ≡ ¬C₁ ⊓ ... ⊓ ¬Cₙ
                let new_concepts: SmallVec<[Box<ClassExpression>; 4]> = operands
                    .iter()
                    .map(|op| Box::new(ClassExpression::ObjectComplementOf((*op).clone())))
                    .collect();
                return Ok(Some((
                    vec![ClassExpression::ObjectIntersectionOf(new_concepts)],
                    Vec::new(),
                )));
            }
            _ => {}
        }

        Ok(None)
    }

    /// Apply {a₁, ..., aₙ} rule: create nominal nodes
    fn apply_one_of_rule(
        &self,
        individuals: &[crate::entities::Individual],
        _node_id: NodeId,
        _graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        // For oneOf, we typically create individual nodes
        // For now, just return the oneOf concept as is
        let new_concepts = vec![ClassExpression::ObjectOneOf(individuals.to_vec().into())];
        Ok(Some((new_concepts, Vec::new())))
    }

    /// Apply ≥ n R.C rule: ensure at least n R-successors with C
    fn apply_min_cardinality_rule(
        &self,
        n: usize,
        property: &ObjectPropertyExpression,
        filler: &ClassExpression,
        node_id: NodeId,
        graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        // Count existing R-successors with C
        let mut matching_successors = 0;
        let mut new_nodes = Vec::new();

        let property_iri = match property {
            ObjectPropertyExpression::ObjectProperty(prop) => prop.iri(),
            ObjectPropertyExpression::ObjectInverseOf(prop) => {
                // For inverse properties, use the original property's IRI
                match prop.as_ref() {
                    ObjectPropertyExpression::ObjectProperty(inner_prop) => inner_prop.iri(),
                    ObjectPropertyExpression::ObjectInverseOf(_) => {
                        // This is a more complex case - for now, use a dummy IRI
                        &IRI::new("http://example.org/inverse")
                            .expect("Failed to create inverse property IRI")
                    }
                }
            }
        };

        if let Some(successors) = graph.get_successors(node_id, property_iri) {
            for successor_id in successors {
                let successor_node = graph.nodes.get(successor_id)
                    .ok_or_else(|| OwlError::ReasoningError(format!("Successor node {} not found in graph", successor_id.0)))?;
                if successor_node.concepts.contains(filler) {
                    matching_successors += 1;
                }
            }
        }

        // Create additional successors if needed
        while matching_successors < (n as u32) {
            let new_node_id = graph.add_node();
            graph.add_edge(node_id, property_iri.clone(), new_node_id);
            graph.add_concept(new_node_id, filler.clone());
            new_nodes.push(new_node_id);
            matching_successors += 1;
        }

        if !new_nodes.is_empty() {
            Ok(Some((Vec::new(), new_nodes)))
        } else {
            Ok(None)
        }
    }

    /// Apply ≤ n R.C rule: ensure at most n R-successors with C
    fn apply_max_cardinality_rule(
        &self,
        _n: usize,
        _property: &ObjectPropertyExpression,
        _filler: &ClassExpression,
        _node_id: NodeId,
        _graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        // This is complex and requires blocking or merging
        // For now, return None (will be implemented later)
        Ok(None)
    }

    /// Apply = n R.C rule: ensure exactly n R-successors with C
    fn apply_exact_cardinality_rule(
        &self,
        n: usize,
        property: &ObjectPropertyExpression,
        filler: &ClassExpression,
        node_id: NodeId,
        graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        // = n R.C is equivalent to ≥ n R.C ⊓ ≤ n R.C
        let mut all_new_concepts = Vec::new();
        let mut all_new_nodes = Vec::new();

        // Apply min cardinality
        if let Some((concepts, nodes)) =
            self.apply_min_cardinality_rule(n, property, filler, node_id, graph)?
        {
            all_new_concepts.extend(concepts);
            all_new_nodes.extend(nodes);
        }

        // Apply max cardinality (not implemented yet)
        // if let Some((concepts, nodes)) = self.apply_max_cardinality_rule(n, property, filler, node_id, graph)? {
        //     all_new_concepts.extend(concepts);
        //     all_new_nodes.extend(nodes);
        // }

        if !all_new_concepts.is_empty() || !all_new_nodes.is_empty() {
            Ok(Some((all_new_concepts, all_new_nodes)))
        } else {
            Ok(None)
        }
    }

    /// Check if two concepts are complementary
    #[allow(dead_code)]
    fn are_complementary(&self, a: &ClassExpression, b: &ClassExpression) -> bool {
        match (a, b) {
            (ClassExpression::Class(iri_a), ClassExpression::ObjectComplementOf(box_b)) => {
                if let ClassExpression::Class(iri_b) = box_b.as_ref() {
                    return iri_a == iri_b;
                }
            }
            (ClassExpression::ObjectComplementOf(box_a), ClassExpression::Class(iri_b)) => {
                if let ClassExpression::Class(iri_a) = box_a.as_ref() {
                    return iri_a == iri_b;
                }
            }
            _ => {}
        }

        false
    }

    /// Extract a model from a completed tableau
    fn extract_model(&self, graph: &TableauxGraph) -> HashMap<IRI, HashSet<ClassExpression>> {
        let mut model = HashMap::new();

        for (node_id, node) in &graph.nodes {
            // Create a dummy IRI for the node
            let iri = IRI::new(&format!("http://example.org/node{}", node_id.0))
                .expect("Failed to create node IRI");

            // Convert the optimized concept storage to HashSet
            let concepts: HashSet<ClassExpression> = node.concepts_iter().cloned().collect();
            model.insert(iri, concepts);
        }

        model
    }
}

impl TableauxGraph {
    /// Create a new tableaux graph
    pub fn new() -> Self {
        let root = NodeId(0);
        let mut nodes = HashMap::new();
        nodes.insert(root, TableauxNode::new(root));

        TableauxGraph {
            nodes,
            edges: EdgeStorage::new(),
            root,
            next_id: 1,
            node_cache: HashMap::default(),
        }
    }

    /// Add a new node to the graph
    pub fn add_node(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        self.nodes.insert(id, TableauxNode::new(id));
        id
    }

    /// Add a concept to a node
    pub fn add_concept(&mut self, node_id: NodeId, concept: ClassExpression) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.add_concept(concept);
        }
    }

    /// Add an edge between nodes
    pub fn add_edge(&mut self, from: NodeId, property: IRI, to: NodeId) {
        self.edges.add_edge(from, &property, to);
    }

    /// Get the parent of a node (simplified - returns first parent found)
    pub fn get_parent(&self, node_id: NodeId) -> Option<NodeId> {
        // Use the flat edge storage for efficient iteration
        for (from, _, to) in &self.edges.edges {
            if *to == node_id {
                return Some(*from);
            }
        }
        None
    }

    /// Get all successors of a node via a property
    pub fn get_successors(&self, node_id: NodeId, property: &IRI) -> Option<&[NodeId]> {
        self.edges.get_targets(node_id, property)
    }

    /// Get all predecessors of a node via a property (optimized)
    pub fn get_predecessors(&self, node_id: NodeId, property: &IRI) -> Vec<NodeId> {
        let mut preds = Vec::new();
        // Use the flat edge storage for efficient iteration
        for (from, prop, to) in &self.edges.edges {
            if prop == property && *to == node_id {
                preds.push(*from);
            }
        }
        preds
    }
}

impl ReasoningCache {
    /// Create a new reasoning cache
    pub fn new(ontology: &Ontology) -> Self {
        let mut cache = ReasoningCache::with_capacity(1000); // Default cache size

        // Pre-compute class hierarchy
        for subclass_axiom in ontology.subclass_axioms() {
            let sub = subclass_axiom.sub_class();
            let sup = subclass_axiom.super_class();

            if let (ClassExpression::Class(sub_class), ClassExpression::Class(sup_class)) =
                (sub, sup)
            {
                let mut parents = SmallVec::new();
                if let Some(existing) = cache.class_hierarchy.get(sub_class.iri()) {
                    for iri in existing.iter() {
                        parents.push(iri.clone());
                    }
                }
                parents.push(sup_class.iri().clone());
                cache.class_hierarchy.insert(sub_class.iri().clone(), parents);
            }
        }

        // Pre-compute property hierarchy
        for subprop_axiom in ontology.subobject_property_axioms() {
            let sub = subprop_axiom.sub_property();
            let sup = subprop_axiom.super_property();

            let mut parents = SmallVec::new();
            if let Some(existing) = cache.property_hierarchy.get(sub) {
                for iri in existing.iter() {
                    parents.push(iri.clone());
                }
            }
            parents.push(sup.clone());
            cache.property_hierarchy.insert(sub.clone(), parents);
        }

        cache
    }
}

impl ReasoningRules {
    /// Create new reasoning rules
    pub fn new(_ontology: &Ontology) -> Self {
        ReasoningRules {
            // Rules will be initialized here
        }
    }
}

impl NodeId {
    /// Get the numeric value of the node ID
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axioms::property_expressions::ObjectPropertyExpression;
    use crate::entities::ObjectProperty;
    use crate::ontology::Ontology;

    #[test]
    fn test_tableaux_reasoner_creation() {
        let ontology = Ontology::new();
        let reasoner = TableauxReasoner::new(ontology);

        assert_eq!(reasoner.ontology.classes().len(), 0);
    }

    #[test]
    fn test_simple_satisfiability() {
        let mut ontology = Ontology::new();
        let class_iri = IRI::new("http://example.org/Person")
            .expect("Failed to create Person IRI");
        let person_class = Class::new(class_iri.clone());
        ontology.add_class(person_class)
            .expect("Failed to add Person class");

        let mut reasoner = TableauxReasoner::new(ontology);
        let result = reasoner.is_class_satisfiable(&class_iri)
            .expect("Failed to check satisfiability");

        assert!(result);
    }

    #[test]
    fn test_tableaux_graph() {
        let mut graph = TableauxGraph::new();
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.root, NodeId(0));

        let node2 = graph.add_node();
        assert_eq!(node2, NodeId(1));
        assert_eq!(graph.nodes.len(), 2);
    }

    #[test]
    fn test_concept_complementarity() {
        let ontology = Ontology::new();
        let reasoner = TableauxReasoner::new(ontology);

        let class_iri = IRI::new("http://example.org/Person")
            .expect("Failed to create Person IRI");
        let person_class = Class::new(class_iri.clone());
        let concept = ClassExpression::Class(person_class.clone());
        let complement = ClassExpression::ObjectComplementOf(Box::new(concept.clone()));

        assert!(reasoner.are_complementary(&concept, &complement));
        assert!(reasoner.are_complementary(&complement, &concept));
    }

    #[test]
    fn test_some_values_from_edge_direction_named() {
        let ontology = Ontology::new();
        let reasoner = TableauxReasoner::new(ontology);

        let mut graph = TableauxGraph::new();
        let root = graph.root;

        let prop_iri = IRI::new("http://example.org/hasFriend")
            .expect("Failed to create hasFriend property IRI");
        let prop = ObjectProperty::new(prop_iri.clone());
        let filler_class = Class::new(IRI::new("http://example.org/Person")
            .expect("Failed to create Person IRI"));
        let filler = ClassExpression::Class(filler_class);

        let new_node = reasoner
            .create_successor_node(
                root,
                &ObjectPropertyExpression::ObjectProperty(prop),
                &filler,
                &mut graph,
            )
            .expect("should create node");

        // Edge should be root --hasFriend--> new_node
        let succs: Vec<NodeId> = graph
            .get_successors(root, &prop_iri)
            .map(|s| s.to_vec())
            .unwrap_or_default();
        assert!(succs.contains(&new_node));

        // Reverse should not exist
        let rev: Vec<NodeId> = graph
            .get_successors(new_node, &prop_iri)
            .map(|s| s.to_vec())
            .unwrap_or_default();
        assert!(!rev.contains(&root));
    }

    #[test]
    fn test_some_values_from_edge_direction_inverse() {
        let ontology = Ontology::new();
        let reasoner = TableauxReasoner::new(ontology);

        let mut graph = TableauxGraph::new();
        let root = graph.root;

        let prop_iri = IRI::new("http://example.org/hasFriend")
            .expect("Failed to create hasFriend property IRI");
        let prop = ObjectProperty::new(prop_iri.clone());
        let filler_class = Class::new(IRI::new("http://example.org/Person")
            .expect("Failed to create Person IRI"));
        let filler = ClassExpression::Class(filler_class);

        // Use inverse property expression
        let new_node = reasoner
            .create_successor_node(
                root,
                &ObjectPropertyExpression::ObjectInverseOf(Box::new(
                    ObjectPropertyExpression::ObjectProperty(prop),
                )),
                &filler,
                &mut graph,
            )
            .expect("should create node");

        // Edge should be new_node --hasFriend--> root
        let succs: Vec<NodeId> = graph
            .get_successors(new_node, &prop_iri)
            .map(|s| s.to_vec())
            .unwrap_or_default();
        assert!(succs.contains(&root));

        // Forward from root should not contain new_node
        let forward: Vec<NodeId> = graph
            .get_successors(root, &prop_iri)
            .map(|s| s.to_vec())
            .unwrap_or_default();
        assert!(!forward.contains(&new_node));
    }

    #[test]
    fn test_all_values_from_applies_to_successors() {
        let ontology = Ontology::new();
        let reasoner = TableauxReasoner::new(ontology);
        let mut graph = TableauxGraph::new();
        let root = graph.root;

        // Build root --p--> succ
        let prop_iri = IRI::new("http://example.org/p")
            .expect("Failed to create property IRI");
        let prop = ObjectProperty::new(prop_iri.clone());
        let succ = graph.add_node();
        graph.add_edge(root, prop_iri.clone(), succ);

        let filler = ClassExpression::Class(Class::new(IRI::new("http://example.org/C")
            .expect("Failed to create class C IRI")));

        // Apply ∀p.C at root
        reasoner
            .apply_all_values_from_rule(
                &ObjectPropertyExpression::ObjectProperty(prop),
                &filler,
                root,
                &mut graph,
            )
            .expect("Failed to apply all values from rule");

        // Successor must contain filler
        let succ_node = graph.nodes.get(&succ)
            .expect("Successor node not found");
        assert!(succ_node.concepts.contains(&filler));
    }

    #[test]
    fn test_all_values_from_inverse_applies_to_predecessors() {
        let ontology = Ontology::new();
        let reasoner = TableauxReasoner::new(ontology);
        let mut graph = TableauxGraph::new();
        let root = graph.root;

        // Build pred --p--> root
        let prop_iri = IRI::new("http://example.org/p")
            .expect("Failed to create property IRI");
        let prop = ObjectProperty::new(prop_iri.clone());
        let pred = graph.add_node();
        graph.add_edge(pred, prop_iri.clone(), root);

        let filler = ClassExpression::Class(Class::new(IRI::new("http://example.org/C")
            .expect("Failed to create class C IRI")));

        // Apply ∀p^-.C at root
        reasoner
            .apply_all_values_from_rule(
                &ObjectPropertyExpression::ObjectInverseOf(Box::new(
                    ObjectPropertyExpression::ObjectProperty(prop),
                )),
                &filler,
                root,
                &mut graph,
            )
            .expect("Failed to apply all values from rule");

        // Predecessor must contain filler
        let pred_node = graph.nodes.get(&pred)
            .expect("Predecessor node not found");
        assert!(pred_node.concepts.contains(&filler));
    }
}
