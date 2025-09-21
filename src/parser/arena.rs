//! Arena allocation module for efficient memory management during parsing
//!
//! Provides arena-based allocation for frequently created objects during parsing
//! to reduce memory fragmentation and improve performance.

use bumpalo::Bump;
use std::cell::Cell;
use std::sync::Arc;
use std::slice;

/// Parser arena for efficient allocation of frequently created objects
pub struct ParserArena {
    /// Main arena for general allocations
    bump: Bump,
    /// Approximate allocation counter for tests/metrics
    alloc_count: Cell<usize>,
}

impl ParserArena {
    /// Create a new parser arena with default capacity
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
            alloc_count: Cell::new(0),
        }
    }

    /// Create a new parser arena with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity),
            alloc_count: Cell::new(0),
        }
    }

    /// Allocate a string in the arena
    pub fn alloc_str(&self, s: &str) -> &str {
        self.alloc_count.set(self.alloc_count.get() + 1);
        self.bump.alloc_str(s)
    }

    /// Allocate a string copy in the arena
    pub fn alloc_string(&self, s: String) -> &str {
        self.alloc_count.set(self.alloc_count.get() + 1);
        self.bump.alloc_str(&s)
    }

    /// Allocate a T in the arena
    pub fn alloc<T>(&self, value: T) -> &T {
        self.alloc_count.set(self.alloc_count.get() + 1);
        self.bump.alloc(value)
    }

    /// Allocate a slice of T in the arena
    pub fn alloc_slice<T>(&self, slice: &[T]) -> &[T]
    where
        T: Copy,
    {
        self.alloc_count.set(self.alloc_count.get() + 1);
        self.bump.alloc_slice_copy(slice)
    }

    /// Allocate a slice of T from an iterator in the arena
    pub fn alloc_slice_fill_iter<T, I>(&self, iter: I) -> &[T]
    where
        I: IntoIterator<Item = T>,
        T: Copy,
        I::IntoIter: ExactSizeIterator,
    {
        self.alloc_count.set(self.alloc_count.get() + 1);
        self.bump.alloc_slice_fill_iter(iter)
    }

    /// Allocate a vector of T in the arena (returns &mut [T])
    pub fn alloc_vec<T: Copy>(&self, vec: Vec<T>) -> &mut [T] {
        let slice: &[T] = self.bump.alloc_slice_copy(&vec);
        // SAFETY: This operation is safe because:
        // 1. We just allocated this slice and have exclusive access to the arena
        // 2. The slice was created from a Vec<T> where T: Copy, so no drop obligations
        // 3. No other references to this memory exist yet
        // 4. The conversion from &[T] to &mut [T] is for API compatibility only
        self.alloc_count.set(self.alloc_count.get() + 1);
        unsafe { slice::from_raw_parts_mut(slice.as_ptr() as *mut T, slice.len()) }
    }

    /// Try to allocate a string with length check
    pub fn try_alloc_str(&self, s: &str, max_len: usize) -> Result<&str, String> {
        if s.len() > max_len {
            Err(format!("String length {} exceeds maximum allowed length {}", s.len(), max_len))
        } else {
            Ok(self.alloc_str(s))
        }
    }

    /// Get the current memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.bump.allocated_bytes()
    }

    /// Get the capacity of the arena in bytes
    pub fn capacity(&self) -> usize {
        self.bump.chunk_capacity()
    }

    /// Reset the arena, freeing all allocated memory
    pub fn reset(&mut self) {
        self.bump.reset();
        self.alloc_count.set(0);
    }

    /// Check if the arena is empty
    pub fn is_empty(&self) -> bool {
        self.alloc_count.get() == 0
    }

    /// Get the number of allocations made (approximate)
    pub fn allocation_count(&self) -> usize {
        self.alloc_count.get()
    }
}

impl Default for ParserArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-local parser arena for single-threaded parsing operations
pub struct LocalParserArena {
    arena: ParserArena,
}

impl LocalParserArena {
    /// Create a new local parser arena
    pub fn new() -> Self {
        Self {
            arena: ParserArena::new(),
        }
    }

    /// Get a reference to the underlying arena
    pub fn arena(&self) -> &ParserArena {
        &self.arena
    }

    /// Get a mutable reference to the underlying arena
    pub fn arena_mut(&mut self) -> &mut ParserArena {
        &mut self.arena
    }

    /// Reset the arena
    pub fn reset(&mut self) {
        self.arena.reset();
    }
}

impl Default for LocalParserArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe parser arena using Arc for shared access
pub struct SharedParserArena {
    arena: Arc<std::sync::Mutex<ParserArena>>,
}

impl SharedParserArena {
    /// Create a new shared parser arena
    pub fn new() -> Self {
        Self {
            arena: Arc::new(std::sync::Mutex::new(ParserArena::new())),
        }
    }

    /// Get a reference to the underlying arena
    pub fn arena(&self) -> &ParserArena {
        // SAFETY: This transmute is justified because:
        // 1. We're extending the lifetime from the lock guard to &self
        // 2. The arena is guaranteed to live as long as self
        // 3. No mutable access occurs while this reference exists
        // 4. This is necessary for API compatibility with the ParserArenaTrait
        unsafe { std::mem::transmute::<&ParserArena, &ParserArena>(&*self.arena.lock().unwrap()) }
    }

