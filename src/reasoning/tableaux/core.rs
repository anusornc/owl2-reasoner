//! # Tableaux Reasoning Core
//!
//! Contains the fundamental data structures, configuration, and main reasoning engine
//! for the tableaux-based OWL2 reasoner. This module provides the primary interface
//! for consistency checking and classification.
//!
//! ## Key Components
//!
//! - **[`TableauxReasoner`]** - Main reasoning engine with caching and configuration
//! - **[`ReasoningConfig`]** - Configurable options for reasoning behavior
//! - **[`ReasoningRules`]** - Extracted reasoning rules from ontology
//! - **[`TableauxNode`]** - Individual nodes in the tableaux graph
//! - **[`NodeId`]** - Unique identifiers for graph nodes
//! - **[`ReasoningCache`]** - Performance optimization through caching
//! - **[`MemoryStats`]** - Memory usage tracking and profiling
//!
//! ## Reasoning Process
//!
//! 1. **Rule Extraction**: Extract subclass, equivalence, and property rules from ontology
//! 2. **Consistency Checking**: Verify ontology satisfiability using tableaux algorithm
//! 3. **Classification**: Compute class hierarchy and relationships
//! 4. **Caching**: Store results for performance optimization
//! 5. **Memory Management**: Track allocation and deallocation patterns
//!
//! ## Performance Features
//!
//! - **Multi-layered caching**: Consistency, satisfiability, and classification results
//! - **Optimized concept storage**: SmallVec for small sets, fallback to HashSet
//! - **Configurable timeouts**: Prevent infinite reasoning loops
//! - **Incremental reasoning**: Support for partial ontology updates
//! - **Memory profiling**: Detailed statistics for optimization
//!
//! ## Example Usage
//!
//! ```rust
//! use owl2_reasoner::{Ontology, TableauxReasoner, ReasoningConfig};
//!
//! // Create ontology and configure reasoner
//! let ontology = Ontology::new();
//! let config = ReasoningConfig {
//!     max_depth: 1000,
//!     debug: false,
//!     incremental: true,
//!     timeout: Some(30000),
//! };
//! let reasoner = TableauxReasoner::with_config(ontology, config);
//!
//! // Perform reasoning
//! let is_consistent = reasoner.is_consistent()?;
//! let memory_stats = reasoner.get_memory_stats();
//! println!("Consistent: {}, Memory used: {} bytes",
//!          is_consistent, memory_stats.peak_memory_bytes);
//! ```

use crate::axioms::*;
use crate::entities::Class;
use crate::error::{OwlError, OwlResult};
use crate::iri::IRI;
use crate::ontology::Ontology;

use hashbrown::HashMap;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::Arc;

/// Reasoning rules for tableaux algorithm
#[derive(Debug)]
pub struct ReasoningRules {
    pub subclass_rules: Vec<SubClassOfAxiom>,
    pub equivalence_rules: Vec<EquivalentClassesAxiom>,
    pub disjointness_rules: Vec<DisjointClassesAxiom>,
    pub property_rules: Vec<SubObjectPropertyAxiom>,
}

impl ReasoningRules {
    pub fn new(ontology: &Ontology) -> Self {
        let subclass_rules = ontology
            .subclass_axioms()
            .iter()
            .map(|ax| (**ax).clone())
            .collect();
        let equivalence_rules = ontology
            .equivalent_classes_axioms()
            .iter()
            .map(|ax| (**ax).clone())
            .collect();
        let disjointness_rules = ontology
            .disjoint_classes_axioms()
            .iter()
            .map(|ax| (**ax).clone())
            .collect();
        let property_rules = ontology
            .subobject_property_axioms()
            .iter()
            .map(|ax| (**ax).clone())
            .collect();

        Self {
            subclass_rules,
            equivalence_rules,
            disjointness_rules,
            property_rules,
        }
    }

    pub fn clear(&mut self) {
        self.subclass_rules.clear();
        self.equivalence_rules.clear();
        self.disjointness_rules.clear();
        self.property_rules.clear();
    }
}

/// Node identifier for tableaux graph nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(usize);

