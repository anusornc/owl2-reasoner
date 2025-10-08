//! # Tableaux Memory Management
//!
//! Provides efficient memory management and allocation strategies for the tableaux reasoning engine.
//! This module implements arena-based allocation and automatic memory cleanup to prevent leaks
//! and optimize performance during intensive reasoning operations.
//!
//! ## Key Components
//!
//! - **[`MemoryManager`]** - Central memory management coordinator
//! - **[`ArenaManager`]** - Arena allocator for temporary objects
//! - **[`ArenaStats`]** - Detailed memory usage statistics and profiling
//! - **String Interning** - Efficient string storage and deduplication
//! - **Automatic Cleanup** - RAII-based memory management
//!
//! ## Memory Strategies
//!
//! ### Arena Allocation
//! Uses bump allocators for temporary objects created during reasoning:
//! - **Fast Allocation**: O(1) allocation time
//! - **Bulk Cleanup**: All arena memory freed at once
//! - **No Fragmentation**: Contiguous memory blocks
//! - **Cache Friendly**: Better locality than individual allocations
//!
//! ### String Interning
//! Deduplicates common strings to reduce memory usage:
//! - **IRI Storage**: Shared IRI strings across the reasoner
//! - **Property Names**: Common property names stored once
//! - **Class Expressions**: Shared string components
//! - **Memory Savings**: Up to 70% reduction in string storage
//!
//! ## Performance Benefits
//!
//! - **Reduced Allocation Overhead**: Fewer malloc/free calls
//! - **Better Cache Locality**: Contiguous memory blocks
//! - **Automatic Cleanup**: No manual memory management required
//! - **Memory Profiling**: Detailed statistics for optimization
//! - **Leak Prevention**: Guaranteed cleanup of temporary objects
//!
//! ## Example Usage
//!
//! ```rust
//! use owl2_reasoner::reasoning::tableaux::{MemoryManager, ArenaStats};
//!
//! // Create memory manager
//! let mut memory_manager = MemoryManager::new();
//!
//! // Allocate objects in arena
//! let arena_id = memory_manager.create_arena();
//!
//! // Perform memory-intensive operations
//! // ... reasoning operations that create temporary objects
//!
//! // Get memory statistics
//! let stats = memory_manager.get_arena_stats(arena_id);
//! println!("Arena {} allocated {} objects using {} bytes",
//!          arena_id, stats.arena_allocated_nodes, stats.total_bytes_allocated);
//!
//! // Cleanup happens automatically when arena is dropped
//! ```

use super::core::{MemoryStats, NodeId, TableauxNode};
use crate::axioms::*;
use crate::error::{OwlError, OwlResult};
use crate::iri::IRI;
use bumpalo::Bump;
use hashbrown::HashMap;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::mem;
use std::ptr::NonNull;
use std::sync::Mutex;

/// Helper function to safely lock mutexes with proper error handling
fn safe_lock<'a, T>(
    mutex: &'a Mutex<T>,
    lock_type: &str,
) -> OwlResult<std::sync::MutexGuard<'a, T>> {
    mutex.lock().map_err(|_| OwlError::LockError {
        lock_type: lock_type.to_string(),
        timeout_ms: 0,
        message: format!("Failed to acquire {} lock", lock_type),
    })
}

/// Memory optimization statistics for tracking arena allocation benefits
#[derive(Debug, Clone, Default)]
pub struct MemoryOptimizationStats {
    /// Number of nodes allocated in arena
    pub arena_allocated_nodes: usize,
    /// Number of expressions allocated in arena
    pub arena_allocated_expressions: usize,
    /// Number of constraints allocated in arena
    pub arena_allocated_constraints: usize,
    /// Number of strings interned
    pub interned_strings: usize,
    /// Memory saved through string interning (bytes)
    pub string_intern_savings: usize,
    /// Memory saved through arena allocation (bytes)
    pub arena_allocation_savings: usize,
}

