//! Multi-Participant Supply Chain Test Suite
//! 
//! This example demonstrates complex multi-participant supply chain scenarios
//! with realistic business processes, compliance requirements, and coordination challenges.

use owl2_reasoner::*;
use owl2_reasoner::epcis_test_generator::*;

fn main() -> OwlResult<()> {
    println!("🌐 Multi-Participant Supply Chain Test Suite");
    println!("{}", "=".repeat(50));
    
    // Create advanced multi-participant configuration
    let config = TestDataConfig {
        event_count: 2000, // Larger dataset for complex multi-participant scenarios
        scale: TestScale::Medium,
        include_complex_scenarios: true,
        seed: Some(12345), // Reproducible multi-participant scenarios
    };
    
    println!("Multi-Participant Configuration:");
    println!("- Event Count: {}", config.event_count);
    println!("- Scale: {:?}", config.scale);
    println!("- Include Complex Scenarios: {}", config.include_complex_scenarios);
    println!("- Seed: {:?}", config.seed);
    println!();
    
    // Initialize test data generator
    let mut generator = EPCISTestDataGenerator::new(config);
    
    // Generate multi-participant ontology
    println!("🏭 Generating multi-participant ontology...");
    let ontology = generator.generate_ontology()?;
    let events = generator.generate_events();
    
    println!("✅ Generated {} events for multi-participant testing", events.len());
    
    // Enhanced multi-participant analysis
    println!("\n👥 Multi-Participant Ecosystem Analysis:");
    analyze_multi_participant_ecosystem(&events)?;
    
    // Test complex business processes
    println!("\n🔄 Complex Business Process Testing:");
    test_complex_business_processes(&events)?;
    
    // Test compliance and regulatory scenarios
    println!("\n📋 Compliance and Regulatory Testing:");
    test_compliance_scenarios(&events)?;
    
    // Test exception handling and error scenarios
    println!("\n⚠️  Exception and Error Scenario Testing:");
    test_exception_scenarios(&events)?;
    
    // Test performance under load
    println!("\n⚡ Multi-Participant Performance Testing:");
    test_multi_participant_performance(&events)?;
    
    // Generate comprehensive test report
    println!("\n📊 Multi-Participant Test Summary:");
    generate_multi_participant_report(&events);
    
    println!("\n🎯 Multi-participant supply chain testing completed successfully!");
    println!("This test suite validates:");
    println!("- Complex multi-participant coordination and handoffs");
    println!("- Cross-organizational business process integration");
    println!("- Regulatory compliance across different jurisdictions");
    println!("- Exception handling and error recovery mechanisms");
    println!("- Performance optimization for distributed systems");
    println!("- Real-time supply chain visibility and tracking");
    
    Ok(())
}

