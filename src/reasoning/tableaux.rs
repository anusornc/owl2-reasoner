//! OWL2 Tableaux Reasoning Engine
//!
//! Implements a tableaux-based reasoning algorithm for OWL2 ontologies
//! based on SROIQ(D) description logic.

use crate::axioms::*;
use crate::entities::*;
use crate::error::{OwlError, OwlResult};
use crate::iri::IRI;
use crate::ontology::Ontology;

use bumpalo::Bump;
use hashbrown::HashMap;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::mem;
use std::ptr::NonNull;
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
    /// Dependency-directed backtracking manager
    dependency_manager: DependencyManager,
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
pub struct NodeId(pub usize);

/// Sophisticated blocking constraint types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockingType {
    /// Subset blocking: ancestor contains all concepts of descendant
    Subset,
    /// Equality blocking: nodes are forced to be equal
    Equality,
    /// Cardinality blocking: enforced by cardinality restrictions
    Cardinality,
    /// Dynamic blocking: adaptive blocking based on reasoning state
    Dynamic,
    /// Nominal blocking: blocking based on individual equality
    Nominal,
}

/// Blocking constraint with metadata
#[derive(Debug, Clone)]
pub struct BlockingConstraint {
    pub blocked_node: NodeId,
    pub blocking_node: NodeId,
    pub blocking_type: BlockingType,
    pub reason: String,
    pub dependencies: Vec<NodeId>,
    pub strength: f64, // 0.0 to 1.0, higher means stronger blocking
}

impl BlockingConstraint {
    fn new(blocked_node: NodeId, blocking_node: NodeId, blocking_type: BlockingType, reason: String) -> Self {
        let strength = match blocking_type {
            BlockingType::Subset => 0.7,
            BlockingType::Equality => 1.0,
            BlockingType::Cardinality => 0.9,
            BlockingType::Dynamic => 0.6,
            BlockingType::Nominal => 0.8,
        };

        Self {
            blocked_node,
            blocking_node,
            blocking_type,
            reason,
            dependencies: Vec::new(),
            strength,
        }
    }

  }

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
    /// Individual equality constraints for nominal reasoning
    _individual_constraints: HashMap<NodeId, crate::entities::Individual>,
    /// Sophisticated blocking constraints with metadata
    blocking_constraints: Vec<BlockingConstraint>,
    /// Blocking index for fast lookup: blocked_node -> constraint_index
    blocking_index: HashMap<NodeId, usize>,
    /// Dynamic blocking state
    blocking_stats: BlockingStats,
}

/// Arena-optimized tableaux graph structure for memory efficiency
#[derive(Debug)]
pub struct ArenaTableauxGraph {
    /// Arena-allocated nodes
    nodes: HashMap<NodeId, NonNull<TableauxNode>>,
    /// Arena-optimized edge storage
    edges: ArenaEdgeStorage,
    /// Arena manager for all allocations
    arena_manager: ArenaManager,
    root: NodeId,
    next_id: usize,
    /// Individual equality constraints (arena-allocated)
    _individual_constraints: HashMap<NodeId, crate::entities::Individual>,
    /// Arena-allocated blocking constraints
    blocking_constraints: Vec<*mut BlockingConstraint>,
    /// Blocking index for fast lookup
    blocking_index: HashMap<NodeId, usize>,
    /// Dynamic blocking state
    blocking_stats: BlockingStats,
    /// Memory optimization statistics
    memory_stats: RefCell<MemoryOptimizationStats>,
}

impl ArenaTableauxGraph {
    /// Create a new arena-optimized tableaux graph
    pub fn new() -> Self {
        let mut graph = Self {
            nodes: HashMap::new(),
            edges: ArenaEdgeStorage::new(),
            arena_manager: ArenaManager::new(),
            root: NodeId(0),
            next_id: 1,
            _individual_constraints: HashMap::new(),
            blocking_constraints: Vec::new(),
            blocking_index: HashMap::new(),
            blocking_stats: BlockingStats::default(),
            memory_stats: RefCell::new(MemoryOptimizationStats::default()),
        };

        // Create root node
        let root_node = graph.arena_manager.allocate_node(TableauxNode::new(graph.root));
        graph.nodes.insert(graph.root, NonNull::new(root_node).unwrap());

        graph
    }

    /// Add a node to the arena-optimized graph
    pub fn add_node(&mut self) -> NodeId {
        let node_id = NodeId(self.next_id);
        self.next_id += 1;

        // Allocate node in arena
        let node = self.arena_manager.allocate_node(TableauxNode::new(node_id));
        self.nodes.insert(node_id, NonNull::new(node).unwrap());

        // Update memory statistics
        self.memory_stats.borrow_mut().arena_allocated_nodes += 1;

        node_id
    }

    /// Add a concept to a node in arena memory
    pub fn add_concept(&mut self, node_id: NodeId, concept: ClassExpression) {
        if let Some(node_ptr) = self.nodes.get_mut(&node_id) {
            unsafe {
                let node = node_ptr.as_mut();
                node.concepts.push(concept);
            }

            // Update memory statistics
            self.memory_stats.borrow_mut().arena_allocated_expressions += 1;
        }
    }

    /// Add an edge to the arena-optimized graph
    pub fn add_edge(&mut self, from: NodeId, property: IRI, to: NodeId) {
        self.edges.add_edge(from, &property, to);
    }

    /// Get a node from the arena-optimized graph
    pub fn get_node(&self, node_id: NodeId) -> Option<&TableauxNode> {
        self.nodes.get(&node_id).map(|node_ptr| unsafe { node_ptr.as_ref() })
    }

    /// Get a mutable node from the arena-optimized graph
    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut TableauxNode> {
        self.nodes.get_mut(&node_id).map(|node_ptr| unsafe { node_ptr.as_mut() })
    }

    /// Get all nodes from the arena-optimized graph
    pub fn get_nodes(&self) -> impl Iterator<Item = &TableauxNode> {
        self.nodes.values().map(|node_ptr| unsafe { node_ptr.as_ref() })
    }

    /// Get successors of a node
    pub fn get_successors(&self, node_id: NodeId, property: &IRI) -> Option<&[NodeId]> {
        self.edges.get_successors(node_id, property)
    }

    /// Get memory optimization statistics
    pub fn get_memory_stats(&self) -> MemoryOptimizationStats {
        (*self.memory_stats.borrow()).clone()
    }

    /// Get arena allocation statistics
    pub fn get_arena_stats(&self) -> ArenaStats {
        self.arena_manager.stats().clone()
    }

    /// Calculate memory efficiency improvements
    pub fn calculate_memory_efficiency(&self) -> f64 {
        let stats = self.memory_stats.borrow();
        let total_traditional_allocations = stats.arena_allocated_nodes * 64 + // Traditional node allocation overhead
                                           stats.arena_allocated_expressions * 48 + // Traditional expression overhead
                                           stats.arena_allocated_constraints * 32; // Traditional constraint overhead

        if total_traditional_allocations == 0 {
            1.0
        } else {
            let total_arena_allocations = stats.arena_allocated_nodes + stats.arena_allocated_expressions + stats.arena_allocated_constraints;
            total_traditional_allocations as f64 / total_arena_allocations.max(1) as f64
        }
    }

