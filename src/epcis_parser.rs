//! EPCIS Document Parser
//! 
//! This module provides parsers for EPCIS documents in XML and JSON formats,
//! converting them into OWL2 ontologies for reasoning.

use crate::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// EPCIS document parser configuration
#[derive(Debug, Clone)]
pub struct EPCISParserConfig {
    /// Whether to validate against EPCIS schema
    pub validate_schema: bool,
    /// Whether to include all extensions and extensions
    pub include_extensions: bool,
    /// Custom namespace mappings
    pub namespace_mappings: HashMap<String, String>,
}

impl Default for EPCISParserConfig {
    fn default() -> Self {
        let mut namespace_mappings = HashMap::new();
        namespace_mappings.insert("epcis".to_string(), "urn:epcglobal:epcis:xsd:2".to_string());
        namespace_mappings.insert("cbvmda".to_string(), "urn:epcglobal:cbv:mda".to_string());
        namespace_mappings.insert("gs1".to_string(), "urn:epcglobal:gs1".to_string());
        
        Self {
            validate_schema: true,
            include_extensions: true,
            namespace_mappings,
        }
    }
}

/// Simple EPCIS event representation
#[derive(Debug, Clone)]
pub struct EPCISSimpleEvent {
    pub event_id: String,
    pub event_type: String,
    pub event_time: String,
    pub epcs: Vec<String>,
    pub biz_step: Option<String>,
    pub disposition: Option<String>,
    pub action: String,
}

/// EPCIS Document Parser - Simplified version for compilation
pub struct EPCISDocumentParser {
    config: EPCISParserConfig,
}

impl EPCISDocumentParser {
    /// Create a new EPCIS document parser
    pub fn new(config: EPCISParserConfig) -> Self {
        Self { config }
    }