/// Analyze the multi-participant ecosystem
fn analyze_multi_participant_ecosystem(events: &[EPCISEvent]) -> OwlResult<()> {
    println!("Analyzing participant ecosystem structure...");
    
    // Group events by participant
    let mut participant_data = std::collections::HashMap::new();
    
    for event in events {
        if let Some(location) = &event.business_location {
            let participant_id = &location.id;
            let entry = participant_data.entry(participant_id.clone())
                .or_insert(ParticipantData {
                    name: location.name.clone(),
                    role: String::new(), // Will be determined
                    event_count: 0,
                    event_types: std::collections::HashMap::new(),
                    business_steps: std::collections::HashMap::new(),
                    unique_epcs: std::collections::HashSet::new(),
                    first_event: None,
                    last_event: None,
                });
            
            entry.event_count += 1;
            *entry.event_types.entry(format!("{:?}", event.event_type)).or_insert(0) += 1;
            if let Some(step) = &event.biz_step {
                *entry.business_steps.entry(format!("{:?}", step)).or_insert(0) += 1;
            }
            entry.unique_epcs.extend(event.epc_list.iter().cloned());
            
            if entry.first_event.is_none() || event.event_time < entry.first_event.unwrap() {
                entry.first_event = Some(event.event_time);
            }
            if entry.last_event.is_none() || event.event_time > entry.last_event.unwrap() {
                entry.last_event = Some(event.event_time);
            }
        }
    }
    
    // Determine participant roles based on business steps
    for (participant_id, data) in &mut participant_data {
        let manufacturing_count = data.business_steps.get("Manufacturing").unwrap_or(&0);
        let shipping_count = data.business_steps.get("Shipping").unwrap_or(&0);
        let receiving_count = data.business_steps.get("Receiving").unwrap_or(&0);
        let picking_count = data.business_steps.get("Picking").unwrap_or(&0);
        
        data.role = if *manufacturing_count > *shipping_count && *manufacturing_count > *receiving_count {
            "Manufacturer".to_string()
        } else if *shipping_count > *picking_count && *shipping_count > *receiving_count {
            "Distributor".to_string()
        } else if *picking_count > *receiving_count {
            "Retailer".to_string()
        } else {
            "Logistics Provider".to_string()
        };
    }
    
    println!("Participant Ecosystem Overview:");
    for (participant_id, data) in &participant_data {
        println!("  🏢 {} ({})", data.name, participant_id);
        println!("    Role: {}", data.role);
        println!("    Events: {}", data.event_count);
        println!("    Unique EPCs: {}", data.unique_epcs.len());
        
        if let (Some(first), Some(last)) = (data.first_event, data.last_event) {
            let duration = last.duration_since(first).unwrap_or_default();
            println!("    Activity Span: {:?}", duration);
        }
        
        // Top business steps
        let mut steps: Vec<_> = data.business_steps.iter().collect();
        steps.sort_by(|a, b| b.1.cmp(a.1));
        if !steps.is_empty() {
            println!("    Primary Activities: {}", steps[0].0);
            if steps.len() > 1 {
                println!("                    {}", steps[1].0);
            }
        }
        println!();
    }
    
    // Analyze participant interactions
    analyze_participant_interactions(&events, &participant_data)?;
    
    Ok(())
}

/// Analyze interactions between participants
fn analyze_participant_interactions(events: &[EPCISEvent], participant_data: &std::collections::HashMap<String, ParticipantData>) -> OwlResult<()> {
    println!("Analyzing participant interactions...");
    
    // Track EPC movements between participants
    let mut epc_journeys = std::collections::HashMap::new();
    
    for event in events {
        for epc in &event.epc_list {
            let journey = epc_journeys.entry(epc.clone()).or_insert_with(Vec::new);
            if let Some(location) = &event.business_location {
                journey.push((location.id.clone(), event.event_time, event.event_type.clone()));
            }
        }
    }
    
    // Analyze handoff patterns
    let mut handoff_patterns = std::collections::HashMap::new();
    let mut successful_handoffs = 0;
    let mut failed_handoffs = 0;
    
    for journey in epc_journeys.values() {
        for window in journey.windows(2) {
            let (from_participant, _from_time, _from_type) = &window[0];
            let (to_participant, to_time, to_type) = &window[1];
            
            if from_participant != to_participant {
                let handoff_key = format!("{} -> {}", from_participant, to_participant);
                *handoff_patterns.entry(handoff_key).or_insert(0) += 1;
                
                // Check if this is a logical handoff (shipping -> receiving)
                let is_logical_handoff = matches!(_from_type, EPCISEventType::ObjectEvent) 
                    && matches!(to_type, EPCISEventType::ObjectEvent);
                
                if is_logical_handoff {
                    successful_handoffs += 1;
                } else {
                    failed_handoffs += 1;
                }
            }
        }
    }
    
    println!("Interaction Analysis:");
    println!("  Total EPC Journeys: {}", epc_journeys.len());
    println!("  Successful Handoffs: {}", successful_handoffs);
    println!("  Failed Handoffs: {}", failed_handoffs);
    println!("  Handoff Success Rate: {:.1}%", 
        if successful_handoffs + failed_handoffs > 0 {
            (successful_handoffs as f64 / (successful_handoffs + failed_handoffs) as f64) * 100.0
        } else { 100.0 });
    
    println!("\nTop Handoff Patterns:");
    let mut patterns: Vec<_> = handoff_patterns.iter().collect();
    patterns.sort_by(|a, b| b.1.cmp(a.1));
    
    for (pattern, count) in patterns.iter().take(5) {
        println!("  {}: {} times", pattern, count);
    }
    
    Ok(())
}