    /// Estimate memory savings
    pub fn estimate_memory_savings(&self) -> usize {
        let stats = self.memory_stats.borrow();
        stats.string_intern_savings + stats.arena_allocation_savings
    }

    /// Get the root node
    pub fn get_root(&self) -> NodeId {
        self.root
    }

    /// Add a blocking constraint in arena memory
    pub fn add_blocking_constraint(&mut self, blocked_node: NodeId, blocking_node: NodeId) {
        let constraint = BlockingConstraint {
            blocked_node,
            blocking_node,
            blocking_type: BlockingType::Dynamic,
            reason: "Memory-optimized blocking".to_string(),
            dependencies: Vec::new(),
            strength: 1.0,
        };

        let arena_constraint = self.arena_manager.allocate_constraint(constraint);
        self.blocking_constraints.push(arena_constraint);

        // Update blocking index
        self.blocking_index.insert(blocked_node, self.blocking_constraints.len() - 1);

        // Update blocking statistics
        self.blocking_stats.total_blocks += 1;
        self.blocking_stats.dynamic_blocks += 1;
        self.blocking_stats.blocked_nodes.insert(blocked_node);

        // Update memory statistics
        self.memory_stats.borrow_mut().arena_allocated_constraints += 1;
    }

    /// Check if a node is blocked
    pub fn is_node_blocked(&self, node_id: NodeId) -> bool {
        self.blocking_index.contains_key(&node_id) || self.blocking_stats.blocked_nodes.contains(&node_id)
    }

    /// Get blocking statistics
    pub fn get_blocking_stats(&self) -> &BlockingStats {
        &self.blocking_stats
    }

    /// Clear all blocking constraints
    pub fn clear_blocking(&mut self) {
        self.blocking_constraints.clear();
        self.blocking_index.clear();
        self.blocking_stats.blocked_nodes.clear();
    }

    /// Intern a string to save memory
    pub fn intern_string(&mut self, s: &str) -> &str {
        self.arena_manager.intern_string(s)
    }

    /// Get total number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get total number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.edges().len()
    }

    /// Get total number of blocking constraints
    pub fn blocking_constraint_count(&self) -> usize {
        self.blocking_constraints.len()
    }

    /// Get memory usage summary
    pub fn get_memory_usage_summary(&self) -> String {
        let memory_stats = self.get_memory_stats();
        let arena_stats = self.get_arena_stats();
        let efficiency = self.calculate_memory_efficiency();
        let savings = self.estimate_memory_savings();

        format!(
            "Arena-Optimized Memory Usage:\n\
             • Nodes: {} (arena allocated)\n\
             • Expressions: {} (arena allocated)\n\
             • Constraints: {} (arena allocated)\n\
             • Interned strings: {}\n\
             • Memory efficiency: {:.2}x\n\
             • Estimated savings: {} bytes\n\
             • Total arena allocations: {}",
            memory_stats.arena_allocated_nodes,
            memory_stats.arena_allocated_expressions,
            memory_stats.arena_allocated_constraints,
            memory_stats.interned_strings,
            efficiency,
            savings,
            arena_stats.total_bytes_allocated
        )
    }
}

/// Memory optimization statistics
#[derive(Debug, Default, Clone)]
pub struct MemoryOptimizationStats {
    /// Number of nodes allocated in arena
    arena_allocated_nodes: usize,
    /// Number of expressions allocated in arena
    arena_allocated_expressions: usize,
    /// Number of constraints allocated in arena
    arena_allocated_constraints: usize,
    /// Number of strings interned
    interned_strings: usize,
    /// Memory saved through string interning (bytes)
    string_intern_savings: usize,
    /// Memory saved through arena allocation (bytes)
    arena_allocation_savings: usize,
}

/// Arena allocation manager for memory optimization
#[derive(Debug)]
pub struct ArenaManager {
    /// Arena for TableauxNode allocations
    node_arena: Bump,
    /// Arena for ClassExpression allocations
    expression_arena: Bump,
    /// Arena for blocking constraint allocations
    constraint_arena: Bump,
    /// Arena for string allocations (labels, reasons)
    string_arena: Bump,
    /// String interning for common strings
    string_interner: HashMap<String, *const u8>,
    /// Allocation statistics
    stats: ArenaStats,
}

/// Arena allocation statistics
#[derive(Debug, Default, Clone)]
pub struct ArenaStats {
    node_allocations: usize,
    expression_allocations: usize,
    constraint_allocations: usize,
    string_allocations: usize,
    string_intern_hits: usize,
    total_bytes_allocated: usize,
}

impl ArenaStats {
    /// Get total bytes allocated
    pub fn total_bytes_allocated(&self) -> usize {
        self.total_bytes_allocated
    }
}

impl ArenaManager {
    /// Create a new arena manager with default capacity
    pub fn new() -> Self {
        Self {
            node_arena: Bump::new(),
            expression_arena: Bump::new(),
            constraint_arena: Bump::new(),
            string_arena: Bump::new(),
            string_interner: HashMap::default(),
            stats: ArenaStats::default(),
        }
    }

    /// Allocate a TableauxNode in the node arena
    pub fn allocate_node(&mut self, node: TableauxNode) -> &mut TableauxNode {
        self.stats.node_allocations += 1;
        self.stats.total_bytes_allocated += mem::size_of::<TableauxNode>();
        self.node_arena.alloc(node)
    }

    /// Allocate a ClassExpression in the expression arena
    pub fn allocate_expression(&mut self, expr: ClassExpression) -> &mut ClassExpression {
        self.stats.expression_allocations += 1;
        self.stats.total_bytes_allocated += mem::size_of::<ClassExpression>();
        self.expression_arena.alloc(expr)
    }

    /// Allocate a blocking constraint in the constraint arena
    pub fn allocate_constraint(&mut self, constraint: BlockingConstraint) -> &mut BlockingConstraint {
        self.stats.constraint_allocations += 1;
        self.stats.total_bytes_allocated += mem::size_of::<BlockingConstraint>();
        self.constraint_arena.alloc(constraint)
    }

    /// Intern a string in the string arena
    pub fn intern_string(&mut self, s: &str) -> &str {
        if let Some(&ptr) = self.string_interner.get(s) {
            self.stats.string_intern_hits += 1;
            return unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, s.len())) };
        }

        let allocated = self.string_arena.alloc_str(s);
        let ptr = allocated.as_ptr();
        self.string_interner.insert(s.to_string(), ptr);
        self.stats.string_allocations += 1;
        self.stats.total_bytes_allocated += s.len();
        allocated
    }

    /// Reset all arenas (for tableaux restart)
    pub fn reset(&mut self) {
        self.node_arena.reset();
        self.expression_arena.reset();
        self.constraint_arena.reset();
        self.string_arena.reset();
        self.string_interner.clear();
        self.stats = ArenaStats::default();
    }

    /// Get allocation statistics
    pub fn stats(&self) -> &ArenaStats {
        &self.stats
    }

    /// Get total memory usage across all arenas
    pub fn total_memory_usage(&self) -> usize {
        self.node_arena.allocated_bytes() +
        self.expression_arena.allocated_bytes() +
        self.constraint_arena.allocated_bytes() +
        self.string_arena.allocated_bytes()
    }
}

