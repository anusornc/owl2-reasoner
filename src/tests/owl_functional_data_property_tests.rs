//! Test for OWL Functional Syntax data property assertion parsing
//!
//! This test verifies that the OWL Functional Syntax parser can now handle
//! DataPropertyAssertion and NegativeDataPropertyAssertion axioms with
//! comprehensive literal support.

use crate::*;

#[test]
fn test_owl_functional_data_property_assertion() -> OwlResult<()> {
    let parser = OwlFunctionalSyntaxParser::new();

    let owl_content = r#"
Prefix(:=<http://example.org/test#>)
Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)

Ontology(<http://example.org/test>

    Declaration(Class(:Person))
    Declaration(DataProperty(:hasAge))
    Declaration(DataProperty(:hasName))
    Declaration(NamedIndividual(:John))
    Declaration(NamedIndividual(:Mary))

    # Test various data property assertions with different literal types
    DataPropertyAssertion(:hasAge :John "25"^^xsd:integer)
    DataPropertyAssertion(:hasName :John "John Doe")
    DataPropertyAssertion(:hasAge :Mary "30.5"^^xsd:double)
    DataPropertyAssertion(:hasName :Mary "Mary Smith"@en)
    DataPropertyAssertion(:hasAge :John "true"^^xsd:boolean)

)
"#;

    let ontology = parser.parse_str(owl_content)?;

    // Check that data property assertions were parsed correctly
    let data_assertions: Vec<_> = ontology
        .axioms()
        .iter()
        .filter(|axiom| matches!(axiom.as_ref(), Axiom::DataPropertyAssertion(_)))
        .collect();

    assert_eq!(
        data_assertions.len(),
        5,
        "Should have 5 data property assertions"
    );

    // Verify specific assertions
    for (i, axiom) in data_assertions.iter().enumerate() {
        if let Axiom::DataPropertyAssertion(assertion) = axiom.as_ref() {
            println!(
                "Assertion {}: {} {} {} = {}",
                i + 1,
                assertion.subject(),
                assertion.property(),
                assertion.value().lexical_form(),
                assertion.value().datatype().as_str()
            );
        }
    }

    // Check for specific types
    let mut integer_found = false;
    let mut string_found = false;
    let mut double_found = false;
    let mut lang_tagged_found = false;
    let mut boolean_found = false;

    for axiom in data_assertions.iter() {
        if let Axiom::DataPropertyAssertion(assertion) = axiom.as_ref() {
            let datatype = assertion.value().datatype();
            if datatype.as_str().contains("integer") {
                integer_found = true;
            } else if datatype.as_str().contains("double") {
                double_found = true;
            } else if datatype.as_str().contains("boolean") {
                boolean_found = true;
            } else if datatype.as_str().contains("string") {
                string_found = true;
            }

            if assertion.value().language_tag().is_some() {
                lang_tagged_found = true;
            }
        }
    }

    assert!(integer_found, "Should have integer literal");
    assert!(string_found, "Should have string literal");
    assert!(double_found, "Should have double literal");
    assert!(lang_tagged_found, "Should have language-tagged literal");
    assert!(boolean_found, "Should have boolean literal");

    println!("✅ OWL Functional Syntax data property assertion parsing works correctly!");
    println!("   - Total assertions: {}", data_assertions.len());
    println!("   - Integer literal: ✓");
    println!("   - String literal: ✓");
    println!("   - Double literal: ✓");
    println!("   - Language-tagged literal: ✓");
    println!("   - Boolean literal: ✓");

    Ok(())
}

#[test]
fn test_owl_functional_negative_data_property_assertion() -> OwlResult<()> {
    let parser = OwlFunctionalSyntaxParser::new();

    let owl_content = r#"
Prefix(:=<http://example.org/test#>)
Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)

Ontology(<http://example.org/test>

    Declaration(Class(:Person))
    Declaration(DataProperty(:hasAge))
    Declaration(NamedIndividual(:John))

    # Test negative data property assertions
    NegativeDataPropertyAssertion(:hasAge :John "25"^^xsd:integer)
    NegativeDataPropertyAssertion(:hasAge :John "30"^^xsd:integer)

)
"#;

    let ontology = parser.parse_str(owl_content)?;

    // Check that negative data property assertions were parsed correctly
    let neg_data_assertions: Vec<_> = ontology
        .axioms()
        .iter()
        .filter(|axiom| matches!(axiom.as_ref(), Axiom::NegativeDataPropertyAssertion(_)))
        .collect();

    assert_eq!(
        neg_data_assertions.len(),
        2,
        "Should have 2 negative data property assertions"
    );

    // Verify specific assertions
    for (i, axiom) in neg_data_assertions.iter().enumerate() {
        if let Axiom::NegativeDataPropertyAssertion(assertion) = axiom.as_ref() {
            println!(
                "Negative Assertion {}: {} {} {} = {}",
                i + 1,
                assertion.subject(),
                assertion.property(),
                assertion.value().lexical_form(),
                assertion.value().datatype().as_str()
            );
        }
    }

    println!("✅ OWL Functional Syntax negative data property assertion parsing works correctly!");
    println!(
        "   - Total negative assertions: {}",
        neg_data_assertions.len()
    );

    Ok(())
}

#[test]
fn test_owl_functional_complex_literals() -> OwlResult<()> {
    let parser = OwlFunctionalSyntaxParser::new();

    let owl_content = r#"
Prefix(:=<http://example.org/test#>)
Prefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)

Ontology(<http://example.org/test>

    Declaration(Class(:Person))
    Declaration(DataProperty(:hasComplexData))
    Declaration(NamedIndividual(:TestPerson))

    # Test complex literal types
    DataPropertyAssertion(:hasComplexData :TestPerson "42"^^xsd:integer)
    DataPropertyAssertion(:hasComplexData :TestPerson "3.14159"^^xsd:double)
    DataPropertyAssertion(:hasComplexData :TestPerson "Hello, World!")
    DataPropertyAssertion(:hasComplexData :TestPerson "Bonjour"@fr)
    DataPropertyAssertion(:hasComplexData :TestPerson "false"^^xsd:boolean)

)
"#;

    let ontology = parser.parse_str(owl_content)?;

    // Check that all assertions were parsed
    let data_assertions: Vec<_> = ontology
        .axioms()
        .iter()
        .filter(|axiom| matches!(axiom.as_ref(), Axiom::DataPropertyAssertion(_)))
        .collect();

    assert_eq!(
        data_assertions.len(),
        5,
        "Should have 5 data property assertions"
    );

    // Verify specific literal values
    let mut values = Vec::new();
    for axiom in data_assertions.iter() {
        if let Axiom::DataPropertyAssertion(assertion) = axiom.as_ref() {
            values.push(assertion.value().lexical_form().to_string());
        }
    }

    assert!(
        values.contains(&"42".to_string()),
        "Should contain integer 42"
    );
    assert!(
        values.contains(&"3.14159".to_string()),
        "Should contain double 3.14159"
    );
    assert!(
        values.contains(&"Hello, World!".to_string()),
        "Should contain string"
    );
    assert!(
        values.contains(&"false".to_string()),
        "Should contain boolean false"
    );

    println!("✅ OWL Functional Syntax complex literal parsing works correctly!");
    println!("   - Parsed values: {:?}", values);

    Ok(())
}
