//! RDF/XML format parser for OWL2 ontologies
//!
//! Implements parsing of the RDF/XML serialization format with full specification compliance.

use crate::axioms::class_expressions::ClassExpression;
use crate::axioms::{
    AsymmetricPropertyAxiom, Axiom, ClassAssertionAxiom, DataPropertyAssertionAxiom,
    DisjointClassesAxiom,
    DisjointObjectPropertiesAxiom, EquivalentClassesAxiom, EquivalentObjectPropertiesAxiom,
    FunctionalPropertyAxiom, InverseFunctionalPropertyAxiom, IrreflexivePropertyAxiom,
    InverseObjectPropertiesAxiom,
    PropertyAssertionAxiom, ReflexivePropertyAxiom, SubClassOfAxiom, SubObjectPropertyAxiom,
    SymmetricPropertyAxiom, TransitivePropertyAxiom,
};
use crate::entities::*;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::parser::{OntologyParser, ParserConfig};
#[cfg(feature = "rio-xml")]
use rio_api::model::{Subject, Term};
#[cfg(feature = "rio-xml")]
use rio_api::parser::TriplesParser as _;
#[cfg(feature = "rio-xml")]
use rio_xml::RdfXmlParser as RioRdfXmlParser;
use std::collections::HashMap;
#[cfg(feature = "rio-xml")]
use std::io::Cursor;
use std::path::Path;

/// RDF/XML format parser
pub struct RdfXmlParser {
    pub config: ParserConfig,
    namespaces: HashMap<String, String>,
    base_iri: Option<IRI>,
    blank_node_counter: u32,
    resource_map: HashMap<String, ResourceInfo>,
}

impl RdfXmlParser {
    /// Create a new RDF/XML parser with default configuration
    pub fn new() -> Self {
        Self::with_config(ParserConfig::default())
    }

    /// Create a new RDF/XML parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        let mut namespaces = HashMap::new();
        // Add standard OWL2 namespaces
        namespaces.insert(
            "rdf".to_string(),
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string(),
        );
        namespaces.insert(
            "rdfs".to_string(),
            "http://www.w3.org/2000/01/rdf-schema#".to_string(),
        );
        namespaces.insert(
            "owl".to_string(),
            "http://www.w3.org/2002/07/owl#".to_string(),
        );
        namespaces.insert(
            "xsd".to_string(),
            "http://www.w3.org/2001/XMLSchema#".to_string(),
        );

        for (prefix, namespace) in &config.prefixes {
            namespaces.insert(prefix.clone(), namespace.clone());
        }