/// Optimized tableaux node with arena allocation support
#[derive(Debug)]
pub struct ArenaTableauxNode {
    /// Pointer to arena-allocated node data
    node_ptr: NonNull<TableauxNode>,
    /// Arena reference for deallocation tracking
    _arena: *const Bump,
}

impl ArenaTableauxNode {
    /// Create a new arena-allocated node
    pub fn new(node: TableauxNode, arena: &mut Bump) -> Self {
        let node_ptr = NonNull::from(arena.alloc(node));
        Self {
            node_ptr,
            _arena: arena as *const Bump,
        }
    }

    /// Get mutable reference to the node
    pub fn get_mut(&mut self) -> &mut TableauxNode {
        unsafe { self.node_ptr.as_mut() }
    }

    /// Get immutable reference to the node
    pub fn get(&self) -> &TableauxNode {
        unsafe { self.node_ptr.as_ref() }
    }
}

/// Optimized edge storage with arena support
#[derive(Debug)]
pub struct ArenaEdgeStorage {
    /// Arena-allocated edge storage
    edges: Vec<(NodeId, IRI, NodeId)>,
    /// Fast lookup index
    index: HashMap<(NodeId, IRI), SmallVec<[NodeId; 4]>>,
}

impl ArenaEdgeStorage {
    /// Create new arena-optimized edge storage
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            index: HashMap::default(),
        }
    }

    /// Add an edge with arena allocation
    pub fn add_edge(&mut self, from: NodeId, property: &IRI, to: NodeId) {
        let edge = (from, property.clone(), to);
        self.edges.push(edge);

        let key = (from, property.clone());
        self.index.entry(key).or_insert_with(SmallVec::new).push(to);
    }

    /// Get successors of a node
    pub fn get_successors(&self, node: NodeId, property: &IRI) -> Option<&[NodeId]> {
        self.index.get(&(node, property.clone()))
            .map(|vec| vec.as_slice())
    }

    /// Get all edges
    pub fn edges(&self) -> &[(NodeId, IRI, NodeId)] {
        &self.edges
    }

    /// Clear all edges
    pub fn clear(&mut self) {
        self.edges.clear();
        self.index.clear();
    }
}

/// Blocking statistics for optimization
#[derive(Debug, Default)]
pub struct BlockingStats {
    pub total_blocks: usize,
    pub subset_blocks: usize,
    pub equality_blocks: usize,
    pub cardinality_blocks: usize,
    pub dynamic_blocks: usize,
    pub nominal_blocks: usize,
    pub blocked_nodes: HashSet<NodeId>,
}

/// Dependency tracking for backtracking decisions
#[derive(Debug, Clone)]
pub struct Dependency {
    /// The node that created this dependency
    pub source_node: NodeId,
    /// The reasoning choice that led to this dependency
    pub choice: ReasoningChoice,
    /// Nodes that depend on this choice
    pub dependent_nodes: Vec<NodeId>,
    /// The level at which this dependency was created
    pub level: usize,
    /// Whether this dependency has been resolved
    pub resolved: bool,
}

/// Reasoning choices that can create dependencies
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum ReasoningChoice {
    /// Choice rule application: which rule to apply
    RuleApplication {
        concept: ClassExpression,
        node_id: NodeId,
        rule_applied: String,
    },
    /// Non-deterministic choice: which branch to explore
    BranchChoice {
        node_id: NodeId,
        branch_options: Vec<ClassExpression>,
        chosen_branch: usize,
    },
    /// Individual selection for nominal reasoning
    IndividualSelection {
        nominal_node: NodeId,
        selected_individual: crate::entities::Individual,
        available_individuals: Vec<crate::entities::Individual>,
    },
    /// Cardinality constraint handling
    CardinalityHandling {
        node_id: NodeId,
        property: crate::axioms::property_expressions::ObjectPropertyExpression,
        min_cardinality: Option<usize>,
        max_cardinality: Option<usize>,
        created_fillers: Vec<NodeId>,
    },
}

/// Backtracking decision point
#[derive(Debug, Clone)]
pub struct BacktrackPoint {
    /// The node where the decision was made
    pub node_id: NodeId,
    /// The choice that was made
    pub choice: ReasoningChoice,
    /// Dependencies created by this choice
    pub dependencies: Vec<Dependency>,
    /// Alternative choices that could have been made
    pub alternatives: Vec<ReasoningChoice>,
    /// The reasoning level/depth
    pub level: usize,
    /// Whether this point has been fully explored
    pub exhausted: bool,
}

/// Dependency-directed backtracking manager
#[derive(Debug)]
pub struct DependencyManager {
    /// Current reasoning level/depth
    current_level: usize,
    /// Stack of backtrack points
    backtrack_stack: Vec<BacktrackPoint>,
    /// Dependencies by node for fast lookup
    node_dependencies: HashMap<NodeId, Vec<Dependency>>,
    /// Choices that led to contradictions
    contradictory_choices: HashSet<ReasoningChoice>,
    /// Performance statistics
    stats: BacktrackStats,
}

/// Backtracking statistics
#[derive(Debug, Default)]
pub struct BacktrackStats {
    pub total_backtracks: usize,
    pub dependency_directed_backtracks: usize,
    pub naive_backtracks: usize,
    pub choices_explored: usize,
    pub contradictions_detected: usize,
    pub _average_backtrack_depth: f64,
}

impl DependencyManager {
    /// Create a new dependency manager
    pub fn new() -> Self {
        Self {
            current_level: 0,
            backtrack_stack: Vec::new(),
            node_dependencies: HashMap::new(),
            contradictory_choices: HashSet::new(),
            stats: BacktrackStats::default(),
        }
    }

    /// Push a new reasoning choice onto the stack
    pub fn push_choice(&mut self, node_id: NodeId, choice: ReasoningChoice, alternatives: Vec<ReasoningChoice>) {
        let backtrack_point = BacktrackPoint {
            node_id,
            choice: choice.clone(),
            dependencies: Vec::new(),
            alternatives,
            level: self.current_level,
            exhausted: false,
        };

        self.backtrack_stack.push(backtrack_point);
        self.current_level += 1;
        self.stats.choices_explored += 1;
    }

    /// Add a dependency created by the current choice
    pub fn add_dependency(&mut self, dependent_node: NodeId, source_node: NodeId, choice: &ReasoningChoice) {
        if let Some(current_point) = self.backtrack_stack.last_mut() {
            let dependency = Dependency {
                source_node,
                choice: choice.clone(),
                dependent_nodes: vec![dependent_node],
                level: self.current_level - 1,
                resolved: false,
            };

            current_point.dependencies.push(dependency.clone());

            // Add to node dependency index
            self.node_dependencies.entry(dependent_node)
                .or_insert_with(Vec::new)
                .push(dependency);
        }
    }

