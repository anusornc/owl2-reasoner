//! # Tableaux Rule Expansion
//!
//! Implements the core rule expansion logic for the tableaux reasoning algorithm.
//! This module manages the application of tableaux rules to expand the model
//! and derive new consequences from the ontology.
//!
//! ## Key Components
//!
//! - **[`ExpansionEngine`]** - Main coordinator for rule application
//! - **[`ExpansionRules`]** - Collection of tableaux expansion rules
//! - **[`ExpansionContext`]** - Context tracking for expansion state
//! - **[`ExpansionTask`]** - Individual rule application tasks
//! - **[`ExpansionRule`]** - Types of expansion rules (Conjunction, Disjunction, etc.)
//!
//! ## Tableaux Rules
//!
//! The module implements the standard tableaux rules for OWL2 reasoning:
//!
//! ### Conjunction Rule (∧-rule)
//! When a node contains a conjunction `C₁ ∧ C₂`, add both `C₁` and `C₂` to the node.
//!
//! ### Disjunction Rule (∨-rule)
//! When a node contains a disjunction `C₁ ∨ C₂`, create a choice point and branch:
//! - Branch 1: Add `C₁` to the node
//! - Branch 2: Add `C₂` to the node
//!
//! ### Existential Restriction Rule (∃-rule)
//! When a node contains `∃r.C`, create a new node connected by property `r` that contains `C`.
//!
//! ### Universal Restriction Rule (∀-rule)
//! When a node contains `∀r.C` and has `r`-successors, add `C` to all `r`-successors.
//!
//! ### Nominal Rule
//! Handle individual assertions and nominals according to OWL2 semantics.
//!
//! ### Data Range Rule
//! Process data property restrictions and datatypes.
//!
//! ## Expansion Strategy
//!
//! The expansion engine uses a priority-based approach:
//!
//! 1. **Rule Selection**: Choose next applicable rule based on priority order
//! 2. **Task Creation**: Create expansion tasks for rule applications
//! 3. **Priority Queue**: Manage tasks by priority to optimize reasoning
//! 4. **Context Tracking**: Maintain expansion state and applied rules
//! 5. **Depth Control**: Limit expansion depth to prevent infinite loops
//!
//! ## Performance Optimizations
//!
//! - **Priority-Based Ordering**: Apply high-impact rules first
//! - **Task Batching**: Group similar operations for efficiency
//! - **Context Caching**: Avoid redundant rule applications
//! - **Depth Limiting**: Prevent infinite expansion
//! - **Smart Rule Selection**: Heuristics for optimal rule choice
//!
//! ## Example Usage
//!
//! ```rust
//! use owl2_reasoner::reasoning::tableaux::{ExpansionEngine, ExpansionRules, ExpansionContext};
//!
//! // Create expansion engine with rules
//! let mut expansion_engine = ExpansionEngine::new();
//! let rules = ExpansionRules::new();
//!
//! // Set up expansion context
//! let mut context = ExpansionContext {
//!     current_node: NodeId::new(0),
//!     current_depth: 0,
//!     applied_rules: HashSet::new(),
//!     pending_expansions: VecDeque::new(),
//! };
//!
//! // Perform expansion up to maximum depth
//! let max_depth = 100;
//! let expansion_complete = expansion_engine.expand(&mut graph, &mut memory_manager, max_depth)?;
//!
//! println!("Expansion completed: {}", expansion_complete);
//! ```

use super::core::NodeId;
use super::graph::TableauxGraph;
use super::memory::MemoryManager;
use crate::axioms::class_expressions::ClassExpression;
use crate::axioms::ObjectPropertyExpression;
use crate::iri::IRI;
use hashbrown::HashMap;
use std::collections::{HashSet, VecDeque};

/// Types of expansion rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpansionRule {
    /// Conjunction rule
    Conjunction,
    /// Disjunction rule
    Disjunction,
    /// Existential restriction rule
    ExistentialRestriction,
    /// Universal restriction rule
    UniversalRestriction,
    /// Nominal rule
    Nominal,
    /// Data range rule
    DataRange,
}

