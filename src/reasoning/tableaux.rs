//! OWL2 Tableaux Reasoning Engine
//! 
//! Implements a tableaux-based reasoning algorithm for OWL2 ontologies
//! based on SROIQ(D) description logic.

use crate::ontology::Ontology;
use crate::iri::IRI;
use crate::entities::*;
use crate::axioms::*;
use crate::error::OwlResult;

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

/// Tableaux reasoning engine for OWL2 ontologies
pub struct TableauxReasoner {
    pub ontology: Arc<Ontology>,
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

/// Individual node in the tableaux
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableauxNode {
    id: NodeId,
    concepts: HashSet<ClassExpression>,
    labels: HashSet<String>,
    blocked_by: Option<NodeId>,
}

/// Node identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(usize);

/// Tableaux graph structure
#[derive(Debug)]
pub struct TableauxGraph {
    nodes: HashMap<NodeId, TableauxNode>,
    edges: HashMap<NodeId, HashMap<IRI, HashSet<NodeId>>>,
    root: NodeId,
    next_id: usize,
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

/// Reasoning cache for performance
#[derive(Debug, Default)]
struct ReasoningCache {
    concept_satisfiability: HashMap<ClassExpression, bool>,
    class_hierarchy: HashMap<IRI, HashSet<IRI>>,
    property_hierarchy: HashMap<IRI, HashSet<IRI>>,
}

/// Built-in reasoning rules
#[derive(Debug)]
struct ReasoningRules {
    // Rule implementations will be added here
}

impl TableauxReasoner {
    /// Create a new tableaux reasoner
    pub fn new(ontology: Ontology) -> Self {
        Self::with_config(ontology, ReasoningConfig::default())
    }
    
    /// Create a new tableaux reasoner with custom configuration
    pub fn with_config(ontology: Ontology, config: ReasoningConfig) -> Self {
        let ontology = Arc::new(ontology);
        let rules = ReasoningRules::new(&ontology);
        let cache = ReasoningCache::new(&ontology);
        
        TableauxReasoner {
            ontology,
            rules,
            cache,
        }
    }
    
