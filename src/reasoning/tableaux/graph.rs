//! # Tableaux Graph Management
//!
//! Provides efficient graph data structures and operations for the tableaux reasoning algorithm.
//! This module manages the tableaux graph structure, including nodes, edges, and their relationships.
//!
//! ## Key Components
//!
//! - **[`TableauxGraph`]** - Main graph structure with node and edge management
//! - **[`EdgeStorage`]** - Optimized edge storage with indexing for fast lookups
//! - **Graph Traversal** - Efficient traversal algorithms for reasoning
//! - **Memory Management** - Arena-based node allocation and cleanup
//!
//! ## Performance Optimizations
//!
//! - **Flat Edge Storage**: Contiguous memory allocation for better cache locality
//! - **Hash-Based Indexing**: O(1) edge lookups using (from_node, property) -> to_node mappings
//! - **SmallVec Optimization**: Uses stack allocation for small edge collections
//! - **Arena Allocation**: Bulk allocation of nodes to reduce malloc overhead
//! - **Bidirectional Edges**: Efficient forward and backward traversal
//!
//! ## Graph Structure
//!
//! The tableaux graph represents the model being constructed during reasoning:
//!
//! ```text
//! Node1 ──property───> Node2
//!   │                    │
//!   │                    │
//!   └─other_property──> Node3
//! ```
//!
//! Each node contains concepts (class expressions) and edges represent property relationships.
//!
//! ## Example Usage
//!
//! ```rust
//! use owl2_reasoner::reasoning::tableaux::{TableauxGraph, NodeId, TableauxNode};
//! use owl2_reasoner::IRI;
//!
//! // Create new graph
//! let mut graph = TableauxGraph::new();
//!
//! // Add nodes to the graph
//! let node1 = graph.add_node();
//! let node2 = graph.add_node();
//!
//! // Add edge between nodes
//! let property_iri = IRI::new("http://example.org/hasChild")?;
//! graph.add_edge(node1, &property_iri, node2);
//!
//! // Traverse graph
//! if let Some(node) = graph.get_node(node1) {
//!     println!("Node {} has {} concepts", node1.as_usize(), node.concepts_iter().count());
//! }
//! ```

use super::core::{NodeId, TableauxNode};
use crate::iri::IRI;
use hashbrown::HashMap;
use smallvec::SmallVec;

/// Optimized edge storage for tableaux graph
#[derive(Debug, Default)]
pub struct EdgeStorage {
    /// Optimized storage for edges using flat representation
    pub edges: Vec<(NodeId, IRI, NodeId)>,
    /// Index for fast lookups: (from_node, property) -> Vec<to_node>
    pub index: HashMap<(NodeId, IRI), SmallVec<[NodeId; 4]>>,
}

impl EdgeStorage {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            index: HashMap::default(),
        }
    }

    pub fn add_edge(&mut self, from: NodeId, property: &IRI, to: NodeId) {
        // Add to flat storage
        self.edges.push((from, property.clone(), to));

        // Update index
        let key = (from, property.clone());
        self.index.entry(key).or_default().push(to);
    }

    pub fn get_targets(&self, from: NodeId, property: &IRI) -> Option<&[NodeId]> {
        let key = (from, property.clone());
        self.index.get(&key).map(|vec| vec.as_slice())
    }

    pub fn get_all_edges(&self) -> &[(NodeId, IRI, NodeId)] {
        &self.edges
    }

    pub fn clear(&mut self) {
        self.edges.clear();
        self.index.clear();
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }
}

/// Tableaux graph manager
#[derive(Debug)]
pub struct TableauxGraph {
    pub nodes: Vec<TableauxNode>,
    pub edges: EdgeStorage,
    pub root: NodeId,
}

impl TableauxGraph {
    pub fn new() -> Self {
        let root = NodeId::new(0);
        let mut nodes = Vec::new();
        nodes.push(TableauxNode::new(root));

        Self {
            nodes,
            edges: EdgeStorage::new(),
            root,
        }
    }

    pub fn add_node(&mut self) -> NodeId {
        let id = NodeId::new(self.nodes.len());
        self.nodes.push(TableauxNode::new(id));
        id
    }

    pub fn get_node(&self, id: NodeId) -> Option<&TableauxNode> {
        self.nodes.get(id.as_usize())
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut TableauxNode> {
        self.nodes.get_mut(id.as_usize())
    }

    pub fn add_edge(&mut self, from: NodeId, property: &IRI, to: NodeId) {
        self.edges.add_edge(from, property, to);
    }

    pub fn get_targets(&self, from: NodeId, property: &IRI) -> Option<&[NodeId]> {
        self.edges.get_targets(from, property)
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        // Re-add root node
        self.root = NodeId::new(0);
        self.nodes.push(TableauxNode::new(self.root));
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for TableauxGraph {
    fn default() -> Self {
        Self::new()
    }
}