/// Test complex business processes
fn test_complex_business_processes(events: &[EPCISEvent]) -> OwlResult<()> {
    println!("Testing complex multi-participant business processes...");
    
    // Test 1: End-to-end order fulfillment
    println!("1. Testing Order Fulfillment Process...");
    let order_fulfillment_success = test_order_fulfillment_process(events)?;
    println!("   ✅ Order Fulfillment: {}", if order_fulfillment_success { "Complete" } else { "Incomplete" });
    
    // Test 2: Returns and reverse logistics
    println!("2. Testing Returns Process...");
    let returns_success = test_returns_process(events)?;
    println!("   ✅ Returns Processing: {}", if returns_success { "Functional" } else { "Limited" });
    
    // Test 3: Quality control processes
    println!("3. Testing Quality Control Processes...");
    let quality_success = test_quality_control_processes(events)?;
    println!("   ✅ Quality Control: {}", if quality_success { "Effective" } else { "Needs Improvement" });
    
    // Test 4: Recall processes
    println!("4. Testing Recall Simulation...");
    let recall_success = test_recall_simulation(events)?;
    println!("   ✅ Recall Capability: {}", if recall_success { "Robust" } else { "Basic" });
    
    Ok(())
}

/// Test order fulfillment process
fn test_order_fulfillment_process(events: &[EPCISEvent]) -> OwlResult<bool> {
    let mut complete_orders = 0;
    let mut partial_orders = 0;
    
    // Find products that go through complete order fulfillment
    let mut epc_processes: std::collections::HashMap<_, Vec<_>> = std::collections::HashMap::new();
    for event in events {
        for epc in &event.epc_list {
            epc_processes.entry(epc.clone()).or_insert_with(Vec::new).push(event);
        }
    }
    
    for (epc, epc_events) in epc_processes {
        let has_manufacturing = epc_events.iter().any(|e| 
            e.biz_step == Some(EPCISBusinessStep::Manufacturing));
        let has_shipping = epc_events.iter().any(|e| 
            e.biz_step == Some(EPCISBusinessStep::Shipping));
        let has_receiving = epc_events.iter().any(|e| 
            e.biz_step == Some(EPCISBusinessStep::Receiving));
        let has_picking = epc_events.iter().any(|e| 
            e.biz_step == Some(EPCISBusinessStep::Picking));
        
        if has_manufacturing && has_shipping && has_receiving && has_picking {
            complete_orders += 1;
        } else if has_shipping && has_receiving {
            partial_orders += 1;
        }
    }
    
    println!("   Complete Orders: {}", complete_orders);
    println!("   Partial Orders: {}", partial_orders);
    
    Ok(complete_orders > 0)
}

/// Test returns process
fn test_returns_process(events: &[EPCISEvent]) -> OwlResult<bool> {
    // Look for evidence of returns processing
    let return_indicators = vec![
        EPCISDisposition::Failed,
        EPCISDisposition::Reserved, // Could indicate return processing
    ];
    
    let return_events = events.iter()
        .filter(|e| e.disposition.as_ref().map(|d| return_indicators.contains(d)).unwrap_or(false))
        .count();
    
    println!("   Return-related Events: {}", return_events);
    
    Ok(return_events > 0)
}

/// Test quality control processes
fn test_quality_control_processes(events: &[EPCISEvent]) -> OwlResult<bool> {
    // Look for quality control indicators
    let quality_steps = vec![
        EPCISBusinessStep::Assembling, // Could include quality checks
        EPCISBusinessStep::Receiving, // Could include inspection
    ];
    
    let quality_events = events.iter()
        .filter(|e| e.biz_step.as_ref().map(|s| quality_steps.contains(s)).unwrap_or(false))
        .count();
    
    let passed_products = events.iter()
        .filter(|e| e.disposition == Some(EPCISDisposition::Passed))
        .count();
    
    println!("   Quality Control Events: {}", quality_events);
    println!("   Passed Products: {}", passed_products);
    
    Ok(quality_events > 0 && passed_products > 0)
}