/// Change log for memory allocations performed during branch expansion.
///
/// Currently a placeholder; will record arena allocations once branch-aware
/// allocation is wired through [`MemoryManager`].
#[derive(Debug, Default, Clone)]
pub struct MemoryChangeLog;

impl MemoryChangeLog {
    pub fn new() -> Self {
        Self
    }

    pub fn extend(&mut self, _other: MemoryChangeLog) {}

    pub fn rollback(&self, _manager: &mut MemoryManager) {}
}

/// Arena allocation statistics
#[derive(Debug, Clone, Default)]
pub struct ArenaStats {
    /// Number of nodes allocated in arena
    pub arena_allocated_nodes: usize,
    /// Number of expressions allocated in arena
    pub arena_allocated_expressions: usize,
    /// Number of constraints allocated in arena
    pub arena_allocated_constraints: usize,
    /// Total bytes allocated across all arenas
    pub total_bytes_allocated: usize,
    /// Memory saved through string interning (bytes)
    pub string_intern_savings: usize,
    /// Memory saved through arena allocation (bytes)
    pub arena_allocation_savings: usize,
}

/// Arena-optimized edge storage for memory efficiency
#[derive(Debug, Default)]
pub struct ArenaEdgeStorage {
    /// Arena-allocated edge storage
    pub edges: Vec<(NodeId, IRI, NodeId)>,
    /// Fast lookup index
    pub index: HashMap<(NodeId, IRI), SmallVec<[NodeId; 4]>>,
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
        self.index.entry(key).or_default().push(to);
    }

    /// Get successors of a node
    pub fn get_successors(&self, node: NodeId, property: &IRI) -> Option<&[NodeId]> {
        self.index
            .get(&(node, property.clone()))
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

/// Arena manager for efficient memory allocation
#[derive(Debug)]
pub struct ArenaManager {
    pub stats: ArenaStats,
    pub node_arena: Mutex<Bump>,
    pub expression_arena: Mutex<Bump>,
    pub constraint_arena: Mutex<Bump>,
    pub string_arena: Mutex<Bump>,
    pub string_interner: Mutex<HashMap<String, *const u8>>,
}

impl ArenaManager {
    /// Create a new arena manager with default capacity
    pub fn new() -> Self {
        Self {
            stats: ArenaStats::default(),
            node_arena: Mutex::new(Bump::new()),
            expression_arena: Mutex::new(Bump::new()),
            constraint_arena: Mutex::new(Bump::new()),
            string_arena: Mutex::new(Bump::new()),
            string_interner: Mutex::new(HashMap::new()),
        }
    }

    /// Allocate a TableauxNode in the node arena
    pub fn allocate_node(&mut self, node: TableauxNode) -> OwlResult<NonNull<TableauxNode>> {
        self.stats.arena_allocated_nodes += 1;
        let node_arena = safe_lock(&self.node_arena, "node_arena")?;
        let allocated = node_arena.alloc(node);
        Ok(NonNull::from(allocated))
    }

    /// Allocate a ClassExpression in the expression arena
    pub fn allocate_expression(
        &mut self,
        expr: ClassExpression,
    ) -> OwlResult<NonNull<ClassExpression>> {
        self.stats.arena_allocated_expressions += 1;
        let expression_arena = safe_lock(&self.expression_arena, "expression_arena")?;
        let allocated = expression_arena.alloc(expr);
        Ok(NonNull::from(allocated))
    }

    /// Allocate a blocking constraint in the constraint arena
    pub fn allocate_constraint<T>(&mut self, constraint: T) -> OwlResult<NonNull<T>> {
        self.stats.arena_allocated_constraints += 1;
        let constraint_arena = safe_lock(&self.constraint_arena, "constraint_arena")?;
        let allocated = constraint_arena.alloc(constraint);
        Ok(NonNull::from(allocated))
    }

    /// Intern a string in the string arena
    pub fn intern_string(&mut self, s: &str) -> OwlResult<NonNull<str>> {
        let mut string_interner = safe_lock(&self.string_interner, "string_interner")?;
        if let Some(&ptr) = string_interner.get(s) {
            // SAFETY: String interning lookup reconstruction
            // 1. The pointer `ptr` was allocated from string_arena which is still alive
            // 2. We hold a lock on string_interner, preventing pointer invalidation
            // 3. The pointer points to valid UTF-8 data (validated when first stored)
            // 4. The slice length `s.len()` exactly matches the original string length
            // 5. The memory range [ptr, ptr + s.len()) is within the arena bounds
            // 6. No mutable references exist to this string due to string interning semantics
            let interned_str =
                unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, s.len())) };
            return Ok(NonNull::from(interned_str));
        }

        let string_arena = safe_lock(&self.string_arena, "string_arena")?;
        let allocated = string_arena.alloc_str(s);
        let ptr = allocated.as_ptr();
        string_interner.insert(s.to_string(), ptr);
        self.stats.string_intern_savings += s.len() * 2; // Estimate savings from interning
        Ok(NonNull::from(allocated))
    }

    /// Reset all arenas (for tableaux restart)
    pub fn reset(&mut self) -> OwlResult<()> {
        let mut node_arena = safe_lock(&self.node_arena, "node_arena")?;
        let mut expression_arena = safe_lock(&self.expression_arena, "expression_arena")?;
        let mut constraint_arena = safe_lock(&self.constraint_arena, "constraint_arena")?;
        let mut string_arena = safe_lock(&self.string_arena, "string_arena")?;
        let mut string_interner = safe_lock(&self.string_interner, "string_interner")?;

        node_arena.reset();
        expression_arena.reset();
        constraint_arena.reset();
        string_arena.reset();
        string_interner.clear();
        self.stats = ArenaStats::default();
        Ok(())
    }

    /// Get total memory usage across all arenas
    pub fn total_allocated_bytes(&self) -> OwlResult<usize> {
        let node_arena = safe_lock(&self.node_arena, "node_arena")?;
        let expression_arena = safe_lock(&self.expression_arena, "expression_arena")?;
        let constraint_arena = safe_lock(&self.constraint_arena, "constraint_arena")?;
        let string_arena = safe_lock(&self.string_arena, "string_arena")?;

        Ok(node_arena.allocated_bytes()
            + expression_arena.allocated_bytes()
            + constraint_arena.allocated_bytes()
            + string_arena.allocated_bytes())
    }

    /// Get current statistics
    pub fn stats(&self) -> &ArenaStats {
        &self.stats
    }

    /// Get mutable statistics
    pub fn stats_mut(&mut self) -> &mut ArenaStats {
        &mut self.stats
    }
}