    /// Parse an EPCIS XML document from file (basic implementation)
    pub fn parse_xml_file<P: AsRef<Path>>(&self, path: P) -> OwlResult<Vec<EPCISSimpleEvent>> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        self.parse_xml_str(&content)
    }

    /// Parse an EPCIS XML document from string (basic implementation)
    pub fn parse_xml_str(&self, content: &str) -> OwlResult<Vec<EPCISSimpleEvent>> {
        // Simple XML parsing for ObjectEvents
        let mut events = Vec::new();
        
        // Extract ObjectEvent data using basic string parsing
        for event_match in content.matches("<ObjectEvent>") {
            let event_start = content.find(event_match).unwrap();
            let event_end = content[event_start..].find("</ObjectEvent>").unwrap() + event_start + 14;
            let event_content = &content[event_start..event_end];
            
            if let Some(event) = self.parse_object_event(event_content) {
                events.push(event);
            }
        }
        
        Ok(events)
    }

    /// Parse a single object event from XML content
    fn parse_object_event(&self, content: &str) -> Option<EPCISSimpleEvent> {
        let event = EPCISSimpleEvent {
            event_id: self.extract_xml_field(content, "eventID").unwrap_or_else(|| format!("event_{}", rand::random::<u64>())),
            event_type: "ObjectEvent".to_string(),
            event_time: self.extract_xml_field(content, "eventTime").unwrap_or_default(),
            epcs: self.extract_epc_list(content),
            biz_step: self.extract_xml_field(content, "bizStep"),
            disposition: self.extract_xml_field(content, "disposition"),
            action: self.extract_xml_field(content, "action").unwrap_or_else(|| "ADD".to_string()),
        };
        
        Some(event)
    }

    /// Extract a field from XML content
    fn extract_xml_field(&self, content: &str, field_name: &str) -> Option<String> {
        let start_tag = format!("<{}>", field_name);
        let end_tag = format!("</{}>", field_name);
        
        if let Some(start) = content.find(&start_tag) {
            if let Some(end) = content[start + start_tag.len()..].find(&end_tag) {
                let value = &content[start + start_tag.len()..start + start_tag.len() + end];
                return Some(value.trim().to_string());
            }
        }
        None
    }

    /// Extract EPC list from XML content
    fn extract_epc_list(&self, content: &str) -> Vec<String> {
        let mut epcs = Vec::new();
        
        if let Some(epc_list_start) = content.find("<epcList>") {
            if let Some(epc_list_end) = content[epc_list_start..].find("</epcList>") {
                let epc_list_content = &content[epc_list_start + 9..epc_list_start + epc_list_end];
                
                // Extract individual EPCs
                for epc_match in epc_list_content.matches("<epc>") {
                    let epc_start = epc_list_content.find(epc_match).unwrap();
                    let epc_end = epc_list_content[epc_start..].find("</epc>").unwrap() + epc_start + 6;
                    let epc_value = &epc_list_content[epc_start + 5..epc_end];
                    if !epc_value.trim().is_empty() {
                        epcs.push(epc_value.trim().to_string());
                    }
                }
            }
        }
        
        epcs
    }

    /// Convert EPCIS events to OWL2 ontology
    pub fn to_ontology(&self, events: &[EPCISSimpleEvent]) -> OwlResult<Ontology> {
        let mut ontology = Ontology::new();

        // Add core EPCIS classes
        self.add_epcis_classes(&mut ontology)?;

        // Add events
        self.add_events(&mut ontology, events)?;

        Ok(ontology)
    }

    /// Add core EPCIS classes to ontology
    fn add_epcis_classes(&self, ontology: &mut Ontology) -> OwlResult<()> {
        // Core event classes
        let event_class = Class::new("http://ns.gs1.org/epcis/Event".to_string());
        let object_event_class = Class::new("http://ns.gs1.org/epcis/ObjectEvent".to_string());
        let aggregation_event_class = Class::new("http://ns.gs1.org/epcis/AggregationEvent".to_string());
        let transaction_event_class = Class::new("http://ns.gs1.org/epcis/TransactionEvent".to_string());
        let transformation_event_class = Class::new("http://ns.gs1.org/epcis/TransformationEvent".to_string());

        // Add class declarations
        ontology.add_class(event_class.clone())?;
        ontology.add_class(object_event_class.clone())?;
        ontology.add_class(aggregation_event_class.clone())?;
        ontology.add_class(transaction_event_class.clone())?;
        ontology.add_class(transformation_event_class.clone())?;

        // Add subclass relationships
        let object_subclass = SubClassOfAxiom::new(
            crate::axioms::class_expressions::ClassExpression::Class(object_event_class),
            crate::axioms::class_expressions::ClassExpression::Class(event_class.clone()),
        );
        let aggregation_subclass = SubClassOfAxiom::new(
            crate::axioms::class_expressions::ClassExpression::Class(aggregation_event_class),
            crate::axioms::class_expressions::ClassExpression::Class(event_class.clone()),
        );
        let transaction_subclass = SubClassOfAxiom::new(
            crate::axioms::class_expressions::ClassExpression::Class(transaction_event_class),
            crate::axioms::class_expressions::ClassExpression::Class(event_class.clone()),
        );
        let transformation_subclass = SubClassOfAxiom::new(
            crate::axioms::class_expressions::ClassExpression::Class(transformation_event_class),
            crate::axioms::class_expressions::ClassExpression::Class(event_class),
        );

        ontology.add_subclass_axiom(object_subclass)?;
        ontology.add_subclass_axiom(aggregation_subclass)?;
        ontology.add_subclass_axiom(transaction_subclass)?;
        ontology.add_subclass_axiom(transformation_subclass)?;

        // Add business step and disposition classes
        let biz_step_class = Class::new("http://ns.gs1.org/cbv/BizStep".to_string());
        let disposition_class = Class::new("http://ns.gs1.org/cbv/Disp".to_string());
        ontology.add_class(biz_step_class)?;
        ontology.add_class(disposition_class)?;

        Ok(())
    }

    /// Add events to ontology
    fn add_events(&self, ontology: &mut Ontology, events: &[EPCISSimpleEvent]) -> OwlResult<()> {
        for event in events {
            self.add_simple_event(ontology, event)?;
        }
        Ok(())
    }

    /// Add a simple event to ontology
    fn add_simple_event(&self, ontology: &mut Ontology, event: &EPCISSimpleEvent) -> OwlResult<()> {
        // Add event as individual
        let event_individual = NamedIndividual::new(&event.event_id[..]);
        ontology.add_named_individual(event_individual)?;

        // Add EPC individuals
        for epc in &event.epcs {
            let epc_individual = NamedIndividual::new(&epc[..]);
            ontology.add_named_individual(epc_individual)?;
        }

        Ok(())
    }
}

/// Helper functions for EPCIS parsing
impl EPCISDocumentParser {
    /// Create parser with default configuration
    pub fn default() -> Self {
        Self::new(EPCISParserConfig::default())
    }

    /// Extract all EPCs from events
    pub fn extract_all_epcs(&self, events: &[EPCISSimpleEvent]) -> Vec<String> {
        let mut epcs = Vec::new();
        for event in events {
            epcs.extend(event.epcs.clone());
        }
        epcs.sort();
        epcs.dedup();
        epcs
    }

    /// Extract events by type
    pub fn extract_events_by_type(&self, events: &[EPCISSimpleEvent]) -> HashMap<String, usize> {
        let mut event_counts = HashMap::new();
        for event in events {
            *event_counts.entry(event.event_type.clone()).or_insert(0) += 1;
        }
        event_counts
    }