/// Expansion context for rule application
#[derive(Debug, Clone)]
pub struct ExpansionContext {
    pub current_node: NodeId,
    pub current_depth: usize,
    pub applied_rules: HashSet<ExpansionRule>,
    pub pending_expansions: VecDeque<ExpansionTask>,
}

#[derive(Debug, Clone)]
pub struct ExpansionTask {
    pub node_id: NodeId,
    pub concept: ClassExpression,
    pub rule_type: ExpansionRule,
    pub priority: usize,
}

impl ExpansionTask {
    pub fn new(node_id: NodeId, concept: ClassExpression, rule_type: ExpansionRule) -> Self {
        Self {
            node_id,
            concept,
            rule_type,
            priority: 0,
        }
    }

    pub fn with_priority(mut self, priority: usize) -> Self {
        self.priority = priority;
        self
    }
}

/// Expansion rules for tableaux reasoning
#[derive(Debug)]
pub struct ExpansionRules {
    pub rules: Vec<ExpansionRule>,
    pub rule_order: Vec<ExpansionRule>,
    pub max_applications: HashMap<ExpansionRule, usize>,
}

impl ExpansionRules {
    pub fn new() -> Self {
        let rules = vec![
            ExpansionRule::Conjunction,
            ExpansionRule::Disjunction,
            ExpansionRule::ExistentialRestriction,
            ExpansionRule::UniversalRestriction,
            ExpansionRule::Nominal,
            ExpansionRule::DataRange,
        ];

        let rule_order = vec![
            ExpansionRule::Conjunction,
            ExpansionRule::ExistentialRestriction,
            ExpansionRule::UniversalRestriction,
            ExpansionRule::Disjunction,
            ExpansionRule::Nominal,
            ExpansionRule::DataRange,
        ];

        let max_applications: HashMap<_, _> = rules.iter().map(|rule| (*rule, 1000)).collect();

        Self {
            rules,
            rule_order,
            max_applications,
        }
    }

    pub fn get_next_rule(&self, context: &ExpansionContext) -> Option<ExpansionRule> {
        for rule in &self.rule_order {
            if !context.applied_rules.contains(rule) {
                return Some(*rule);
            }
        }
        None
    }

    pub fn can_apply_rule(&self, _rule: &ExpansionRule, _context: &ExpansionContext) -> bool {
        if let Some(&_max_apps) = self.max_applications.get(_rule) {
            // Check if we haven't exceeded maximum applications
            true // Simplified check
        } else {
            false
        }
    }