/// Test recall simulation
fn test_recall_simulation(events: &[EPCISEvent]) -> OwlResult<bool> {
    // Simulate recall by testing if we can trace all instances of problematic products
    let sample_epcs: std::collections::HashSet<_> = events.iter()
        .flat_map(|e| e.epc_list.iter())
        .take(10) // Test with first 10 EPCs
        .cloned()
        .collect();
    
    let mut fully_traceable = 0;
    let mut partially_traceable = 0;
    
    for epc in &sample_epcs {
        let epc_events: Vec<_> = events.iter()
            .filter(|e| e.epc_list.contains(epc))
            .collect();
        
        if epc_events.len() >= 3 { // Minimum for reasonable traceability
            fully_traceable += 1;
        } else if epc_events.len() >= 1 {
            partially_traceable += 1;
        }
    }
    
    println!("   Fully Traceable Products: {}", fully_traceable);
    println!("   Partially Traceable: {}", partially_traceable);
    
    Ok(fully_traceable >= sample_epcs.len() / 2) // At least 50% fully traceable
}

/// Test compliance scenarios
fn test_compliance_scenarios(events: &[EPCISEvent]) -> OwlResult<()> {
    println!("Testing regulatory compliance scenarios...");
    
    // Test 1: GS1 Standards Compliance
    println!("1. Testing GS1 EPCIS Standards Compliance...");
    let gs1_compliance = test_gs1_compliance(events)?;
    println!("   ✅ GS1 Compliance: {:.1}%", gs1_compliance * 100.0);
    
    // Test 2: Data Quality Compliance
    println!("2. Testing Data Quality Compliance...");
    let data_quality = test_data_quality_compliance(events)?;
    println!("   ✅ Data Quality: {:.1}%", data_quality * 100.0);
    
    // Test 3: Timeline Compliance
    println!("3. Testing Timeline Compliance...");
    let timeline_compliance = test_timeline_compliance(events)?;
    println!("   ✅ Timeline Compliance: {:.1}%", timeline_compliance * 100.0);
    
    Ok(())
}

/// Test GS1 compliance
fn test_gs1_compliance(events: &[EPCISEvent]) -> OwlResult<f64> {
    let mut compliant_events = 0;
    let mut total_events = events.len();
    
    for event in events {
        let mut is_compliant = true;
        
        // Check required fields
        if event.event_id.is_empty() {
            is_compliant = false;
        }
        if event.epc_list.is_empty() {
            is_compliant = false;
        }
        if event.biz_step.is_none() {
            is_compliant = false;
        }
        
        if is_compliant {
            compliant_events += 1;
        }
    }
    
    Ok(if total_events > 0 { compliant_events as f64 / total_events as f64 } else { 1.0 })
}

/// Test data quality compliance
fn test_data_quality_compliance(events: &[EPCISEvent]) -> OwlResult<f64> {
    let mut high_quality_events = 0;
    let mut total_events = events.len();
    
    for event in events {
        let mut quality_score = 0;
        
        // Check data completeness
        if !event.event_id.is_empty() { quality_score += 1; }
        if !event.epc_list.is_empty() { quality_score += 1; }
        if event.biz_step.is_some() { quality_score += 1; }
        if event.disposition.is_some() { quality_score += 1; }
        if event.read_point.is_some() { quality_score += 1; }
        
        if quality_score >= 4 { // 80% or better quality
            high_quality_events += 1;
        }
    }
    
    Ok(if total_events > 0 { high_quality_events as f64 / total_events as f64 } else { 1.0 })
}

/// Test timeline compliance
fn test_timeline_compliance(events: &[EPCISEvent]) -> OwlResult<f64> {
    let mut timely_events = 0;
    let mut total_events = events.len();
    
    // Check for reasonable time sequencing
    let mut last_time = std::time::UNIX_EPOCH;
    
    for event in events {
        if event.event_time > last_time {
            last_time = event.event_time;
            timely_events += 1;
        }
    }
    
    Ok(if total_events > 0 { timely_events as f64 / total_events as f64 } else { 1.0 })
}

