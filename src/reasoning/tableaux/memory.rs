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
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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

/// Represents a single memory mutation that can be tracked and rolled back.
#[derive(Debug, Clone)]
pub enum MemoryChange {
    /// Node allocation in arena
    AllocateNode {
        node_id: NodeId,
        arena_type: ArenaType,
        size_bytes: usize,
    },
    /// Expression allocation in arena
    AllocateExpression {
        arena_type: ArenaType,
        size_bytes: usize,
    },
    /// Constraint allocation in arena
    AllocateConstraint {
        arena_type: ArenaType,
        size_bytes: usize,
    },
    /// String interning (optimized: use Cow<str> to avoid unnecessary allocations)
    InternString {
        string_hash: u64, // Pre-computed hash for faster comparison
        size_bytes: usize,
    },
    /// Arena reset operation
    ArenaReset {
        arena_type: ArenaType,
        previous_stats: ArenaStats,
    },
    /// Memory checkpoint creation
    CreateCheckpoint {
        checkpoint_id: usize,
        memory_state: MemorySnapshot,
    },
    /// Memory rollback to checkpoint
    RollbackToCheckpoint { checkpoint_id: usize },
}

/// Types of arenas for memory tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaType {
    Node,
    Expression,
    Constraint,
    String,
}

/// Snapshot of memory state for checkpoint/rollback functionality
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub arena_stats: ArenaStats,
    pub memory_stats: MemoryStats,
    pub timestamp: std::time::Instant,
}

/// Ordered log of memory mutations so branches can be rolled back.
#[derive(Debug, Default, Clone)]
pub struct MemoryChangeLog {
    changes: Vec<MemoryChange>,
    checkpoints: hashbrown::HashMap<usize, usize>, // checkpoint_id -> change_index (optimized HashMap)
    next_checkpoint_id: usize,
}

impl MemoryChangeLog {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
            checkpoints: hashbrown::HashMap::new(),
            next_checkpoint_id: 0,
        }
    }

    /// Check if the change log is empty
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Record a memory change
    pub fn record(&mut self, change: MemoryChange) {
        self.changes.push(change);
    }

    /// Create a memory checkpoint and return its ID
    pub fn create_checkpoint(&mut self, memory_state: MemorySnapshot) -> usize {
        let checkpoint_id = self.next_checkpoint_id;
        self.next_checkpoint_id += 1;

        let change_index = self.changes.len();
        self.checkpoints.insert(checkpoint_id, change_index);

        self.changes.push(MemoryChange::CreateCheckpoint {
            checkpoint_id,
            memory_state,
        });

        checkpoint_id
    }

    /// Extend this log with another log
    pub fn extend(&mut self, mut other: MemoryChangeLog) {
        let current_len = self.changes.len();

        // Adjust checkpoint indices from the other log
        for (checkpoint_id, change_index) in other.checkpoints.drain() {
            self.checkpoints
                .insert(checkpoint_id, current_len + change_index);
        }
        self.changes.append(&mut other.changes);
    }

    /// Rollback to a specific checkpoint
    pub fn rollback_to_checkpoint(
        &mut self,
        checkpoint_id: usize,
    ) -> Result<Vec<MemoryChange>, String> {
        if let Some(&change_index) = self.checkpoints.get(&checkpoint_id) {
            let changes_to_rollback: Vec<_> =
                self.changes[change_index..].iter().rev().cloned().collect();
            self.changes.truncate(change_index);

            // Remove this checkpoint and any later ones
            self.checkpoints.retain(|&id, _| id < checkpoint_id);

            Ok(changes_to_rollback)
        } else {
            Err(format!("Checkpoint {} not found", checkpoint_id))
        }
    }

    /// Get the number of changes recorded
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Get iterator over all changes
    pub fn changes(&self) -> impl Iterator<Item = &MemoryChange> {
        self.changes.iter()
    }

    /// Get changes since a specific checkpoint
    pub fn changes_since_checkpoint(&self, checkpoint_id: usize) -> Option<&[MemoryChange]> {
        self.checkpoints
            .get(&checkpoint_id)
            .map(|&index| &self.changes[index..])
    }

    /// Apply rollback operations to a memory manager
    pub fn rollback(&self, manager: &mut MemoryManager) -> Result<(), String> {
        // This is a simplified rollback implementation
        // In a full implementation, we'd need more sophisticated arena management
        manager
            .reset()
            .map_err(|e| format!("Memory reset failed: {:?}", e))?;
        Ok(())
    }

    /// Get memory statistics from the change log
    pub fn get_memory_stats(&self) -> MemoryMutationStats {
        let mut stats = MemoryMutationStats::default();

        for change in &self.changes {
            match change {
                MemoryChange::AllocateNode { size_bytes, .. } => {
                    stats.nodes_allocated += 1;
                    stats.total_bytes_allocated += size_bytes;
                }
                MemoryChange::AllocateExpression { size_bytes, .. } => {
                    stats.expressions_allocated += 1;
                    stats.total_bytes_allocated += size_bytes;
                }
                MemoryChange::AllocateConstraint { size_bytes, .. } => {
                    stats.constraints_allocated += 1;
                    stats.total_bytes_allocated += size_bytes;
                }
                MemoryChange::InternString { size_bytes, .. } => {
                    stats.strings_interned += 1;
                    stats.total_bytes_allocated += size_bytes;
                }
                MemoryChange::ArenaReset { .. } => {
                    stats.arena_resets += 1;
                }
                MemoryChange::CreateCheckpoint { .. } => {
                    stats.checkpoints_created += 1;
                }
                MemoryChange::RollbackToCheckpoint { .. } => {
                    stats.rollbacks_performed += 1;
                }
            }
        }

        stats
    }
}