    /// Extract business steps from events
    pub fn extract_business_steps(&self, events: &[EPCISSimpleEvent]) -> Vec<String> {
        let mut steps = std::collections::HashSet::new();
        for event in events {
            if let Some(step) = &event.biz_step {
                steps.insert(step.clone());
            }
        }
        let mut result: Vec<_> = steps.into_iter().collect();
        result.sort();
        result
    }
}

/// Simple EPCIS Document Writer
pub struct EPCISDocumentWriter {
    base_uri: String,
}

impl EPCISDocumentWriter {
    /// Create a new EPCIS document writer
    pub fn new() -> Self {
        Self {
            base_uri: "http://ns.gs1.org/epcis/".to_string(),
        }
    }

    /// Write ontology to EPCIS XML format (placeholder)
    pub fn write_xml(&self, _ontology: &Ontology) -> OwlResult<String> {
        Ok(r#"<?xml version="1.0" encoding="UTF-8"?>
<EPCISDocument xmlns="urn:epcglobal:epcis:xsd:2" schemaVersion="2.0">
    <EventList>
    </EventList>
</EPCISDocument>"#.to_string())
    }

    /// Write ontology to EPCIS JSON format (placeholder)
    pub fn write_json(&self, _ontology: &Ontology) -> OwlResult<String> {
        Ok(r#"{
    "@context": "https://gs1.github.io/EPCIS/epcis-context.jsonld",
    "schemaVersion": "2.0",
    "EventList": []
}"#.to_string())
    }
}

impl Default for EPCISDocumentWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parser_initialization() {
        let parser = EPCISDocumentParser::new(EPCISParserConfig::default());
        assert_eq!(parser.config.validate_schema, true);
    }
    
    #[test]
    fn test_writer_initialization() {
        let writer = EPCISDocumentWriter::new();
        assert_eq!(writer.base_uri, "http://ns.gs1.org/epcis/");
    }
    
    #[test]
    fn test_config_initialization() {
        let config = EPCISParserConfig::default();
        assert_eq!(config.validate_schema, true);
        assert_eq!(config.include_extensions, true);
        assert!(config.namespace_mappings.contains_key("epcis"));
    }
    
    #[test]
    fn test_simple_epcis_document_parsing() {
        let simple_xml = r#"
        <EPCISDocument xmlns="urn:epcglobal:epcis:xsd:2" schemaVersion="2.0">
            <EPCISBody>
                <EventList>
                    <ObjectEvent>
                        <eventTime>2023-01-01T00:00:00Z</eventTime>
                        <recordTime>2023-01-01T00:00:00Z</recordTime>
                        <eventTimeZoneOffset>+00:00</eventTimeZoneOffset>
                        <epcList>
                            <epc>urn:epc:id:sgtin:0614141.107346.2018</epc>
                        </epcList>
                        <action>ADD</action>
                        <bizStep>urn:epcglobal:cbv:bizstep:receiving</bizStep>
                        <disposition>urn:epcglobal:cbv:disp:in_progress</disposition>
                        <readPoint>
                            <id>urn:epc:id:sgln:0614141.00001.0</id>
                        </readPoint>
                    </ObjectEvent>
                </EventList>
            </EPCISBody>
        </EPCISDocument>
        "#;
        
        let parser = EPCISDocumentParser::new(EPCISParserConfig {
            validate_schema: false,
            include_extensions: false,
            namespace_mappings: HashMap::new(),
        });
        
        // Test parsing
        let result = parser.parse_xml_str(simple_xml);
        assert!(result.is_ok());
        
        if let Ok(events) = result {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].event_type, "ObjectEvent");
            assert_eq!(events[0].epcs.len(), 1);
            assert_eq!(events[0].epcs[0], "urn:epc:id:sgtin:0614141.107346.2018");
            
            // Test ontology conversion
            let ontology_result = parser.to_ontology(&events);
            assert!(ontology_result.is_ok());
        }
    }
    
    #[test]
    fn test_field_extraction() {
        let parser = EPCISDocumentParser::default();
        let content = r#"<ObjectEvent>
            <eventTime>2023-01-01T00:00:00Z</eventTime>
            <action>ADD</action>
            <epcList>
                <epc>urn:epc:id:sgtin:0614141.107346.2018</epc>
            </epcList>
        </ObjectEvent>"#;
        
        let event_time = parser.extract_xml_field(content, "eventTime");
        let action = parser.extract_xml_field(content, "action");
        let epcs = parser.extract_epc_list(content);
        
        assert_eq!(event_time, Some("2023-01-01T00:00:00Z".to_string()));
        assert_eq!(action, Some("ADD".to_string()));
        assert_eq!(epcs, vec!["urn:epc:id:sgtin:0614141.107346.2018"]);
    }
}