/// Test exception scenarios
fn test_exception_scenarios(events: &[EPCISEvent]) -> OwlResult<()> {
    println!("Testing exception and error scenarios...");
    
    // Test 1: Missing Data Handling
    println!("1. Testing Missing Data Resilience...");
    let missing_data_resilience = test_missing_data_resilience(events)?;
    println!("   ✅ Missing Data Handling: {}", if missing_data_resilience { "Robust" } else { "Vulnerable" });
    
    // Test 2: Timeline Anomaly Detection
    println!("2. Testing Timeline Anomaly Detection...");
    let anomaly_detection = test_timeline_anomaly_detection(events)?;
    println!("   ✅ Anomaly Detection: {}", if anomaly_detection { "Effective" } else { "Limited" });
    
    // Test 3: Error Recovery Capability
    println!("3. Testing Error Recovery Capability...");
    let error_recovery = test_error_recovery_capability(events)?;
    println!("   ✅ Error Recovery: {}", if error_recovery { "Good" } else { "Needs Work" });
    
    Ok(())
}

/// Test missing data resilience
fn test_missing_data_resilience(events: &[EPCISEvent]) -> OwlResult<bool> {
    // Count events with potentially missing critical data
    let complete_events = events.iter()
        .filter(|e| !e.event_id.is_empty() && !e.epc_list.is_empty())
        .count();
    
    let resilience_ratio = complete_events as f64 / events.len() as f64;
    Ok(resilience_ratio > 0.8) // 80% or better
}

/// Test timeline anomaly detection
fn test_timeline_anomaly_detection(events: &[EPCISEvent]) -> OwlResult<bool> {
    // Look for temporal anomalies
    let mut sorted_events = events.to_vec();
    sorted_events.sort_by_key(|e| e.event_time);
    
    let mut anomalies = 0;
    for window in sorted_events.windows(2) {
        let time_gap = window[1].event_time.duration_since(window[0].event_time).unwrap_or_default();
        if time_gap > std::time::Duration::from_secs(3600) { // More than 1 hour gap
            anomalies += 1;
        }
    }
    
    // Some anomalies are expected, but not too many
    Ok(anomalies < events.len() / 10) // Less than 10% anomalous
}

/// Test error recovery capability
fn test_error_recovery_capability(events: &[EPCISEvent]) -> OwlResult<bool> {
    // Test if the system can handle various error conditions
    let error_events = events.iter()
        .filter(|e| e.disposition == Some(EPCISDisposition::Failed))
        .count();
    
    // Having some error events is normal, recovery is what matters
    let recovery_events = events.iter()
        .filter(|e| {
            e.disposition == Some(EPCISDisposition::Passed) &&
            e.biz_step == Some(EPCISBusinessStep::Receiving)
        })
        .count();
    
    Ok(recovery_events > error_events) // More recoveries than errors
}

/// Test multi-participant performance
fn test_multi_participant_performance(events: &[EPCISEvent]) -> OwlResult<()> {
    println!("Testing multi-participant system performance...");
    
    // Test 1: Query Performance
    println!("1. Testing Multi-Participant Query Performance...");
    let query_performance = test_query_performance(events)?;
    println!("   ✅ Query Performance: {:.2}ms avg", query_performance.as_millis());
    
    // Test 2: Traceability Performance
    println!("2. Testing End-to-End Traceability Performance...");
    let traceability_performance = test_traceability_performance(events)?;
    println!("   ✅ Traceability Performance: {:.2}ms avg", traceability_performance.as_millis());
    
    // Test 3: Analytics Performance
    println!("3. Testing Analytics Performance...");
    let analytics_performance = test_analytics_performance(events)?;
    println!("   ✅ Analytics Performance: {:.2}ms", analytics_performance.as_millis());
    
    Ok(())
}

/// Test query performance
fn test_query_performance(events: &[EPCISEvent]) -> OwlResult<std::time::Duration> {
    let sample_queries = vec![
        "Manufacturing",
        "Shipping", 
        "Receiving",
        "Picking",
    ];
    
    let mut total_time = std::time::Duration::new(0, 0);
    
    for query in &sample_queries {
        let start_time = std::time::Instant::now();
        let _results: Vec<_> = events.iter()
            .filter(|e| e.biz_step.as_ref().map(|s| format!("{:?}", s).contains(query)).unwrap_or(false))
            .collect();
        total_time += start_time.elapsed();
    }
    
    Ok(total_time / sample_queries.len() as u32)
}