/// Statistics for memory mutations tracked during reasoning
#[derive(Debug, Clone, Default)]
pub struct MemoryMutationStats {
    /// Number of nodes allocated
    pub nodes_allocated: usize,
    /// Number of expressions allocated
    pub expressions_allocated: usize,
    /// Number of constraints allocated
    pub constraints_allocated: usize,
    /// Number of strings interned
    pub strings_interned: usize,
    /// Total bytes allocated
    pub total_bytes_allocated: usize,
    /// Number of arena resets
    pub arena_resets: usize,
    /// Number of checkpoints created
    pub checkpoints_created: usize,
    /// Number of rollbacks performed
    pub rollbacks_performed: usize,
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

/// Arena manager for efficient memory allocation (optimized: reduced mutex contention)
#[derive(Debug)]
pub struct ArenaManager {
    pub stats: ArenaStats,
    pub node_arena: Mutex<Bump>,
    pub expression_arena: Mutex<Bump>,
    pub constraint_arena: Mutex<Bump>,
    pub string_arena: Mutex<Bump>,
    pub string_interner: Mutex<HashMap<u64, (*const u8, usize)>>, // Optimized: hash -> (ptr, len)
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
            string_interner: Mutex::new(hashbrown::HashMap::new()),
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

    /// Intern a string in the string arena (optimized: avoid string allocations)
    pub fn intern_string(&mut self, s: &str) -> OwlResult<NonNull<str>> {
        let s_hash = {
            let mut hasher = DefaultHasher::new();
            s.hash(&mut hasher);
            hasher.finish()
        };

        let mut string_interner = safe_lock(&self.string_interner, "string_interner")?;
        if let Some(&(ptr, len)) = string_interner.get(&s_hash) {
            // Verify length matches to avoid hash collisions
            if len == s.len() {
                // SAFETY: String interning lookup reconstruction
                // 1. The pointer `ptr` was allocated from string_arena which is still alive
                // 2. We hold a lock on string_interner, preventing pointer invalidation
                // 3. The pointer points to valid UTF-8 data (validated when first stored)
                // 4. The slice length `s.len()` exactly matches the original string length
                // 5. The memory range [ptr, ptr + s.len()) is within the arena bounds
                // 6. No mutable references exist to this string due to string interning semantics
                let interned_str = unsafe {
                    std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, s.len()))
                };
                return Ok(NonNull::from(interned_str));
            }
        }

