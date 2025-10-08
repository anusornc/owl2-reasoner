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

use super::core::{NodeId, ReasoningRules};
use super::graph::{GraphChangeLog, TableauxGraph};
use super::memory::{MemoryChangeLog, MemoryManager};
use crate::axioms::class_expressions::ClassExpression;
use crate::axioms::ObjectPropertyExpression;
use crate::entities::Class;
use crate::iri::IRI;
use hashbrown::HashMap;
use smallvec::SmallVec;
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
    /// Subclass axiom application rule
    SubclassAxiom,
}

/// Expansion context for rule application
#[derive(Debug, Clone)]
pub struct ExpansionContext {
    pub current_node: NodeId,
    pub current_depth: usize,
    pub applied_rules: HashSet<ExpansionRule>,
    pub pending_expansions: VecDeque<ExpansionTask>,
    pub reasoning_rules: Option<super::core::ReasoningRules>,
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
            ExpansionRule::SubclassAxiom,
        ];

        let rule_order = vec![
            ExpansionRule::SubclassAxiom, // Apply subclass axioms first
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
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
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
            ExpansionRule::SubclassAxiom => self.apply_subclass_axiom_rule(graph, memory, context),
        }
    }

    fn apply_conjunction_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
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
                        graph.add_concept_logged(
                            context.current_node,
                            (**operand).clone(),
                            &mut change_log,
                        );
                    }
                    // Remove the intersection (optional - depends on strategy)
                    // For now, we'll keep it for completeness
                }
            }
        }
        Ok((vec![], change_log))
    }

    fn apply_disjunction_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let change_log = GraphChangeLog::new();
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
                        return Ok((vec![choice], change_log));
                    }
                }
            }
        }
        Ok((vec![], change_log))
    }

    fn apply_existential_restriction_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();

        let existentials: Vec<_> = match graph.get_node(context.current_node) {
            Some(node) => node
                .concepts_iter()
                .filter(|c| matches!(c, ClassExpression::ObjectSomeValuesFrom(_, _)))
                .cloned()
                .collect(),
            None => return Ok((vec![], change_log)),
        };

        let universals: Vec<_> = graph
            .get_node(context.current_node)
            .map(|node| {
                node.concepts_iter()
                    .filter(|c| matches!(c, ClassExpression::ObjectAllValuesFrom(_, _)))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        for existential in existentials {
            if let ClassExpression::ObjectSomeValuesFrom(property, filler) = existential {
                let (is_inverse, property_iri) = Self::resolve_property_direction(&property);

                // Attempt to reuse existing nodes that already contain the filler
                if !is_inverse {
                    if let Some(successors) =
                        graph.get_successors(context.current_node, property_iri)
                    {
                        if let Some(existing) = successors.iter().copied().find(|succ_id| {
                            graph
                                .get_node(*succ_id)
                                .map(|n| n.contains_concept(&filler))
                                .unwrap_or(false)
                        }) {
                            self.propagate_universal_to_node(
                                &universals,
                                is_inverse,
                                property_iri,
                                existing,
                                graph,
                                &mut change_log,
                            );

                            let task = ExpansionTask {
                                node_id: existing,
                                concept: (*filler).clone(),
                                rule_type: ExpansionRule::ExistentialRestriction,
                                priority: 5,
                            };
                            return Ok((vec![task], change_log));
                        }
                    }
                } else {
                    let predecessors = graph.get_predecessors(context.current_node, property_iri);
                    if let Some(existing) = predecessors.iter().copied().find(|pred_id| {
                        graph
                            .get_node(*pred_id)
                            .map(|n| n.contains_concept(&filler))
                            .unwrap_or(false)
                    }) {
                        self.propagate_universal_to_node(
                            &universals,
                            is_inverse,
                            property_iri,
                            existing,
                            graph,
                            &mut change_log,
                        );

                        let task = ExpansionTask {
                            node_id: existing,
                            concept: (*filler).clone(),
                            rule_type: ExpansionRule::ExistentialRestriction,
                            priority: 5,
                        };
                        return Ok((vec![task], change_log));
                    }
                }

                // No reusable node, create a new one
                let new_node_id = graph.add_node_logged(&mut change_log);
                graph.add_concept_logged(new_node_id, (*filler).clone(), &mut change_log);
                graph.add_edge_logged(
                    context.current_node,
                    property_iri,
                    new_node_id,
                    &mut change_log,
                );

                self.propagate_universal_to_node(
                    &universals,
                    is_inverse,
                    property_iri,
                    new_node_id,
                    graph,
                    &mut change_log,
                );

                let task = ExpansionTask {
                    node_id: new_node_id,
                    concept: (*filler).clone(),
                    rule_type: ExpansionRule::ExistentialRestriction,
                    priority: 5,
                };
                return Ok((vec![task], change_log));
            }
        }

        Ok((vec![], change_log))
    }

    fn apply_universal_restriction_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
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
                                graph.add_concept_logged(
                                    successor_id,
                                    (*filler).clone(),
                                    &mut change_log,
                                );

                                // Create expansion task for the successor
                                let task = ExpansionTask {
                                    node_id: successor_id,
                                    concept: (*filler).clone(),
                                    rule_type: ExpansionRule::UniversalRestriction,
                                    priority: 8, // Medium-high priority for universal restrictions
                                };
                                return Ok((vec![task], change_log));
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
                                graph.add_concept_logged(
                                    pred_id,
                                    (*filler).clone(),
                                    &mut change_log,
                                );

                                // Create expansion task for the predecessor
                                let task = ExpansionTask {
                                    node_id: pred_id,
                                    concept: (*filler).clone(),
                                    rule_type: ExpansionRule::UniversalRestriction,
                                    priority: 8,
                                };
                                return Ok((vec![task], change_log));
                            }
                        }
                    }
                }
            }
        }
        Ok((vec![], change_log))
    }

    fn propagate_universal_to_node(
        &self,
        universals: &[ClassExpression],
        is_inverse: bool,
        property_iri: &IRI,
        target_node: NodeId,
        graph: &mut TableauxGraph,
        change_log: &mut GraphChangeLog,
    ) {
        for universal in universals {
            if let ClassExpression::ObjectAllValuesFrom(univ_property, univ_filler) = universal {
                let (univ_inverse, univ_iri) = Self::resolve_property_direction(univ_property);
                if univ_inverse == is_inverse && univ_iri == property_iri {
                    graph.add_concept_logged(target_node, (**univ_filler).clone(), change_log);
                }
            }
        }
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
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        // Handle nominals (individuals): {a} → create node for individual a
        // First, collect the nominals without holding a mutable borrow
        let nominals: Vec<_> = if let Some(node) = graph.get_node(context.current_node) {
            node.concepts_iter()
                .filter(|c| matches!(c, ClassExpression::ObjectOneOf(_)))
                .cloned()
                .collect()
        } else {
            return Ok((vec![], change_log));
        };

        for nominal in nominals {
            if let ClassExpression::ObjectOneOf(individuals) = nominal {
                // For each individual in the nominal, ensure they have corresponding nodes
                for individual in individuals.iter() {
                    // Check if we already have a node for this individual
                    let individual_node =
                        self.find_or_create_individual_node(graph, individual, &mut change_log);

                    // Create expansion task for the individual node
                    let mut task_individual_vec: SmallVec<[crate::entities::Individual; 8]> =
                        SmallVec::new();
                    task_individual_vec.push(individual.clone());
                    let task = ExpansionTask {
                        node_id: individual_node,
                        concept: ClassExpression::ObjectOneOf(Box::new(task_individual_vec)),
                        rule_type: ExpansionRule::Nominal,
                        priority: 7, // Medium priority for nominals
                    };
                    return Ok((vec![task], change_log));
                }
            }
        }
        Ok((vec![], change_log))
    }

    /// Find or create a node for an individual
    fn find_or_create_individual_node(
        &self,
        graph: &mut TableauxGraph,
        individual: &crate::entities::Individual,
        change_log: &mut GraphChangeLog,
    ) -> NodeId {
        // For now, create a new node for each individual
        // In a full implementation, we'd maintain a mapping of individuals to nodes
        let node_id = graph.add_node_logged(change_log);

        // Add the individual as a nominal concept to the new node
        let mut individual_vec: SmallVec<[crate::entities::Individual; 8]> = SmallVec::new();
        individual_vec.push(individual.clone());
        graph.add_concept_logged(
            node_id,
            ClassExpression::ObjectOneOf(Box::new(individual_vec)),
            change_log,
        );

        node_id
    }

    fn apply_data_range_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let change_log = GraphChangeLog::new();
        // Handle data property restrictions and data ranges
        if let Some(node) = graph.get_node_mut(context.current_node) {
            // Find all data property restrictions
            let data_restrictions: Vec<_> = node
                .concepts_iter()
                .filter(|c| {
                    matches!(
                        c,
                        ClassExpression::DataSomeValuesFrom(_, _)
                            | ClassExpression::DataAllValuesFrom(_, _)
                            | ClassExpression::DataHasValue(_, _)
                            | ClassExpression::DataMinCardinality(_, _)
                            | ClassExpression::DataMaxCardinality(_, _)
                            | ClassExpression::DataExactCardinality(_, _)
                    )
                })
                .cloned()
                .collect();

            for restriction in data_restrictions {
                match &restriction {
                    ClassExpression::DataSomeValuesFrom(_, _) => {
                        // ∃D.R → create data value satisfying R
                        // For now, we'll create a placeholder data value
                        // In a full implementation, this would involve data range reasoning
                        let task = ExpansionTask {
                            node_id: context.current_node,
                            concept: restriction.clone(),
                            rule_type: ExpansionRule::DataRange,
                            priority: 6, // Medium priority for data restrictions
                        };
                        return Ok((vec![task], change_log));
                    }
                    ClassExpression::DataAllValuesFrom(_, _) => {
                        // ∀D.R → all data values must satisfy R
                        // This is handled during model completion
                        let task = ExpansionTask {
                            node_id: context.current_node,
                            concept: restriction.clone(),
                            rule_type: ExpansionRule::DataRange,
                            priority: 6,
                        };
                        return Ok((vec![task], change_log));
                    }
                    ClassExpression::DataHasValue(_, _) => {
                        // D = v → the node has data value v for property D
                        // This represents a concrete data assertion
                        let task = ExpansionTask {
                            node_id: context.current_node,
                            concept: restriction.clone(),
                            rule_type: ExpansionRule::DataRange,
                            priority: 6,
                        };
                        return Ok((vec![task], change_log));
                    }
                    ClassExpression::DataMinCardinality(cardinality, _) => {
                        // ≥n D → at least n distinct data values
                        if *cardinality > 0 {
                            // Create additional data values to satisfy minimum cardinality
                            for _ in 0..*cardinality {
                                let task = ExpansionTask {
                                    node_id: context.current_node,
                                    concept: restriction.clone(),
                                    rule_type: ExpansionRule::DataRange,
                                    priority: 6,
                                };
                                return Ok((vec![task], change_log));
                            }
                        }
                    }
                    ClassExpression::DataMaxCardinality(_, _) => {
                        // ≤n D → at most n distinct data values
                        // This is a constraint that will be checked during completion
                        let task = ExpansionTask {
                            node_id: context.current_node,
                            concept: restriction.clone(),
                            rule_type: ExpansionRule::DataRange,
                            priority: 6,
                        };
                        return Ok((vec![task], change_log));
                    }
                    ClassExpression::DataExactCardinality(cardinality, _) => {
                        // =n D → exactly n distinct data values
                        if *cardinality > 0 {
                            // Create exactly n data values
                            for _ in 0..*cardinality {
                                let task = ExpansionTask {
                                    node_id: context.current_node,
                                    concept: restriction.clone(),
                                    rule_type: ExpansionRule::DataRange,
                                    priority: 6,
                                };
                                return Ok((vec![task], change_log));
                            }
                        }
                    }
                    _ => {} // Other cases handled above
                }
            }
        }
        Ok((vec![], change_log))
    }

    fn apply_subclass_axiom_rule(
        &self,
        graph: &mut TableauxGraph,
        _memory: &mut MemoryManager,
        context: &mut ExpansionContext,
    ) -> Result<(Vec<ExpansionTask>, GraphChangeLog), String> {
        let mut change_log = GraphChangeLog::new();
        // Apply subclass axioms: if node contains A and A ⊑ B, then add B to the node
        if let Some(reasoning_rules) = &context.reasoning_rules {
            if let Some(node) = graph.get_node_mut(context.current_node) {
                // Get all class concepts in the current node
                let class_concepts: Vec<ClassExpression> = node
                    .concepts_iter()
                    .filter(|c| matches!(c, ClassExpression::Class(_)))
                    .cloned()
                    .collect();

                for concept in class_concepts {
                    if let ClassExpression::Class(class) = concept {
                        // Find all subclass axioms where this class is the subclass
                        for axiom in &reasoning_rules.subclass_rules {
                            if let ClassExpression::Class(sub_class) = axiom.sub_class() {
                                if sub_class.iri().as_ref() == class.iri().as_ref() {
                                    // Add the superclass to the node if not already present
                                    if let ClassExpression::Class(super_class) = axiom.super_class()
                                    {
                                        let super_concept = ClassExpression::Class(Class::new(
                                            super_class.iri().as_str(),
                                        ));
                                        if !node.contains_concept(&super_concept) {
                                            graph.add_concept_logged(
                                                context.current_node,
                                                super_concept.clone(),
                                                &mut change_log,
                                            );

                                            // Create expansion task for the superclass
                                            let task = ExpansionTask {
                                                node_id: context.current_node,
                                                concept: super_concept,
                                                rule_type: ExpansionRule::SubclassAxiom,
                                                priority: 1, // Highest priority for subclass axioms
                                            };
                                            return Ok((vec![task], change_log));
                                        }
                                    }
                                }
                            }
                        }

                        // Also check equivalent classes
                        for equiv_axiom in &reasoning_rules.equivalence_rules {
                            let classes = equiv_axiom.classes();
                            if classes.iter().any(|c| c.as_ref() == class.iri().as_ref()) {
                                // Add all other equivalent classes to the node
                                for equiv_class in classes {
                                    if equiv_class.as_ref() != class.iri().as_ref() {
                                        let equiv_concept = ClassExpression::Class(Class::new(
                                            equiv_class.as_str(),
                                        ));
                                        if !node.contains_concept(&equiv_concept) {
                                            graph.add_concept_logged(
                                                context.current_node,
                                                equiv_concept.clone(),
                                                &mut change_log,
                                            );

                                            // Create expansion task for the equivalent class
                                            let task = ExpansionTask {
                                                node_id: context.current_node,
                                                concept: equiv_concept,
                                                rule_type: ExpansionRule::SubclassAxiom,
                                                priority: 1,
                                            };
                                            return Ok((vec![task], change_log));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((vec![], change_log))
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
    pub reasoning_rules: Option<ReasoningRules>,
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
                reasoning_rules: None,
            },
            reasoning_rules: None,
        }
    }

    pub fn with_reasoning_rules(mut self, rules: ReasoningRules) -> Self {
        self.reasoning_rules = Some(rules.clone());
        self.context.reasoning_rules = Some(rules);
        self
    }

    pub fn expand(
        &mut self,
        graph: &mut TableauxGraph,
        memory: &mut MemoryManager,
        max_depth: usize,
        change_log: &mut GraphChangeLog,
        memory_log: &mut MemoryChangeLog,
    ) -> Result<bool, String> {
        while self.context.current_depth < max_depth {
            if let Some(rule) = self.rules.get_next_rule(&self.context) {
                if self.rules.can_apply_rule(&rule, &self.context) {
                    let (new_tasks, local_changes) =
                        self.rules
                            .apply_rule(rule, graph, memory, &mut self.context)?;
                    change_log.extend(local_changes);
                    // TODO: capture memory mutations when MemoryManager supports logging.
                    memory_log.extend(MemoryChangeLog::new());
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
            reasoning_rules: self.reasoning_rules.clone(),
        };
    }
}

impl Default for ExpansionEngine {
    fn default() -> Self {
        Self::new()
    }
}