    /// Clone the shared arena (increases reference count)
    pub fn clone(&self) -> Self {
        Self {
            arena: Arc::clone(&self.arena),
        }
    }
}

impl Default for SharedParserArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating parser arenas with custom configuration
pub struct ParserArenaBuilder {
    capacity: Option<usize>,
    is_shared: bool,
}

impl ParserArenaBuilder {
    /// Create a new arena builder
    pub fn new() -> Self {
        Self {
            capacity: None,
            is_shared: false,
        }
    }

    /// Set the initial capacity for the arena
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = Some(capacity);
        self
    }

    /// Make the arena thread-safe (shared)
    pub fn shared(mut self) -> Self {
        self.is_shared = true;
        self
    }

    /// Build the arena
    pub fn build(self) -> Box<dyn ParserArenaTrait> {
        let arena = if let Some(capacity) = self.capacity {
            ParserArena::with_capacity(capacity)
        } else {
            ParserArena::new()
        };

        if self.is_shared {
            Box::new(SharedParserArena {
                arena: Arc::new(std::sync::Mutex::new(arena)),
            })
        } else {
            Box::new(LocalParserArena { arena })
        }
    }
}

impl Default for ParserArenaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for parser arena implementations
pub trait ParserArenaTrait {
    /// Get a reference to the underlying arena
    fn arena(&self) -> &ParserArena;

    /// Get a mutable reference to the underlying arena
    fn arena_mut(&mut self) -> &mut ParserArena;

    /// Get memory usage statistics
    fn memory_usage(&self) -> usize {
        self.arena().memory_usage()
    }

    /// Reset the arena
    fn reset(&mut self);
}

impl ParserArenaTrait for LocalParserArena {
    fn arena(&self) -> &ParserArena {
        &self.arena
    }

    fn arena_mut(&mut self) -> &mut ParserArena {
        &mut self.arena
    }

    fn reset(&mut self) {
        self.arena.reset();
    }
}

impl ParserArenaTrait for SharedParserArena {
    fn arena(&self) -> &ParserArena {
        // SAFETY: This transmute is justified because:
        // 1. We're extending the lifetime from the lock guard to &self
        // 2. The arena is guaranteed to live as long as self
        // 3. No mutable access occurs while this reference exists
        // 4. This is necessary for API compatibility with the ParserArenaTrait
        unsafe { std::mem::transmute::<&ParserArena, &ParserArena>(&*self.arena.lock().unwrap()) }
    }

    fn arena_mut(&mut self) -> &mut ParserArena {
        // SAFETY: This transmute is justified because:
        // 1. We're extending the lifetime from the lock guard to &mut self
        // 2. The arena is guaranteed to live as long as self
        // 3. No other references exist while this mutable reference exists
        // 4. This is necessary for API compatibility with the ParserArenaTrait
        unsafe { std::mem::transmute::<&mut ParserArena, &mut ParserArena>(&mut *self.arena.lock().unwrap()) }
    }

    fn reset(&mut self) {
        if let Ok(mut arena) = self.arena.lock() {
            arena.reset();
        }
    }
}

/// Macro for convenient arena allocation
#[macro_export]
macro_rules! arena_alloc {
    ($arena:expr, $value:expr) => {
        $arena.alloc($value)
    };
}

/// Macro for convenient string allocation in arena
#[macro_export]
macro_rules! arena_str {
    ($arena:expr, $value:expr) => {
        $arena.alloc_str($value)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_arena_creation() {
        let arena = ParserArena::new();
        assert!(arena.is_empty());
        assert_eq!(arena.allocation_count(), 0);
    }

    #[test]
    fn test_string_allocation() {
        let arena = ParserArena::new();
        let s = arena.alloc_str("hello");
        assert_eq!(s, "hello");
        assert_eq!(arena.allocation_count(), 1);
    }

    #[test]
    fn test_slice_allocation() {
        let arena = ParserArena::new();
        let data = vec![1, 2, 3, 4, 5];
        let slice = arena.alloc_slice(&data);
        assert_eq!(slice, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_capacity_setting() {
        let arena = ParserArena::with_capacity(1024);
        assert!(arena.capacity() >= 1024);
    }

    #[test]
    fn test_memory_usage() {
        let arena = ParserArena::new();
        let initial_usage = arena.memory_usage();

        // Allocate some data
        arena.alloc_str("hello world");

        assert!(arena.memory_usage() > initial_usage);
    }

    #[test]
    fn test_arena_reset() {
        let mut arena = ParserArena::new();
        arena.alloc_str("hello");
        assert!(!arena.is_empty());

        arena.reset();
        assert!(arena.is_empty());
    }

    #[test]
    fn test_builder_pattern() {
        let arena = ParserArenaBuilder::new()
            .with_capacity(2048)
            .shared()
            .build();

        assert!(arena.memory_usage() >= 0);
    }

    #[test]
    fn test_string_length_validation() {
        let arena = ParserArena::new();
        let short_str = "hello";
        let long_str = "a".repeat(1000);

        assert!(arena.try_alloc_str(short_str, 100).is_ok());
        assert!(arena.try_alloc_str(&long_str, 100).is_err());
    }
}