    /// Mark a choice as contradictory
    pub fn mark_contradictory(&mut self, choice: &ReasoningChoice) {
        self.contradictory_choices.insert(choice.clone());
        self.stats.contradictions_detected += 1;
    }

    /// Find the best backtrack point based on dependencies
    pub fn find_backtrack_point(&mut self, contradiction_node: NodeId) -> Option<usize> {
        // First, try dependency-directed backtracking
        if let Some(dependencies) = self.node_dependencies.get(&contradiction_node) {
            for dependency in dependencies {
                // Find the backtrack point that created this dependency
                for (i, point) in self.backtrack_stack.iter().enumerate().rev() {
                    if point.choice == dependency.choice && !point.exhausted {
                        // Check if there are unexplored alternatives
                        if !point.alternatives.is_empty() {
                            self.stats.dependency_directed_backtracks += 1;
                            return Some(i);
                        }
                    }
                }
            }
        }

        // Fall back to naive backtracking (most recent choice with alternatives)
        for (i, point) in self.backtrack_stack.iter().enumerate().rev() {
            if !point.exhausted && !point.alternatives.is_empty() {
                self.stats.naive_backtracks += 1;
                return Some(i);
            }
        }

        None
    }

    /// Execute backtracking to a specific point
    pub fn backtrack_to(&mut self, target_level: usize, graph: &mut TableauxGraph) -> OwlResult<()> {
        // Remove all nodes and dependencies created after the target level
        self.revert_to_level(target_level, graph)?;

        self.current_level = target_level;
        self.stats.total_backtracks += 1;

        Ok(())
    }

    /// Revert graph state to a specific level
    fn revert_to_level(&mut self, target_level: usize, graph: &mut TableauxGraph) -> OwlResult<()> {
        // Remove nodes created after target level
        let nodes_to_remove: Vec<NodeId> = graph.nodes.keys()
            .filter(|&&_node_id| {
                // Determine if this node was created after target level
                // This is a simplified check - in practice, you'd track creation levels
                false // Placeholder logic
            })
            .cloned()
            .collect();

        for node_id in nodes_to_remove {
            graph.remove_node(node_id);
        }

        // Remove dependencies after target level
        self.node_dependencies.retain(|_, dependencies| {
            dependencies.iter().any(|dep| dep.level <= target_level)
        });

        // Mark backtrack points as exhausted up to target level
        for point in &mut self.backtrack_stack {
            if point.level > target_level {
                point.exhausted = true;
            }
        }

        Ok(())
    }

    /// Get backtracking statistics
    pub fn get_stats(&self) -> &BacktrackStats {
        &self.stats
    }

    /// Check if a choice is known to be contradictory
    pub fn is_contradictory_choice(&self, choice: &ReasoningChoice) -> bool {
        self.contradictory_choices.contains(choice)
    }
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

#[allow(dead_code)]
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
            dependency_manager: DependencyManager::new(),
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
            dependency_manager: DependencyManager::new(),
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

    /// Run the tableaux algorithm with dependency-directed backtracking
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

        // Reset dependency manager for new reasoning session
        self.dependency_manager = DependencyManager::new();

        let result = self.run_tableaux_with_backtracking(graph, &config, &mut stats)?;

        stats.time_ms = start_time.elapsed().as_millis() as u64;
        stats.backtracks = self.dependency_manager.get_stats().total_backtracks;