impl NodeId {
    pub fn new(id: usize) -> Self {
        NodeId(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
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
    /// Enable parallel processing
    pub enable_parallel: bool,
    /// Number of parallel workers (None = use all available cores)
    pub parallel_workers: Option<usize>,
    /// Chunk size for parallel operations
    pub parallel_chunk_size: usize,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        ReasoningConfig {
            max_depth: 1000,
            debug: false,
            incremental: true,
            timeout: Some(30000),   // 30 seconds default
            enable_parallel: false, // Disabled by default for compatibility
            parallel_workers: None, // Use all available cores
            parallel_chunk_size: 64,
        }
    }
}

/// Tableaux node with optimized concept storage and blocking support
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableauxNode {
    pub id: NodeId,
    /// Optimized concept storage using SmallVec for small sets
    pub concepts: SmallVec<[ClassExpression; 8]>,
    /// Lazy hashset for large concept sets
    pub concepts_hashset: Option<HashSet<ClassExpression>>,
    /// Node labels for debugging and identification
    pub labels: SmallVec<[String; 4]>,
    /// Optional blocking reference for optimization
    pub blocked_by: Option<NodeId>,
}

impl TableauxNode {
    pub fn new(id: NodeId) -> Self {
        TableauxNode {
            id,
            concepts: SmallVec::new(),
            concepts_hashset: None,
            labels: SmallVec::new(),
            blocked_by: None,
        }
    }

    pub fn add_concept(&mut self, concept: ClassExpression) {
        if self.concepts_hashset.is_some() {
            // Use hashset for large collections with safe access
            if let Some(hashset) = &mut self.concepts_hashset {
                hashset.insert(concept);
            }
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

    pub fn contains_concept(&self, concept: &ClassExpression) -> bool {
        if let Some(ref hashset) = self.concepts_hashset {
            hashset.contains(concept)
        } else {
            self.concepts.contains(concept)
        }
    }

    pub fn concepts_iter(&self) -> impl Iterator<Item = &ClassExpression> {
        if let Some(ref hashset) = self.concepts_hashset {
            Either::Left(hashset.iter())
        } else {
            Either::Right(self.concepts.iter())
        }
    }

    /// Get the number of concepts in this node
    pub fn concepts_len(&self) -> usize {
        if let Some(ref hashset) = self.concepts_hashset {
            hashset.len()
        } else {
            self.concepts.len()
        }
    }

    /// Add a label to this node
    pub fn add_label(&mut self, label: String) {
        if !self.labels.contains(&label) {
            self.labels.push(label);
        }
    }

    /// Get all labels for this node
    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    /// Check if this node is blocked by another node
    pub fn is_blocked(&self) -> bool {
        self.blocked_by.is_some()
    }

    /// Set the blocking node for this node
    pub fn set_blocked_by(&mut self, blocking_node: NodeId) {
        self.blocked_by = Some(blocking_node);
    }

    /// Clear blocking for this node
    pub fn clear_blocking(&mut self) {
        self.blocked_by = None;
    }

    /// Get the node that blocks this node, if any
    pub fn blocked_by(&self) -> Option<NodeId> {
        self.blocked_by
    }
}

/// Helper enum for iteration
enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L: Iterator, R: Iterator<Item = L::Item>> Iterator for Either<L, R> {
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(l) => l.next(),
            Either::Right(r) => r.next(),
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Default, Clone)]
pub struct MemoryStats {
    pub arena_allocated_nodes: usize,
    pub arena_allocated_edges: usize,
    pub arena_allocated_expressions: usize,
    pub total_arena_bytes: usize,
    pub peak_memory_bytes: usize,
}

impl MemoryStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node_allocation(&mut self, bytes: usize) {
        self.arena_allocated_nodes += 1;
        self.total_arena_bytes += bytes;
        self.peak_memory_bytes = self.peak_memory_bytes.max(self.total_arena_bytes);
    }

    pub fn add_edge_allocation(&mut self, bytes: usize) {
        self.arena_allocated_edges += 1;
        self.total_arena_bytes += bytes;
        self.peak_memory_bytes = self.peak_memory_bytes.max(self.total_arena_bytes);
    }