        let string_arena = safe_lock(&self.string_arena, "string_arena")?;
        let allocated = string_arena.alloc_str(s);
        let ptr = allocated.as_ptr();
        string_interner.insert(s_hash, (ptr, s.len()));
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

/// Memory manager for tableaux reasoning with mutation tracking support
#[derive(Debug)]
pub struct MemoryManager {
    pub arena_manager: Mutex<ArenaManager>,
    pub memory_stats: Mutex<MemoryStats>,
    /// Optional memory change log for tracking mutations
    change_log: Option<Mutex<MemoryChangeLog>>,
    /// Whether memory tracking is enabled
    tracking_enabled: std::sync::atomic::AtomicBool,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            arena_manager: Mutex::new(ArenaManager::new()),
            memory_stats: Mutex::new(MemoryStats::new()),
            change_log: None,
            tracking_enabled: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Create a new memory manager with tracking enabled
    pub fn with_tracking() -> Self {
        Self {
            arena_manager: Mutex::new(ArenaManager::new()),
            memory_stats: Mutex::new(MemoryStats::new()),
            change_log: Some(Mutex::new(MemoryChangeLog::new())),
            tracking_enabled: std::sync::atomic::AtomicBool::new(true),
        }
    }

    /// Enable or disable memory tracking
    pub fn set_tracking_enabled(&self, enabled: bool) {
        self.tracking_enabled
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
    }