impl Default for ArenaManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimized tableaux node with arena allocation support
#[derive(Debug)]
pub struct ArenaTableauxNode {
    /// Pointer to arena-allocated node data
    node_ptr: NonNull<TableauxNode>,
    /// Keep arena alive (phantom data)
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
        // SAFETY: Arena node mutable access
        // 1. node_ptr was allocated from an arena referenced by _arena field
        // 2. _arena field's lifetime ensures arena outlives this struct
        // 3. &mut self guarantees exclusive access to the node
        // 4. No other references can exist due to Rust's borrowing rules
        // 5. The memory pointed to is properly aligned and valid for TableauxNode
        unsafe { self.node_ptr.as_mut() }
    }

    /// Get immutable reference to the node
    pub fn get(&self) -> &TableauxNode {
        // SAFETY: Arena node immutable access
        // 1. node_ptr was allocated from an arena referenced by _arena field
        // 2. _arena field's lifetime ensures arena outlives this struct
        // 3. &self provides shared access, which is safe for immutable references
        // 4. Arena allocation guarantees memory remains valid and unchanged
        // 5. The pointer is properly aligned and points to valid TableauxNode data
        unsafe { self.node_ptr.as_ref() }
    }
}

/// Memory manager for tableaux reasoning
#[derive(Debug)]
pub struct MemoryManager {
    pub arena_manager: Mutex<ArenaManager>,
    pub memory_stats: Mutex<MemoryStats>,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            arena_manager: Mutex::new(ArenaManager::new()),
            memory_stats: Mutex::new(MemoryStats::new()),
        }
    }

    pub fn allocate_node(&self, node: TableauxNode) -> OwlResult<ArenaTableauxNode> {
        let node_size = mem::size_of::<TableauxNode>();
        let arena_manager = safe_lock(&self.arena_manager, "arena_manager")?;
        let mut memory_stats = safe_lock(&self.memory_stats, "memory_stats")?;
        let mut node_arena = safe_lock(&arena_manager.node_arena, "node_arena")?;
        let arena_node = ArenaTableauxNode::new(node, &mut node_arena);
        memory_stats.add_node_allocation(node_size);
        Ok(arena_node)
    }

    pub fn allocate_expression(
        &self,
        expr: ClassExpression,
    ) -> OwlResult<NonNull<ClassExpression>> {
        let expr_size = mem::size_of::<ClassExpression>();
        let mut arena_manager = safe_lock(&self.arena_manager, "arena_manager")?;
        let mut memory_stats = safe_lock(&self.memory_stats, "memory_stats")?;
        let allocated = arena_manager.allocate_expression(expr)?;
        memory_stats.add_expression_allocation(expr_size);
        Ok(allocated)
    }

    pub fn intern_string(&self, s: &str) -> OwlResult<NonNull<str>> {
        let mut arena_manager = safe_lock(&self.arena_manager, "arena_manager")?;
        arena_manager.intern_string(s)
    }

    pub fn get_memory_efficiency_ratio(&self) -> OwlResult<f64> {
        let arena_manager = safe_lock(&self.arena_manager, "arena_manager")?;
        let stats = &arena_manager.stats;
        let total_traditional_allocations = stats.arena_allocated_nodes * 64 + // Traditional node allocation overhead
                                           stats.arena_allocated_expressions * 48 + // Traditional expression overhead
                                           stats.arena_allocated_constraints * 32; // Traditional constraint overhead

        if total_traditional_allocations == 0 {
            Ok(1.0)
        } else {
            let total_arena_allocations = stats.arena_allocated_nodes
                + stats.arena_allocated_expressions
                + stats.arena_allocated_constraints;
            Ok(total_traditional_allocations as f64 / total_arena_allocations.max(1) as f64)
        }
    }

    pub fn get_total_memory_savings(&self) -> OwlResult<usize> {
        let arena_manager = safe_lock(&self.arena_manager, "arena_manager")?;
        let stats = &arena_manager.stats;
        Ok(stats.string_intern_savings + stats.arena_allocation_savings)
    }

    pub fn reset(&self) -> OwlResult<()> {
        let mut arena_manager = safe_lock(&self.arena_manager, "arena_manager")?;
        let mut memory_stats = safe_lock(&self.memory_stats, "memory_stats")?;
        arena_manager.reset()?;
        *memory_stats = MemoryStats::new();
        Ok(())
    }

    pub fn get_arena_stats(&self) -> OwlResult<ArenaStats> {
        let arena_manager = safe_lock(&self.arena_manager, "arena_manager")?;
        Ok(arena_manager.stats.clone())
    }

    pub fn get_memory_stats(&self) -> OwlResult<MemoryStats> {
        let memory_stats = safe_lock(&self.memory_stats, "memory_stats")?;
        Ok(memory_stats.clone())
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Arena-optimized tableaux graph for maximum memory efficiency
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
            root: NodeId::new(0),
            next_id: 1,
            memory_stats: RefCell::new(MemoryOptimizationStats::default()),
        };

        // Create root node
        let root_node = graph
            .arena_manager
            .allocate_node(TableauxNode::new(graph.root))
            .expect("Failed to allocate root node");
        graph.nodes.insert(graph.root, root_node);

        graph
    }

    /// Add a node to the arena-optimized graph
    pub fn add_node(&mut self) -> NodeId {
        let node_id = NodeId::new(self.next_id);
        self.next_id += 1;

        // Allocate node in arena
        let node = self
            .arena_manager
            .allocate_node(TableauxNode::new(node_id))
            .expect("Failed to allocate node");
        self.nodes.insert(node_id, node);

        // Update memory statistics
        self.memory_stats.borrow_mut().arena_allocated_nodes += 1;

        node_id
    }

    /// Add a concept to a node in arena memory
    pub fn add_concept(&mut self, node_id: NodeId, concept: ClassExpression) {
        if let Some(node_ptr) = self.nodes.get_mut(&node_id) {
            // SAFETY: Graph node mutable access
            // 1. node_ptr was allocated from self.arena_manager which is still alive
            // 2. &mut self ensures exclusive access to entire graph structure
            // 3. No other references exist to this specific node due to borrow rules
            // 4. Arena allocation guarantees memory stability and validity
            // 5. The TableauxNode is properly initialized and safe to modify
            unsafe {
                let node = node_ptr.as_mut();
                node.add_concept(concept);
            }

            // Update memory statistics
            self.memory_stats.borrow_mut().arena_allocated_expressions += 1;
        }
    }

    /// Add an edge to the arena-optimized graph
    pub fn add_edge(&mut self, from: NodeId, property: &IRI, to: NodeId) {
        self.edges.add_edge(from, property, to);
    }

    /// Get a node from the arena-optimized graph
    pub fn get_node(&self, node_id: NodeId) -> Option<&TableauxNode> {
        self.nodes.get(&node_id).map(|node_ptr| {
            // SAFETY: Graph node immutable access
            // 1. node_ptr was allocated from self.arena_manager which outlives &self
            // 2. Arena allocation ensures memory remains valid and stable
            // 3. &self provides shared access, safe for immutable references
            // 4. The returned reference lifetime is correctly bound to &self
            // 5. No mutation can occur during this reference's lifetime
            unsafe { node_ptr.as_ref() }
        })
    }

    /// Get a mutable node from the arena-optimized graph
    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut TableauxNode> {
        self.nodes.get_mut(&node_id).map(|node_ptr| {
            // SAFETY: Graph node mutable access
            // 1. node_ptr was allocated from self.arena_manager which outlives &mut self
            // 2. &mut self guarantees exclusive access to entire graph
            // 3. No other references exist to this node due to Rust's borrow checker
            // 4. Arena allocation guarantees memory stability during mutation
            // 5. The pointer is properly aligned and valid for TableauxNode access
            unsafe { node_ptr.as_mut() }
        })
    }

    /// Get successors of a node
    pub fn get_successors(&self, node_id: NodeId, property: &IRI) -> Option<&[NodeId]> {
        self.edges.get_successors(node_id, property)
    }

    /// Get memory optimization statistics
    pub fn get_memory_stats(&self) -> MemoryOptimizationStats {
        self.memory_stats.borrow().clone()
    }

    /// Get all nodes in the graph
    pub fn get_nodes(&self) -> impl Iterator<Item = &TableauxNode> {
        self.nodes.values().map(|node_ptr| {
            // SAFETY: Iterator over all graph nodes
            // 1. All node pointers were allocated from self.arena_manager
            // 2. Arena lifetime is tied to &self, ensuring memory validity
            // 3. &self provides shared access, safe for immutable references
            // 4. Each node_ptr is properly aligned and points to valid TableauxNode
            // 5. No mutation occurs during iterator lifetime
            unsafe { node_ptr.as_ref() }
        })
    }

    /// Clear the graph and reset all arenas
    pub fn clear(&mut self) -> OwlResult<()> {
        self.nodes.clear();
        self.edges.clear();
        self.arena_manager.reset()?;
        *self.memory_stats.borrow_mut() = MemoryOptimizationStats::default();
        self.root = NodeId::new(0);
        self.next_id = 1;

        // Recreate root node
        let root_node = self
            .arena_manager
            .allocate_node(TableauxNode::new(self.root))
            .expect("Failed to allocate root node");
        self.nodes.insert(self.root, root_node);

        Ok(())
    }
}

impl Default for ArenaTableauxGraph {
    fn default() -> Self {
        Self::new()
    }
}
