//! OWL2 Reasoner EPCIS Ecosystem Integration Examples
//!
//! This example demonstrates how to use the OWL2 reasoner with EPCIS data
//! across different ecosystem integration points.

use owl2_reasoner::epcis_parser::*;
use owl2_reasoner::reasoning::SimpleReasoner;
use owl2_reasoner::profiles::Owl2Profile;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 **OWL2 Reasoner EPCIS Ecosystem Integration**");
    println!("{}", "=".repeat(60));

    // Create sample EPCIS events for demonstration
    let sample_events = create_sample_epcis_events();

    println!("📊 **Sample EPCIS Events Created**");
    println!("   {} events spanning supply chain lifecycle", sample_events.len());

    // Example 1: Core EPCIS Processing with OWL2 Reasoning
    println!("\n1️⃣ **Core EPCIS Processing Example**");
    println!("{}", "=".repeat(40));
    demonstrate_core_processing(&sample_events)?;

    // Example 2: Python Integration Pattern
    println!("\n2️⃣ **Python Integration Example**");
    println!("{}", "=".repeat(40));
    demonstrate_python_integration()?;

    // Example 3: Web Service Integration Pattern
    println!("\n3️⃣ **Web Service Integration Example**");
    println!("{}", "=".repeat(40));
    demonstrate_web_service_integration()?;

    // Example 4: Data Processing Pipeline
    println!("\n4️⃣ **Data Processing Pipeline Example**");
    println!("{}", "=".repeat(40));
    demonstrate_data_pipeline(&sample_events)?;

    // Example 5: Multi-Language Client Integration
    println!("\n5️⃣ **Multi-Language Client Integration**");
    println!("{}", "=".repeat(40));
    demonstrate_client_integration()?;

    println!("\n🎉 **EPCIS Ecosystem Integration Complete**");
    println!("The OWL2 reasoner provides comprehensive EPCIS integration capabilities");
    println!("across multiple platforms and languages.");

    Ok(())
}

fn create_sample_epcis_events() -> Vec<EPCISSimpleEvent> {
    let mut events = Vec::new();

    // Manufacturing event
    events.push(EPCISSimpleEvent {
        event_id: "mfg_001".to_string(),
        event_type: "ObjectEvent".to_string(),
        event_time: "2023-01-01T08:00:00Z".to_string(),
        epcs: vec!["urn:epc:id:sgtin:0614141.107346.1001".to_string()],
        biz_step: Some("urn:epcglobal:cbv:bizstep:manufacturing".to_string()),
        disposition: Some("urn:epcglobal:cbv:disp:in_progress".to_string()),
        action: "ADD".to_string(),
    });

    // Quality inspection event
    events.push(EPCISSimpleEvent {
        event_id: "qc_001".to_string(),
        event_type: "ObjectEvent".to_string(),
        event_time: "2023-01-01T09:00:00Z".to_string(),
        epcs: vec!["urn:epc:id:sgtin:0614141.107346.1001".to_string()],
        biz_step: Some("urn:epcglobal:cbv:bizstep:inspecting".to_string()),
        disposition: Some("urn:epcglobal:cbv:disp:in_progress".to_string()),
        action: "OBSERVE".to_string(),
    });

    // Shipping event
    events.push(EPCISSimpleEvent {
        event_id: "ship_001".to_string(),
        event_type: "ObjectEvent".to_string(),
        event_time: "2023-01-01T10:00:00Z".to_string(),
        epcs: vec!["urn:epc:id:sgtin:0614141.107346.1001".to_string()],
        biz_step: Some("urn:epcglobal:cbv:bizstep:shipping".to_string()),
        disposition: Some("urn:epcglobal:cbv:disp:in_transit".to_string()),
        action: "OBSERVE".to_string(),
    });

    // Receiving event at distributor
    events.push(EPCISSimpleEvent {
        event_id: "recv_001".to_string(),
        event_type: "ObjectEvent".to_string(),
        event_time: "2023-01-02T14:00:00Z".to_string(),
        epcs: vec!["urn:epc:id:sgtin:0614141.107346.1001".to_string()],
        biz_step: Some("urn:epcglobal:cbv:bizstep:receiving".to_string()),
        disposition: Some("urn:epcglobal:cbv:disp:in_stock".to_string()),
        action: "OBSERVE".to_string(),
    });

    // Retail sale event
    events.push(EPCISSimpleEvent {
        event_id: "sale_001".to_string(),
        event_type: "ObjectEvent".to_string(),
        event_time: "2023-01-03T16:00:00Z".to_string(),
        epcs: vec!["urn:epc:id:sgtin:0614141.107346.1001".to_string()],
        biz_step: Some("urn:epcglobal:cbv:bizstep:selling".to_string()),
        disposition: Some("urn:epcglobal:cbv:disp:sold".to_string()),
        action: "OBSERVE".to_string(),
    });

    events
}