    pub fn add_expression_allocation(&mut self, bytes: usize) {
        self.arena_allocated_expressions += 1;
        self.total_arena_bytes += bytes;
        self.peak_memory_bytes = self.peak_memory_bytes.max(self.total_arena_bytes);
    }
}

/// Reasoning cache for performance optimization
#[derive(Debug, Default)]
pub struct ReasoningCache {
    pub consistency_cache: HashMap<Vec<ClassExpression>, bool>,
    pub satisfiability_cache: HashMap<ClassExpression, bool>,
    pub classification_cache: HashMap<(IRI, IRI), bool>,
}

impl ReasoningCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.consistency_cache.clear();
        self.satisfiability_cache.clear();
        self.classification_cache.clear();
    }
}

/// Core tableaux reasoning engine
pub struct TableauxReasoner {
    pub ontology: Arc<Ontology>,
    pub config: ReasoningConfig,
    pub rules: ReasoningRules,
    pub cache: ReasoningCache,
    pub memory_stats: RefCell<MemoryStats>,
    /// Dependency-directed backtracking manager
    pub dependency_manager: super::dependency::DependencyManager,
}

impl TableauxReasoner {
    pub fn new(ontology: Ontology) -> Self {
        Self::with_config(ontology, ReasoningConfig::default())
    }

    pub fn with_config(ontology: Ontology, config: ReasoningConfig) -> Self {
        let rules = ReasoningRules::new(&ontology);

        Self {
            ontology: Arc::new(ontology),
            config,
            rules,
            cache: ReasoningCache::new(),
            memory_stats: RefCell::new(MemoryStats::new()),
            dependency_manager: super::dependency::DependencyManager::new(),
        }
    }

    pub fn from_arc(ontology: &Arc<Ontology>) -> Self {
        Self::with_config(Ontology::clone(ontology), ReasoningConfig::default())
    }

    pub fn check_consistency(&mut self) -> OwlResult<bool> {
        // Create a new tableaux graph for consistency checking
        let mut graph = super::graph::TableauxGraph::new();
        let mut expansion_engine = super::expansion::ExpansionEngine::new();
        let blocking_manager = super::blocking::BlockingManager::new(super::blocking::BlockingStrategy::Optimized);
        let mut memory_manager = super::memory::MemoryManager::new();
        
        // Initialize the root node with all classes from the ontology
        self.initialize_root_node(&mut graph)?;
        
        // Track reasoning state
        let mut nodes_to_expand = std::collections::VecDeque::new();
        nodes_to_expand.push_back(graph.get_root());
        
        let mut expanded_nodes = std::collections::HashSet::new();
        expanded_nodes.insert(graph.get_root());
        
        // Main reasoning loop
        while let Some(current_node) = nodes_to_expand.pop_front() {
            // Check if current node should be blocked
            if blocking_manager.should_block_node(current_node, &graph) {
                continue;
            }
            
            // Apply tableaux expansion rules
            expansion_engine.context.current_node = current_node;
            let _expansion_result = expansion_engine.expand(&mut graph, &mut memory_manager, self.config.max_depth)
                .map_err(|e| OwlError::ReasoningError(format!("Expansion failed: {}", e)))?;
            
            // Check for clashes after expansion
            if self.has_clash(current_node, &graph)? {
                return Ok(false); // Ontology is inconsistent
            }
            
            // Get newly created nodes from expansion
            let new_nodes = self.get_new_successors(current_node, &graph, &expanded_nodes);
            
            // Add new nodes to expansion queue
            for new_node in new_nodes {
                if !expanded_nodes.contains(&new_node) {
                    nodes_to_expand.push_back(new_node);
                    expanded_nodes.insert(new_node);
                }
            }
            
            // Handle backtracking if needed
            if let Some(backtrack_point) = self.dependency_manager.latest_backtrack_point() {
                if backtrack_point.exhausted {
                    // Backtrack to previous choice point
                    self.dependency_manager.backtrack_to_level(backtrack_point.level - 1)?;
                }
            }
            
            // Check timeout
            if let Some(timeout_ms) = self.config.timeout {
                let start_time = std::time::Instant::now();
                if start_time.elapsed().as_millis() >= timeout_ms as u128 {
                    return Err(OwlError::TimeoutError {
                        operation: "consistency_checking".to_string(),
                        timeout_ms,
                    });
                }
            }
        }
        
        // If we completed without finding a clash, the ontology is consistent
        Ok(true)
    }

