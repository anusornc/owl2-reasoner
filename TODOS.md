# Project Todos

## Active
- [ ] Refactor tableaux.rs - Split 2,981-line file into logical submodules | Due: next week
- [ ] Fix unsafe code in profiles.rs line 1934 - Replace with safe alternative | Due: next week
- [ ] Document remaining unsafe code blocks in tableaux.rs and memory.rs | Due: next week
- [ ] Create tableaux core module - Extract core data structures and config | Due: 2 weeks
- [ ] Create tableaux graph module - Separate graph operations and edge management | Due: 2 weeks
- [ ] Create tableaux memory module - Isolate arena allocation logic | Due: 3 weeks
- [ ] Create tableaux blocking module - Separate blocking strategy logic | Due: 3 weeks
- [ ] Create tableaux dependency module - Extract dependency management | Due: 3 weeks
- [ ] Create tableaux expansion module - Separate rule expansion logic | Due: 4 weeks

## Completed
- [x] Fix all 152 Clippy linting errors (unused variables, large enum variants, redundant closures) | Due: 2025-01-29
- [x] Refactor axiom_type() method with 46 match arms in src/axioms/mod.rs:172-219 | Due: 2025-01-31
- [x] Break down 78-line match_triple_pattern_optimized method in src/reasoning/query.rs:305-383 | Due: 2025-01-31
- [x] Box large enum variants for memory efficiency (ObjectOneOf, ObjectProperty) | Due: 2025-02-05
- [x] Replace all unwrap() and expect() calls with proper error handling | Due: 2025-02-07
- [x] Document unsafe code blocks in reasoning/tableaux.rs, parser/turtle.rs, parser/arena.rs | Due: 2025-02-10
- [x] Define constants for magic numbers (file sizes, timeouts, RDF vocabulary) | Due: 2025-02-14
- [x] Add comprehensive documentation for public APIs | Due: 2025-02-21
- [x] Implement property-based testing for edge cases | Due: 2025-02-28
- [x] Add performance benchmarks and regression tests | Due: 2025-03-07
- [x] Create centralized configuration module | Due: 2025-03-14
- [x] Standardize naming conventions across modules | Due: 2025-03-21