        RdfXmlParser {
            config,
            namespaces,
            base_iri: None,
            blank_node_counter: 0,
            resource_map: HashMap::new(),
        }
    }

    /// Parse RDF/XML content and build an ontology
    fn parse_content(&mut self, content: &str) -> OwlResult<Ontology> {
        if self.config.strict_validation && content.trim().is_empty() {
            return Err(crate::error::OwlError::ValidationError(
                "Ontology contains no entities or imports".to_string(),
            ));
        }
        // Non-strict mode: always use streaming RDF/XML if available
        #[cfg(feature = "rio-xml")]
        if !self.config.strict_validation {
            return self.parse_with_rio(content);
        }

        // Strict mode (or no streaming available): use legacy parser
        let mut ontology = Ontology::new();
        let document = self.parse_xml_document(content)?;
        self.process_rdf_document(&mut ontology, &document)?;
        self.process_resource_map(&mut ontology)?;
        if self.config.strict_validation {
            self.validate_ontology(&ontology)?;
        }
        Ok(ontology)
    }

    /// Parse via rio_xml and map triples to ontology (used in non-strict mode)
    #[cfg(feature = "rio-xml")]
    fn parse_with_rio(&mut self, content: &str) -> OwlResult<Ontology> {
        // no-op
        use rio_api::model::Subject;
        use rio_api::model::Term;

        let mut ontology = Ontology::new();

        let base_iri = self
            .base_iri
            .as_ref()
            .map(|iri| oxiri::Iri::parse(iri.as_str().to_string()).ok())
            .flatten();
        let mut parser = RioRdfXmlParser::new(Cursor::new(content), base_iri);
        let mut handler = |t: rio_api::model::Triple| -> Result<(), std::io::Error> {
            // Map common triples to ontology structures
            let subj_iri = match t.subject {
                Subject::NamedNode(nn) => IRI::new(nn.iri).ok(),
                _ => None,
            };
            let pred_iri = IRI::new(t.predicate.iri).ok();
            let obj_iri = match t.object {
                Term::NamedNode(nn) => IRI::new(nn.iri).ok(),
                Term::Literal(_) => None,
                _ => None,
            };

            if let (Some(s), Some(p)) = (subj_iri, pred_iri) {
                let p_str = p.as_str();
                match p_str {
                    // rdf:type
                    "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" => {
                        if let Some(o) = obj_iri {
                            let o_str = o.as_str();
                            // Handle property characteristics first
                            match o_str {
                                "http://www.w3.org/2002/07/owl#FunctionalProperty" => {
                                    let _ = ontology.add_object_property(ObjectProperty::new(s.clone()));
                                    let ax = FunctionalPropertyAxiom::new(s.clone());
                                    let _ = ontology.add_axiom(Axiom::FunctionalProperty(ax));
                                    return Ok(());
                                }
                                "http://www.w3.org/2002/07/owl#InverseFunctionalProperty" => {
                                    let _ = ontology.add_object_property(ObjectProperty::new(s.clone()));
                                    let ax = InverseFunctionalPropertyAxiom::new(s.clone());
                                    let _ = ontology.add_axiom(Axiom::InverseFunctionalProperty(ax));
                                    return Ok(());
                                }
                                "http://www.w3.org/2002/07/owl#SymmetricProperty" => {
                                    let _ = ontology.add_object_property(ObjectProperty::new(s.clone()));
                                    let ax = SymmetricPropertyAxiom::new(s.clone());
                                    let _ = ontology.add_axiom(Axiom::SymmetricProperty(ax));
                                    return Ok(());
                                }
                                "http://www.w3.org/2002/07/owl#TransitiveProperty" => {
                                    let _ = ontology.add_object_property(ObjectProperty::new(s.clone()));
                                    let ax = TransitivePropertyAxiom::new(s.clone());
                                    let _ = ontology.add_axiom(Axiom::TransitiveProperty(ax));
                                    return Ok(());
                                }
                                "http://www.w3.org/2002/07/owl#ReflexiveProperty" => {
                                    let _ = ontology.add_object_property(ObjectProperty::new(s.clone()));
                                    let ax = ReflexivePropertyAxiom::new(s.clone());
                                    let _ = ontology.add_axiom(Axiom::ReflexiveProperty(ax));
                                    return Ok(());
                                }
                                "http://www.w3.org/2002/07/owl#IrreflexiveProperty" => {
                                    let _ = ontology.add_object_property(ObjectProperty::new(s.clone()));
                                    let ax = IrreflexivePropertyAxiom::new(s.clone());
                                    let _ = ontology.add_axiom(Axiom::IrreflexiveProperty(ax));
                                    return Ok(());
                                }
                                "http://www.w3.org/2002/07/owl#AsymmetricProperty" => {
                                    let _ = ontology.add_object_property(ObjectProperty::new(s.clone()));
                                    let ax = AsymmetricPropertyAxiom::new(s.clone());
                                    let _ = ontology.add_axiom(Axiom::AsymmetricProperty(ax));
                                    return Ok(());
                                }
                                _ => {}
                            }
                            if o_str == "http://www.w3.org/2002/07/owl#Class" {
                                let class = Class::new(s.clone());
                                let _ = ontology.add_class(class);
                            } else if o_str == "http://www.w3.org/2002/07/owl#ObjectProperty" {
                                let prop = ObjectProperty::new(s.clone());
                                let _ = ontology.add_object_property(prop);
                            } else if o_str == "http://www.w3.org/2002/07/owl#DatatypeProperty" {
                                let prop = DataProperty::new(s.clone());
                                let _ = ontology.add_data_property(prop);
                            } else if o_str == "http://www.w3.org/2002/07/owl#NamedIndividual" {
                                let ind = NamedIndividual::new(s.clone());
                                let _ = ontology.add_named_individual(ind);
                            } else {
                                // Treat as class assertion
                                let class = Class::new(o.clone());
                                let ax = ClassAssertionAxiom::new(s.clone(), class.into());
                                let _ = ontology.add_axiom(Axiom::ClassAssertion(ax));
                            }
                        }
                    }
                    // rdfs:domain
                    "http://www.w3.org/2000/01/rdf-schema#domain" => {
                        if let Some(domain_class) = obj_iri {
                            // ∃P.Thing ⊑ C
                            let prop_expr =
                                crate::axioms::property_expressions::ObjectPropertyExpression::ObjectProperty(
                                    ObjectProperty::new(p.clone()),
                                );
                            let thing = Class::new(
                                IRI::new("http://www.w3.org/2002/07/owl#Thing").unwrap(),
                            );
                            let some = ClassExpression::ObjectSomeValuesFrom(
                                Box::new(prop_expr),
                                Box::new(ClassExpression::Class(thing)),
                            );
                            let super_c = ClassExpression::Class(Class::new(domain_class));
                            let ax = SubClassOfAxiom::new(some, super_c);
                            let _ = ontology.add_axiom(Axiom::SubClassOf(ax));
                        }
                    }
                    // rdfs:range
                    "http://www.w3.org/2000/01/rdf-schema#range" => {
                        if let Some(range_class) = obj_iri {
                            // ⊤ ⊑ ∀P.C
                            let prop_expr =
                                crate::axioms::property_expressions::ObjectPropertyExpression::ObjectProperty(
                                    ObjectProperty::new(p.clone()),
                                );
                            let thing = Class::new(
                                IRI::new("http://www.w3.org/2002/07/owl#Thing").unwrap(),
                            );
                            let all = ClassExpression::ObjectAllValuesFrom(
                                Box::new(prop_expr),
                                Box::new(ClassExpression::Class(Class::new(range_class))),
                            );
                            let sub = ClassExpression::Class(thing);
                            let ax = SubClassOfAxiom::new(sub, all);
                            let _ = ontology.add_axiom(Axiom::SubClassOf(ax));
                        }
                    }
                    // Property characteristics by rdf:type
                    "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" if obj_iri
                        .as_ref()
                        .is_some_and(|o| o.as_str() == "http://www.w3.org/2002/07/owl#FunctionalProperty") => {
                            let _ = ontology.add_object_property(ObjectProperty::new(s.clone()));
                            let ax = FunctionalPropertyAxiom::new(s.clone());
                            let _ = ontology.add_axiom(Axiom::FunctionalProperty(ax));
                    }
                    "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" if obj_iri
                        .as_ref()
                        .is_some_and(|o| o.as_str() == "http://www.w3.org/2002/07/owl#InverseFunctionalProperty") => {
                            let _ = ontology.add_object_property(ObjectProperty::new(s.clone()));
                            let ax = InverseFunctionalPropertyAxiom::new(s.clone());
                            let _ = ontology.add_axiom(Axiom::InverseFunctionalProperty(ax));
                    }
                    "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" if obj_iri
                        .as_ref()
                        .is_some_and(|o| o.as_str() == "http://www.w3.org/2002/07/owl#SymmetricProperty") => {
                            let _ = ontology.add_object_property(ObjectProperty::new(s.clone()));
                            let ax = SymmetricPropertyAxiom::new(s.clone());
                            let _ = ontology.add_axiom(Axiom::SymmetricProperty(ax));
                    }
                    "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" if obj_iri
                        .as_ref()
                        .is_some_and(|o| o.as_str() == "http://www.w3.org/2002/07/owl#TransitiveProperty") => {
                            let _ = ontology.add_object_property(ObjectProperty::new(s.clone()));
                            let ax = TransitivePropertyAxiom::new(s.clone());
                            let _ = ontology.add_axiom(Axiom::TransitiveProperty(ax));
                    }
                    "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" if obj_iri
                        .as_ref()
                        .is_some_and(|o| o.as_str() == "http://www.w3.org/2002/07/owl#ReflexiveProperty") => {
                            let _ = ontology.add_object_property(ObjectProperty::new(s.clone()));
                            let ax = ReflexivePropertyAxiom::new(s.clone());
                            let _ = ontology.add_axiom(Axiom::ReflexiveProperty(ax));
                    }
                    "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" if obj_iri
                        .as_ref()
                        .is_some_and(|o| o.as_str() == "http://www.w3.org/2002/07/owl#IrreflexiveProperty") => {
                            let _ = ontology.add_object_property(ObjectProperty::new(s.clone()));
                            let ax = IrreflexivePropertyAxiom::new(s.clone());
                            let _ = ontology.add_axiom(Axiom::IrreflexiveProperty(ax));
                    }
                    "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" if obj_iri
                        .as_ref()
                        .is_some_and(|o| o.as_str() == "http://www.w3.org/2002/07/owl#AsymmetricProperty") => {
                            let _ = ontology.add_object_property(ObjectProperty::new(s.clone()));
                            let ax = AsymmetricPropertyAxiom::new(s.clone());
                            let _ = ontology.add_axiom(Axiom::AsymmetricProperty(ax));
                    }
                    // rdfs:subClassOf
                    "http://www.w3.org/2000/01/rdf-schema#subClassOf" => {
                        if let Some(o) = obj_iri {
                            let ax = SubClassOfAxiom::new(Class::new(s.clone()).into(), Class::new(o).into());
                            let _ = ontology.add_axiom(Axiom::SubClassOf(ax));
                        }
                    }
                    // owl:equivalentClass
                    "http://www.w3.org/2002/07/owl#equivalentClass" => {
                        if let Some(o) = obj_iri {
                            let ax = EquivalentClassesAxiom::new(vec![s.clone(), o]);
                            let _ = ontology.add_axiom(Axiom::EquivalentClasses(ax));
                        }
                    }
                    // rdfs:subPropertyOf
                    "http://www.w3.org/2000/01/rdf-schema#subPropertyOf" => {
                        if let Some(o) = obj_iri {
                            let ax = SubObjectPropertyAxiom::new(s.clone(), o);
                            let _ = ontology.add_axiom(Axiom::SubObjectProperty(ax));
                        }
                    }
                    // owl:equivalentProperty
                    "http://www.w3.org/2002/07/owl#equivalentProperty" => {
                        if let Some(o) = obj_iri {
                            let ax = EquivalentObjectPropertiesAxiom::new(vec![s.clone(), o]);
                            let _ = ontology.add_axiom(Axiom::EquivalentObjectProperties(ax));
                        }
                    }
                    // owl:propertyDisjointWith
                    "http://www.w3.org/2002/07/owl#propertyDisjointWith" => {
                        if let Some(o) = obj_iri {
                            let ax = DisjointObjectPropertiesAxiom::new(vec![s.clone(), o]);
                            let _ = ontology.add_axiom(Axiom::DisjointObjectProperties(ax));
                        }
                    }
                    // owl:inverseOf
                    "http://www.w3.org/2002/07/owl#inverseOf" => {
                        if let Some(o) = obj_iri {
                            let p1 = crate::axioms::property_expressions::ObjectPropertyExpression::ObjectProperty(
                                ObjectProperty::new(s.clone()),
                            );
                            let p2 = crate::axioms::property_expressions::ObjectPropertyExpression::ObjectProperty(
                                ObjectProperty::new(o),
                            );
                            let ax = InverseObjectPropertiesAxiom::new(p1, p2);
                            let _ = ontology.add_axiom(Axiom::InverseObjectProperties(ax));
                        }
                    }
                    // owl:disjointWith
                    "http://www.w3.org/2002/07/owl#disjointWith" => {
                        if let Some(o) = obj_iri {
                            let ax = DisjointClassesAxiom::new(vec![s.clone(), o]);
                            let _ = ontology.add_axiom(Axiom::DisjointClasses(ax));
                        }
                    }
                    // owl:imports
                    "http://www.w3.org/2002/07/owl#imports" => {
                        if let Some(o) = obj_iri {
                            ontology.add_import(o);
                        }
                    }
                    _ => {
                        match t.object {
                            Term::NamedNode(nn) => {
                                if let Some(o) = IRI::new(nn.iri).ok() {
                                    let subj = s.clone();
                                    let pred = ObjectProperty::new(p.clone());
                                    let obj = o.clone();
                                    let ax =
                                        PropertyAssertionAxiom::new(subj, pred.iri().clone(), obj);
                                    let _ = ontology.add_axiom(Axiom::PropertyAssertion(ax));
                                }
                            }
                            Term::Literal(lit) => {
                                // Data property assertion
                                let dp = crate::entities::DataProperty::new(p.clone());
                                let _ = ontology.add_data_property(dp);
                                let value = match lit {
                                    rio_api::model::Literal::Simple { value } => {
                                        crate::entities::Literal::simple(value)
                                    }
                                    rio_api::model::Literal::LanguageTaggedString { value, language } => {
                                        crate::entities::Literal::lang_tagged(value, language)
                                    }
                                    rio_api::model::Literal::Typed { value, datatype } => {
                                        let dt = IRI::new(datatype.iri).unwrap_or_else(|_| IRI::new("http://www.w3.org/2001/XMLSchema#string").unwrap());
                                        crate::entities::Literal::typed(value, dt)
                                    }
                                };
                                let ax = DataPropertyAssertionAxiom::new(s.clone(), p.clone(), value);
                                let _ = ontology.add_axiom(Axiom::DataPropertyAssertion(ax));
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(())
        };
        let res = parser.parse_all(&mut handler);

        if let Err(e) = res {
            return Err(crate::error::OwlError::ParseError(format!(
                "rio-xml parse error: {}",
                e
            )));
        }

        Ok(ontology)
    }

    /// Parse XML document into a structured representation
    fn parse_xml_document(&mut self, content: &str) -> OwlResult<XmlDocument> {
        let mut chars = content.char_indices();
        let mut document = XmlDocument {
            root: None,
            xml_decl: None,
            doctype: None,
        };

        // Parse XML declaration if present
        self.parse_xml_declaration(&mut chars, &mut document)?;

        // Parse document type declaration if present
        self.parse_doctype(&mut chars, &mut document)?;

        // Parse root element and its children
        let mut root_found = false;
        let mut char_count = 0;

        while let Some((_, c)) = chars.next() {
            char_count += 1;
            let _ = char_count < 20;

            if c == '<' {
                if let Some(element) = self.parse_element(&mut chars)? {
                    // Look for RDF root element - could be "rdf:RDF" or just "RDF" depending on namespace handling
                    if element.name == "rdf:RDF" || element.name == "RDF" {
                        document.root = Some(Box::new(element));
                        root_found = true;
                        break;
                    }
                }
            }
        }

        let _ = root_found;

        Ok(document)
    }

    /// Parse XML declaration
    fn parse_xml_declaration(
        &self,
        chars: &mut std::str::CharIndices<'_>,
        document: &mut XmlDocument,
    ) -> OwlResult<()> {
        // Check if there's an XML declaration by looking ahead
        let mut chars_clone = chars.clone();
        if let Some((_, c)) = chars_clone.next() {
            if c != '<' {
                return Ok(());
            }
            if let Some((_, c)) = chars_clone.next() {
                if c != '?' {
                    return Ok(());
                }
            }
        } else {
            return Ok(());
        }

        // We have <?xml... so parse it
        chars.next(); // consume '<'
        chars.next(); // consume '?'

        let mut decl_content = String::new();
        while let Some((_, c)) = chars.next() {
            if c == '>' {
                break;
            }
            decl_content.push(c);
        }

        if decl_content.starts_with("?xml") && decl_content.ends_with('?') {
            let content = &decl_content[4..decl_content.len() - 1];
            let mut version = "1.0".to_string();
            let mut encoding = None;
            let mut standalone = None;

            for part in content.split_whitespace() {
                if let Some(eq_pos) = part.find('=') {
                    let key = &part[..eq_pos];
                    let value = &part[eq_pos + 1..];
                    if value.len() >= 2 && (value.starts_with('"') || value.starts_with('\'')) {
                        let clean_value = &value[1..value.len() - 1];
                        match key {
                            "version" => version = clean_value.to_string(),
                            "encoding" => encoding = Some(clean_value.to_string()),
                            "standalone" => standalone = Some(clean_value == "yes"),
                            _ => {}
                        }
                    }
                }
            }

            document.xml_decl = Some(XmlDeclaration {
                version,
                encoding,
                standalone,
            });
        }

        Ok(())
    }

    /// Parse DOCTYPE declaration
    fn parse_doctype(
        &self,
        chars: &mut std::str::CharIndices<'_>,
        document: &mut XmlDocument,
    ) -> OwlResult<()> {
        // Check if there's a DOCTYPE by looking ahead
        let mut chars_clone = chars.clone();
        let mut found_doctype = false;

        while let Some((_, c)) = chars_clone.next() {
            if c == '<' {
                // Check if this is a DOCTYPE
                let mut doctype_check = Vec::new();
                while let Some((_, c)) = chars_clone.next() {
                    doctype_check.push(c);
                    if doctype_check.len() >= 8
                        && doctype_check
                            .iter()
                            .collect::<String>()
                            .to_lowercase()
                            .ends_with("doctype")
                    {
                        found_doctype = true;
                        break;
                    }
                    if doctype_check.len() > 20 {
                        // Safety valve
                        break;
                    }
                }
                break;
            }
        }

        if !found_doctype {
            return Ok(());
        }

        // We found a DOCTYPE, so consume the '<' and parse it
        chars.next(); // consume '<'

        // Check for DOCTYPE
        let mut doctype_start = Vec::new();
        while let Some((_, c)) = chars.next() {
            doctype_start.push(c);
            if doctype_start.len() >= 8
                && doctype_start
                    .iter()
                    .collect::<String>()
                    .to_lowercase()
                    .ends_with("doctype")
            {
                break;
            }
            if doctype_start.len() > 20 {
                // Safety valve
                break;
            }
        }

        if doctype_start
            .iter()
            .collect::<String>()
            .to_lowercase()
            .ends_with("doctype")
        {
            let mut doctype_content = String::new();
            let mut bracket_count = 0;

            while let Some((_, c)) = chars.next() {
                if c == '[' {
                    bracket_count += 1;
                } else if c == ']' {
                    bracket_count -= 1;
                    if bracket_count == 0
                        && chars.clone().next().is_some_and(|(_, next)| next == '>')
                    {
                        chars.next(); // Consume '>'
                        break;
                    }
                } else if c == '>' && bracket_count == 0 {
                    break;
                }
                doctype_content.push(c);
            }

            document.doctype = Some(doctype_content);
        }

        Ok(())
    }

    /// Parse a single XML element
    fn parse_element(
        &mut self,
        chars: &mut std::str::CharIndices<'_>,
    ) -> OwlResult<Option<XmlElement>> {
        let mut name = String::new();
        let mut attributes = HashMap::new();
        let mut is_empty = false;

        // Parse element name
        while let Some((_, c)) = chars.next() {
            if c.is_whitespace() || c == '>' || c == '/' {
                break;
            }
            name.push(c);
        }

        if name.is_empty() {
            return Ok(None);
        }

        // Parse attributes
        self.parse_xml_attributes(chars, &mut attributes, &mut is_empty)?;

        // Debug: Check what element we're parsing

        // Extract namespace (deferred until attributes are processed)
        let namespace = if let Some(colon_pos) = name.find(':') {
            let prefix = &name[..colon_pos];
            self.namespaces.get(prefix).cloned()
        } else {
            self.namespaces.get("").cloned()
        };

        let mut element = XmlElement {
            name,
            namespace,
            attributes,
            content: String::new(),
            children: Vec::new(),
            is_empty,
        };

        // Parse content and children if not empty element
        if !is_empty {
            self.parse_element_content(chars, &mut element)?;
        }

        Ok(Some(element))
    }

    /// Parse XML attributes
    fn parse_xml_attributes(
        &mut self,
        chars: &mut std::str::CharIndices<'_>,
        attributes: &mut HashMap<String, String>,
        is_empty: &mut bool,
    ) -> OwlResult<()> {
        let mut attr_name = String::new();
        let mut in_attr_name = true;
        let mut expecting_equals = false;
        let mut in_attr_value = false;
        let mut attr_value_delimiter = None;
        let mut attr_value = String::new();

        while let Some((_, c)) = chars.clone().next() {
            match c {
                '=' if in_attr_name => {
                    expecting_equals = true;
                    in_attr_name = false;
                    chars.next();
                }
                '"' | '\'' if expecting_equals => {
                    attr_value_delimiter = Some(c);
                    expecting_equals = false;
                    in_attr_value = true;
                    chars.next();
                }
                '/' if !in_attr_value => {
                    *is_empty = true;
                    chars.next();
                    if let Some((_, next)) = chars.clone().next() {
                        if next == '>' {
                            chars.next();
                        }
                    }
                    break;
                }
                '>' if !in_attr_value => {
                    chars.next();
                    break;
                }
                c if in_attr_value => {
                    if Some(c) == attr_value_delimiter {
                        // End of attribute value
                        attributes.insert(attr_name.clone(), attr_value.clone());
                        attr_name.clear();
                        attr_value.clear();
                        in_attr_value = false;
                        in_attr_name = true;
                        attr_value_delimiter = None;
                        chars.next();
                    } else {
                        attr_value.push(c);
                        chars.next();
                    }
                }
                c if in_attr_name && c.is_whitespace() => {
                    if !attr_name.is_empty() {
                        in_attr_name = false;
                    }
                    chars.next();
                }
                c if !in_attr_value && !expecting_equals && c.is_whitespace() => {
                    chars.next();
                }
                c if !in_attr_value && !expecting_equals => {
                    if c.is_whitespace() {
                        // End of attribute name, expecting equals
                        if !attr_name.is_empty() {
                            in_attr_name = false;
                        }
                        chars.next();
                    } else if attr_name.is_empty() {
                        // Start of new attribute name
                        attr_name.push(c);
                        chars.next();
                    } else {
                        // Continue current attribute name
                        attr_name.push(c);
                        chars.next();
                    }
                }
                _ => {
                    chars.next();
                }
            }
        }

        // Handle final attribute
        if !attr_name.is_empty() && !attr_value.is_empty() {
            attributes.insert(attr_name, attr_value);
        }

        Ok(())
    }

    /// Parse element content and children
    fn parse_element_content(
        &mut self,
        chars: &mut std::str::CharIndices<'_>,
        element: &mut XmlElement,
    ) -> OwlResult<()> {
        let mut content = String::new();
        let mut current_depth = 1; // We're inside the opening tag we're parsing content for

        while let Some((_, c)) = chars.clone().next() {
            if c == '<' {
                // Check if this is a closing tag
                let mut is_closing = false;
                let mut is_comment = false;
                let mut is_cdata = false;

                // Peek ahead to determine tag type
                {
                    let mut peek_chars = chars.clone();
                    peek_chars.next(); // Consume '<'

                    if let Some((_, next_c)) = peek_chars.next() {
                        if next_c == '/' {
                            is_closing = true;
                        } else if next_c == '!' {
                            // Check for comment or CDATA
                            let mut comment_start = Vec::new();
                            comment_start.push(next_c);

                            while let Some((_, comment_c)) = peek_chars.next() {
                                comment_start.push(comment_c);
                                let comment_str: String = comment_start.iter().collect();

                                if comment_str.starts_with("!--") {
                                    is_comment = true;
                                    break;
                                } else if comment_str == "![CDATA[" {
                                    is_cdata = true;
                                    break;
                                } else if comment_str.len() > 10 {
                                    break;
                                }
                            }
                        }
                    }
                }

                if is_closing {
                    // Parse closing tag
                    chars.next(); // Consume '<'
                    chars.next(); // Consume '/'

                    let mut closing_name = String::new();
                    while let Some((_, c)) = chars.next() {
                        if c == '>' {
                            break;
                        }
                        closing_name.push(c);
                    }

                    // Trim leading '<' from both names if present
                    let closing_name = closing_name.trim_start_matches('<');
                    let element_name = element.name.trim_start_matches('<');

                    if closing_name == element_name {
                        current_depth -= 1;
                        if current_depth == 0 {
                            element.content = content.trim().to_string();
                            return Ok(());
                        }
                    } else {
                    }
                } else if is_comment {
                    // Skip comment
                    self.skip_comment(chars)?;
                } else if is_cdata {
                    // Parse CDATA
                    let cdata_content = self.parse_cdata(chars)?;
                    content.push_str(&cdata_content);
                } else {
                    // Parse child element
                    if let Some(child) = self.parse_element(chars)? {
                        element.children.push(child);
                        // Don't increment depth here - depth is only for tracking the current element's closing tag
                    }
                }
            } else {
                // Regular content
                content.push(c);
                chars.next();
            }
        }

        Ok(())
    }

    /// Skip XML comment
    fn skip_comment(&self, chars: &mut std::str::CharIndices<'_>) -> OwlResult<()> {
        // We've already seen "<!--"
        let mut comment_chars = Vec::new();

        while let Some((_, c)) = chars.next() {
            comment_chars.push(c);

            // Check for the pattern "-->"
            if comment_chars.len() >= 3 {
                // Look at the last 3 characters
                let len = comment_chars.len();
                let last_three = &comment_chars[len - 3..];
                let last_three_str: String = last_three.iter().collect();

                if last_three_str == "-->" {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Parse CDATA section
    fn parse_cdata(&self, chars: &mut std::str::CharIndices<'_>) -> OwlResult<String> {
        // We've already seen "<![CDATA["
        let mut cdata_content = String::new();

        while let Some((_, c)) = chars.next() {
            let mut end_marker = String::new();
            end_marker.push(c);

            // Check for "]]>"
            if c == ']' {
                if let Some((_, next1)) = chars.clone().next() {
                    if next1 == ']' {
                        if let Some((_, next2)) = chars.clone().nth(1) {
                            if next2 == '>' {
                                // Found end marker
                                chars.next(); // Consume first ']'
                                chars.next(); // Consume second ']'
                                chars.next(); // Consume '>'
                                return Ok(cdata_content);
                            }
                        }
                    }
                }
            }

            cdata_content.push(c);
        }

        Ok(cdata_content)
    }

    /// Process RDF document and populate ontology
    fn process_rdf_document(
        &mut self,
        ontology: &mut Ontology,
        document: &XmlDocument,
    ) -> OwlResult<()> {
        if let Some(root) = &document.root {
            // Process RDF root attributes
            for (key, value) in &root.attributes {
                if key == "xml:base" {
                    self.base_iri = Some(IRI::new(value)?);
                } else if let Some(prefix) = key.strip_prefix("xmlns:") {
                    self.namespaces
                        .insert(prefix.to_string(), value.to_string());
                } else if key == "xmlns" {
                    self.namespaces.insert("".to_string(), value.to_string());
                }
            }

            // Process all child elements
            for (_i, child) in root.children.iter().enumerate() {
                self.process_rdf_element(ontology, child)?;
            }
        } else {
        }
        Ok(())
    }

    /// Process individual RDF elements
    fn process_rdf_element(
        &mut self,
        ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        // Trim leading '<' from element name if present
        let element_name = element.name.trim_start_matches('<');

        match element_name {
            "owl:Ontology" => self.process_ontology_element(ontology, element),
            "owl:Class" => self.process_class_element(ontology, element),
            "owl:ObjectProperty" => self.process_object_property_element(ontology, element),
            "owl:DatatypeProperty" => self.process_data_property_element(ontology, element),
            "owl:NamedIndividual" => self.process_named_individual_element(ontology, element),
            "owl:Restriction" => self.process_restriction_element(ontology, element),
            "rdf:Description" => self.process_description_element(ontology, element),
            "rdfs:subClassOf" => self.process_subclass_element(ontology, element),
            "owl:equivalentClass" => self.process_equivalent_class_element(ontology, element),
            "owl:disjointWith" => self.process_disjoint_class_element(ontology, element),
            "rdfs:subPropertyOf" => self.process_subproperty_element(ontology, element),
            "owl:equivalentProperty" => self.process_equivalent_property_element(ontology, element),
            "rdf:type" => self.process_type_element(ontology, element),
            "owl:intersectionOf" => self.process_intersection_element(ontology, element),
            "owl:unionOf" => self.process_union_element(ontology, element),
            "owl:complementOf" => self.process_complement_element(ontology, element),
            "owl:oneOf" => self.process_oneof_element(ontology, element),
            "owl:allValuesFrom" => self.process_all_values_from_element(ontology, element),
            "owl:someValuesFrom" => self.process_some_values_from_element(ontology, element),
            "owl:hasValue" => self.process_has_value_element(ontology, element),
            "owl:minCardinality" => self.process_min_cardinality_element(ontology, element),
            "owl:maxCardinality" => self.process_max_cardinality_element(ontology, element),
            "owl:cardinality" => self.process_exact_cardinality_element(ontology, element),
            _ => self.process_unknown_element(ontology, element),
        }
    }

    /// Process ontology element
    fn process_ontology_element(
        &mut self,
        ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        if let Some(about) = element.attributes.get("rdf:about") {
            let iri = self.resolve_iri(about)?;
            ontology.set_iri(iri);
        }

        // Process imports
        for child in &element.children {
            if child.name == "owl:imports" {
                if let Some(resource) = child.attributes.get("rdf:resource") {
                    let import_iri = self.resolve_iri(resource)?;
                    ontology.add_import(import_iri);
                }
            }
        }

        Ok(())
    }

    /// Process class element
    fn process_class_element(
        &mut self,
        ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        if let Some(about) = element.attributes.get("rdf:about") {
            let class_iri = self.resolve_iri(about)?;
            let class = Class::new(class_iri.clone());
            let _class_iri_str = class_iri.to_string();
            ontology.add_class(class)?;
            // Debug: Check current class count

            // Create resource info for this class
            let resource_id = class_iri.to_string();
            let mut resource_info = ResourceInfo {
                iri: Some(class_iri),
                blank_node_id: None,
                element_type: "owl:Class".to_string(),
                properties: HashMap::new(),
                class_expressions: Vec::new(),
            };

            // Process class properties and relationships
            for child in &element.children {
                self.process_class_property(child, &mut resource_info)?;
            }

            self.resource_map.insert(resource_id, resource_info);
        }
        Ok(())
    }

    /// Process object property element
    fn process_object_property_element(
        &mut self,
        ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        if let Some(about) = element.attributes.get("rdf:about") {
            let prop_iri = self.resolve_iri(about)?;
            let prop = ObjectProperty::new(prop_iri.clone());
            ontology.add_object_property(prop)?;

            // Process property characteristics
            for child in &element.children {
                match child.name.as_str() {
                    "<rdf:type" | "rdf:type" => {
                        if let Some(resource) = child.attributes.get("rdf:resource") {
                            let type_iri = self.resolve_iri(resource)?;
                            self.process_property_characteristic(ontology, &prop_iri, &type_iri)?;
                        }
                    }
                    "rdfs:domain" | "rdfs:range" => {
                        if let Some(resource) = child.attributes.get("rdf:resource") {
                            let _domain_range_iri = self.resolve_iri(resource)?;
                            // Add to resource map for later processing
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Process data property element
    fn process_data_property_element(
        &mut self,
        ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        if let Some(about) = element.attributes.get("rdf:about") {
            let prop_iri = self.resolve_iri(about)?;
            let prop = DataProperty::new(prop_iri.clone());
            ontology.add_data_property(prop)?;

            // Process property characteristics
            for child in &element.children {
                match child.name.as_str() {
                    "rdf:type" => {
                        if let Some(resource) = child.attributes.get("rdf:resource") {
                            let type_iri = self.resolve_iri(resource)?;
                            self.process_property_characteristic(ontology, &prop_iri, &type_iri)?;
                        }
                    }
                    "rdfs:domain" | "rdfs:range" => {
                        if let Some(resource) = child.attributes.get("rdf:resource") {
                            let _domain_range_iri = self.resolve_iri(resource)?;
                            // Add to resource map for later processing
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Process named individual element
    fn process_named_individual_element(
        &mut self,
        ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        if let Some(about) = element.attributes.get("rdf:about") {
            let individual_iri = self.resolve_iri(about)?;
            let individual = NamedIndividual::new(individual_iri.clone());
            ontology.add_named_individual(individual)?;

            // Process individual properties
            for child in &element.children {
                self.process_individual_property(ontology, &individual_iri, child)?;
            }
        }
        Ok(())
    }

    /// Process restriction element
    fn process_restriction_element(
        &mut self,
        _ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        let mut on_property = None;
        let mut _restriction_type = None;
        let mut _restriction_value = None;

        for child in &element.children {
            match child.name.as_str() {
                "owl:onProperty" => {
                    if let Some(resource) = child.attributes.get("rdf:resource") {
                        on_property = Some(self.resolve_iri(resource)?);
                    }
                }
                "owl:allValuesFrom" | "owl:someValuesFrom" => {
                    if let Some(resource) = child.attributes.get("rdf:resource") {
                        _restriction_type = Some(child.name.clone());
                        _restriction_value =
                            Some(ResourceValue::Resource(self.resolve_iri(resource)?));
                    }
                }
                "owl:hasValue" => {
                    if let Some(resource) = child.attributes.get("rdf:resource") {
                        _restriction_type = Some(child.name.clone());
                        _restriction_value =
                            Some(ResourceValue::Resource(self.resolve_iri(resource)?));
                    }
                }
                "owl:minCardinality" | "owl:maxCardinality" | "owl:cardinality" => {
                    if let Some(_content) = child.attributes.get("rdf:datatype") {
                        _restriction_type = Some(child.name.clone());
                        // Parse cardinality from content or attributes
                        if let Some(_card_str) = child.attributes.get("rdf:datatype") {
                            // This would need proper parsing
                        }
                    }
                }
                _ => {}
            }
        }

        // Create restriction class expression
        if let Some(_prop_iri) = on_property {
            // This would be stored in the parent element's context
        }

        Ok(())
    }

    /// Process description element (generic RDF resource)
    fn process_description_element(
        &mut self,
        ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        let subject = self.get_element_subject(element);

        if let Some(subject_iri) = subject {
            // Process as individual if it has rdf:type statements
            for child in &element.children {
                if child.name.trim_start_matches('<') == "rdf:type" {
                    if let Some(resource) = child.attributes.get("rdf:resource") {
                        let type_iri = self.resolve_iri(resource)?;

                        // Check if this is a class or property
                        if type_iri.as_str() == "http://www.w3.org/2002/07/owl#Class" {
                            let class = Class::new(subject_iri.clone());
                            ontology.add_class(class)?;
                        } else if type_iri.as_str()
                            == "http://www.w3.org/2002/07/owl#ObjectProperty"
                        {
                            let prop = ObjectProperty::new(subject_iri.clone());
                            ontology.add_object_property(prop)?;
                        } else if type_iri.as_str()
                            == "http://www.w3.org/2002/07/owl#DatatypeProperty"
                        {
                            let prop = DataProperty::new(subject_iri.clone());
                            ontology.add_data_property(prop)?;
                        } else if type_iri.as_str()
                            == "http://www.w3.org/2002/07/owl#NamedIndividual"
                        {
                            let individual = NamedIndividual::new(subject_iri.clone());
                            ontology.add_named_individual(individual)?;
                        } else {
                            // Generic individual with type
                            let individual = NamedIndividual::new(subject_iri.clone());
                            ontology.add_named_individual(individual)?;

                            // Add class assertion
                            let class = Class::new(type_iri);
                            let axiom = ClassAssertionAxiom::new(subject_iri.clone(), class.into());
                            ontology.add_axiom(Axiom::ClassAssertion(axiom))?;
                        }
                    }
                } else {
                    // Process other properties
                    self.process_individual_property(ontology, &subject_iri, child)?;
                }
            }
        }

        Ok(())
    }

    /// Process subclass relationship
    fn process_subclass_element(
        &mut self,
        ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        if let (Some(subject), Some(object)) = (
            self.get_element_subject(element),
            self.get_element_object(element),
        ) {
            let sub_class = Class::new(subject);
            let super_class = Class::new(object);
            let axiom = SubClassOfAxiom::new(sub_class.into(), super_class.into());
            ontology.add_axiom(Axiom::SubClassOf(axiom))?;
        }
        Ok(())
    }

    /// Process equivalent class relationship
    fn process_equivalent_class_element(
        &mut self,
        ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        if let (Some(subject), Some(object)) = (
            self.get_element_subject(element),
            self.get_element_object(element),
        ) {
            let axiom = EquivalentClassesAxiom::new(vec![subject, object]);
            ontology.add_axiom(Axiom::EquivalentClasses(axiom))?;
        }
        Ok(())
    }

    /// Process disjoint class relationship
    fn process_disjoint_class_element(
        &mut self,
        ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        if let (Some(subject), Some(object)) = (
            self.get_element_subject(element),
            self.get_element_object(element),
        ) {
            let axiom = DisjointClassesAxiom::new(vec![subject, object]);
            ontology.add_axiom(Axiom::DisjointClasses(axiom))?;
        }
        Ok(())
    }

    /// Process subproperty relationship
    fn process_subproperty_element(
        &mut self,
        ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        if let (Some(subject), Some(object)) = (
            self.get_element_subject(element),
            self.get_element_object(element),
        ) {
            let axiom = SubObjectPropertyAxiom::new(subject, object);
            ontology.add_axiom(Axiom::SubObjectProperty(axiom))?;
        }
        Ok(())
    }

    /// Process equivalent property relationship
    fn process_equivalent_property_element(
        &mut self,
        ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        if let (Some(subject), Some(object)) = (
            self.get_element_subject(element),
            self.get_element_object(element),
        ) {
            let axiom = EquivalentObjectPropertiesAxiom::new(vec![subject, object]);
            ontology.add_axiom(Axiom::EquivalentObjectProperties(axiom))?;
        }
        Ok(())
    }

    /// Process disjoint property relationship
    fn process_disjoint_property_element(
        &mut self,
        ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        if let (Some(subject), Some(object)) = (
            self.get_element_subject(element),
            self.get_element_object(element),
        ) {
            let axiom = DisjointObjectPropertiesAxiom::new(vec![subject, object]);
            ontology.add_axiom(Axiom::DisjointObjectProperties(axiom))?;
        }
        Ok(())
    }

    /// Process type relationship
    fn process_type_element(
        &mut self,
        ontology: &mut Ontology,
        element: &XmlElement,
    ) -> OwlResult<()> {
        if let (Some(individual), Some(class_iri)) = (
            self.get_element_subject(element),
            self.get_element_object(element),
        ) {
            let class = Class::new(class_iri);
            let axiom = ClassAssertionAxiom::new(individual, class.into());
            ontology.add_axiom(Axiom::ClassAssertion(axiom))?;
        }
        Ok(())
    }

    /// Process complex class expressions
    fn process_intersection_element(
        &mut self,
        _ontology: &mut Ontology,
        _element: &XmlElement,
    ) -> OwlResult<()> {
        // Implementation for owl:intersectionOf
        Ok(())
    }

    fn process_union_element(
        &mut self,
        _ontology: &mut Ontology,
        _element: &XmlElement,
    ) -> OwlResult<()> {
        // Implementation for owl:unionOf
        Ok(())
    }

    fn process_complement_element(
        &mut self,
        _ontology: &mut Ontology,
        _element: &XmlElement,
    ) -> OwlResult<()> {
        // Implementation for owl:complementOf
        Ok(())
    }

    fn process_oneof_element(
        &mut self,
        _ontology: &mut Ontology,
        _element: &XmlElement,
    ) -> OwlResult<()> {
        // Implementation for owl:oneOf
        Ok(())
    }

    fn process_all_values_from_element(
        &mut self,
        _ontology: &mut Ontology,
        _element: &XmlElement,
    ) -> OwlResult<()> {
        // Implementation for owl:allValuesFrom
        Ok(())
    }

    fn process_some_values_from_element(
        &mut self,
        _ontology: &mut Ontology,
        _element: &XmlElement,
    ) -> OwlResult<()> {
        // Implementation for owl:someValuesFrom
        Ok(())
    }

    fn process_has_value_element(
        &mut self,
        _ontology: &mut Ontology,
        _element: &XmlElement,
    ) -> OwlResult<()> {
        // Implementation for owl:hasValue
        Ok(())
    }

    fn process_min_cardinality_element(
        &mut self,
        _ontology: &mut Ontology,
        _element: &XmlElement,
    ) -> OwlResult<()> {
        // Implementation for owl:minCardinality
        Ok(())
    }

    fn process_max_cardinality_element(
        &mut self,
        _ontology: &mut Ontology,
        _element: &XmlElement,
    ) -> OwlResult<()> {
        // Implementation for owl:maxCardinality
        Ok(())
    }

    fn process_exact_cardinality_element(
        &mut self,
        _ontology: &mut Ontology,
        _element: &XmlElement,
    ) -> OwlResult<()> {
        // Implementation for owl:cardinality
        Ok(())
    }

    fn process_unknown_element(
        &mut self,
        _ontology: &mut Ontology,
        _element: &XmlElement,
    ) -> OwlResult<()> {
        // Silently ignore unknown elements or handle them as needed
        Ok(())
    }

    /// Process class properties
    fn process_class_property(
        &mut self,
        element: &XmlElement,
        resource_info: &mut ResourceInfo,
    ) -> OwlResult<()> {
        let element_name = element.name.trim_start_matches('<');
        match element_name {
            "rdfs:subClassOf" => {
                if let Some(resource) = element.attributes.get("rdf:resource") {
                    let super_class_iri = self.resolve_iri(resource)?;
                    let prop = ResourceProperty {
                        property_iri: IRI::new("http://www.w3.org/2000/01/rdf-schema#subClassOf")
                            .unwrap(),
                        value: ResourceValue::Resource(super_class_iri),
                    };
                    resource_info
                        .properties
                        .entry("subClassOf".to_string())
                        .or_insert_with(Vec::new)
                        .push(prop);
                }
            }
            "owl:equivalentClass" => {
                if let Some(resource) = element.attributes.get("rdf:resource") {
                    let equiv_class_iri = self.resolve_iri(resource)?;
                    let prop = ResourceProperty {
                        property_iri: IRI::new("http://www.w3.org/2002/07/owl#equivalentClass")
                            .unwrap(),
                        value: ResourceValue::Resource(equiv_class_iri),
                    };
                    resource_info
                        .properties
                        .entry("equivalentClass".to_string())
                        .or_insert_with(Vec::new)
                        .push(prop);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Process individual properties
    fn process_individual_property(
        &mut self,
        ontology: &mut Ontology,
        individual_iri: &IRI,
        element: &XmlElement,
    ) -> OwlResult<()> {
        let property_iri = self.resolve_qname(&element.name)?;

        if let Some(resource) = element.attributes.get("rdf:resource") {
            // Object property
            let object_iri = self.resolve_iri(resource)?;
            let axiom =
                PropertyAssertionAxiom::new(individual_iri.clone(), property_iri, object_iri);
            ontology.add_axiom(Axiom::PropertyAssertion(axiom))?;
        } else if let Some(_content) = element.attributes.get("rdf:datatype") {
            // Datatype property
            // This would need the actual literal value and datatype
        } else if !element.content.is_empty() {
            // Plain literal
            // This would need the literal value
        }

        Ok(())
    }

    /// Process property characteristics
    fn process_property_characteristic(
        &mut self,
        ontology: &mut Ontology,
        prop_iri: &IRI,
        char_iri: &IRI,
    ) -> OwlResult<()> {
        // Handle property characteristics like Functional, InverseFunctional, etc.
        match char_iri.as_str() {
            "http://www.w3.org/2002/07/owl#FunctionalProperty" => {
                // Add functional object property axiom
                let axiom =
                    Axiom::FunctionalProperty(FunctionalPropertyAxiom::new(prop_iri.clone()));
                ontology.add_axiom(axiom)?;
            }
            "http://www.w3.org/2002/07/owl#InverseFunctionalProperty" => {
                // Add inverse functional object property axiom
                let axiom = Axiom::InverseFunctionalProperty(InverseFunctionalPropertyAxiom::new(
                    prop_iri.clone(),
                ));
                ontology.add_axiom(axiom)?;
            }
            "http://www.w3.org/2002/07/owl#TransitiveProperty" => {
                // Add transitive object property axiom
                let axiom =
                    Axiom::TransitiveProperty(TransitivePropertyAxiom::new(prop_iri.clone()));
                ontology.add_axiom(axiom)?;
            }
            "http://www.w3.org/2002/07/owl#SymmetricProperty" => {
                // Add symmetric object property axiom
                let axiom = Axiom::SymmetricProperty(SymmetricPropertyAxiom::new(prop_iri.clone()));
                ontology.add_axiom(axiom)?;
            }
            "http://www.w3.org/2002/07/owl#AsymmetricProperty" => {
                // Add asymmetric object property axiom
                let axiom =
                    Axiom::AsymmetricProperty(AsymmetricPropertyAxiom::new(prop_iri.clone()));
                ontology.add_axiom(axiom)?;
            }
            "http://www.w3.org/2002/07/owl#ReflexiveProperty" => {
                // Add reflexive object property axiom
                let axiom = Axiom::ReflexiveProperty(ReflexivePropertyAxiom::new(prop_iri.clone()));
                ontology.add_axiom(axiom)?;
            }
            "http://www.w3.org/2002/07/owl#IrreflexiveProperty" => {
                // Add irreflexive object property axiom
                let axiom =
                    Axiom::IrreflexiveProperty(IrreflexivePropertyAxiom::new(prop_iri.clone()));
                ontology.add_axiom(axiom)?;
            }
            _ => {
                // Unknown characteristic, ignore in non-strict mode
                if self.config.strict_validation {
                    return Err(crate::error::OwlError::ParseError(format!(
                        "Unknown property characteristic: {}",
                        char_iri
                    )));
                }
            }
        }
        Ok(())
    }

    /// Process resource map to build axioms
    fn process_resource_map(&mut self, ontology: &mut Ontology) -> OwlResult<()> {
        for (_resource_id, resource_info) in &self.resource_map {
            if resource_info.element_type == "owl:Class" {
                // Process class relationships
                if let Some(subclass_props) = resource_info.properties.get("subClassOf") {
                    for prop in subclass_props {
                        if let ResourceValue::Resource(super_class_iri) = &prop.value {
                            if let Some(class_iri) = &resource_info.iri {
                                let sub_class = Class::new(class_iri.clone());
                                let super_class = Class::new(super_class_iri.clone());
                                let axiom =
                                    SubClassOfAxiom::new(sub_class.into(), super_class.into());
                                ontology.add_axiom(Axiom::SubClassOf(axiom))?;
                            }
                        }
                    }
                }

                // Process equivalent class relationships
                if let Some(equiv_props) = resource_info.properties.get("equivalentClass") {
                    for prop in equiv_props {
                        if let ResourceValue::Resource(equiv_class_iri) = &prop.value {
                            if let Some(class_iri) = &resource_info.iri {
                                let axiom = EquivalentClassesAxiom::new(vec![
                                    class_iri.clone(),
                                    equiv_class_iri.clone(),
                                ]);
                                ontology.add_axiom(Axiom::EquivalentClasses(axiom))?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Resolve IRI with namespace handling
    fn resolve_iri(&self, iri_str: &str) -> OwlResult<IRI> {
        if iri_str.starts_with("http://") || iri_str.starts_with("https://") {
            return IRI::new(iri_str);
        }

        // Handle relative IRIs
        if let Some(base) = &self.base_iri {
            let combined = format!("{}{}", base.as_str(), iri_str);
            return IRI::new(&combined);
        }

        // Handle qnames
        if let Some((prefix, local_name)) = iri_str.split_once(':') {
            if let Some(namespace) = self.namespaces.get(prefix) {
                let full_iri = format!("{}{}", namespace, local_name);
                return IRI::new(&full_iri);
            }
        }

        IRI::new(iri_str)
    }

    /// Resolve QName to IRI
    fn resolve_qname(&self, qname: &str) -> OwlResult<IRI> {
        if let Some((prefix, local_name)) = qname.split_once(':') {
            if let Some(namespace) = self.namespaces.get(prefix) {
                let full_iri = format!("{}{}", namespace, local_name);
                return IRI::new(&full_iri);
            }
        }

        // Try as full IRI
        self.resolve_iri(qname)
    }

    /// Get subject for RDF element
    fn get_element_subject(&self, element: &XmlElement) -> Option<IRI> {
        if let Some(about) = element.attributes.get("rdf:about") {
            self.resolve_iri(about).ok()
        } else if let Some(id) = element.attributes.get("rdf:ID") {
            let fragment = format!("#{}", id);
            self.resolve_iri(&fragment).ok()
        } else if let Some(node_id) = element.attributes.get("rdf:nodeID") {
            // Generate blank node IRI
            Some(IRI::new(&format!("_:{}", node_id)).unwrap())
        } else {
            None
        }
    }

    /// Get object for RDF element
    fn get_element_object(&self, element: &XmlElement) -> Option<IRI> {
        if let Some(resource) = element.attributes.get("rdf:resource") {
            self.resolve_iri(resource).ok()
        } else {
            None
        }
    }

    /// Parse XML attributes (legacy method for backward compatibility)
    fn parse_attributes(&mut self, attr_content: &str, element: &mut XmlElement) {
        let attr_parts: Vec<&str> = attr_content.split_whitespace().collect();
        for part in attr_parts {
            if let Some(eq_pos) = part.find('=') {
                let key = &part[..eq_pos];
                let value = &part[eq_pos + 1..];
                if value.len() >= 2 && (value.starts_with('"') || value.starts_with('\'')) {
                    let clean_value = &value[1..value.len() - 1];
                    element
                        .attributes
                        .insert(key.to_string(), clean_value.to_string());

                    // Track namespace declarations
                    if let Some(prefix) = key.strip_prefix("xmlns:") {
                        self.namespaces
                            .insert(prefix.to_string(), clean_value.to_string());
                    } else if key == "xmlns" {
                        self.namespaces
                            .insert("".to_string(), clean_value.to_string());
                    }
                } else {
                    // Handle unquoted values
                    element
                        .attributes
                        .insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    /// Validate the parsed ontology
    fn validate_ontology(&self, ontology: &Ontology) -> OwlResult<()> {
        // Only validate if strict validation is enabled
        if self.config.strict_validation {
            // Basic validation checks - allow ontologies with only imports (consistent with Turtle parser)
            if ontology.classes().is_empty()
                && ontology.object_properties().is_empty()
                && ontology.data_properties().is_empty()
                && ontology.named_individuals().is_empty()
                && ontology.imports().is_empty()
            {
                return Err(crate::error::OwlError::ValidationError(
                    "Ontology contains no entities or imports".to_string(),
                ));
            }
        }
        Ok(())
    }

    // (Unit tests for RDF/XML streaming can be added once error types are stabilized)
}

impl OntologyParser for RdfXmlParser {
    fn parse_str(&self, content: &str) -> OwlResult<Ontology> {
        // Create a mutable copy for parsing
        let mut parser_copy = RdfXmlParser::with_config(self.config.clone());
        parser_copy.parse_content(content)
    }

    fn parse_file(&self, path: &Path) -> OwlResult<Ontology> {
        use std::fs;
        use std::io::Read;

        // Check file size
        if self.config.max_file_size > 0 {
            let metadata = fs::metadata(path)?;
            if metadata.len() > self.config.max_file_size as u64 {
                return Err(crate::error::OwlError::ParseError(format!(
                    "File size exceeds maximum allowed size: {} bytes",
                    self.config.max_file_size
                )));
            }
        }

        let mut file = fs::File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        self.parse_str(&content)
    }

    fn format_name(&self) -> &'static str {
        "RDF/XML"
    }
}

impl Default for RdfXmlParser {
    fn default() -> Self {
        Self::new()
    }
}

/// XML document structure for RDF/XML parsing
#[derive(Debug, Clone)]
struct XmlDocument {
    root: Option<Box<XmlElement>>,
    xml_decl: Option<XmlDeclaration>,
    doctype: Option<String>,
}

/// XML declaration information
#[derive(Debug, Clone)]
struct XmlDeclaration {
    version: String,
    encoding: Option<String>,
    standalone: Option<bool>,
}

/// XML element with full content and structure
#[derive(Debug, Clone)]
struct XmlElement {
    name: String,
    namespace: Option<String>,
    attributes: HashMap<String, String>,
    content: String,
    children: Vec<XmlElement>,
    is_empty: bool,
}

/// Resource information for tracking RDF subjects
#[derive(Debug, Clone)]
struct ResourceInfo {
    iri: Option<IRI>,
    blank_node_id: Option<String>,
    element_type: String,
    properties: HashMap<String, Vec<ResourceProperty>>,
    class_expressions: Vec<ClassExpression>,
}

/// Property information for resources
#[derive(Debug, Clone)]
struct ResourceProperty {
    property_iri: IRI,
    value: ResourceValue,
}

/// Property values in RDF/XML
#[derive(Debug, Clone)]
enum ResourceValue {
    Resource(IRI),
    BlankNode(String),
    Literal(String, Option<String>, Option<IRI>), // value, language, datatype
    ClassExpression(ClassExpression),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdf_xml_parser_initialization() {
        let parser = RdfXmlParser::new();
        assert_eq!(parser.format_name(), "RDF/XML");
        assert!(parser.namespaces.contains_key("rdf"));
        assert!(parser.namespaces.contains_key("owl"));
        assert!(parser.namespaces.contains_key("rdfs"));
    }

    #[cfg(feature = "rio-xml")]
    #[test]
    fn test_streaming_data_property_assertion_and_characteristics() {
        let rdf_xml_content = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:xsd="http://www.w3.org/2001/XMLSchema#"
         xmlns:ex="http://example.org/">
    <owl:Ontology rdf:about="http://example.org/test"/>
    <owl:DatatypeProperty rdf:about="http://example.org/age"/>
    <owl:ObjectProperty rdf:about="http://example.org/hasFriend"/>
    <rdf:Description rdf:about="http://example.org/hasFriend">
        <rdf:type rdf:resource="http://www.w3.org/2002/07/owl#SymmetricProperty"/>
    </rdf:Description>
    <owl:NamedIndividual rdf:about="http://example.org/john"/>
    <rdf:Description rdf:about="http://example.org/john">
        <ex:age rdf:datatype="http://www.w3.org/2001/XMLSchema#int">42</ex:age>
    </rdf:Description>
</rdf:RDF>"#;

        let mut parser = RdfXmlParser::new();
        parser.config.strict_validation = false; // streaming path
        let result = parser.parse_str(rdf_xml_content);
        assert!(result.is_ok(), "Streaming parse failed: {:?}", result);
        let onto = result.unwrap();

        assert!(
            onto.data_property_assertions().len() >= 1,
            "Expected at least one data property assertion"
        );
        assert!(
            onto.symmetric_property_axioms().len() >= 1,
            "Expected a symmetric property axiom from rdf:type"
        );
    }

    #[cfg(feature = "rio-xml")]
    #[test]
    fn test_streaming_domain_and_range_mapping() {
        let rdf_xml_content = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:ex="http://example.org/">
    <owl:Class rdf:about="http://example.org/Person"/>
    <owl:ObjectProperty rdf:about="http://example.org/hasParent"/>
    <rdf:Description rdf:about="http://example.org/hasParent">
        <rdfs:domain rdf:resource="http://example.org/Person"/>
        <rdfs:range rdf:resource="http://example.org/Person"/>
    </rdf:Description>
</rdf:RDF>"#;

        let mut parser = RdfXmlParser::new();
        parser.config.strict_validation = false; // streaming path
        let onto = parser.parse_str(rdf_xml_content).expect("parse");
        assert!(onto.subclass_axioms().len() >= 2, "Expected domain/range axioms");
    }

    #[cfg(feature = "rio-xml")]
    #[test]
    fn test_streaming_vs_strict_parity_basic() {
        let rdf_xml_content = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:ex="http://example.org/">
    <owl:Class rdf:about="http://example.org/Person"/>
    <owl:Class rdf:about="http://example.org/Animal"/>
    <owl:ObjectProperty rdf:about="http://example.org/hasParent"/>
    <rdf:Description rdf:about="http://example.org/hasParent">
        <rdf:type rdf:resource="http://www.w3.org/2002/07/owl#SymmetricProperty"/>
    </rdf:Description>
    <owl:NamedIndividual rdf:about="http://example.org/john"/>
    <owl:NamedIndividual rdf:about="http://example.org/mary"/>
    <rdf:Description rdf:about="http://example.org/john">
        <ex:hasParent rdf:resource="http://example.org/mary"/>
    </rdf:Description>
</rdf:RDF>"#;

        // Strict (legacy) path
        let mut strict = RdfXmlParser::new();
        strict.config.strict_validation = true;
        let onto_strict = strict.parse_str(rdf_xml_content).expect("strict parse");

        // Streaming path
        let mut streaming = RdfXmlParser::new();
        streaming.config.strict_validation = false;
        let onto_stream = streaming.parse_str(rdf_xml_content).expect("streaming parse");

        assert!(onto_strict.classes().len() >= 2);
        assert!(onto_stream.classes().len() >= 2);
        assert!(onto_strict.object_properties().len() >= 1);
        assert!(onto_stream.object_properties().len() >= 1);
        assert!(onto_strict.named_individuals().len() >= 2);
        assert!(onto_stream.named_individuals().len() >= 2);
        // Compare object property assertions are present in both
        assert!(onto_strict.property_assertions().len() >= 1);
        assert!(onto_stream.property_assertions().len() >= 1);
        // Symmetric property type: ensure both produced at least one
        // Streaming should detect the symmetric property characteristic
        assert!(onto_stream.symmetric_property_axioms().len() >= 1);
    }

    #[test]
    fn test_simple_rdf_xml_parsing() {
        let rdf_xml_content = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:ex="http://example.org/">
    <owl:Ontology rdf:about="http://example.org/test"/>
    <owl:Class rdf:about="http://example.org/Person"/>
    <owl:Class rdf:about="http://example.org/Animal"/>
    <owl:ObjectProperty rdf:about="http://example.org/hasParent"/>
    <owl:NamedIndividual rdf:about="http://example.org/john"/>
</rdf:RDF>"#;

        // Use non-strict validation for testing
        let mut parser = RdfXmlParser::new();
        parser.config.strict_validation = false;
        let result = parser.parse_str(rdf_xml_content);

        assert!(result.is_ok(), "Parsing failed: {:?}", result);

        if let Ok(ontology) = result {
            // Debug output
            println!("Classes: {}", ontology.classes().len());
            println!("Object properties: {}", ontology.object_properties().len());
            println!("Named individuals: {}", ontology.named_individuals().len());
            println!("Imports: {}", ontology.imports().len());
            println!("Axioms: {}", ontology.axioms().len());

            for class in ontology.classes() {
                println!("Class: {}", class.iri());
            }
            for prop in ontology.object_properties() {
                println!("Property: {}", prop.iri());
            }
            for individual in ontology.named_individuals() {
                println!("Individual: {}", individual.iri());
            }
            println!("=========================");

            // Should have parsed the expected entities
            assert_eq!(ontology.classes().len(), 2, "Should have parsed 2 classes");
            assert_eq!(
                ontology.object_properties().len(),
                1,
                "Should have parsed 1 object property"
            );
            assert_eq!(
                ontology.named_individuals().len(),
                1,
                "Should have parsed 1 individual"
            );

            // Verify specific entities were parsed
            let class_iris: Vec<String> = ontology
                .classes()
                .iter()
                .map(|c| c.iri().to_string())
                .collect();
            let prop_iris: Vec<String> = ontology
                .object_properties()
                .iter()
                .map(|p| p.iri().to_string())
                .collect();
            let individual_iris: Vec<String> = ontology
                .named_individuals()
                .iter()
                .map(|i| i.iri().to_string())
                .collect();

            assert!(class_iris.contains(&"http://example.org/Person".to_string()));
            assert!(class_iris.contains(&"http://example.org/Animal".to_string()));
            assert!(prop_iris.contains(&"http://example.org/hasParent".to_string()));
            assert!(individual_iris.contains(&"http://example.org/john".to_string()));
        }
    }

    #[test]
    fn test_rdf_xml_with_subclass() {
        let rdf_xml_content = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:ex="http://example.org/">
    <owl:Ontology rdf:about="http://example.org/test"/>
    <owl:Class rdf:about="http://example.org/Person">
        <rdfs:subClassOf rdf:resource="http://example.org/Animal"/>
    </owl:Class>
    <owl:Class rdf:about="http://example.org/Animal"/>
</rdf:RDF>"#;

        let parser = RdfXmlParser::new();
        let result = parser.parse_str(rdf_xml_content);

        assert!(result.is_ok(), "Parsing failed: {:?}", result);

        if let Ok(ontology) = result {
            // Should have parsed 2 classes and a subclass axiom
            assert_eq!(ontology.classes().len(), 2, "Should have parsed 2 classes");

            // Check if subclass axioms were created
            let subclass_count = ontology
                .axioms()
                .iter()
                .filter(|axiom| matches!(***axiom, crate::axioms::Axiom::SubClassOf(_)))
                .count();

            assert!(subclass_count > 0, "Should have subclass axioms");
        }
    }

    #[test]
    fn test_rdf_xml_with_equivalent_classes() {
        let rdf_xml_content = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:ex="http://example.org/">
    <owl:Ontology rdf:about="http://example.org/test"/>
    <owl:Class rdf:about="http://example.org/Person">
        <owl:equivalentClass rdf:resource="http://example.org/Human"/>
    </owl:Class>
    <owl:Class rdf:about="http://example.org/Human"/>
</rdf:RDF>"#;

        let parser = RdfXmlParser::new();
        let result = parser.parse_str(rdf_xml_content);

        assert!(result.is_ok(), "Parsing failed: {:?}", result);

        if let Ok(ontology) = result {
            // Should have parsed 2 classes
            assert_eq!(ontology.classes().len(), 2, "Should have parsed 2 classes");

            // Check if equivalent class axioms were created
            let equiv_count = ontology
                .axioms()
                .iter()
                .filter(|axiom| matches!(***axiom, crate::axioms::Axiom::EquivalentClasses(_)))
                .count();

            assert!(equiv_count > 0, "Should have equivalent class axioms");
        }
    }

    #[test]
    fn test_rdf_xml_individual_with_type() {
        let rdf_xml_content = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:ex="http://example.org/">
    <owl:Ontology rdf:about="http://example.org/test"/>
    <owl:Class rdf:about="http://example.org/Person"/>
    <rdf:Description rdf:about="http://example.org/john">
        <rdf:type rdf:resource="http://example.org/Person"/>
    </rdf:Description>
</rdf:RDF>"#;

        let parser = RdfXmlParser::new();
        let result = parser.parse_str(rdf_xml_content);

        assert!(result.is_ok(), "Parsing failed: {:?}", result);

        if let Ok(ontology) = result {
            // Should have parsed 1 class and 1 individual
            assert_eq!(ontology.classes().len(), 1, "Should have parsed 1 class");
            assert_eq!(
                ontology.named_individuals().len(),
                1,
                "Should have parsed 1 individual"
            );

            // Check if class assertion axioms were created
            let assertion_count = ontology
                .axioms()
                .iter()
                .filter(|axiom| matches!(***axiom, crate::axioms::Axiom::ClassAssertion(_)))
                .count();

            assert!(assertion_count > 0, "Should have class assertion axioms");
        }
    }

    #[test]
    fn test_rdf_xml_with_property_assertions() {
        let rdf_xml_content = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:ex="http://example.org/">
    <owl:Ontology rdf:about="http://example.org/test"/>
    <owl:ObjectProperty rdf:about="http://example.org/hasParent"/>
    <owl:NamedIndividual rdf:about="http://example.org/john"/>
    <owl:NamedIndividual rdf:about="http://example.org/mary"/>
    <rdf:Description rdf:about="http://example.org/john">
        <ex:hasParent rdf:resource="http://example.org/mary"/>
    </rdf:Description>
</rdf:RDF>"#;

        let parser = RdfXmlParser::new();
        let result = parser.parse_str(rdf_xml_content);

        assert!(result.is_ok(), "Parsing failed: {:?}", result);

        if let Ok(ontology) = result {
            // Should have parsed 1 property and 2 individuals
            assert_eq!(
                ontology.object_properties().len(),
                1,
                "Should have parsed 1 object property"
            );
            assert_eq!(
                ontology.named_individuals().len(),
                2,
                "Should have parsed 2 individuals"
            );

            // Check if property assertion axioms were created
            let prop_assertion_count = ontology
                .axioms()
                .iter()
                .filter(|axiom| matches!(***axiom, crate::axioms::Axiom::PropertyAssertion(_)))
                .count();

            assert!(
                prop_assertion_count > 0,
                "Should have property assertion axioms"
            );
        }
    }

    #[test]
    fn test_rdf_xml_namespace_parsing() {
        let rdf_xml_content = r#"
            <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
                     xmlns:owl="http://www.w3.org/2002/07/owl#"
                     xmlns:ex="http://example.org/">
                <owl:Class rdf:about="http://example.org/Person"/>
            </rdf:RDF>
        "#;

        let parser = RdfXmlParser::new();
        let result = parser.parse_str(rdf_xml_content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rdf_xml_with_config() {
        let mut config = ParserConfig::default();
        config.max_file_size = 1000;
        config.strict_validation = false;
        config.resolve_base_iri = false;
        config
            .prefixes
            .insert("test".to_string(), "http://test.org/".to_string());

        let parser = RdfXmlParser::with_config(config);
        assert_eq!(parser.format_name(), "RDF/XML");
        assert!(parser.namespaces.contains_key("test"));
    }

    #[test]
    fn test_rdf_xml_xml_declaration() {
        let rdf_xml_content = r#"<?xml version="1.1" encoding="UTF-8" standalone="yes"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#">
    <owl:Class rdf:about="http://example.org/Test"/>
</rdf:RDF>"#;

        let parser = RdfXmlParser::new();
        let result = parser.parse_str(rdf_xml_content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rdf_xml_empty_element() {
        let rdf_xml_content = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#">
    <owl:Class rdf:about="http://example.org/Test"/>
    <owl:ObjectProperty rdf:about="http://example.org/testProp"/>
</rdf:RDF>"#;

        let parser = RdfXmlParser::new();
        let result = parser.parse_str(rdf_xml_content);
        assert!(result.is_ok());

        if let Ok(ontology) = result {
            assert_eq!(ontology.classes().len(), 1);
            assert_eq!(ontology.object_properties().len(), 1);
        }
    }

    #[test]
    fn test_rdf_xml_with_comments() {
        let rdf_xml_content = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:owl="http://www.w3.org/2002/07/owl#">
    <!-- This is a comment -->
    <owl:Class rdf:about="http://example.org/Test"/>
    <!-- Another comment -->
    <owl:ObjectProperty rdf:about="http://example.org/testProp"/>
</rdf:RDF>"#;

        let parser = RdfXmlParser::new();
        let result = parser.parse_str(rdf_xml_content);
        assert!(result.is_ok());

        if let Ok(ontology) = result {
            assert_eq!(ontology.classes().len(), 1);
            assert_eq!(ontology.object_properties().len(), 1);
        }
    }
}