fn demonstrate_core_processing(events: &[EPCISSimpleEvent]) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 **Processing EPCIS Events with OWL2 Reasoning**");

    // Parse EPCIS events and create ontology
    let parser = EPCISDocumentParser::default();
    let ontology = parser.to_ontology(events)?;

    // Create reasoner
    let mut reasoner = SimpleReasoner::new(ontology);

    // Extract and display basic statistics
    let stats = get_basic_statistics(&reasoner);
    println!("   📈 Ontology Statistics:");
    for (key, value) in stats {
        println!("      {}: {}", key, value);
    }

    // Perform consistency checking
    let is_consistent = reasoner.is_consistent()?;
    println!("   ✅ Consistency Check: {}", if is_consistent { "PASS" } else { "FAIL" });

    // Validate against OWL2 profiles
    let profiles = vec!["EL", "QL", "RL"];
    for profile in profiles {
        let result = match profile {
            "EL" => reasoner.validate_profile(Owl2Profile::EL),
            "QL" => reasoner.validate_profile(Owl2Profile::QL),
            "RL" => reasoner.validate_profile(Owl2Profile::RL),
            _ => continue,
        };

        match result {
            Ok(validation) => println!("   📋 {} Profile: {}", profile,
                if validation.is_valid { "VALID" } else { "INVALID" }),
            Err(e) => println!("   ❌ {} Profile Error: {}", profile, e),
        }
    }

    // Extract supply chain insights
    let insights = extract_supply_chain_insights(events);
    println!("   🔍 Supply Chain Insights:");
    for insight in insights {
        println!("      • {}", insight);
    }

    Ok(())
}

fn demonstrate_python_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐍 **Python Integration Pattern**");

    // Show Python code example
    let python_example = r#"""
# Python example using OWL2 reasoner bindings
import owl2_reasoner_python
import json

# Create EPCIS parser
parser = owl2_reasoner_python.PyEPCISParser()

# Parse EPCIS XML data
events = parser.parse_xml_string("""
<EPCISDocument>
    <EPCISBody>
        <EventList>
            <ObjectEvent>
                <eventTime>2023-01-01T10:00:00Z</eventTime>
                <epcList>
                    <epc>urn:epc:id:sgtin:0614141.107346.1001</epc>
                </epcList>
                <action>ADD</action>
                <bizStep>urn:epcglobal:cbv:bizstep:receiving</bizStep>
            </ObjectEvent>
        </EventList>
    </EPCISBody>
</EPCISDocument>
""")

print(f"Parsed {len(events)} EPCIS events")

# Create OWL2 reasoner
reasoner = owl2_reasoner_python.PyOWL2Reasoner()

# Load EPCIS events into reasoner
reasoner.load_epcis_events(events)

# Perform reasoning operations
is_consistent = reasoner.is_consistent()
print(f"Ontology consistency: {is_consistent}")

# Validate OWL2 profiles
el_valid = reasoner.validate_el_profile()
ql_valid = reasoner.validate_ql_profile()
rl_valid = reasoner.validate_rl_profile()