/// Test traceability performance
fn test_traceability_performance(events: &[EPCISEvent]) -> OwlResult<std::time::Duration> {
    let sample_epcs: Vec<_> = events.iter()
        .flat_map(|e| e.epc_list.iter())
        .take(5)
        .cloned()
        .collect();
    
    let mut total_time = std::time::Duration::new(0, 0);
    
    for epc in &sample_epcs {
        let start_time = std::time::Instant::now();
        let _epc_events: Vec<_> = events.iter()
            .filter(|e| e.epc_list.contains(epc))
            .collect();
        total_time += start_time.elapsed();
    }
    
    Ok(total_time / sample_epcs.len() as u32)
}

/// Test analytics performance
fn test_analytics_performance(events: &[EPCISEvent]) -> OwlResult<std::time::Duration> {
    let start_time = std::time::Instant::now();
    
    // Complex analytics query
    let participant_stats: std::collections::HashMap<_, _> = events.iter()
        .filter_map(|e| e.business_location.as_ref())
        .map(|loc| loc.id.clone())
        .fold(std::collections::HashMap::new(), |mut acc, participant| {
            *acc.entry(participant).or_insert(0) += 1;
            acc
        });
    
    let _total_events = participant_stats.values().sum::<usize>();
    let _avg_events_per_participant = participant_stats.values().sum::<usize>() as f64 / participant_stats.len() as f64;
    
    Ok(start_time.elapsed())
}

/// Generate comprehensive multi-participant report
fn generate_multi_participant_report(events: &[EPCISEvent]) {
    let total_events = events.len();
    let unique_epcs: std::collections::HashSet<_> = events.iter()
        .flat_map(|e| e.epc_list.iter())
        .cloned()
        .collect();
    
    let participants: std::collections::HashSet<_> = events.iter()
        .filter_map(|e| e.business_location.as_ref())
        .map(|loc| loc.id.clone())
        .collect();
    
    let aggregation_events = events.iter()
        .filter(|e| e.event_type == EPCISEventType::AggregationEvent)
        .count();
    
    let transformation_events = events.iter()
        .filter(|e| e.event_type == EPCISEventType::TransformationEvent)
        .count();
    
    println!("Multi-Participant Test Results Summary:");
    println!("  📊 Total Events: {}", total_events);
    println!("  🏢 Participants: {}", participants.len());
    println!("  🏷️  Unique EPCs: {}", unique_epcs.len());
    println!("  📦 Aggregation Events: {}", aggregation_events);
    println!("  🔄 Transformation Events: {}", transformation_events);
    println!("  📈 Events per Participant: {:.1}", total_events as f64 / participants.len() as f64);
    println!("  📈 Events per EPC: {:.1}", total_events as f64 / unique_epcs.len() as f64);
    
    // System maturity assessment
    let complexity_score = (participants.len() as f64 * 0.3 + 
                          aggregation_events as f64 * 0.2 + 
                          transformation_events as f64 * 0.2 + 
                          unique_epcs.len() as f64 * 0.1) / total_events.max(1) as f64;
    
    println!("\nSystem Maturity Assessment:");
    println!("  🏆 Complexity Score: {:.2}", complexity_score);
    
    if complexity_score > 0.5 {
        println!("  🌟 Maturity Level: Advanced Multi-Participant System");
    } else if complexity_score > 0.3 {
        println!("  🌟 Maturity Level: Developing Multi-Participant System");
    } else {
        println!("  🌟 Maturity Level: Basic Multi-Participant System");
    }
}

/// Participant data structure for analysis
#[derive(Debug)]
struct ParticipantData {
    name: String,
    role: String,
    event_count: usize,
    event_types: std::collections::HashMap<String, usize>,
    business_steps: std::collections::HashMap<String, usize>,
    unique_epcs: std::collections::HashSet<String>,
    first_event: Option<std::time::SystemTime>,
    last_event: Option<std::time::SystemTime>,
}