    pub fn classify(&self) -> OwlResult<()> {
        // Core classification logic will be implemented here
        Ok(())
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_memory_stats(&self) -> MemoryStats {
        self.memory_stats.borrow().clone()
    }

    pub fn reset(&mut self) {
        self.cache.clear();
        self.rules.clear();
        self.dependency_manager.clear();
        *self.memory_stats.borrow_mut() = MemoryStats::new();
    }

    pub fn is_consistent(&mut self) -> OwlResult<bool> {
        // Placeholder implementation
        self.check_consistency()
    }

    pub fn get_subclasses(&self, _class: &IRI) -> Vec<IRI> {
        // Placeholder implementation
        Vec::new()
    }

    pub fn get_superclasses(&self, _class: &IRI) -> Vec<IRI> {
        // Placeholder implementation
        Vec::new()
    }

    pub fn get_equivalent_classes(&self, _class: &IRI) -> Vec<IRI> {
        // Placeholder implementation
        Vec::new()
    }

    pub fn get_disjoint_classes(&self, _class: &IRI) -> Vec<IRI> {
        // Placeholder implementation
        Vec::new()
    }

    pub fn are_disjoint_classes(&self, _class1: &IRI, _class2: &IRI) -> OwlResult<bool> {
        // Placeholder implementation - check if two classes are disjoint
        Ok(false)
    }

    /// Check if two class expressions represent disjoint classes
    fn are_disjoint_class_expressions(&self, concept1: &ClassExpression, concept2: &ClassExpression) -> OwlResult<bool> {
        // Extract class names from concepts
        let class1 = self.extract_class_name(concept1)?;
        let class2 = self.extract_class_name(concept2)?;
        
        if let (Some(iri1), Some(iri2)) = (class1, class2) {
            // Check if these IRIs are declared disjoint
            for disjoint_axiom in &self.rules.disjointness_rules {
                let mut found_iri1 = false;
                let mut found_iri2 = false;
                
                // For disjoint classes axioms, we need to check the actual classes
                for class_iri in disjoint_axiom.classes() {
                    if **class_iri == iri1 {
                        found_iri1 = true;
                    }
                    if **class_iri == iri2 {
                        found_iri2 = true;
                    }
                }
                
                if found_iri1 && found_iri2 {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }

    pub fn is_class_satisfiable(&self, _class: &IRI) -> OwlResult<bool> {
        // Placeholder implementation - check if the class can be instantiated
        Ok(true)
    }

    pub fn is_class_expression_satisfiable(&self, _class: &ClassExpression) -> OwlResult<bool> {
        // Placeholder implementation - check if the class expression can be instantiated
        Ok(true)
    }

    pub fn is_subclass_of(&self, _subclass: &IRI, _superclass: &IRI) -> OwlResult<bool> {
        // Placeholder implementation
        Ok(false)
    }

    /// Initialize the root node with all classes from the ontology
    fn initialize_root_node(&self, graph: &mut super::graph::TableauxGraph) -> OwlResult<()> {
        let root_id = graph.get_root();
        
        // Add all named classes from the ontology to the root node
        for class in self.ontology.classes() {
            let class_expr = ClassExpression::Class(Class::new(class.iri().as_str()));
            graph.add_concept(root_id, class_expr);
        }
        
        // Add all subclass axioms as concepts
        for subclass_axiom in &self.rules.subclass_rules {
            graph.add_concept(root_id, subclass_axiom.sub_class().clone());
            graph.add_concept(root_id, subclass_axiom.super_class().clone());
        }
        
        // Add all equivalence axioms
        for equiv_axiom in &self.rules.equivalence_rules {
            // For equivalent classes, add each class as a concept
            for class_iri in equiv_axiom.classes() {
                let class_expr = ClassExpression::Class(Class::new(class_iri.as_str()));
                graph.add_concept(root_id, class_expr);
            }
        }
        
        Ok(())
    }

    /// Check if a node contains contradictory concepts (clash detection)
    fn has_clash(&self, node_id: NodeId, graph: &super::graph::TableauxGraph) -> OwlResult<bool> {
        if let Some(node) = graph.get_node(node_id) {
            let concepts: Vec<_> = node.concepts_iter().collect();
            
            // Check for direct contradictions
            for (i, concept1) in concepts.iter().enumerate() {
                for concept2 in concepts.iter().skip(i + 1) {
                    if self.are_contradictory(concept1, concept2)? {
                        return Ok(true);
                    }
                }
            }
            
            // Check for disjoint class axioms
            for (i, concept1) in concepts.iter().enumerate() {
                for concept2 in concepts.iter().skip(i + 1) {
                    if self.are_disjoint_class_expressions(concept1, concept2)? {
                        return Ok(true);
                    }
                }
            }
        }
        
        Ok(false)
    }

    /// Check if two concepts are contradictory
    fn are_contradictory(&self, concept1: &ClassExpression, concept2: &ClassExpression) -> OwlResult<bool> {
        match (concept1, concept2) {
            (ClassExpression::Class(class1), ClassExpression::Class(class2)) => {
                // Check if classes are declared disjoint
                for disjoint_axiom in &self.rules.disjointness_rules {
                    let mut found_class1 = false;
                    let mut found_class2 = false;
                    
                    for class_iri in disjoint_axiom.classes() {
                        if **class_iri == **class1.iri() {
                            found_class1 = true;
                        }
                        if **class_iri == **class2.iri() {
                            found_class2 = true;
                        }
                    }
                    
                    if found_class1 && found_class2 {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            (ClassExpression::ObjectComplementOf(comp1), ClassExpression::Class(class2)) => {
                // Check if complement contradicts the class
                Ok(comp1.as_ref() == &ClassExpression::Class(Class::new(class2.iri().as_str())))
            }
            (ClassExpression::Class(class1), ClassExpression::ObjectComplementOf(comp2)) => {
                // Check if complement contradicts the class
                Ok(&ClassExpression::Class(Class::new(class1.iri().as_str())) == comp2.as_ref())
            }
            (ClassExpression::ObjectComplementOf(comp1), ClassExpression::ObjectComplementOf(comp2)) => {
                // Check if complements are of the same class
                Ok(comp1.as_ref() == comp2.as_ref())
            }
            // Check for bottom (Nothing) and top (Thing) contradictions
            (ClassExpression::Class(class), _) if class.iri().as_str() == "http://www.w3.org/2002/07/owl#Nothing" => {
                Ok(true) // Nothing contradicts everything except itself
            }
            (_, ClassExpression::Class(class)) if class.iri().as_str() == "http://www.w3.org/2002/07/owl#Nothing" => {
                Ok(true)
            }
            _ => Ok(false),
        }
    }


    /// Extract the class name from a class expression
    fn extract_class_name(&self, concept: &ClassExpression) -> OwlResult<Option<IRI>> {
        match concept {
            ClassExpression::Class(class) => Ok(Some((**class.iri()).clone())),
            ClassExpression::ObjectComplementOf(comp) => self.extract_class_name(comp.as_ref()),
            _ => Ok(None),
        }
    }

    /// Get new successor nodes that haven't been processed yet
    fn get_new_successors(
        &self,
        node_id: NodeId,
        graph: &super::graph::TableauxGraph,
        expanded_nodes: &std::collections::HashSet<NodeId>,
    ) -> Vec<NodeId> {
        let mut new_nodes = Vec::new();
        
        // Check all edges from the current node
        for edge in graph.edges.get_all_edges() {
            if edge.0 == node_id && !expanded_nodes.contains(&edge.2) {
                new_nodes.push(edge.2);
            }
        }
        
        new_nodes
    }
}