print(f"EL Profile: {el_valid}")
print(f"QL Profile: {ql_valid}")
print(f"RL Profile: {rl_valid}")

# Get statistics
stats = reasoner.get_statistics()
print("Ontology Statistics:")
for key, value in stats.items():
    print(f"  {key}: {value}")

# Data science integration with pandas
import pandas as pd

# Convert events to DataFrame
event_data = []
for event in events:
    event_data.append({
        'event_id': event.event_id,
        'event_type': event.event_type,
        'timestamp': event.event_time,
        'epc_count': len(event.epcs),
        'business_step': event.biz_step,
        'disposition': event.disposition
    })

df = pd.DataFrame(event_data)
print("\nEPCIS Events DataFrame:")
print(df)

# Analyze supply chain patterns
print("\nBusiness Step Distribution:")
print(df['business_step'].value_counts())
    """#;

    println!("   📝 Python Integration Example:");
    println!("   ```python{}", python_example);

    println!("   ```");
    println!("   🔧 **Key Integration Points**:");
    println!("      • Native Python bindings via PyO3");
    println!("      • Seamless EPCIS XML parsing");
    println!("      • OWL2 reasoning operations");
    println!("      • Pandas/NumPy integration");
    println!("      • Data science workflow support");

    Ok(())
}

fn demonstrate_web_service_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 **Web Service Integration Pattern**");

    // Show API usage examples
    println!("   📡 **REST API Endpoints**:");

    let api_examples = vec![
        ("POST /epcis", "Upload EPCIS XML data"),
        ("POST /reasoning", "Perform reasoning operations"),
        ("POST /analysis", "Analyze traceability"),
        ("GET /statistics", "Get ontology statistics"),
        ("GET /health", "Service health check"),
    ];

    for (endpoint, description) in api_examples {
        println!("      {} - {}", endpoint, description);
    }

    // Show curl examples
    let curl_examples = r#"""
# Upload EPCIS data
curl -X POST http://localhost:3030/epcis \
  -H "Content-Type: application/json" \
  -d '{
    "xml_content": "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
      <EPCISDocument>...</EPCISDocument>"
  }'

# Perform reasoning
curl -X POST http://localhost:3030/reasoning \
  -H "Content-Type: application/json" \
  -d '{
    "check_consistency": true,
    "validate_profiles": ["EL", "QL", "RL"],
    "get_statistics": true
  }'

# Analyze traceability
curl -X POST http://localhost:3030/analysis \
  -H "Content-Type: application/json" \
  -d '{
    "extract_epcs": true,
    "traceability_analysis": true,
    "business_steps": true
  }'
    """#;

    println!("\n   📜 **Client Usage Examples**:");
    println!("   ```bash{}", curl_examples);

    println!("   ```");

    // Show JavaScript/Node.js example
    let js_example = r#"""
// JavaScript/Node.js integration
const fetch = require('node-fetch');

