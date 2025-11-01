//! Tests for equality reasoning in tableaux expansion module
//! This module tests the enhanced equality reasoning capabilities for clash detection.

use owl2_reasoner::reasoning::tableaux::expansion::{
    ExpansionEngine, ExpansionContext, EqualityTracker, ExpansionRules
};
use owl2_reasoner::reasoning::tableaux::core::NodeId;
use owl2_reasoner::reasoning::TableauxGraph;
use owl2_reasoner::reasoning::tableaux::memory::MemoryManager;
use owl2_reasoner::axioms::{SameIndividualAxiom, DifferentIndividualsAxiom};
use owl2_reasoner::reasoning::core::ReasoningRules;
use owl2_reasoner::iri::IRI;
use std::sync::Arc;

fn create_test_graph_and_engine() -> (TableauxGraph, ExpansionEngine, MemoryManager) {
    let mut graph = TableauxGraph::new();
    let memory = MemoryManager::new();
    let engine = ExpansionEngine::new();

    (graph, engine, memory)
}

#[test]
fn test_equality_tracker_basic_functionality() {
    let mut tracker = EqualityTracker::new();

    // Test initial state
    assert!(!tracker.are_same(NodeId::new(0), NodeId::new(1)));
    assert!(!tracker.are_different(NodeId::new(0), NodeId::new(1)));

    // Test merging nodes
    tracker.merge_nodes(NodeId::new(0), NodeId::new(1)).unwrap();
    assert!(tracker.are_same(NodeId::new(0), NodeId::new(1)));
    assert_eq!(tracker.get_canonical(NodeId::new(0)), tracker.get_canonical(NodeId::new(1)));

    // Test marking different
    tracker.mark_different(NodeId::new(2), NodeId::new(3)).unwrap();
    assert!(tracker.are_different(NodeId::new(2), NodeId::new(3)));

    // Test clash detection: trying to merge nodes that are marked different
    tracker.mark_different(NodeId::new(4), NodeId::new(5)).unwrap();
    let result = tracker.merge_nodes(NodeId::new(4), NodeId::new(5));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Clash"));
}

#[test]
fn test_equality_tracker_equivalence_classes() {
    let mut tracker = EqualityTracker::new();

    // Create equivalence class: node0 = node1 = node2
    tracker.merge_nodes(NodeId::new(0), NodeId::new(1)).unwrap();
    tracker.merge_nodes(NodeId::new(1), NodeId::new(2)).unwrap();

    // All should be in the same equivalence class
    assert!(tracker.are_same(NodeId::new(0), NodeId::new(1)));
    assert!(tracker.are_same(NodeId::new(1), NodeId::new(2)));
    assert!(tracker.are_same(NodeId::new(0), NodeId::new(2)));

    // All should have the same canonical representative
    let canon0 = tracker.get_canonical(NodeId::new(0));
    let canon1 = tracker.get_canonical(NodeId::new(1));
    let canon2 = tracker.get_canonical(NodeId::new(2));
    assert_eq!(canon0, canon1);
    assert_eq!(canon1, canon2);

    // Test get_equivalent_nodes
    let equivalents = tracker.get_equivalent_nodes(NodeId::new(0));
    assert_eq!(equivalents.len(), 3);
    assert!(equivalents.contains(&NodeId::new(0)));
    assert!(equivalents.contains(&NodeId::new(1)));
    assert!(equivalents.contains(&NodeId::new(2)));
}

#[test]
fn test_same_individual_axiom_processing() {
    let (mut graph, mut engine, mut memory) = create_test_graph_and_engine();

    // Create reasoning rules with same individual axiom
    let individual1_iri = Arc::new(IRI::new("http://example.org/person1").unwrap());
    let individual2_iri = Arc::new(IRI::new("http://example.org/person2").unwrap());

    let same_individual_axiom = SameIndividualAxiom::new(vec![individual1_iri.clone(), individual2_iri.clone()]);

    let mut reasoning_rules = ReasoningRules::new(&owl2_reasoner::ontology::Ontology::new());
    reasoning_rules.same_individual_axioms.push(same_individual_axiom);

    engine.context.reasoning_rules = Some(reasoning_rules);

    // Add nodes for the individuals
    let node1 = graph.add_node();
    let node2 = graph.add_node();

    // Add labels to identify the individuals
    if let Some(node) = graph.nodes.get_mut(node1.as_usize()) {
        node.labels.push(individual1_iri.as_str().to_string());
    }
    if let Some(node) = graph.nodes.get_mut(node2.as_usize()) {
        node.labels.push(individual2_iri.as_str().to_string());
    }

    // Apply same individual rule
    let rules = ExpansionRules::new();
    let (tasks, _change_log) = rules.apply_same_individual_rule(&mut graph, &mut memory, &mut engine.context).unwrap();

    // Verify that the nodes are marked as equivalent in the equality tracker
    assert!(engine.context.equality_tracker.are_same(node1, node2));
}

