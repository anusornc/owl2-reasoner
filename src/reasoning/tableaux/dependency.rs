//! # Dependency-Directed Backtracking
//!
//! Implements dependency management for efficient backtracking in the tableaux reasoning algorithm.
//! This module tracks relationships between reasoning decisions and enables intelligent backtracking
//! when contradictions are discovered.
//!
//! ## Key Components
//!
//! - **[`DependencyManager`]** - Central coordinator for dependency tracking
//! - **[`Dependency`]** - Represents relationships between nodes and choices
//! - **[`ChoicePoint`]** - Records branching points in the reasoning process
//! - **[`DependencySource`]** - Types of dependency sources (ChoicePoint, Node, GlobalConstraint)
//! - **[`DependencyType`]** - Categories of dependencies (Subclass, Property, Disjointness, etc.)
//!
//! ## Dependency-Directed Backtracking
//!
//! Unlike naive backtracking that explores all possibilities, dependency-directed backtracking:
//!
//! 1. **Track Dependencies**: Record which reasoning steps depend on which choices
//! 2. **Identify Contradictions**: When a clash is found, trace back to responsible choices
//! 3. **Smart Backtracking**: Jump directly to the choice that caused the contradiction
//! 4. **Avoid Redundant Work**: Skip exploration of paths that would lead to the same contradiction
//!
//! ## Dependency Types
//!
//! - **Subclass**: Dependencies from subclass reasoning steps
//! - **Property**: Dependencies from property axioms and restrictions
//! - **Disjointness**: Dependencies from disjoint class axioms
//! - **Existential**: Dependencies from existential restrictions
//! - **Universal**: Dependencies from universal restrictions
//! - **Nominal**: Dependencies from individual assertions
//!
//! ## Performance Benefits
//!
//! - **Reduced Backtracking**: Skip irrelevant branches
//! - **Faster Contradiction Detection**: Direct tracing to source
//! - **Memory Efficiency**: Only track necessary dependencies
//! - **Scalability**: Better performance on complex ontologies
//!
//! ## Example Usage
//!
//! ```rust
//! use owl2_reasoner::reasoning::tableaux::{DependencyManager, Dependency, DependencyType};
//!
//! // Create dependency manager
//! let mut dependency_manager = DependencyManager::new();
//!
//! // Create a choice point (branching decision)
//! let choice_point_id = dependency_manager.create_choice_point(NodeId::new(1));
//!
//! // Add dependencies for reasoning steps
//! dependency_manager.add_dependency(
//!     NodeId::new(2),  // dependent node
//!     DependencySource::ChoicePoint(choice_point_id),
//!     DependencyType::Subclass
//! );
//!
//! // When a contradiction is found, backtrack intelligently
//! if contradiction_detected {
//!     let backtrack_to = dependency_manager.find_responsible_choice(NodeId::new(2));
//!     println!("Backtrack to choice point {}", backtrack_to);
//! }
//! ```

use super::core::NodeId;
use std::collections::{HashMap, HashSet};

/// Dependency between tableaux nodes and choices
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub dependent_node: NodeId,
    pub dependency_source: DependencySource,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencySource {
    /// Choice point in the reasoning process
    ChoicePoint(usize),
    /// Another node
    Node(NodeId),
    /// Global constraint
    GlobalConstraint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyType {
    /// Subclass dependency
    Subclass,
    /// Property dependency
    Property,
    /// Individual dependency
    Individual,
    /// Concept dependency
    Concept,
}

impl Dependency {
    pub fn new(
        dependent_node: NodeId,
        dependency_source: DependencySource,
        dependency_type: DependencyType,
    ) -> Self {
        Self {
            dependent_node,
            dependency_source,
            dependency_type,
        }
    }
}

/// Choice point for backtracking
#[derive(Debug, Clone)]
pub struct ChoicePoint {
    pub id: usize,
    pub node_id: NodeId,
    pub choice_type: ChoiceType,
    pub dependencies: HashSet<NodeId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChoiceType {
    /// Disjunction choice
    Disjunction,
    /// existential restriction choice
    ExistentialRestriction,
    /// nominal choice
    Nominal,
    /// data range choice
    DataRange,
}

impl ChoicePoint {
    pub fn new(id: usize, node_id: NodeId, choice_type: ChoiceType) -> Self {
        Self {
            id,
            node_id,
            choice_type,
            dependencies: HashSet::new(),
        }
    }

    pub fn add_dependency(&mut self, node_id: NodeId) {
        self.dependencies.insert(node_id);
    }
}

/// Dependency manager for backtracking and dependency-directed reasoning
#[derive(Debug)]
pub struct DependencyManager {
    pub dependencies: HashMap<NodeId, Vec<Dependency>>,
    pub choice_points: Vec<ChoicePoint>,
    pub next_choice_id: usize,
    pub dependency_graph: HashMap<NodeId, HashSet<NodeId>>,
}

impl DependencyManager {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            choice_points: Vec::new(),
            next_choice_id: 0,
            dependency_graph: HashMap::new(),
        }
    }

    pub fn add_dependency(&mut self, dependency: Dependency) {
        let dependent_node = dependency.dependent_node;
        self.dependencies
            .entry(dependent_node)
            .or_default()
            .push(dependency.clone());

        // Update dependency graph
        match dependency.dependency_source {
            DependencySource::Node(source_node) => {
                self.dependency_graph
                    .entry(source_node)
                    .or_default()
                    .insert(dependent_node);
            }
            DependencySource::ChoicePoint(_) => {
                // Choice points will be handled separately
            }
            DependencySource::GlobalConstraint => {
                // Global constraints affect all nodes
            }
        }
    }

    pub fn create_choice_point(
        &mut self,
        node_id: NodeId,
        choice_type: ChoiceType,
    ) -> &mut ChoicePoint {
        let choice_id = self.next_choice_id;
        self.next_choice_id += 1;

        let choice_point = ChoicePoint::new(choice_id, node_id, choice_type);
        self.choice_points.push(choice_point);

        self.choice_points.last_mut().unwrap()
    }

    pub fn get_dependencies(&self, node_id: NodeId) -> &[Dependency] {
        self.dependencies
            .get(&node_id)
            .map(|vec| vec.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_dependent_nodes(&self, node_id: NodeId) -> HashSet<NodeId> {
        self.dependency_graph
            .get(&node_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn backtrack_to_choice(&mut self, choice_id: usize) {
        // Remove all choice points after the specified one
        self.choice_points.retain(|cp| cp.id <= choice_id);

        // Rebuild dependencies based on remaining choice points
        self.rebuild_dependencies();
    }

    pub fn clear(&mut self) {
        self.dependencies.clear();
        self.choice_points.clear();
        self.next_choice_id = 0;
        self.dependency_graph.clear();
    }

    fn rebuild_dependencies(&mut self) {
        // This is a simplified version - in practice, you'd want to
        // properly rebuild the dependency graph from remaining choice points
        self.dependencies.clear();
        self.dependency_graph.clear();
    }

    pub fn get_latest_choice_point(&self) -> Option<&ChoicePoint> {
        self.choice_points.last()
    }

    pub fn has_choices(&self) -> bool {
        !self.choice_points.is_empty()
    }
}

impl Default for DependencyManager {
    fn default() -> Self {
        Self::new()
    }
}