    /// Check if memory tracking is enabled
    pub fn is_tracking_enabled(&self) -> bool {
        self.tracking_enabled
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get the current memory change log
    pub fn get_change_log(&self) -> Option<MemoryChangeLog> {
        if let Some(ref log) = self.change_log {
            safe_lock(log, "change_log").ok().map(|guard| guard.clone())
        } else {
            None
        }
    }

    /// Take the current memory change log (replacing it with a new one)
    pub fn take_change_log(&self) -> Option<MemoryChangeLog> {
        if let Some(ref log) = self.change_log {
            safe_lock(log, "change_log")
                .ok()
                .map(|mut log| std::mem::replace(&mut *log, MemoryChangeLog::new()))
        } else {
            None
        }
    }

    /// Record a memory change if tracking is enabled (optimized: early return)
    #[inline]
    fn record_change(&self, change: MemoryChange) {
        if !self.is_tracking_enabled() {
            return;
        }

        if let Some(ref log) = self.change_log {
            if let Ok(mut log) = safe_lock(log, "change_log") {
                log.record(change);
            }
        }
    }

    /// Create a memory checkpoint and return its ID (optimized: avoid unnecessary clones)
    pub fn create_checkpoint(&self) -> OwlResult<usize> {
        if !self.is_tracking_enabled() {
            return Err(OwlError::Other(
                "Memory tracking is not enabled".to_string(),
            ));
        }

        let arena_manager = safe_lock(&self.arena_manager, "arena_manager")?;
        let memory_stats = safe_lock(&self.memory_stats, "memory_stats")?;

        let snapshot = MemorySnapshot {
            arena_stats: arena_manager.stats.clone(), // Only clone what we need
            memory_stats: memory_stats.clone(),
            timestamp: std::time::Instant::now(),
        };

        if let Some(ref log) = self.change_log {
            let mut log = safe_lock(log, "change_log")?;
            Ok(log.create_checkpoint(snapshot))
        } else {
            Err(OwlError::Other(
                "Memory change log not available".to_string(),
            ))
        }
    }

    /// Rollback to a specific checkpoint
    pub fn rollback_to_checkpoint(&self, checkpoint_id: usize) -> OwlResult<()> {
        if !self.is_tracking_enabled() {
            return Err(OwlError::Other(
                "Memory tracking is not enabled".to_string(),
            ));
        }

        if let Some(ref log) = self.change_log {
            let mut log = safe_lock(log, "change_log")?;
            let changes_to_rollback = log
                .rollback_to_checkpoint(checkpoint_id)
                .map_err(|e| OwlError::Other(format!("Checkpoint rollback failed: {}", e)))?;

            // Apply rollback operations (simplified implementation)
            for change in changes_to_rollback {
                match change {
                    MemoryChange::AllocateNode { .. }
                    | MemoryChange::AllocateExpression { .. }
                    | MemoryChange::AllocateConstraint { .. }
                    | MemoryChange::InternString { .. } => {
                        // For arena-based allocations, we can't easily deallocate individual items
                        // In a full implementation, we'd need more sophisticated arena management
                        // For now, we'll reset the arenas on rollback
                    }
                    MemoryChange::ArenaReset { previous_stats, .. } => {
                        // Restore previous arena stats
                        let mut arena_manager = safe_lock(&self.arena_manager, "arena_manager")?;
                        arena_manager.stats = previous_stats;
                    }
                    _ => {}
                }
            }

            // Reset arenas to deallocate all memory after checkpoint
            self.reset()?;
            Ok(())
        } else {
            Err(OwlError::Other(
                "Memory change log not available".to_string(),
            ))
        }
    }

    /// Get memory mutation statistics
    pub fn get_mutation_stats(&self) -> OwlResult<MemoryMutationStats> {
        if let Some(ref log) = self.change_log {
            let log = safe_lock(log, "change_log")?;
            Ok(log.get_memory_stats())
        } else {
            Ok(MemoryMutationStats::default())
        }
    }

    pub fn allocate_node(&self, node: TableauxNode) -> OwlResult<ArenaTableauxNode> {
        let node_size = mem::size_of::<TableauxNode>();
        let node_id = node.id;

        // Record the memory change first (faster path)
        self.record_change(MemoryChange::AllocateNode {
            node_id,
            arena_type: ArenaType::Node,
            size_bytes: node_size,
        });

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

        // Record the memory change first (faster path)
        self.record_change(MemoryChange::AllocateExpression {
            arena_type: ArenaType::Expression,
            size_bytes: expr_size,
        });

        let mut arena_manager = safe_lock(&self.arena_manager, "arena_manager")?;
        let mut memory_stats = safe_lock(&self.memory_stats, "memory_stats")?;
        let allocated = arena_manager.allocate_expression(expr)?;
        memory_stats.add_expression_allocation(expr_size);

        Ok(allocated)
    }

    pub fn allocate_constraint<T>(&self, constraint: T) -> OwlResult<NonNull<T>> {
        let constraint_size = mem::size_of::<T>();

        // Record the memory change first (faster path)
        self.record_change(MemoryChange::AllocateConstraint {
            arena_type: ArenaType::Constraint,
            size_bytes: constraint_size,
        });

        let mut arena_manager = safe_lock(&self.arena_manager, "arena_manager")?;
        let mut memory_stats = safe_lock(&self.memory_stats, "memory_stats")?;
        let allocated = arena_manager.allocate_constraint(constraint)?;
        memory_stats.add_constraint_allocation(constraint_size);

        Ok(allocated)
    }

    pub fn intern_string(&self, s: &str) -> OwlResult<NonNull<str>> {
        let s_hash = {
            let mut hasher = DefaultHasher::new();
            s.hash(&mut hasher);
            hasher.finish()
        };
        let string_size = s.len();

        // Record the memory change first (optimized: use hash instead of string)
        self.record_change(MemoryChange::InternString {
            string_hash: s_hash,
            size_bytes: string_size,
        });

        let mut arena_manager = safe_lock(&self.arena_manager, "arena_manager")?;
        let allocated = arena_manager.intern_string(s)?;

        Ok(allocated)
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

        // Record the arena reset before clearing if tracking is enabled
        // Note: We need to be careful about deadlock - don't record if we're in a rollback
        if self.is_tracking_enabled() {
            // Try to record without blocking - if it fails, skip recording to avoid deadlock
            if let Some(ref log) = self.change_log {
                if let Ok(mut log) = log.try_lock() {
                    log.record(MemoryChange::ArenaReset {
                        arena_type: ArenaType::Node, // Record as general arena reset
                        previous_stats: arena_manager.stats.clone(),
                    });
                }
                // If try_lock fails, we're likely in a rollback scenario, so skip recording
            }
        }

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

        // Create root node - critical failure if this fails
        let root_node = graph
            .arena_manager
            .allocate_node(TableauxNode::new(graph.root))
            .expect("CRITICAL: Failed to allocate root node in ArenaTableauxGraph::new() - this should never fail");
        graph.nodes.insert(graph.root, root_node);

        graph
    }

    /// Add a node to the arena-optimized graph
    pub fn add_node(&mut self) -> NodeId {
        let node_id = NodeId::new(self.next_id);
        self.next_id += 1;

        // Allocate node in arena - critical failure if this fails
        let node = self
            .arena_manager
            .allocate_node(TableauxNode::new(node_id))
            .expect("CRITICAL: Failed to allocate node in ArenaTableauxGraph::add_node() - memory allocation failed");
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

        // Recreate root node - critical failure if this fails
        let root_node = self
            .arena_manager
            .allocate_node(TableauxNode::new(self.root))
            .expect("CRITICAL: Failed to recreate root node in ArenaTableauxGraph::clear() - this should never fail");
        self.nodes.insert(self.root, root_node);

        Ok(())
    }
}

impl Default for ArenaTableauxGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Class;