async function processEPCISData(xmlContent) {
    try {
        // Upload EPCIS data
        const uploadResponse = await fetch('http://localhost:3030/epcis', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ xml_content: xmlContent })
        });

        const uploadResult = await uploadResponse.json();
        console.log('Upload result:', uploadResult);

        // Perform reasoning
        const reasoningResponse = await fetch('http://localhost:3030/reasoning', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                check_consistency: true,
                validate_profiles: ['EL', 'QL', 'RL'],
                get_statistics: true
            })
        });

        const reasoningResult = await reasoningResponse.json();
        console.log('Reasoning result:', reasoningResult);

        return reasoningResult;
    } catch (error) {
        console.error('Error:', error);
    }
}
    """#;

    println!("   📱 **JavaScript Integration**:");
    println!("   ```javascript{}", js_example);

    println!("   ```");

    Ok(())
}

fn demonstrate_data_pipeline(events: &[EPCISSimpleEvent]) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️ **Data Processing Pipeline Pattern**");

    println!("   🔄 **Pipeline Stages**:");
    println!("      1. Data Ingestion - EPCIS XML parsing");
    println!("      2. Validation - Schema and business rule validation");
    println!("      3. Reasoning - OWL2 inference and consistency checking");
    println!("      4. Analysis - Traceability and pattern detection");
    println!("      5. Output - JSON, XML, CSV, or database storage");

    // Simulate pipeline processing
    println!("\n   📊 **Pipeline Processing Simulation**:");

    // Stage 1: Data Ingestion
    println!("      Stage 1 - Data Ingestion: ✅ {} events ingested", events.len());

    // Stage 2: Validation
    let validation_results = validate_epcis_events(events);
    println!("      Stage 2 - Validation: ✅ {} valid events", validation_results.valid_count);
    if !validation_results.errors.is_empty() {
        println!("                        ❌ {} validation errors", validation_results.errors.len());
    }

    // Stage 3: Reasoning
    let parser = EPCISDocumentParser::default();
    let ontology = parser.to_ontology(events)?;
    let reasoner = SimpleReasoner::new(ontology);
    let is_consistent = reasoner.is_consistent()?;
    println!("      Stage 3 - Reasoning: ✅ Consistency check {}", if is_consistent { "PASS" } else { "FAIL" });

    // Stage 4: Analysis
    let analysis_results = analyze_epcis_data(events, &reasoner);
    println!("      Stage 4 - Analysis: ✅ {} insights generated", analysis_results.len());

    // Stage 5: Output
    println!("      Stage 5 - Output: ✅ Ready for export");

    // Show pipeline configuration
    let pipeline_config = r#"""
# Pipeline Configuration
[pipeline]
name = "EPCIS Supply Chain Pipeline"
batch_size = 1000
parallel_processing = true

[inputs]
epcis_xml_files = ["data/*.xml"]
api_endpoints = ["https://api.supplychain.com/epcis"]

[processing]
validation_level = "strict"
reasoning_profiles = ["EL", "QL"]
traceability_analysis = true

[outputs]
database_url = "postgresql://localhost/epcis_db"
export_formats = ["json", "csv", "xml"]
real_time_streaming = true
    """#;

    println!("\n   ⚙️ **Pipeline Configuration**:");
    println!("   ```toml{}", pipeline_config);

    println!("   ```");

    Ok(())
}

fn demonstrate_client_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 **Multi-Language Client Integration**");

    // Show integration patterns for different languages
    let integration_examples = vec![
        ("Python", "PyO3 native bindings with pandas integration"),
        ("JavaScript/Node.js", "REST API client with WebSocket support"),
        ("Java", "REST API client with Apache HttpClient"),
        ("C#", "REST API client with HttpClient"),
        ("Go", "REST API client with net/http"),
        ("Rust", "Native library with FFI support"),
        ("Ruby", "REST API client with Net::HTTP"),
        ("PHP", "REST API client with Guzzle"),
    ];

    for (language, description) in integration_examples {
        println!("      🌐 {} - {}", language, description);
    }

    // Show client architecture
    println!("\n   🏗️ **Client Architecture Pattern**:");
    println!("      ┌─────────────────┐    ┌─────────────────┐");
    println!("      │   Client App    │    │   Web Browser   │");
    println!("      │  (Python/JS/Java)│    │  (React/Vue/Ang)│");
    println!("      └─────────┬───────┘    └─────────┬───────┘");
    println!("                │                      │");
    println!("                ▼                      ▼");
    println!("      ┌─────────────────┐    ┌─────────────────┐");
    println!("      │  Native Libs   │    │  REST API       │");
    println!("      │  (PyO3/FFI)    │    │  (Warp Server)   │");
    println!("      └─────────┬───────┘    └─────────┬───────┘");
    println!("                │                      │");
    println!("                └──────┬─────────────────┘");
    println!("                       ▼");
    println!("              ┌─────────────────┐");
    println!("              │ OWL2 Reasoner   │");
    println!("              │   Core Engine   │");
    println!("              └─────────────────┘");

    // Show integration code examples
    println!("\n   💻 **Integration Code Examples**:");

    let java_example = r#"""