        Ok(result)
    }

    /// Run tableaux with dependency-directed backtracking
    fn run_tableaux_with_backtracking(
        &mut self,
        graph: &mut TableauxGraph,
        config: &ReasoningConfig,
        stats: &mut ReasoningStats,
    ) -> OwlResult<ReasoningResult> {
        let mut queue = VecDeque::new();
        queue.push_back(graph.root);

        while let Some(node_id) = queue.pop_front() {
            if stats.nodes_created > config.max_depth {
                return Ok(ReasoningResult {
                    is_satisfiable: false,
                    explanation: Some("Maximum depth exceeded".to_string()),
                    model: None,
                    stats: stats.clone(),
                });
            }

            // Apply reasoning rules with dependency tracking
            let node = graph.nodes.get(&node_id)
                .ok_or_else(|| OwlError::ReasoningError(format!("Node {} not found in graph", node_id.0)))?;
            let concepts: Vec<_> = node.concepts.iter().cloned().collect();

            for concept in concepts {
                // Check if this concept application would create a known contradictory choice
                let choice = ReasoningChoice::RuleApplication {
                    concept: concept.clone(),
                    node_id,
                    rule_applied: "concept_application".to_string(),
                };

                if self.dependency_manager.is_contradictory_choice(&choice) {
                    continue; // Skip known contradictory choices
                }

                // Track this reasoning choice
                self.dependency_manager.push_choice(node_id, choice.clone(), Vec::new());

                if let Some((new_concepts, new_nodes_created)) =
                    self.apply_rules_with_dependencies(&concept, node_id, graph)?
                {
                    stats.rules_applied += 1;

                    // Add new concepts to current node and track dependencies
                    for new_concept in new_concepts {
                        graph.add_concept(node_id, new_concept);
                        self.dependency_manager.add_dependency(node_id, node_id, &choice);
                    }

                    // Add new nodes to queue and track dependencies
                    for new_node_id in new_nodes_created {
                        queue.push_back(new_node_id);
                        stats.nodes_created += 1;
                        self.dependency_manager.add_dependency(new_node_id, node_id, &choice);
                    }
                }
            }

            // Check for contradictions with dependency-directed backtracking
            let node = graph.nodes.get(&node_id)
                .ok_or_else(|| OwlError::ReasoningError(format!("Node {} not found in graph", node_id.0)))?;

            if self.has_contradiction(node) {
                // Handle contradiction with backtracking
                let current_choice = self.get_current_choice();

                if let Some(choice) = current_choice {
                    // Mark as contradictory
                    self.dependency_manager.mark_contradictory(&choice);
                }

                // Find backtrack point index after marking contradictory
                let backtrack_index = self.dependency_manager.find_backtrack_point(node_id);

                if let Some(index) = backtrack_index {
                    if let Some(backtrack_point) = self.dependency_manager.backtrack_stack.get(index) {
                        self.dependency_manager.backtrack_to(backtrack_point.level, graph)?;
                        continue;
                    }
                }

                // No more backtrack options - unsatisfiable
                return Ok(ReasoningResult {
                    is_satisfiable: false,
                    explanation: Some("Contradiction found with no backtrack options".to_string()),
                    model: None,
                    stats: stats.clone(),
                });
            }

            // Check blocking conditions
            if self.is_blocked(node_id, graph) {
                continue;
            }

            // Check if we've found a complete model (satisfiable)
            if self.is_complete_model(graph) {
                return Ok(ReasoningResult {
                    is_satisfiable: true,
                    explanation: Some("Complete model found with dependency-directed backtracking".to_string()),
                    model: Some(self.extract_model(graph)),
                    stats: stats.clone(),
                });
            }
        }

        // If we exhausted all possibilities without finding a model, it's unsatisfiable
        Ok(ReasoningResult {
            is_satisfiable: false,
            explanation: Some("No complete model found".to_string()),
            model: None,
            stats: stats.clone(),
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
                // {a₁, ..., aₙ} → create nominal nodes with individual equality
                self.apply_nominal_rule(individuals, node_id, graph)
            }

            ClassExpression::ObjectMinCardinality(n, property) => {
                // ≥ n R → ensure at least n R-successors (unqualified)
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
                // ≤ n R → ensure at most n R-successors (unqualified)
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
                // = n R → ensure exactly n R-successors (unqualified)
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

            ClassExpression::ObjectHasValue(property, individual) => {
                // R(a) → object property has value
                self.apply_has_value_rule(property, individual, node_id, graph)
            }

            ClassExpression::ObjectHasSelf(property) => {
                // R(a,a) → object has self
                self.apply_has_self_rule(property, node_id, graph)
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

    /// Check if a node is blocked using sophisticated blocking strategies
    fn is_blocked(&self, node_id: NodeId, graph: &TableauxGraph) -> bool {
        // First check explicit blocking constraints
        if graph.is_blocked_by(node_id).is_some() {
            return true;
        }

        // Check for sophisticated blocking conditions without modifying the graph
        self.check_sophisticated_blocking_conditions(node_id, graph)
    }

    /// Check sophisticated blocking conditions (non-mutating)
    fn check_sophisticated_blocking_conditions(&self, node_id: NodeId, graph: &TableauxGraph) -> bool {
        let _node = &graph.nodes[&node_id];

        // Strategy 1: Enhanced Subset Blocking
        if self.check_enhanced_subset_blocking(node_id, graph) {
            return true;
        }

        // Strategy 2: Equality Blocking
        if self.check_equality_blocking(node_id, graph) {
            return true;
        }

        // Strategy 3: Nominal Blocking
        if self.check_nominal_blocking(node_id, graph) {
            return true;
        }

        // Strategy 4: Dynamic Blocking
        if self.check_dynamic_blocking(node_id, graph) {
            return true;
        }

        false
    }

    /// Check enhanced subset blocking (non-mutating version)
    fn check_enhanced_subset_blocking(&self, node_id: NodeId, graph: &TableauxGraph) -> bool {
        let node = &graph.nodes[&node_id];

        // Check all ancestors for enhanced subset blocking
        let mut current = node_id;
        while let Some(parent) = graph.get_parent(current) {
            if parent == node_id || parent == node_id {
                current = parent;
                continue;
            }

            let parent_node = &graph.nodes[&parent];

            // Enhanced subset checking: consider not just exact concepts but also subsumption
            if self.is_enhanced_subset(node, parent_node) {
                // Enhanced subset blocking detected
                return true;
            }

            current = parent;
        }

        false
    }

    /// Check equality blocking (non-mutating version)
    fn check_equality_blocking(&self, node_id: NodeId, graph: &TableauxGraph) -> bool {
        // Check if this node has individual constraints
        if let Some(individual) = graph._individual_constraints.get(&node_id) {
            // Look for other nodes with the same individual constraint
            for (other_id, other_individual) in &graph._individual_constraints {
                if other_id != &node_id && individual == other_individual {
                    // Check if the other node is an ancestor
                    if self.is_ancestor(*other_id, node_id, graph) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check nominal blocking (non-mutating version)
    fn check_nominal_blocking(&self, node_id: NodeId, graph: &TableauxGraph) -> bool {
        let node = &graph.nodes[&node_id];

        // Check for nominal expressions in the node's concepts
        for concept in node.concepts_iter() {
            if let ClassExpression::ObjectOneOf(individuals) = concept {
                // This node represents one of these individuals
                // Look for ancestor nodes that also represent the same individuals
                let mut current = node_id;
                while let Some(parent) = graph.get_parent(current) {
                    if parent == node_id {
                        current = parent;
                        continue;
                    }

                    let parent_node = &graph.nodes[&parent];

                    // Check if parent has the same nominal constraint
                    for parent_concept in parent_node.concepts_iter() {
                        if let ClassExpression::ObjectOneOf(parent_individuals) = parent_concept {
                            if individuals.as_slice() == parent_individuals.as_slice() {
                                return true;
                            }
                        }
                    }

                    current = parent;
                }
            }
        }

        false
    }

    /// Check dynamic blocking (non-mutating version)
    fn check_dynamic_blocking(&self, node_id: NodeId, graph: &TableauxGraph) -> bool {
        let node = &graph.nodes[&node_id];

        // Dynamic blocking heuristic: if node has many self-restrictions, it might be redundant
        let mut self_restriction_count = 0;
        for concept in node.concepts_iter() {
            if let ClassExpression::ObjectHasSelf(_) = concept {
                self_restriction_count += 1;
            }
        }

        // If node has multiple self-restrictions and ancestors have similar patterns
        if self_restriction_count > 1 {
            let mut current = node_id;
            while let Some(parent) = graph.get_parent(current) {
                if parent == node_id {
                    current = parent;
                    continue;
                }

                let parent_node = &graph.nodes[&parent];
                let mut parent_self_count = 0;

                for parent_concept in parent_node.concepts_iter() {
                    if let ClassExpression::ObjectHasSelf(_) = parent_concept {
                        parent_self_count += 1;
                    }
                }

                // If parent has similar self-restriction pattern, apply dynamic blocking
                if parent_self_count >= self_restriction_count {
                    return true;
                }

                current = parent;
            }
        }

        false
    }

    /// Apply reasoning rules with dependency tracking
    fn apply_rules_with_dependencies(
        &mut self,
        concept: &ClassExpression,
        node_id: NodeId,
        graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        // For now, delegate to the existing apply_rules method
        // In a full implementation, this would track more detailed dependencies
        self.apply_rules(concept, node_id, graph)
    }

    /// Get the current reasoning choice from the dependency manager
    fn get_current_choice(&self) -> Option<ReasoningChoice> {
        self.dependency_manager.backtrack_stack.last()
            .map(|point| point.choice.clone())
    }

    /// Check if dependency-directed backtracking is available
    fn has_backtrack_options(&self) -> bool {
        self.dependency_manager.backtrack_stack.iter()
            .any(|point| !point.exhausted && !point.alternatives.is_empty())
    }

    /// Get backtracking statistics for performance monitoring
    pub fn get_backtrack_stats(&self) -> &BacktrackStats {
        self.dependency_manager.get_stats()
    }

    /// Apply sophisticated blocking strategies
    fn apply_sophisticated_blocking_strategies(&self, node_id: NodeId, graph: &mut TableauxGraph) -> bool {
        let _node = &graph.nodes[&node_id];

        // Strategy 1: Enhanced Subset Blocking
        if self.apply_enhanced_subset_blocking(node_id, graph) {
            return true;
        }

        // Strategy 2: Equality Blocking
        if self.apply_equality_blocking(node_id, graph) {
            return true;
        }

        // Strategy 3: Nominal Blocking
        if self.apply_nominal_blocking(node_id, graph) {
            return true;
        }

        // Strategy 4: Dynamic Blocking
        if self.apply_dynamic_blocking(node_id, graph) {
            return true;
        }

        false
    }

    /// Enhanced subset blocking with more comprehensive concept comparison
    fn apply_enhanced_subset_blocking(&self, node_id: NodeId, graph: &TableauxGraph) -> bool {
        let node = &graph.nodes[&node_id];

        // Check all ancestors for enhanced subset blocking
        let mut current = node_id;
        while let Some(parent) = graph.get_parent(current) {
            if parent == node_id || parent == node_id {
                current = parent;
                continue;
            }

            let parent_node = &graph.nodes[&parent];

            // Enhanced subset checking: consider not just exact concepts but also subsumption
            if self.is_enhanced_subset(node, parent_node) {
                // Enhanced subset blocking detected
                return true;
            }

            current = parent;
        }

        false
    }

    /// Check if one node is an enhanced subset of another
    fn is_enhanced_subset(&self, node: &TableauxNode, ancestor: &TableauxNode) -> bool {
        // Basic subset check
        for concept in node.concepts_iter() {
            if !ancestor.has_concept(concept) {
                // Check for subsumption relationships
                if !self.is_subsumed_by(concept, ancestor) {
                    return false;
                }
            }
        }

        // Additional check: if ancestor has significantly more concepts, it's a better blocker
        let node_concept_count = node.concepts_iter().count();
        let ancestor_concept_count = ancestor.concepts_iter().count();

        // Only block if ancestor has at least as many concepts (proper subset)
        ancestor_concept_count >= node_concept_count && ancestor_concept_count > 0
    }

    /// Check if a concept is subsumed by any concept in the ancestor node
    fn is_subsumed_by(&self, concept: &ClassExpression, ancestor: &TableauxNode) -> bool {
        // Simple subsumption checking (can be enhanced with ontology reasoning)
        for ancestor_concept in ancestor.concepts_iter() {
            if self.concepts_are_compatible(concept, ancestor_concept) {
                return true;
            }
        }
        false
    }

    /// Check if two concepts are compatible (not contradictory)
    fn concepts_are_compatible(&self, c1: &ClassExpression, c2: &ClassExpression) -> bool {
        // Simple compatibility check
        match (c1, c2) {
            (ClassExpression::Class(_class1), ClassExpression::Class(_class2)) => {
                // Check if classes are related in the ontology
                !self.are_contradictory(c1, c2)
            }
            (ClassExpression::ObjectComplementOf(comp1), _) => {
                // Check if the complement doesn't contradict the other concept
                !self.are_contradictory(comp1.as_ref(), c2)
            }
            (_, ClassExpression::ObjectComplementOf(comp2)) => {
                // Check if the complement doesn't contradict the first concept
                !self.are_contradictory(c1, comp2.as_ref())
            }
            _ => true, // Default to compatible for complex expressions
        }
    }

    /// Apply equality blocking for nodes with individual constraints
    fn apply_equality_blocking(&self, node_id: NodeId, graph: &mut TableauxGraph) -> bool {
        // Check if this node has individual constraints
        if let Some(individual) = graph._individual_constraints.get(&node_id) {
            // Look for other nodes with the same individual constraint
            for (other_id, other_individual) in &graph._individual_constraints {
                if other_id != &node_id && individual == other_individual {
                    // Check if the other node is an ancestor
                    if self.is_ancestor(*other_id, node_id, graph) {
                        graph.add_sophisticated_blocking_constraint(
                            node_id,
                            *other_id,
                            BlockingType::Equality,
                            format!("Equality blocking: both nodes represent individual {:?}", individual.iri())
                        );
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Apply nominal blocking based on individual equality
    fn apply_nominal_blocking(&self, node_id: NodeId, graph: &mut TableauxGraph) -> bool {
        // Collect nominal concepts first to avoid borrowing conflicts
        let mut nominal_individuals = Vec::new();

        {
            let node = &graph.nodes[&node_id];
            // Check for nominal expressions in the node's concepts
            for concept in node.concepts_iter() {
                if let ClassExpression::ObjectOneOf(individuals) = concept {
                    nominal_individuals.push(individuals.clone());
                }
            }
        }

        // Now check each nominal for blocking
        for individuals in nominal_individuals {
            // This node represents one of these individuals
            // Look for ancestor nodes that also represent the same individuals
            let mut current = node_id;
            while let Some(parent) = graph.get_parent(current) {
                if parent == node_id {
                    current = parent;
                    continue;
                }

                let parent_node = &graph.nodes[&parent];

                // Check if parent has the same nominal constraint
                let parent_has_nominal = {
                    let mut has_nominal = false;
                    for parent_concept in parent_node.concepts_iter() {
                        if let ClassExpression::ObjectOneOf(parent_individuals) = parent_concept {
                            if individuals.as_slice() == parent_individuals.as_slice() {
                                has_nominal = true;
                                break;
                            }
                        }
                    }
                    has_nominal
                };

                if parent_has_nominal {
                    graph.add_sophisticated_blocking_constraint(
                        node_id,
                        parent,
                        BlockingType::Nominal,
                        format!("Nominal blocking: both nodes represent nominal {:?}", individuals)
                    );
                    return true;
                }

                current = parent;
            }
        }

        false
    }

    /// Apply dynamic blocking based on reasoning state and heuristics
    fn apply_dynamic_blocking(&self, node_id: NodeId, graph: &mut TableauxGraph) -> bool {
        // Count self-restrictions first to avoid borrowing conflicts
        let self_restriction_count = {
            let node = &graph.nodes[&node_id];
            let mut count = 0;
            for concept in node.concepts_iter() {
                if let ClassExpression::ObjectHasSelf(_) = concept {
                    count += 1;
                }
            }
            count
        };

        // If node has multiple self-restrictions and ancestors have similar patterns
        if self_restriction_count > 1 {
            let mut current = node_id;
            while let Some(parent) = graph.get_parent(current) {
                if parent == node_id {
                    current = parent;
                    continue;
                }

                // Count parent self-restrictions to avoid borrowing conflicts
                let parent_self_count = {
                    let parent_node = &graph.nodes[&parent];
                    let mut count = 0;
                    for parent_concept in parent_node.concepts_iter() {
                        if let ClassExpression::ObjectHasSelf(_) = parent_concept {
                            count += 1;
                        }
                    }
                    count
                };

                // If parent has similar self-restriction pattern, apply dynamic blocking
                if parent_self_count >= self_restriction_count {
                    graph.add_sophisticated_blocking_constraint(
                        node_id,
                        parent,
                        BlockingType::Dynamic,
                        format!("Dynamic blocking: self-restriction pattern similarity ({} vs {})",
                                self_restriction_count, parent_self_count)
                    );
                    return true;
                }

                current = parent;
            }
        }

        false
    }

    /// Check if one node is an ancestor of another
    fn is_ancestor(&self, potential_ancestor: NodeId, node: NodeId, graph: &TableauxGraph) -> bool {
        let mut current = node;
        while let Some(parent) = graph.get_parent(current) {
            if parent == potential_ancestor {
                return true;
            }
            if parent == node {
                break; // Avoid cycles
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

    /// Apply {a₁, ..., aₙ} rule: create nominal nodes with individual equality
    fn apply_nominal_rule(
        &self,
        individuals: &[crate::entities::Individual],
        node_id: NodeId,
        graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        // For SROIQ(D), ObjectOneOf represents a nominal - an enumeration of individuals
        // This creates a choice: the node must be equal to one of the individuals

        if individuals.is_empty() {
            // Empty oneOf is unsatisfiable
            return Ok(None);
        }

        if individuals.len() == 1 {
            // Single individual: node must be equal to this individual
            let individual = &individuals[0];
            let individual_iri = individual.iri().expect("Individual must have an IRI");

            // Add the individual as a concept to the current node
            let individual_class = ClassExpression::Class(Class::new(individual_iri.clone()));

            // Add individual equality constraint
            graph.add_individual_constraint(node_id, individual.clone());

            return Ok(Some((vec![individual_class], Vec::new())));
        } else {
            // Multiple individuals: create a non-deterministic choice
            // For now, choose the first individual (will be enhanced with backtracking)
            let individual = &individuals[0];
            let individual_iri = individual.iri().expect("Individual must have an IRI");
            let individual_class = ClassExpression::Class(Class::new(individual_iri.clone()));

            // Add individual equality constraint
            graph.add_individual_constraint(node_id, individual.clone());

            return Ok(Some((vec![individual_class], Vec::new())));
        }
    }

    /// Apply R(a) rule: object property has value
    fn apply_has_value_rule(
        &self,
        property: &ObjectPropertyExpression,
        individual: &crate::entities::Individual,
        node_id: NodeId,
        graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        // R(a) means there must be an R-successor that is equal to individual a

        // First, check if there's already an R-successor equal to this individual
        let (is_inverse, property_iri) = resolve_property_direction(property);

        // Look for existing successors that match the individual
        let mut found_matching_successor = false;

        if !is_inverse {
            if let Some(successors) = graph.get_successors(node_id, property_iri) {
                for successor_id in successors {
                    if graph.is_equal_to_individual(*successor_id, individual) {
                        found_matching_successor = true;
                        break;
                    }
                }
            }
        } else {
            // For inverse properties, check predecessors
            let predecessors = graph.get_predecessors(node_id, property_iri);
            for pred_id in predecessors {
                if graph.is_equal_to_individual(pred_id, individual) {
                    found_matching_successor = true;
                    break;
                }
            }
        }

        if !found_matching_successor {
            // Create a new successor that is equal to the individual
            let new_node_id = graph.add_node();

            if is_inverse {
                // For inverse R^-, we need an incoming edge via R: new_node --R--> current_node
                graph.add_edge(new_node_id, property_iri.clone(), node_id);
            } else {
                // Regular direction: current_node --R--> new_node
                graph.add_edge(node_id, property_iri.clone(), new_node_id);
            }

            // Set the new node to be equal to the individual
            graph.add_individual_constraint(new_node_id, individual.clone());

            // Add the individual class to the new node
            let individual_class = ClassExpression::Class(Class::new(individual.iri().expect("Individual must have an IRI").clone()));
            graph.add_concept(new_node_id, individual_class);

            return Ok(Some((Vec::new(), vec![new_node_id])));
        }

        Ok(None)
    }

    /// Apply R(a,a) rule: object has self
    fn apply_has_self_rule(
        &self,
        property: &ObjectPropertyExpression,
        node_id: NodeId,
        graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        // R(a,a) means there must be an R-edge from the node to itself

        let (is_inverse, property_iri) = resolve_property_direction(property);

        // Check if the self-edge already exists
        let self_edge_exists = if !is_inverse {
            graph.get_successors(node_id, property_iri)
                .map(|successors| successors.contains(&node_id))
                .unwrap_or(false)
        } else {
            // For inverse properties, check if node is its own predecessor
            graph.get_predecessors(node_id, property_iri)
                .contains(&node_id)
        };

        if !self_edge_exists {
            // Add the self-edge
            if is_inverse {
                // For inverse properties, self-edge means predecessor relationship
                graph.add_edge(node_id, property_iri.clone(), node_id);
            } else {
                graph.add_edge(node_id, property_iri.clone(), node_id);
            }

            // Return that we've modified the graph structure
            return Ok(Some((Vec::new(), Vec::new())));
        }

        Ok(None)
    }

    /// Apply ≥ n R rule: ensure at least n R-successors (unqualified)
    fn apply_min_cardinality_rule(
        &self,
        n: usize,
        property: &ObjectPropertyExpression,
        filler: &ClassExpression,
        node_id: NodeId,
        graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        // For backward compatibility, delegate to qualified version with owl:Thing filler
        self.apply_qualified_min_cardinality_rule(n, property, filler, node_id, graph)
    }

    /// Apply ≥ n R.C rule: ensure at least n R-successors with C (qualified)
    fn apply_qualified_min_cardinality_rule(
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

        let (is_inverse, property_iri) = resolve_property_direction(property);

        if !is_inverse {
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
            while matching_successors < n {
                let new_node_id = graph.add_node();
                graph.add_edge(node_id, property_iri.clone(), new_node_id);
                graph.add_concept(new_node_id, filler.clone());
                new_nodes.push(new_node_id);
                matching_successors += 1;
            }
        } else {
            // For inverse properties, count predecessors and create new predecessors if needed
            let predecessors = graph.get_predecessors(node_id, property_iri);
            for pred_id in predecessors {
                let pred_node = graph.nodes.get(&pred_id)
                    .ok_or_else(|| OwlError::ReasoningError(format!("Predecessor node {} not found in graph", pred_id.0)))?;
                if pred_node.concepts.contains(filler) {
                    matching_successors += 1;
                }
            }

            // Create additional predecessors if needed
            while matching_successors < n {
                let new_node_id = graph.add_node();
                graph.add_edge(new_node_id, property_iri.clone(), node_id);
                graph.add_concept(new_node_id, filler.clone());
                new_nodes.push(new_node_id);
                matching_successors += 1;
            }
        }

        if !new_nodes.is_empty() {
            Ok(Some((Vec::new(), new_nodes)))
        } else {
            Ok(None)
        }
    }

    /// Apply ≤ n R rule: ensure at most n R-successors (unqualified)
    fn apply_max_cardinality_rule(
        &self,
        n: usize,
        property: &ObjectPropertyExpression,
        filler: &ClassExpression,
        node_id: NodeId,
        graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        // For backward compatibility, delegate to qualified version
        self.apply_qualified_max_cardinality_rule(n, property, filler, node_id, graph)
    }

    /// Apply ≤ n R.C rule: ensure at most n R-successors with C (qualified)
    fn apply_qualified_max_cardinality_rule(
        &self,
        n: usize,
        property: &ObjectPropertyExpression,
        filler: &ClassExpression,
        node_id: NodeId,
        graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        // Advanced SROIQ(D) max cardinality with sophisticated blocking
        let (is_inverse, property_iri) = resolve_property_direction(property);

        // Get all relevant successors/predecessors
        let relevant_nodes = if !is_inverse {
            graph.get_successors(node_id, property_iri)
                .map(|s| s.to_vec())
                .unwrap_or_default()
        } else {
            graph.get_predecessors(node_id, property_iri)
        };

        // Filter nodes that match the filler
        let mut matching_nodes = Vec::new();
        for node in relevant_nodes {
            if let Some(node_data) = graph.nodes.get(&node) {
                if node_data.concepts.contains(filler) {
                    matching_nodes.push(node);
                }
            }
        }

        if matching_nodes.len() <= n {
            // Constraint already satisfied
            return Ok(None);
        }

        // Need to apply blocking or merging - this is complex in SROIQ(D)
        // For now, apply pairwise blocking to satisfy the constraint
        let nodes_to_block = &matching_nodes[n..];

        for (i, node_to_block) in nodes_to_block.iter().enumerate() {
            // Find a node to block with (first n nodes)
            let blocking_node = &matching_nodes[i % n];

            // Apply blocking constraint
            graph.add_blocking_constraint(*node_to_block, *blocking_node);
        }

        // Return that we've modified the graph structure
        Ok(Some((Vec::new(), Vec::new())))
    }

    /// Apply = n R rule: ensure exactly n R-successors (unqualified)
    fn apply_exact_cardinality_rule(
        &self,
        n: usize,
        property: &ObjectPropertyExpression,
        filler: &ClassExpression,
        node_id: NodeId,
        graph: &mut TableauxGraph,
    ) -> OwlResult<Option<(Vec<ClassExpression>, Vec<NodeId>)>> {
        // For backward compatibility, delegate to qualified version
        self.apply_qualified_exact_cardinality_rule(n, property, filler, node_id, graph)
    }

    /// Apply = n R.C rule: ensure exactly n R-successors with C (qualified)
    fn apply_qualified_exact_cardinality_rule(
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

        // Apply qualified min cardinality
        if let Some((concepts, nodes)) =
            self.apply_qualified_min_cardinality_rule(n, property, filler, node_id, graph)?
        {
            all_new_concepts.extend(concepts);
            all_new_nodes.extend(nodes);
        }

        // Apply qualified max cardinality
        if let Some((concepts, nodes)) =
            self.apply_qualified_max_cardinality_rule(n, property, filler, node_id, graph)?
        {
            all_new_concepts.extend(concepts);
            all_new_nodes.extend(nodes);
        }

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
            _individual_constraints: HashMap::default(),
            blocking_constraints: Vec::new(),
            blocking_index: HashMap::default(),
            blocking_stats: BlockingStats::default(),
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

    /// Add individual equality constraint for nominal reasoning
    pub fn add_individual_constraint(&mut self, node_id: NodeId, individual: crate::entities::Individual) {
        self._individual_constraints.insert(node_id, individual);
    }

    /// Check if a node is equal to an individual
    pub fn is_equal_to_individual(&self, node_id: NodeId, individual: &crate::entities::Individual) -> bool {
        self._individual_constraints.get(&node_id) == Some(individual)
    }

    /// Add sophisticated blocking constraint
    pub fn add_blocking_constraint(&mut self, blocked_node: NodeId, blocking_node: NodeId) {
        self.add_sophisticated_blocking_constraint(
            blocked_node,
            blocking_node,
            BlockingType::Cardinality,
            "Cardinality restriction".to_string()
        );
    }

    /// Add sophisticated blocking constraint with metadata
    pub fn add_sophisticated_blocking_constraint(
        &mut self,
        blocked_node: NodeId,
        blocking_node: NodeId,
        blocking_type: BlockingType,
        reason: String
    ) {
        let constraint = BlockingConstraint::new(blocked_node, blocking_node, blocking_type.clone(), reason);
        let constraint_index = self.blocking_constraints.len();

        self.blocking_constraints.push(constraint);
        self.blocking_index.insert(blocked_node, constraint_index);

        // Update statistics
        self.blocking_stats.total_blocks += 1;
        self.blocking_stats.blocked_nodes.insert(blocked_node);

        match blocking_type {
            BlockingType::Subset => self.blocking_stats.subset_blocks += 1,
            BlockingType::Equality => self.blocking_stats.equality_blocks += 1,
            BlockingType::Cardinality => self.blocking_stats.cardinality_blocks += 1,
            BlockingType::Dynamic => self.blocking_stats.dynamic_blocks += 1,
            BlockingType::Nominal => self.blocking_stats.nominal_blocks += 1,
        }
    }

    /// Check if a node is blocked and return the strongest blocking constraint
    pub fn is_blocked_by(&self, node_id: NodeId) -> Option<&BlockingConstraint> {
        self.blocking_index.get(&node_id)
            .and_then(|&index| self.blocking_constraints.get(index))
    }

    /// Check if a node is blocked by a specific type
    pub fn is_blocked_by_type(&self, node_id: NodeId, blocking_type: BlockingType) -> bool {
        self.blocking_index.get(&node_id)
            .and_then(|&index| self.blocking_constraints.get(index))
            .map(|constraint| constraint.blocking_type == blocking_type)
            .unwrap_or(false)
    }

    /// Remove blocking constraint (for dynamic blocking)
    pub fn remove_blocking_constraint(&mut self, node_id: NodeId) -> Option<BlockingConstraint> {
        if let Some(&index) = self.blocking_index.get(&node_id) {
            let constraint = self.blocking_constraints.remove(index);
            self.blocking_index.remove(&node_id);
            self.blocking_stats.blocked_nodes.remove(&node_id);
            Some(constraint)
        } else {
            None
        }
    }

    /// Get all blocking constraints
    pub fn blocking_constraints(&self) -> &[BlockingConstraint] {
        &self.blocking_constraints
    }

    /// Get blocking statistics
    pub fn blocking_stats(&self) -> &BlockingStats {
        &self.blocking_stats
    }

    /// Get all individual constraints
    pub fn individual_constraints(&self) -> &HashMap<NodeId, crate::entities::Individual> {
        &self._individual_constraints
    }

    /// Remove a node and its associated data from the graph
    pub fn remove_node(&mut self, node_id: NodeId) {
        // Remove the node
        self.nodes.remove(&node_id);

        // Remove any edges associated with this node
        self.edges.edges.retain(|(from, _, to)| {
            *from != node_id && *to != node_id
        });

        // Remove any individual constraints
        self._individual_constraints.remove(&node_id);

        // Remove any blocking constraints
        self.remove_blocking_constraint(node_id);

        // Clear from cache
        self.node_cache.remove(&node_id);
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