#[test]
fn test_different_individuals_axiom_processing() {
    let (mut graph, mut engine, mut memory) = create_test_graph_and_engine();

    // Create reasoning rules with different individuals axiom
    let individual1_iri = Arc::new(IRI::new("http://example.org/person1").unwrap());
    let individual2_iri = Arc::new(IRI::new("http://example.org/person2").unwrap());

    let different_individuals_axiom = DifferentIndividualsAxiom::new(vec![individual1_iri.clone(), individual2_iri.clone()]);

    let mut reasoning_rules = ReasoningRules::new(&owl2_reasoner::ontology::Ontology::new());
    reasoning_rules.different_individuals_axioms.push(different_individuals_axiom);

    engine.context.reasoning_rules = Some(reasoning_rules);

    // Add nodes for the individuals
    let node1 = graph.add_node();
    let node2 = graph.add_node();

    // Add labels to identify the individuals
    if let Some(node) = graph.nodes.get_mut(node1.as_usize()) {
        node.labels.push(individual1_iri.as_str().to_string());
    }
    if let Some(node) = graph.nodes.get_mut(node2.as_usize()) {
        node.labels.push(individual2_iri.as_str().to_string());
    }

    // Apply different individuals rule
    let rules = ExpansionRules::new();
    let (tasks, _change_log) = rules.apply_different_individuals_rule(&mut graph, &mut memory, &mut engine.context).unwrap();

    // Verify that the nodes are marked as different in the equality tracker
    assert!(engine.context.equality_tracker.are_different(node1, node2));
}

#[test]
fn test_different_individuals_clash_detection() {
    let (mut graph, mut engine, mut memory) = create_test_graph_and_engine();

    // Create reasoning rules with different individuals axiom
    let individual1_iri = Arc::new(IRI::new("http://example.org/person1").unwrap());
    let individual2_iri = Arc::new(IRI::new("http://example.org/person2").unwrap());

    let different_individuals_axiom = DifferentIndividualsAxiom::new(vec![individual1_iri.clone(), individual2_iri.clone()]);

    let mut reasoning_rules = ReasoningRules::new(&owl2_reasoner::ontology::Ontology::new());
    reasoning_rules.different_individuals_axioms.push(different_individuals_axiom);

    engine.context.reasoning_rules = Some(reasoning_rules);

    // Add a single node with both individual labels (simulating they were merged)
    let node = graph.add_node();
    if let Some(node_ref) = graph.nodes.get_mut(node.as_usize()) {
        node_ref.labels.push(individual1_iri.as_str().to_string());
        node_ref.labels.push(individual2_iri.as_str().to_string());
    }

    // Apply different individuals rule - should detect a clash
    let rules = ExpansionRules::new();
    let result = rules.apply_different_individuals_rule(&mut graph, &mut memory, &mut engine.context);

    // Should fail with a clash
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Clash"));
}

#[test]
fn test_functional_property_equality_reasoning() {
    let (mut graph, mut engine, mut memory) = create_test_graph_and_engine();

    // Create a functional property
    let property_iri = Arc::new(IRI::new("http://example.org/hasMother").unwrap());
    let mut reasoning_rules = ReasoningRules::new(&owl2_reasoner::ontology::Ontology::new());
    reasoning_rules.functional_properties.insert(property_iri.clone());

    engine.context.reasoning_rules = Some(reasoning_rules);

    // Create three nodes
    let source = graph.add_node();
    let target1 = graph.add_node();
    let target2 = graph.add_node();

    // Add edges: source --hasMother--> target1 and source --hasMother--> target2
    graph.add_edge(source, &property_iri, target1);
    graph.add_edge(source, &property_iri, target2);

    // Apply functional property rule
    let rules = ExpansionRules::new();
    let result = rules.apply_functional_property_rule(&mut graph, &mut memory, &mut engine.context);

    // Should detect a clash since we have two different targets
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("Clash"));
    assert!(error_msg.contains("Functional property"));
}

#[test]
fn test_functional_property_with_equal_targets() {
    let (mut graph, mut engine, mut memory) = create_test_graph_and_engine();

    // Create a functional property
    let property_iri = Arc::new(IRI::new("http://example.org/hasMother").unwrap());
    let mut reasoning_rules = ReasoningRules::new(&owl2_reasoner::ontology::Ontology::new());
    reasoning_rules.functional_properties.insert(property_iri.clone());

    engine.context.reasoning_rules = Some(reasoning_rules);

    // Create three nodes
    let source = graph.add_node();
    let target1 = graph.add_node();
    let target2 = graph.add_node();

    // Mark target1 and target2 as the same individual
    engine.context.equality_tracker.merge_nodes(target1, target2).unwrap();

    // Add edges: source --hasMother--> target1 and source --hasMother--> target2
    graph.add_edge(source, &property_iri, target1);
    graph.add_edge(source, &property_iri, target2);

    // Apply functional property rule
    let rules = ExpansionRules::new();
    let result = rules.apply_functional_property_rule(&mut graph, &mut memory, &mut engine.context);

    // Should NOT detect a clash since targets are the same individual
    assert!(result.is_ok());
}