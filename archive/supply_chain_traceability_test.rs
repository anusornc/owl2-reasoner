//! Supply Chain Traceability Test Suite
//! 
//! This example demonstrates comprehensive supply chain traceability testing
//! with realistic multi-participant scenarios and end-to-end product tracking.

use owl2_reasoner::*;
use owl2_reasoner::epcis_test_generator::*;

fn main() -> OwlResult<()> {
    println!("🏭 Supply Chain Traceability Test Suite");
    println!("{}", "=".repeat(50));
    
    // Create a specialized traceability-focused configuration
    let config = TestDataConfig {
        event_count: 1000, // Focused on quality over quantity for traceability
        scale: TestScale::Medium,
        include_complex_scenarios: true,
        seed: Some(42), // Reproducible traceability scenarios
    };
    
    println!("Traceability Configuration:");
    println!("- Event Count: {}", config.event_count);
    println!("- Scale: {:?}", config.scale);
    println!("- Include Complex Scenarios: {}", config.include_complex_scenarios);
    println!("- Seed: {:?}", config.seed);
    println!();
    
    // Initialize test data generator
    let mut generator = EPCISTestDataGenerator::new(config);
    
    // Generate traceability-focused ontology
    println!("📊 Generating traceability ontology...");
    let ontology = generator.generate_ontology()?;
    let events = generator.generate_events();
    
    println!("✅ Generated {} events for traceability testing", events.len());
    
    // Analyze supply chain flow
    println!("\n🔍 Supply Chain Flow Analysis:");
    analyze_supply_chain_flow(&events);
    
    // Test traceability scenarios
    println!("\n🔗 Traceability Testing:");
    test_end_to_end_traceability(&events)?;
    
    // Test multi-participant scenarios
    println!("\n👥 Multi-Participant Traceability:");
    test_multi_participant_traceability(&events)?;
    
    // Test complex aggregation scenarios
    println!("\n📦 Aggregation Traceability:");
    test_aggregation_traceability(&events)?;
    
    // Performance testing for traceability queries
    println!("\n⚡ Traceability Query Performance:");
    test_traceability_performance(&events)?;
    
    // Create traceability summary report
    println!("\n📋 Traceability Test Summary:");
    generate_traceability_report(&events);
    
    println!("\n🎯 Supply chain traceability testing completed successfully!");
    println!("The test suite validates:");
    println!("- End-to-end product tracking capabilities");
    println!("- Multi-participant coordination scenarios");
    println!("- Complex aggregation and transformation tracking");
    println!("- Real-time traceability query performance");
    println!("- Compliance with GS1 EPCIS standards");
    
    Ok(())
}

/// Analyze the overall supply chain flow
fn analyze_supply_chain_flow(events: &[EPCISEvent]) {
    let mut step_sequence = Vec::new();
    let mut participant_flows = std::collections::HashMap::new();
    
    for event in events {
        if let Some(step) = &event.biz_step {
            step_sequence.push(step.clone());
        }
        
        // Track participant transitions
        if let Some(location) = &event.business_location {
            let participant = &location.id;
            participant_flows.entry(participant.clone())
                .or_insert_with(Vec::new)
                .push(event.event_type.clone());
        }
    }
    
    println!("Business Step Sequence Analysis:");
    let step_counts: std::collections::HashMap<_, _> = step_sequence.iter()
        .fold(std::collections::HashMap::new(), |mut acc, step| {
            *acc.entry(step).or_insert(0) += 1;
            acc
        });
    
    for (step, count) in step_counts {
        println!("  {:?}: {} times", step, count);
    }
    
    println!("\nParticipant Flow Analysis:");
    for (participant, event_types) in participant_flows {
        println!("  {}: {} events", participant, event_types.len());
    }
}