// Java integration example
import java.net.http.*;
import java.net.URI;
import com.fasterxml.jackson.databind.*;

public class EPCISClient {
    private final HttpClient client;
    private final ObjectMapper mapper;
    private final String baseUrl;

    public EPCISClient(String baseUrl) {
        this.client = HttpClient.newHttpClient();
        this.mapper = new ObjectMapper();
        this.baseUrl = baseUrl;
    }

    public void uploadEPCISData(String xmlContent) throws Exception {
        var requestBody = Map.of("xml_content", xmlContent);
        var request = HttpRequest.newBuilder()
            .uri(URI.create(baseUrl + "/epcis"))
            .header("Content-Type", "application/json")
            .POST(HttpRequest.BodyPublishers.ofString(
                mapper.writeValueAsString(requestBody)))
            .build();

        var response = client.send(request,
            HttpResponse.BodyHandlers.ofString());
        System.out.println("Upload response: " + response.body());
    }
}
    """#;

    println!("   ☕ **Java Client Example**:");
    println!("   ```java{}", java_example);

    println!("   ```");

    Ok(())
}

// Helper functions
fn get_basic_statistics(reasoner: &SimpleReasoner) -> HashMap<String, usize> {
    let mut stats = HashMap::new();
    stats.insert("classes".to_string(), reasoner.ontology.classes().len());
    stats.insert("object_properties".to_string(), reasoner.ontology.object_properties().len());
    stats.insert("data_properties".to_string(), reasoner.ontology.data_properties().len());
    stats.insert("individuals".to_string(), reasoner.ontology.named_individuals().len());
    stats.insert("axioms".to_string(), reasoner.ontology.axioms().len());
    stats
}

struct ValidationResult {
    valid_count: usize,
    errors: Vec<String>,
}

fn validate_epcis_events(events: &[EPCISSimpleEvent]) -> ValidationResult {
    let mut errors = Vec::new();
    let mut valid_count = 0;

    for event in events {
        if event.event_id.is_empty() {
            errors.push(format!("Event missing ID: {:?}", event));
            continue;
        }
        if event.epcs.is_empty() {
            errors.push(format!("Event missing EPCs: {}", event.event_id));
            continue;
        }
        valid_count += 1;
    }

    ValidationResult { valid_count, errors }
}

fn extract_supply_chain_insights(events: &[EPCISSimpleEvent]) -> Vec<String> {
    let mut insights = Vec::new();

    // Count events by type
    let mut event_counts = HashMap::new();
    for event in events {
        *event_counts.entry(&event.event_type).or_insert(0) += 1;
    }

    insights.push(format!("Event type distribution: {:?}", event_counts));

    // Count unique EPCs
    let unique_epcs: std::collections::HashSet<_> = events
        .iter()
        .flat_map(|e| &e.epcs)
        .collect();
    insights.push(format!("Unique EPCs tracked: {}", unique_epcs.len()));

    // Analyze business steps
    let business_steps: std::collections::HashSet<_> = events
        .iter()
        .filter_map(|e| e.biz_step.as_ref())
        .collect();
    insights.push(format!("Business steps involved: {}", business_steps.len()));

    insights
}

fn analyze_epcis_data(events: &[EPCISSimpleEvent], _reasoner: &SimpleReasoner) -> Vec<String> {
    let mut insights = Vec::new();

    // Time span analysis
    if let (Some(first), Some(last)) = (events.first(), events.last()) {
        insights.push(format!("Time span from {} to {}", first.event_time, last.event_time));
    }

    // Action distribution
    let action_counts: HashMap<_, _> = events
        .iter()
        .map(|e| &e.action)
        .fold(HashMap::new(), |mut acc, action| {
            *acc.entry(action).or_insert(0) += 1;
            acc
        });
    insights.push(format!("Action distribution: {:?}", action_counts));

    insights
}