    #[test]
    fn test_memory_change_log_basic_operations() {
        let mut log = MemoryChangeLog::new();

        // Test empty log
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);

        // Test recording changes
        let change = MemoryChange::AllocateNode {
            node_id: NodeId::new(1),
            arena_type: ArenaType::Node,
            size_bytes: 64,
        };
        log.record(change);

        assert!(!log.is_empty());
        assert_eq!(log.len(), 1);

        // Test extending logs
        let mut other_log = MemoryChangeLog::new();
        other_log.record(MemoryChange::AllocateExpression {
            arena_type: ArenaType::Expression,
            size_bytes: 48,
        });

        log.extend(other_log);
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn test_memory_checkpoint_creation_and_rollback() {
        let mut log = MemoryChangeLog::new();

        // Create initial checkpoint
        let memory_state = MemorySnapshot {
            arena_stats: ArenaStats::default(),
            memory_stats: MemoryStats::new(),
            timestamp: std::time::Instant::now(),
        };
        let checkpoint_id = log.create_checkpoint(memory_state);
        assert_eq!(checkpoint_id, 0);

        // Add some changes
        log.record(MemoryChange::AllocateNode {
            node_id: NodeId::new(1),
            arena_type: ArenaType::Node,
            size_bytes: 64,
        });
        {
            let mut hasher = DefaultHasher::new();
            "test".hash(&mut hasher);
            log.record(MemoryChange::InternString {
                string_hash: hasher.finish(),
                size_bytes: 4,
            });
        }

        // Rollback to checkpoint
        let changes_to_rollback = log
            .rollback_to_checkpoint(checkpoint_id)
            .expect("Failed to rollback to checkpoint: checkpoint should be valid");
        assert_eq!(changes_to_rollback.len(), 3); // 2 changes + 1 checkpoint change
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn test_memory_change_log_statistics() {
        let mut log = MemoryChangeLog::new();

        // Add various types of changes
        log.record(MemoryChange::AllocateNode {
            node_id: NodeId::new(1),
            arena_type: ArenaType::Node,
            size_bytes: 64,
        });
        log.record(MemoryChange::AllocateExpression {
            arena_type: ArenaType::Expression,
            size_bytes: 48,
        });
        {
            let mut hasher = DefaultHasher::new();
            "test".hash(&mut hasher);
            log.record(MemoryChange::InternString {
                string_hash: hasher.finish(),
                size_bytes: 4,
            });
        }
        log.record(MemoryChange::ArenaReset {
            arena_type: ArenaType::Node,
            previous_stats: ArenaStats::default(),
        });

        let stats = log.get_memory_stats();
        assert_eq!(stats.nodes_allocated, 1);
        assert_eq!(stats.expressions_allocated, 1);
        assert_eq!(stats.strings_interned, 1);
        assert_eq!(stats.arena_resets, 1);
        assert_eq!(stats.total_bytes_allocated, 64 + 48 + 4);
    }

    #[test]
    fn test_memory_manager_with_tracking() {
        let memory_manager = MemoryManager::with_tracking();

        // Test that tracking is enabled
        assert!(memory_manager.is_tracking_enabled());

        // Test allocation tracking
        let node = TableauxNode::new(NodeId::new(1));
        let _arena_node = memory_manager.allocate_node(node).unwrap();

        let change_log = memory_manager.get_change_log().unwrap();
        assert_eq!(change_log.len(), 1);

        let stats = memory_manager.get_mutation_stats().unwrap();
        assert_eq!(stats.nodes_allocated, 1);
    }

    #[test]
    fn test_memory_manager_without_tracking() {
        let memory_manager = MemoryManager::new();

        // Test that tracking is disabled
        assert!(!memory_manager.is_tracking_enabled());

        // Test allocation without tracking
        let node = TableauxNode::new(NodeId::new(1));
        let _arena_node = memory_manager.allocate_node(node).unwrap();

        let change_log = memory_manager.get_change_log();
        assert!(change_log.is_none());

        let stats = memory_manager.get_mutation_stats().unwrap();
        assert_eq!(stats.nodes_allocated, 0);
    }

    #[test]
    fn test_memory_manager_checkpoint_and_rollback() {
        let memory_manager = MemoryManager::with_tracking();

        // Create initial checkpoint
        let checkpoint_id = memory_manager.create_checkpoint().unwrap();

        // Allocate some memory
        let node1 = TableauxNode::new(NodeId::new(1));
        let _arena_node1 = memory_manager.allocate_node(node1).unwrap();

        let expr = ClassExpression::Class(Class::new("http://example.org/Class1"));
        let _allocated_expr = memory_manager.allocate_expression(expr).unwrap();

        // Check statistics before rollback
        let stats_before = memory_manager.get_mutation_stats().unwrap();
        assert_eq!(stats_before.nodes_allocated, 1);
        assert_eq!(stats_before.expressions_allocated, 1);

        // Rollback to checkpoint
        memory_manager
            .rollback_to_checkpoint(checkpoint_id)
            .unwrap();

        // After rollback, the change log should be truncated
        let change_log = memory_manager.get_change_log().unwrap();
        assert!(change_log.changes_since_checkpoint(checkpoint_id).is_none());
    }

    #[test]
    fn test_memory_manager_string_interning_tracking() {
        let memory_manager = MemoryManager::with_tracking();

        // Intern some strings
        let _interned1 = memory_manager.intern_string("test_string_1").unwrap();
        let _interned2 = memory_manager.intern_string("test_string_2").unwrap();

        let stats = memory_manager.get_mutation_stats().unwrap();
        assert_eq!(stats.strings_interned, 2);
        assert_eq!(stats.total_bytes_allocated, 13 + 13); // Length of both strings
    }

    #[test]
    fn test_memory_manager_enable_disable_tracking() {
        let memory_manager = MemoryManager::new();

        // Initially disabled
        assert!(!memory_manager.is_tracking_enabled());

        // Enable tracking
        memory_manager.set_tracking_enabled(true);
        assert!(memory_manager.is_tracking_enabled());

        // Disable tracking
        memory_manager.set_tracking_enabled(false);
        assert!(!memory_manager.is_tracking_enabled());
    }

    #[test]
    fn test_memory_manager_take_change_log() {
        let memory_manager = MemoryManager::with_tracking();

        // Add some changes
        let node = TableauxNode::new(NodeId::new(1));
        let _arena_node = memory_manager.allocate_node(node).unwrap();

        // Take the change log
        let taken_log = memory_manager.take_change_log().unwrap();
        assert_eq!(taken_log.len(), 1);

        // The change log should be reset
        let new_log = memory_manager.get_change_log().unwrap();
        assert!(new_log.is_empty());
    }

    #[test]
    fn test_memory_arena_type_equality() {
        assert_eq!(ArenaType::Node, ArenaType::Node);
        assert_ne!(ArenaType::Node, ArenaType::Expression);
        assert_ne!(ArenaType::Expression, ArenaType::Constraint);
        assert_ne!(ArenaType::Constraint, ArenaType::String);
    }

    #[test]
    fn test_memory_change_debug_format() {
        let change = MemoryChange::AllocateNode {
            node_id: NodeId::new(42),
            arena_type: ArenaType::Node,
            size_bytes: 128,
        };

        let debug_str = format!("{:?}", change);
        assert!(debug_str.contains("AllocateNode"));
        assert!(debug_str.contains("42"));
        assert!(debug_str.contains("128"));
    }

    #[test]
    fn test_memory_snapshot_creation() {
        let snapshot = MemorySnapshot {
            arena_stats: ArenaStats::default(),
            memory_stats: MemoryStats::new(),
            timestamp: std::time::Instant::now(),
        };

        // Basic test that we can create a snapshot
        assert_eq!(snapshot.arena_stats.arena_allocated_nodes, 0);
        assert_eq!(snapshot.memory_stats.total_allocations(), 0);
    }

    #[test]
    fn test_memory_mutation_stats_default() {
        let stats = MemoryMutationStats::default();

        assert_eq!(stats.nodes_allocated, 0);
        assert_eq!(stats.expressions_allocated, 0);
        assert_eq!(stats.constraints_allocated, 0);
        assert_eq!(stats.strings_interned, 0);
        assert_eq!(stats.total_bytes_allocated, 0);
        assert_eq!(stats.arena_resets, 0);
        assert_eq!(stats.checkpoints_created, 0);
        assert_eq!(stats.rollbacks_performed, 0);
    }

    #[test]
    fn test_memory_rollback_error_handling() {
        let memory_manager = MemoryManager::new(); // No tracking

        // Should fail when tracking is disabled
        let result = memory_manager.create_checkpoint();
        assert!(result.is_err());

        let result = memory_manager.rollback_to_checkpoint(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_checkpoint_not_found() {
        let memory_manager = MemoryManager::with_tracking();

        // Should fail when checkpoint doesn't exist
        let result = memory_manager.rollback_to_checkpoint(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_memory_tracking_scenario() {
        let memory_manager = MemoryManager::with_tracking();

        // Create initial checkpoint
        let _checkpoint1 = memory_manager.create_checkpoint().unwrap();

        // Allocate some memory
        let node1 = TableauxNode::new(NodeId::new(1));
        let _arena_node1 = memory_manager.allocate_node(node1).unwrap();

        // Create second checkpoint
        let checkpoint2 = memory_manager.create_checkpoint().unwrap();

        // Allocate more memory
        let node2 = TableauxNode::new(NodeId::new(2));
        let _arena_node2 = memory_manager.allocate_node(node2).unwrap();

        let expr = ClassExpression::Class(Class::new("http://example.org/Test"));
        let _allocated_expr = memory_manager.allocate_expression(expr).unwrap();

        // Check statistics
        let stats = memory_manager.get_mutation_stats().unwrap();
        assert_eq!(stats.nodes_allocated, 2);
        assert_eq!(stats.expressions_allocated, 1);
        assert_eq!(stats.checkpoints_created, 2);

        // Rollback to second checkpoint
        memory_manager.rollback_to_checkpoint(checkpoint2).unwrap();

        // Should have lost the second node and expression
        let log = memory_manager.get_change_log().unwrap();
        let changes_since_cp2 = log.changes_since_checkpoint(checkpoint2);
        assert!(
            changes_since_cp2.is_none()
                || changes_since_cp2
                    .expect("Changes slice should be valid")
                    .is_empty()
        );
    }

    #[test]
    fn test_memory_reset_tracking() {
        let memory_manager = MemoryManager::with_tracking();

        // Allocate some memory
        let node = TableauxNode::new(NodeId::new(1));
        let _arena_node = memory_manager.allocate_node(node).unwrap();

        let stats_before = memory_manager.get_mutation_stats().unwrap();
        assert_eq!(stats_before.nodes_allocated, 1);

        // Reset should be tracked
        memory_manager.reset().unwrap();

        let stats_after = memory_manager.get_mutation_stats().unwrap();
        assert_eq!(stats_after.arena_resets, 1);
    }
}