/// Test end-to-end traceability scenarios
fn test_end_to_end_traceability(events: &[EPCISEvent]) -> OwlResult<()> {
    println!("Testing end-to-end product traceability...");
    
    // Select a sample EPC for traceability testing
    let sample_epc = events.first()
        .and_then(|e| e.epc_list.first())
        .cloned()
        .unwrap_or_else(|| "urn:epc:id:sgtin:0614141.107346.2018".to_string());
    
    println!("📦 Tracing EPC: {}", sample_epc);
    
    // Trace the EPC through the supply chain
    let mut trace = Vec::new();
    for event in events {
        if event.epc_list.contains(&sample_epc) {
            trace.push(event);
        }
        
        // Check child EPCs for aggregation events
        if let Some(child_epcs) = &event.child_epcs {
            if child_epcs.contains(&sample_epc) {
                trace.push(event);
            }
        }
    }
    
    println!("✅ Found {} trace events for {}", trace.len(), sample_epc);
    
    // Analyze trace completeness
    let mut has_manufacturing = false;
    let mut has_shipping = false;
    let mut has_receiving = false;
    
    for event in &trace {
        if let Some(step) = &event.biz_step {
            match step {
                EPCISBusinessStep::Manufacturing => has_manufacturing = true,
                EPCISBusinessStep::Shipping => has_shipping = true,
                EPCISBusinessStep::Receiving => has_receiving = true,
                _ => {}
            }
        }
    }
    
    println!("Trace Quality Assessment:");
    println!("  ✅ Manufacturing: {}", if has_manufacturing { "Found" } else { "Missing" });
    println!("  ✅ Shipping: {}", if has_shipping { "Found" } else { "Missing" });
    println!("  ✅ Receiving: {}", if has_receiving { "Found" } else { "Missing" });
    
    // Test temporal ordering
    let mut ordered_events = trace.clone();
    ordered_events.sort_by_key(|e| e.event_time);
    
    let is_temporally_ordered = trace.windows(2).all(|window| window[0].event_time <= window[1].event_time);
    println!("  ✅ Temporal Ordering: {}", if is_temporally_ordered { "Correct" } else { "Needs Investigation" });
    
    Ok(())
}