    /// Check if a class expression is satisfiable
    pub fn is_satisfiable(&mut self, concept: &ClassExpression) -> OwlResult<bool> {
        // Check cache first
        if let Some(result) = self.cache.concept_satisfiability.get(concept) {
            return Ok(*result);
        }
        
        // Create tableaux graph
        let mut graph = TableauxGraph::new();
        let root = graph.add_node();
        graph.add_concept(root, concept.clone());
        
        // Run tableaux algorithm
        let result = self.run_tableaux(&mut graph, ReasoningConfig::default())?;
        
        // Cache result
        self.cache.concept_satisfiability.insert(concept.clone(), result.is_satisfiable);
        
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
        let intersection = ClassExpression::ObjectIntersectionOf(vec![
            sub_concept,
            ClassExpression::ObjectComplementOf(Box::new(sup_concept)),
        ]);

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
        let intersection = ClassExpression::ObjectIntersectionOf(vec![a_concept, b_concept]);

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
    fn run_tableaux(&mut self, graph: &mut TableauxGraph, config: ReasoningConfig) -> OwlResult<ReasoningResult> {
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
            let concepts: Vec<_> = graph.nodes.get(&node_id).unwrap().concepts.iter().cloned().collect();
            let mut new_nodes: Vec<NodeId> = Vec::new();

            for concept in concepts {
                if let Some((new_concepts, new_nodes_created)) = self.apply_rules(&concept, node_id, graph)? {
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
            if self.has_contradiction(graph.nodes.get(&node_id).unwrap()) {
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
    fn apply_rules(&self, concept: &ClassExpression, node_id: NodeId, graph: &mut TableauxGraph)
        -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>>
    {
        match concept {
            ClassExpression::ObjectIntersectionOf(operands) => {
                // Decompose intersection: C ⊓ D → C, D
                Ok(Some((operands.clone(), Vec::new())))
            }
            
            ClassExpression::ObjectUnionOf(operands) => {
                // Non-deterministic choice for union: C ⊔ D → C or D
                // For now, choose the first operand
                if !operands.is_empty() {
                    Ok(Some((vec![operands[0].clone()], Vec::new())))
                } else {
                    Ok(None)
                }
            }
            
            ClassExpression::ObjectSomeValuesFrom(property, filler) => {
                // ∃R.C → create new node with C and R-edge
                if let Some(new_node_id) = self.create_successor_node(property, filler, graph) {
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
                self.apply_min_cardinality_rule(*n as usize, property, &ClassExpression::Class(Class::new(IRI::new("http://www.w3.org/2002/07/owl#Thing").unwrap())), node_id, graph)
            }

            ClassExpression::ObjectMaxCardinality(n, property) => {
                // ≤ n R → ensure at most n R-successors
                self.apply_max_cardinality_rule(*n as usize, property, &ClassExpression::Class(Class::new(IRI::new("http://www.w3.org/2002/07/owl#Thing").unwrap())), node_id, graph)
            }

            ClassExpression::ObjectExactCardinality(n, property) => {
                // = n R → ensure exactly n R-successors
                self.apply_exact_cardinality_rule(*n as usize, property, &ClassExpression::Class(Class::new(IRI::new("http://www.w3.org/2002/07/owl#Thing").unwrap())), node_id, graph)
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
            },
            (ObjectComplementOf(comp1), ObjectComplementOf(comp2)) => {
                // ¬¬C ≡ C, so check if the inner expressions are contradictory
                self.are_contradictory(comp1.as_ref(), comp2.as_ref())
            },
            (ObjectComplementOf(comp), other) | (other, ObjectComplementOf(comp)) => {
                // Check if C and ¬C are contradictory (this is the main case)
                if let (ClassExpression::Class(class1), ClassExpression::Class(class2)) = (comp.as_ref(), other) {
                    class1.iri() == class2.iri()
                } else if let (ClassExpression::Class(class1), ClassExpression::Class(class2)) = (other, comp.as_ref()) {
                    class1.iri() == class2.iri()
                } else {
                    false
                }
            },
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
            if parent_node.concepts.is_superset(&node.concepts) {
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
        !graph.nodes.values().any(|node| self.has_contradiction(node))
    }

    /// Create a successor node for existential restrictions
    fn create_successor_node(&self, property: &ObjectPropertyExpression, filler: &ClassExpression, graph: &mut TableauxGraph)
        -> Option<NodeId>
    {
        // Check if a suitable successor already exists
        let node_id = NodeId(graph.next_id);
        graph.add_node();

        let property_iri = match property {
            ObjectPropertyExpression::ObjectProperty(prop) => prop.iri(),
            ObjectPropertyExpression::ObjectInverseOf(prop) => {
                // For inverse properties, use the original property's IRI
                match prop.as_ref() {
                    ObjectPropertyExpression::ObjectProperty(inner_prop) => inner_prop.iri(),
                    ObjectPropertyExpression::ObjectInverseOf(_) => {
                        // This is a more complex case - for now, use a dummy IRI
                        &IRI::new("http://example.org/inverse").unwrap()
                    }
                }
            }
        };

        graph.add_edge(graph.root, property_iri.clone(), node_id);
        graph.add_concept(node_id, filler.clone());
        Some(node_id)
    }

    /// Apply ∀R.C rule: check all R-successors have C
    fn apply_all_values_from_rule(&self, property: &ObjectPropertyExpression, filler: &ClassExpression, node_id: NodeId, graph: &mut TableauxGraph)
        -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>>
    {
        // Get all R-successors of the current node
        let property_iri = match property {
            ObjectPropertyExpression::ObjectProperty(prop) => prop.iri(),
            ObjectPropertyExpression::ObjectInverseOf(prop) => {
                // For inverse properties, use the original property's IRI
                match prop.as_ref() {
                    ObjectPropertyExpression::ObjectProperty(inner_prop) => inner_prop.iri(),
                    ObjectPropertyExpression::ObjectInverseOf(_) => {
                        // This is a more complex case - for now, use a dummy IRI
                        &IRI::new("http://example.org/inverse").unwrap()
                    }
                }
            }
        };

        if let Some(successors) = graph.get_successors(node_id, property_iri) {
            // For each successor, ensure it has the filler concept
            let mut new_concepts = Vec::new();
            for successor_id in successors {
                // Check if successor already has filler concept
                let successor_node = graph.nodes.get(successor_id).unwrap();
                if !successor_node.concepts.contains(filler) {
                    new_concepts.push(filler.clone());
                }
            }

            if !new_concepts.is_empty() {
                return Ok(Some((new_concepts, Vec::new())));
            }
        }

        Ok(None)
    }

    /// Apply ¬C rule: check for contradiction and propagate
    fn apply_complement_rule(&self, concept: &ClassExpression, node_id: NodeId, graph: &mut TableauxGraph)
        -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>>
    {
        // Check if the complement concept exists in the node (contradiction)
        let node = graph.nodes.get(&node_id).unwrap();

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
                let new_concepts: Vec<ClassExpression> = operands.iter()
                    .map(|op| ClassExpression::ObjectComplementOf(Box::new(op.clone())))
                    .collect();
                return Ok(Some((new_concepts, Vec::new())));
            }
            ClassExpression::ObjectUnionOf(operands) => {
                // De Morgan's law: ¬(C₁ ⊔ ... ⊔ Cₙ) ≡ ¬C₁ ⊓ ... ⊓ ¬Cₙ
                let new_concepts: Vec<ClassExpression> = operands.iter()
                    .map(|op| ClassExpression::ObjectComplementOf(Box::new(op.clone())))
                    .collect();
                return Ok(Some((vec![ClassExpression::ObjectIntersectionOf(new_concepts)], Vec::new())));
            }
            _ => {}
        }

        Ok(None)
    }

    /// Apply {a₁, ..., aₙ} rule: create nominal nodes
    fn apply_one_of_rule(&self, individuals: &[crate::entities::Individual], node_id: NodeId, graph: &mut TableauxGraph)
        -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>>
    {
        // For oneOf, we typically create individual nodes
        // For now, just return the oneOf concept as is
        let new_concepts = vec![ClassExpression::ObjectOneOf(individuals.to_vec())];
        Ok(Some((new_concepts, Vec::new())))
    }

    /// Apply ≥ n R.C rule: ensure at least n R-successors with C
    fn apply_min_cardinality_rule(&self, n: usize, property: &ObjectPropertyExpression, filler: &ClassExpression, node_id: NodeId, graph: &mut TableauxGraph)
        -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>>
    {
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
                        &IRI::new("http://example.org/inverse").unwrap()
                    }
                }
            }
        };

        if let Some(successors) = graph.get_successors(node_id, property_iri) {
            for successor_id in successors {
                let successor_node = graph.nodes.get(successor_id).unwrap();
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
    fn apply_max_cardinality_rule(&self, n: usize, property: &ObjectPropertyExpression, filler: &ClassExpression, node_id: NodeId, graph: &mut TableauxGraph)
        -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>>
    {
        // This is complex and requires blocking or merging
        // For now, return None (will be implemented later)
        Ok(None)
    }

    /// Apply = n R.C rule: ensure exactly n R-successors with C
    fn apply_exact_cardinality_rule(&self, n: usize, property: &ObjectPropertyExpression, filler: &ClassExpression, node_id: NodeId, graph: &mut TableauxGraph)
        -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>>
    {
        // = n R.C is equivalent to ≥ n R.C ⊓ ≤ n R.C
        let mut all_new_concepts = Vec::new();
        let mut all_new_nodes = Vec::new();

        // Apply min cardinality
        if let Some((concepts, nodes)) = self.apply_min_cardinality_rule(n, property, filler, node_id, graph)? {
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
            let iri = IRI::new(&format!("http://example.org/node{}", node_id.0)).unwrap();
            model.insert(iri, node.concepts.clone());
        }
        
        model
    }
}

impl TableauxGraph {
    /// Create a new tableaux graph
    pub fn new() -> Self {
        let root = NodeId(0);
        let mut nodes = HashMap::new();
        nodes.insert(root, TableauxNode {
            id: root,
            concepts: HashSet::new(),
            labels: HashSet::new(),
            blocked_by: None,
        });
        
        TableauxGraph {
            nodes,
            edges: HashMap::new(),
            root,
            next_id: 1,
        }
    }
    
    /// Add a new node to the graph
    pub fn add_node(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        
        self.nodes.insert(id, TableauxNode {
            id,
            concepts: HashSet::new(),
            labels: HashSet::new(),
            blocked_by: None,
        });
        
        id
    }
    
    /// Add a concept to a node
    pub fn add_concept(&mut self, node_id: NodeId, concept: ClassExpression) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.concepts.insert(concept);
        }
    }
    
    /// Add an edge between nodes
    pub fn add_edge(&mut self, from: NodeId, property: IRI, to: NodeId) {
        self.edges.entry(from).or_insert_with(HashMap::new)
            .entry(property).or_insert_with(HashSet::new)
            .insert(to);
    }

    /// Get the parent of a node (simplified - returns first parent found)
    pub fn get_parent(&self, node_id: NodeId) -> Option<NodeId> {
        // Search through all edges to find which node has this node as a child
        for (parent, edges) in &self.edges {
            for children in edges.values() {
                if children.contains(&node_id) {
                    return Some(*parent);
                }
            }
        }
        None
    }
    
    /// Get all successors of a node via a property
    pub fn get_successors(&self, node_id: NodeId, property: &IRI) -> Option<&HashSet<NodeId>> {
        self.edges.get(&node_id).and_then(|edges| edges.get(property))
    }
}

impl ReasoningCache {
    /// Create a new reasoning cache
    pub fn new(ontology: &Ontology) -> Self {
        let mut cache = ReasoningCache::default();
        
        // Pre-compute class hierarchy
        for subclass_axiom in ontology.subclass_axioms() {
            let sub = subclass_axiom.sub_class();
            let sup = subclass_axiom.super_class();
            
            if let (ClassExpression::Class(sub_class), ClassExpression::Class(sup_class)) = (sub, sup) {
                cache.class_hierarchy.entry(sub_class.iri().clone()).or_insert_with(HashSet::new).insert(sup_class.iri().clone());
            }
        }
        
        // Pre-compute property hierarchy
        for subprop_axiom in ontology.subobject_property_axioms() {
            let sub = subprop_axiom.sub_property();
            let sup = subprop_axiom.super_property();
            
            cache.property_hierarchy.entry(sub.clone()).or_insert_with(HashSet::new).insert(sup.clone());
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
        let class_iri = IRI::new("http://example.org/Person").unwrap();
        let person_class = Class::new(class_iri.clone());
        ontology.add_class(person_class).unwrap();
        
        let mut reasoner = TableauxReasoner::new(ontology);
        let result = reasoner.is_class_satisfiable(&class_iri).unwrap();
        
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
        
        let class_iri = IRI::new("http://example.org/Person").unwrap();
        let person_class = Class::new(class_iri.clone());
        let concept = ClassExpression::Class(person_class.clone());
        let complement = ClassExpression::ObjectComplementOf(Box::new(concept.clone()));
        
        assert!(reasoner.are_complementary(&concept, &complement));
        assert!(reasoner.are_complementary(&complement, &concept));
    }
}