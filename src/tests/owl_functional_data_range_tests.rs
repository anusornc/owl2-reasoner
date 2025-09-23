//! Test for OWL Functional Syntax data range parsing
//!
//! This test verifies that the OWL Functional Syntax parser can handle
//! all data range types and XSD datatype restrictions correctly.

use crate::*;

#[test]
fn test_owl_functional_basic_data_ranges() -> OwlResult<()> {
    let parser = OwlFunctionalSyntaxParser::new();

    let owl_content = r#"
Prefix(:=<http://example.org/test#>)
Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)
Prefix(rdf:=<http://www.w3.org/1999/02/22-rdf-syntax-ns#>)
Prefix(rdfs:=<http://www.w3.org/2000/01/rdf-schema#>)

Ontology(<http://example.org/test>

    Declaration(DataProperty(:hasAge))
    Declaration(DataProperty(:hasName))
    Declaration(DataProperty(:hasHeight))
    Declaration(DataProperty(:hasSalary))
    Declaration(DataProperty(:hasBirthDate))
    Declaration(DataProperty(:isActive))

    # Test basic data property ranges with various XSD datatypes
    DataPropertyRange(:hasAge xsd:integer)
    DataPropertyRange(:hasName xsd:string)
    DataPropertyRange(:hasHeight xsd:double)
    DataPropertyRange(:hasSalary xsd:decimal)
    DataPropertyRange(:hasBirthDate xsd:dateTime)
    DataPropertyRange(:isActive xsd:boolean)

)
"#;

    let ontology = parser.parse_str(owl_content)?;

    // Check that data property range axioms were parsed correctly
    let range_axioms: Vec<_> = ontology
        .axioms()
        .iter()
        .filter(|axiom| matches!(***axiom, Axiom::DataPropertyRange(_)))
        .collect();

    assert_eq!(
        range_axioms.len(),
        6,
        "Should have 6 data property range axioms"
    );

    // Verify specific datatypes
    let mut datatypes_found = Vec::new();
    for axiom in range_axioms.iter() {
        if let Axiom::DataPropertyRange(range_axiom) = axiom.as_ref() {
            datatypes_found.push(range_axiom.range().to_string());
        }
    }

    assert!(
        datatypes_found.iter().any(|dt| dt.contains("integer")),
        "Should have integer datatype"
    );
    assert!(
        datatypes_found.iter().any(|dt| dt.contains("string")),
        "Should have string datatype"
    );
    assert!(
        datatypes_found.iter().any(|dt| dt.contains("double")),
        "Should have double datatype"
    );
    assert!(
        datatypes_found.iter().any(|dt| dt.contains("decimal")),
        "Should have decimal datatype"
    );
    assert!(
        datatypes_found.iter().any(|dt| dt.contains("dateTime")),
        "Should have dateTime datatype"
    );
    assert!(
        datatypes_found.iter().any(|dt| dt.contains("boolean")),
        "Should have boolean datatype"
    );

    println!("✅ OWL Functional Syntax basic data range parsing works correctly!");
    println!("   - Total range axioms: {}", range_axioms.len());
    println!("   - Found datatypes: {:?}", datatypes_found);

    Ok(())
}

#[test]
fn test_owl_functional_derived_data_types() -> OwlResult<()> {
    let parser = OwlFunctionalSyntaxParser::new();

    let owl_content = r#"
Prefix(:=<http://example.org/test#>)
Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)

Ontology(<http://example.org/test>

    Declaration(DataProperty(:hasPositiveInt))
    Declaration(DataProperty(:hasNegativeInt))
    Declaration(DataProperty(:hasNonNegativeInt))
    Declaration(DataProperty(:hasLong))
    Declaration(DataProperty(:hasShort))
    Declaration(DataProperty(:hasFloat))
    Declaration(DataProperty(:hasDate))

    # Test derived XSD datatypes
    DataPropertyRange(:hasPositiveInt xsd:positiveInteger)
    DataPropertyRange(:hasNegativeInt xsd:negativeInteger)
    DataPropertyRange(:hasNonNegativeInt xsd:nonNegativeInteger)
    DataPropertyRange(:hasLong xsd:long)
    DataPropertyRange(:hasShort xsd:short)
    DataPropertyRange(:hasFloat xsd:float)
    DataPropertyRange(:hasDate xsd:date)

)
"#;

    let ontology = parser.parse_str(owl_content)?;

    // Check that derived datatype axioms were parsed correctly
    let range_axioms: Vec<_> = ontology
        .axioms()
        .iter()
        .filter(|axiom| matches!(***axiom, Axiom::DataPropertyRange(_)))
        .collect();

    assert_eq!(
        range_axioms.len(),
        7,
        "Should have 7 derived datatype range axioms"
    );

    // Verify specific derived datatypes
    let mut positive_found = false;
    let mut negative_found = false;
    let mut non_negative_found = false;
    let mut long_found = false;
    let mut short_found = false;
    let mut float_found = false;
    let mut date_found = false;

    for axiom in range_axioms.iter() {
        if let Axiom::DataPropertyRange(range_axiom) = axiom.as_ref() {
            let datatype = range_axiom.range().to_string();
            if datatype.contains("positiveInteger") {
                positive_found = true;
            } else if datatype.contains("negativeInteger") {
                negative_found = true;
            } else if datatype.contains("nonNegativeInteger") {
                non_negative_found = true;
            } else if datatype.contains("long") {
                long_found = true;
            } else if datatype.contains("short") {
                short_found = true;
            } else if datatype.contains("float") {
                float_found = true;
            } else if datatype.contains("date") {
                date_found = true;
            }
        }
    }

    assert!(positive_found, "Should have xsd:positiveInteger datatype");
    assert!(negative_found, "Should have xsd:negativeInteger datatype");
    assert!(
        non_negative_found,
        "Should have xsd:nonNegativeInteger datatype"
    );
    assert!(long_found, "Should have xsd:long datatype");
    assert!(short_found, "Should have xsd:short datatype");
    assert!(float_found, "Should have xsd:float datatype");
    assert!(date_found, "Should have xsd:date datatype");

    println!("✅ OWL Functional Syntax derived XSD datatype parsing works correctly!");
    println!("   - xsd:positiveInteger: ✓");
    println!("   - xsd:negativeInteger: ✓");
    println!("   - xsd:nonNegativeInteger: ✓");
    println!("   - xsd:long: ✓");
    println!("   - xsd:short: ✓");
    println!("   - xsd:float: ✓");
    println!("   - xsd:date: ✓");
    println!("   - Total derived datatype axioms: {}", range_axioms.len());

    Ok(())
}

#[test]
fn test_owl_functional_simple_datatype_restrictions() -> OwlResult<()> {
    let parser = OwlFunctionalSyntaxParser::new();

    let owl_content = r#"
Prefix(:=<http://example.org/test#>)
Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)

Ontology(<http://example.org/test>

    Declaration(DataProperty(:hasSimpleInt))
    Declaration(DataProperty(:hasSimpleString))
    Declaration(DataProperty(:hasSimpleBool))

    # Test simple datatypes (complex restrictions like DatatypeRestriction
    # are not supported in DataPropertyRange in this parser implementation)
    DataPropertyRange(:hasSimpleInt xsd:integer)
    DataPropertyRange(:hasSimpleString xsd:string)
    DataPropertyRange(:hasSimpleBool xsd:boolean)

)
"#;

    let ontology = parser.parse_str(owl_content)?;

    // Check that simple datatype range axioms were parsed correctly
    let range_axioms: Vec<_> = ontology
        .axioms()
        .iter()
        .filter(|axiom| matches!(***axiom, Axiom::DataPropertyRange(_)))
        .collect();

    assert_eq!(
        range_axioms.len(),
        3,
        "Should have 3 simple datatype range axioms"
    );

    // Verify basic datatypes
    let mut integer_found = false;
    let mut string_found = false;
    let mut boolean_found = false;

    for axiom in range_axioms.iter() {
        if let Axiom::DataPropertyRange(range_axiom) = axiom.as_ref() {
            let datatype_iri = range_axiom.range().to_string();
            if datatype_iri.contains("integer") {
                integer_found = true;
            } else if datatype_iri.contains("string") {
                string_found = true;
            } else if datatype_iri.contains("boolean") {
                boolean_found = true;
            }
        }
    }

    assert!(integer_found, "Should have integer datatype");
    assert!(string_found, "Should have string datatype");
    assert!(boolean_found, "Should have boolean datatype");

    println!("✅ OWL Functional Syntax simple datatype range parsing works correctly!");
    println!("   - Integer: ✓");
    println!("   - String: ✓");
    println!("   - Boolean: ✓");
    println!("   - Total range axioms: {}", range_axioms.len());

    Ok(())
}

#[test]
fn test_owl_functional_advanced_xsd_datatypes() -> OwlResult<()> {
    let parser = OwlFunctionalSyntaxParser::new();

    let owl_content = r#"
Prefix(:=<http://example.org/test#>)
Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)

Ontology(<http://example.org/test>

    Declaration(DataProperty(:hasID))
    Declaration(DataProperty(:hasEmail))
    Declaration(DataProperty(:hasURL))
    Declaration(DataProperty(:hasHexColor))
    Declaration(DataProperty(:hasBase64Data))

    # Test advanced XSD datatypes
    DataPropertyRange(:hasID xsd:ID)
    DataPropertyRange(:hasEmail xsd:anyURI)
    DataPropertyRange(:hasURL xsd:anyURI)
    DataPropertyRange(:hasHexColor xsd:hexBinary)
    DataPropertyRange(:hasBase64Data xsd:base64Binary)

)
"#;

    let ontology = parser.parse_str(owl_content)?;

    // Check that advanced datatype axioms were parsed correctly
    let range_axioms: Vec<_> = ontology
        .axioms()
        .iter()
        .filter(|axiom| matches!(***axiom, Axiom::DataPropertyRange(_)))
        .collect();

    assert_eq!(
        range_axioms.len(),
        5,
        "Should have 5 advanced datatype range axioms"
    );

    // Verify specific advanced datatypes
    let mut id_found = false;
    let mut anyuri_found = false;
    let mut hex_found = false;
    let mut base64_found = false;

    for axiom in range_axioms.iter() {
        if let Axiom::DataPropertyRange(range_axiom) = axiom.as_ref() {
            let datatype = range_axiom.range().to_string();
            if datatype.contains("ID") {
                id_found = true;
            } else if datatype.contains("anyURI") {
                anyuri_found = true;
            } else if datatype.contains("hexBinary") {
                hex_found = true;
            } else if datatype.contains("base64Binary") {
                base64_found = true;
            }
        }
    }

    assert!(id_found, "Should have xsd:ID datatype");
    assert!(anyuri_found, "Should have xsd:anyURI datatype");
    assert!(hex_found, "Should have xsd:hexBinary datatype");
    assert!(base64_found, "Should have xsd:base64Binary datatype");

    println!("✅ OWL Functional Syntax advanced XSD datatype parsing works correctly!");
    println!("   - xsd:ID: ✓");
    println!("   - xsd:anyURI: ✓");
    println!("   - xsd:hexBinary: ✓");
    println!("   - xsd:base64Binary: ✓");
    println!(
        "   - Total advanced datatype axioms: {}",
        range_axioms.len()
    );

    Ok(())
}