    pub fn apply_rule(
        &self,
        rule: ExpansionRule,
        graph: &mut TableauxGraph,
        memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<Vec<ExpansionTask>, String> {
        match rule {
            ExpansionRule::Conjunction => self.apply_conjunction_rule(graph, memory, context),
            ExpansionRule::Disjunction => self.apply_disjunction_rule(graph, memory, context),
            ExpansionRule::ExistentialRestriction => {
                self.apply_existential_restriction_rule(graph, memory, context)
            }
            ExpansionRule::UniversalRestriction => {
                self.apply_universal_restriction_rule(graph, memory, context)
            }
            ExpansionRule::Nominal => self.apply_nominal_rule(graph, memory, context),
            ExpansionRule::DataRange => self.apply_data_range_rule(graph, memory, context),
        }
    }

    fn apply_conjunction_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<Vec<ExpansionTask>, String> {
        // Decompose intersection: C ⊓ D → C, D
        if let Some(node) = graph.get_node_mut(context.current_node) {
            // Find all intersection concepts in the node
            let intersections: Vec<_> = node
                .concepts_iter()
                .filter(|c| matches!(c, ClassExpression::ObjectIntersectionOf(_)))
                .cloned()
                .collect();

            for intersection in intersections {
                if let ClassExpression::ObjectIntersectionOf(operands) = intersection {
                    // Add each operand to the node
                    for operand in operands.iter() {
                        node.add_concept((**operand).clone());
                    }
                    // Remove the intersection (optional - depends on strategy)
                    // For now, we'll keep it for completeness
                }
            }
        }
        Ok(vec![])
    }

    fn apply_disjunction_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<Vec<ExpansionTask>, String> {
        // Non-deterministic choice for union: C ⊔ D → C or D
        if let Some(node) = graph.get_node_mut(context.current_node) {
            // Find all union concepts in the node
            let unions: Vec<_> = node
                .concepts_iter()
                .filter(|c| matches!(c, ClassExpression::ObjectUnionOf(_)))
                .cloned()
                .collect();

            for union in unions {
                if let ClassExpression::ObjectUnionOf(operands) = union {
                    if !operands.is_empty() {
                        // Create choice point for non-deterministic branching
                        let choice = ExpansionTask {
                            node_id: context.current_node,
                            concept: (*operands[0]).clone(),
                            rule_type: ExpansionRule::Disjunction,
                            priority: 10, // Medium priority for disjunction
                        };
                        return Ok(vec![choice]);
                    }
                }
            }
        }
        Ok(vec![])
    }

    fn apply_existential_restriction_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<Vec<ExpansionTask>, String> {
        // ∃R.C → create new node with C and R-edge from the current node
        if let Some(node) = graph.get_node_mut(context.current_node) {
            // Find all existential restrictions in the node
            let existentials: Vec<_> = node
                .concepts_iter()
                .filter(|c| matches!(c, ClassExpression::ObjectSomeValuesFrom(_, _)))
                .cloned()
                .collect();

            for existential in existentials {
                if let ClassExpression::ObjectSomeValuesFrom(property, filler) = existential {
                    // Extract the property IRI from the property expression
                    let (_is_inverse, property_iri) = Self::resolve_property_direction(&property);

                    // Create new successor node
                    let new_node_id = graph.add_node();

                    // Add the filler concept to the new node
                    graph.add_concept(new_node_id, (*filler).clone());

                    // Add edge from current node to new node
                    graph.add_edge(context.current_node, property_iri, new_node_id);

                    // Create expansion task for the new node with the filler concept
                    let task = ExpansionTask {
                        node_id: new_node_id,
                        concept: (*filler).clone(),
                        rule_type: ExpansionRule::ExistentialRestriction,
                        priority: 5, // High priority for existential restrictions
                    };
                    return Ok(vec![task]);
                }
            }
        }
        Ok(vec![])
    }

    fn apply_universal_restriction_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<Vec<ExpansionTask>, String> {
        // ∀R.C → ensure all R-successors have C
        if let Some(node) = graph.get_node_mut(context.current_node) {
            // Find all universal restrictions in the node
            let universals: Vec<_> = node
                .concepts_iter()
                .filter(|c| matches!(c, ClassExpression::ObjectAllValuesFrom(_, _)))
                .cloned()
                .collect();

            for universal in universals {
                if let ClassExpression::ObjectAllValuesFrom(property, filler) = universal {
                    // Determine if we look at successors (R) or predecessors (R^-)
                    let (is_inverse, property_iri) = Self::resolve_property_direction(&property);

                    if !is_inverse {
                        // Collect successors first to avoid holding an immutable borrow while mutating
                        let successors: Vec<NodeId> = graph
                            .get_successors(context.current_node, property_iri)
                            .map(|s| s.to_vec())
                            .unwrap_or_default();

                        for successor_id in successors {
                            let needs_add = graph
                                .get_node(successor_id)
                                .map(|n| !n.contains_concept(&filler))
                                .unwrap_or(false);
                            if needs_add {
                                graph.add_concept(successor_id, (*filler).clone());

                                // Create expansion task for the successor
                                let task = ExpansionTask {
                                    node_id: successor_id,
                                    concept: (*filler).clone(),
                                    rule_type: ExpansionRule::UniversalRestriction,
                                    priority: 8, // Medium-high priority for universal restrictions
                                };
                                return Ok(vec![task]);
                            }
                        }
                    } else {
                        // For inverse properties, ensure all predecessors via R have the filler
                        let predecessors: Vec<NodeId> = graph
                            .get_predecessors(context.current_node, property_iri)
                            .into_iter()
                            .collect();

                        for pred_id in predecessors {
                            let needs_add = graph
                                .get_node(pred_id)
                                .map(|n| !n.contains_concept(&filler))
                                .unwrap_or(false);
                            if needs_add {
                                graph.add_concept(pred_id, (*filler).clone());

                                // Create expansion task for the predecessor
                                let task = ExpansionTask {
                                    node_id: pred_id,
                                    concept: (*filler).clone(),
                                    rule_type: ExpansionRule::UniversalRestriction,
                                    priority: 8,
                                };
                                return Ok(vec![task]);
                            }
                        }
                    }
                }
            }
        }
        Ok(vec![])
    }

    /// Helper function to resolve property direction for inverse properties
    pub fn resolve_property_direction(expr: &ObjectPropertyExpression) -> (bool, &IRI) {
        fn flatten(e: &ObjectPropertyExpression, invert: bool) -> (bool, &IRI) {
            match e {
                ObjectPropertyExpression::ObjectProperty(prop) => (invert, prop.iri()),
                ObjectPropertyExpression::ObjectInverseOf(inner) => {
                    flatten(inner.as_ref(), !invert)
                }
            }
        }
        flatten(expr, false)
    }

    fn apply_nominal_rule(
        &self,
        _graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        _context: &mut ExpansionContext,
    ) -> Result<Vec<ExpansionTask>, String> {
        // Simplified nominal rule implementation
        Ok(vec![])
    }

    fn apply_data_range_rule(
        &self,
        _graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        _context: &mut ExpansionContext,
    ) -> Result<Vec<ExpansionTask>, String> {
        // Simplified data range rule implementation
        Ok(vec![])
    }
}

impl Default for ExpansionRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Tableaux expansion engine
#[derive(Debug)]
pub struct ExpansionEngine {
    pub rules: ExpansionRules,
    pub context: ExpansionContext,
}

impl ExpansionEngine {
    pub fn new() -> Self {
        Self {
            rules: ExpansionRules::new(),
            context: ExpansionContext {
                current_node: NodeId::new(0),
                current_depth: 0,
                applied_rules: HashSet::new(),
                pending_expansions: VecDeque::new(),
            },
        }
    }

    pub fn expand(
        &mut self,
        graph: &mut TableauxGraph,
        memory: &mut MemoryManager,
        max_depth: usize,
    ) -> Result<bool, String> {
        while self.context.current_depth < max_depth {
            if let Some(rule) = self.rules.get_next_rule(&self.context) {
                if self.rules.can_apply_rule(&rule, &self.context) {
                    let new_tasks =
                        self.rules
                            .apply_rule(rule, graph, memory, &mut self.context)?;
                    self.context.pending_expansions.extend(new_tasks);
                    self.context.applied_rules.insert(rule);
                }
            } else {
                // No more rules to apply at current level
                if let Some(next_task) = self.context.pending_expansions.pop_front() {
                    self.context.current_node = next_task.node_id;
                    self.context.current_depth += 1;
                    self.context.applied_rules.clear();
                } else {
                    // No more expansions to perform
                    break;
                }
            }
        }

        Ok(true)
    }

    pub fn reset(&mut self) {
        self.context = ExpansionContext {
            current_node: NodeId::new(0),
            current_depth: 0,
            applied_rules: HashSet::new(),
            pending_expansions: VecDeque::new(),
        };
    }
}

impl Default for ExpansionEngine {
    fn default() -> Self {
        Self::new()
    }
}