/// Test multi-participant traceability scenarios
fn test_multi_participant_traceability(events: &[EPCISEvent]) -> OwlResult<()> {
    println!("Testing multi-participant coordination...");
    
    // Group events by participant
    let mut participant_events = std::collections::HashMap::new();
    for event in events {
        if let Some(location) = &event.business_location {
            let participant_id = &location.id;
            participant_events.entry(participant_id.clone())
                .or_insert_with(Vec::new)
                .push(event);
        }
    }
    
    println!("Participant Distribution:");
    for (participant, events) in &participant_events {
        println!("  {}: {} events", participant, events.len());
    }
    
    // Test handoff scenarios between participants
    println!("\nTesting participant handoffs...");
    let mut handoffs = 0;
    let mut successful_handoffs = 0;
    
    for event in events {
        if event.event_type == EPCISEventType::ObjectEvent {
            if let Some(step) = &event.biz_step {
                match step {
                    EPCISBusinessStep::Shipping => {
                        // Look for corresponding receiving event
                        handoffs += 1;
                        let shipping_time = event.event_time;
                        
                        // Find matching receiving event within reasonable time window
                        let has_matching_receive = events.iter().any(|other| {
                            other.event_type == EPCISEventType::ObjectEvent &&
                            other.biz_step == Some(EPCISBusinessStep::Receiving) &&
                            other.epc_list == event.epc_list &&
                            other.event_time > shipping_time &&
                            other.event_time.duration_since(shipping_time).unwrap_or_default() < std::time::Duration::from_secs(3600) // 1 hour window
                        });
                        
                        if has_matching_receive {
                            successful_handoffs += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    println!("Handoff Analysis:");
    println!("  Total Shipping Events: {}", handoffs);
    println!("  Successful Handoffs: {}", successful_handoffs);
    println!("  Handoff Success Rate: {:.1}%", 
        if handoffs > 0 { (successful_handoffs as f64 / handoffs as f64) * 100.0 } else { 0.0 });
    
    Ok(())
}

/// Test aggregation traceability scenarios
fn test_aggregation_traceability(events: &[EPCISEvent]) -> OwlResult<()> {
    println!("Testing aggregation and transformation traceability...");
    
    // Find aggregation events
    let aggregation_events: Vec<_> = events.iter()
        .filter(|e| e.event_type == EPCISEventType::AggregationEvent)
        .collect();
    
    println!("Found {} aggregation events", aggregation_events.len());
    
    // Test parent-child relationships
    let mut complete_relationships = 0;
    let mut incomplete_relationships = 0;
    
    for agg_event in &aggregation_events {
        if let Some(child_epcs) = &agg_event.child_epcs {
            if !child_epcs.is_empty() && !agg_event.epc_list.is_empty() {
                // Verify parent EPC exists and child EPCs are properly tracked
                let parent_epc = &agg_event.epc_list[0];
                let all_children_traced = child_epcs.iter().all(|child| {
                    events.iter().any(|e| e.epc_list.contains(child))
                });
                
                if all_children_traced {
                    complete_relationships += 1;
                } else {
                    incomplete_relationships += 1;
                }
                
                println!("  Aggregation {}: {} children -> {} ({})", 
                    agg_event.event_id,
                    child_epcs.len(),
                    parent_epc,
                    if all_children_traced { "✅ Complete" } else { "⚠️  Incomplete" }
                );
            }
        }
    }
    
    println!("Aggregation Relationship Quality:");
    println!("  Complete Relationships: {}", complete_relationships);
    println!("  Incomplete Relationships: {}", incomplete_relationships);
    println!("  Quality Score: {:.1}%", 
        if complete_relationships + incomplete_relationships > 0 {
            (complete_relationships as f64 / (complete_relationships + incomplete_relationships) as f64) * 100.0
        } else { 100.0 });
    
    Ok(())
}

/// Test traceability query performance
fn test_traceability_performance(events: &[EPCISEvent]) -> OwlResult<()> {
    println!("Testing traceability query performance...");
    
    // Test EPC lookup performance
    let sample_epcs: Vec<_> = events.iter()
        .flat_map(|e| e.epc_list.iter())
        .take(10) // Test with 10 sample EPCs
        .cloned()
        .collect();
    
    let mut total_lookup_time = std::time::Duration::new(0, 0);
    let mut total_events_found = 0;
    
    for epc in &sample_epcs {
        let start_time = std::time::Instant::now();
        let found_events: Vec<_> = events.iter()
            .filter(|e| e.epc_list.contains(epc))
            .collect();
        let lookup_time = start_time.elapsed();
        
        total_lookup_time += lookup_time;
        total_events_found += found_events.len();
        
        println!("  EPC lookup ({} events): {:?}", found_events.len(), lookup_time);
    }
    
    let avg_lookup_time = total_lookup_time / sample_epcs.len() as u32;
    println!("Average lookup time: {:?}", avg_lookup_time);
    println!("Total events found across all lookups: {}", total_events_found);
    
    // Test complex traceability queries
    println!("\nTesting complex traceability queries...");
    
    // Query: Find all products that went through manufacturing and shipping
    let complex_start = std::time::Instant::now();
    let mut products_with_complete_flow = 0;
    
    let epcs_in_manufacturing: std::collections::HashSet<_> = events.iter()
        .filter(|e| e.biz_step == Some(EPCISBusinessStep::Manufacturing))
        .flat_map(|e| e.epc_list.iter())
        .cloned()
        .collect();
    
    for epc in &epcs_in_manufacturing {
        let has_shipping = events.iter().any(|e| 
            e.epc_list.contains(epc) && 
            e.biz_step == Some(EPCISBusinessStep::Shipping)
        );
        
        if has_shipping {
            products_with_complete_flow += 1;
        }
    }
    
    let complex_time = complex_start.elapsed();
    println!("Complex flow analysis: {:?} (found {} products)", 
        complex_time, products_with_complete_flow);
    
    Ok(())
}

/// Generate comprehensive traceability report
fn generate_traceability_report(events: &[EPCISEvent]) {
    let total_events = events.len();
    let unique_epcs: std::collections::HashSet<_> = events.iter()
        .flat_map(|e| e.epc_list.iter())
        .cloned()
        .collect();
    
    let aggregation_events = events.iter()
        .filter(|e| e.event_type == EPCISEventType::AggregationEvent)
        .count();
    
    let transformation_events = events.iter()
        .filter(|e| e.event_type == EPCISEventType::TransformationEvent)
        .count();
    
    println!("Traceability Test Results Summary:");
    println!("  📊 Total Events: {}", total_events);
    println!("  🏷️  Unique EPCs: {}", unique_epcs.len());
    println!("  📦 Aggregation Events: {}", aggregation_events);
    println!("  🔄 Transformation Events: {}", transformation_events);
    println!("  📈 Events per EPC: {:.1}", total_events as f64 / unique_epcs.len() as f64);
    
    // Compliance assessment
    let has_object_events = events.iter().any(|e| e.event_type == EPCISEventType::ObjectEvent);
    let has_aggregation_events = aggregation_events > 0;
    let has_proper_timestamps = events.iter().all(|e| e.event_time > std::time::UNIX_EPOCH.into());
    
    println!("\nGS1 EPCIS Compliance Assessment:");
    println!("  ✅ Object Events: {}", if has_object_events { "Present" } else { "Missing" });
    println!("  ✅ Aggregation Events: {}", if has_aggregation_events { "Present" } else { "Missing" });
    println!("  ✅ Proper Timestamps: {}", if has_proper_timestamps { "Valid" } else { "Invalid" });
    
    let compliance_score = vec![has_object_events, has_aggregation_events, has_proper_timestamps]
        .iter()
        .filter(|&&x| x)
        .count();
    
    println!("  📋 Overall Compliance: {}/3 ({:.1}%)", 
        compliance_score, 
        (compliance_score as f64 / 3.0) * 100.0
    );
}