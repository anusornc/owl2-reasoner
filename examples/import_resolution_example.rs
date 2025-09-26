//! Example demonstrating OWL2 Import Resolution functionality
//!
//! This example shows how to:
//! - Create ontologies with import statements
//! - Use the ImportResolver to resolve imports
//! - Configure import resolution settings
//! - Handle circular dependencies and errors

use owl2_reasoner::*;

fn main() -> OwlResult<()> {
    // Initialize logging
    env_logger::init();

    println!("OWL2 Import Resolution Example");
    println!("================================");

    // Example 1: Create an ontology with imports
    create_import_example()?;

    // Example 2: Configure import resolver
    configure_import_resolver()?;

    // Example 3: Handle circular dependencies
    handle_circular_dependencies()?;

    // Example 4: Custom import sources
    custom_import_sources()?;

    // Example 5: Cache management
    cache_management()?;
    create_turtle_with_imports()?;

    Ok(())
}

fn create_import_example() -> OwlResult<()> {
    println!("\n1. Creating Ontology with Imports");

    // Create a main ontology that imports other ontologies
    let mut main_ontology = Ontology::new();
    main_ontology.set_iri("http://example.org/main-ontology");

    // Add some entities to the main ontology
    let person_class = Class::new("http://example.org/Person");
    main_ontology.add_class(person_class.clone())?;

    let has_name_prop = DataProperty::new("http://example.org/hasName");
    main_ontology.add_data_property(has_name_prop.clone())?;

    // Add import statements
    main_ontology.add_import("http://example.org/biomedical-ontology");
    main_ontology.add_import("http://example.org/location-ontology");

    println!(
        "Main ontology has {} import statements",
        main_ontology.imports().len()
    );

    // Show the import IRIs
    for import_iri in main_ontology.imports() {
        println!("  - Import: {}", import_iri);
    }

    println!("Created main ontology with:");
    println!("  - {} classes", main_ontology.classes().len());
    println!(
        "  - {} data properties",
        main_ontology.data_properties().len()
    );
    println!("  - {} imports", main_ontology.imports().len());

    Ok(())
}

fn configure_import_resolver() -> OwlResult<()> {
    println!("\n2. Configuring Import Resolver");

    // Create custom import resolver configuration
    use parser::ImportResolverConfig;

    let config = ImportResolverConfig {
        max_depth: 5,                                    // Limit import depth
        timeout: std::time::Duration::from_secs(15),     // 15 second timeout
        max_cache_size: 50,                              // Cache up to 50 ontologies
        cache_ttl: std::time::Duration::from_secs(1800), // 30 minute TTL
        enable_concurrent_resolution: true,              // Enable concurrent resolution
        max_concurrent_resolutions: 4,                   // Max 4 concurrent resolutions
        follow_redirects: true,                          // Follow HTTP redirects
        max_redirects: 3,                                // Max 3 redirects
        user_agent: "OWL2-Reasoner-Example/1.0".to_string(),
    };

    // Create import resolver with custom configuration
    let mut resolver = parser::ImportResolver::with_config(config)?;

    println!("Created import resolver with configuration:");
    println!("  - Max depth: {}", resolver.config().max_depth);
    println!("  - Timeout: {:?}", resolver.config().timeout);
    println!("  - Cache size: {}", resolver.config().max_cache_size);
    println!(
        "  - Concurrent resolution: {}",
        resolver.config().enable_concurrent_resolution
    );

    // Test resolver with a simple ontology
    let mut test_ontology = Ontology::new();
    test_ontology.set_iri("http://example.org/test");
    test_ontology.add_import("http://example.org/external-ontology");

    println!("Attempting to resolve imports (may fail due to missing file)...");
    match resolver.resolve_imports(&mut test_ontology) {
        Ok(_) => println!("✓ Import resolution successful"),
        Err(e) => println!("⚠ Import resolution failed (expected): {}", e),
    }

    // Show statistics
    let stats = resolver.stats();
    println!("Resolution statistics:");
    println!("  - Imports resolved: {}", stats.imports_resolved);
    println!("  - Cache hits: {}", stats.cache_hits);
    println!("  - Cache misses: {}", stats.cache_misses);
    println!("  - Failed resolutions: {}", stats.failed_resolutions);

    Ok(())
}

fn handle_circular_dependencies() -> OwlResult<()> {
    println!("\n3. Handling Circular Dependencies");

    // Create ontologies with circular imports
    let mut ontology_a = Ontology::new();
    let mut ontology_b = Ontology::new();

    let iri_a = IRI::new("http://example.org/ontology-a")?;
    let iri_b = IRI::new("http://example.org/ontology-b")?;

    ontology_a.set_iri(iri_a.clone());
    ontology_b.set_iri(iri_b.clone());

    // Create circular dependency: A imports B, B imports A
    ontology_a.add_import(iri_b.clone());
    ontology_b.add_import(iri_a.clone());

    // Add some content to each ontology
    let class_a = Class::new("http://example.org/ClassA");
    ontology_a.add_class(class_a)?;

    let class_b = Class::new("http://example.org/ClassB");
    ontology_b.add_class(class_b)?;

    // Try to resolve imports - should detect circular dependency
    let mut resolver = parser::ImportResolver::new()?;

    println!("Testing circular dependency detection...");
    match resolver.resolve_imports(&mut ontology_a) {
        Ok(_) => println!("⚠ Expected circular dependency was not detected"),
        Err(OwlError::ImportResolutionError { message, .. }) => {
            println!("✓ Circular dependency detected: {}", message);
        }
        Err(e) => println!("✓ Import resolution failed (as expected): {}", e),
    }

    let stats = resolver.stats();
    println!("Circular dependency statistics:");
    println!(
        "  - Circular dependencies detected: {}",
        stats.circular_dependencies_detected
    );

    Ok(())
}

fn custom_import_sources() -> OwlResult<()> {
    println!("\n4. Custom Import Sources");

    // Create a custom file system source with specific directories
    let mut file_source = parser::FileSystemImportSource::new();

    // Add common ontology directories
    file_source.add_base_directory("examples/ontologies");
    file_source.add_base_directory("tests/fixtures");
    file_source.add_base_directory("/usr/share/ontologies");

    // Add custom file extensions
    file_source.add_file_extension("omn");
    file_source.add_file_extension("obo");

    println!("Created custom file system source with:");
    println!(
        "  - some base directories (internal)" // Placeholder since base_directories is private
    );
    println!("  - some file extensions (internal)");

    // Test which IRIs the source can resolve
    let test_iris = vec![
        "file://test.owl",
        "relative/path.ttl",
        "http://example.org/remote.owl", // Should not be resolved by file source
    ];

    for iri_str in test_iris {
        let iri = IRI::new(iri_str).unwrap();
        println!(
            "  - {}: {}",
            iri_str,
            if file_source.can_resolve(&iri) {
                "✓ Can resolve"
            } else {
                "✗ Cannot resolve"
            }
        );
    }

    // Create import resolver and add custom source
    let mut resolver = parser::ImportResolver::new()?;
    resolver.add_source(Box::new(file_source));

    println!("Added custom file system source to resolver");

    Ok(())
}

fn cache_management() -> OwlResult<()> {
    println!("\n5. Cache Management");

    // Create import resolver
    let mut resolver = parser::ImportResolver::new()?;

    // Show initial cache stats
    let initial_stats = resolver.cache_stats();
    println!("Initial cache stats:");
    println!("  - Entries: {}", initial_stats.entries);
    println!("  - Total size: {} bytes", initial_stats.total_size);
    println!("  - Max size: {} bytes", initial_stats.max_size);

    // Test cache operations
    let mut test_ontology = Ontology::new();
    test_ontology.set_iri("http://example.org/cached-ontology");

    let test_class = Class::new("http://example.org/TestClass");
    test_ontology.add_class(test_class)?;

    // The cache is normally managed internally, but we can demonstrate clearing it
    println!("Clearing cache...");
    resolver.clear_cache();

    let cleared_stats = resolver.cache_stats();
    println!("After clearing:");
    println!("  - Entries: {}", cleared_stats.entries);
    println!("  - Total size: {} bytes", cleared_stats.total_size);

    // Demonstrate configuration access
    let config = resolver.config();
    println!("Current configuration:");
    println!("  - Max depth: {}", config.max_depth);
    println!("  - Cache TTL: {:?}", config.cache_ttl);

    // Modify configuration
    let config_mut = resolver.config_mut();
    config_mut.max_depth = 20;
    config_mut.cache_ttl = std::time::Duration::from_secs(7200); // 2 hours

    println!("After modification:");
    println!("  - Max depth: {}", resolver.config().max_depth);
    println!("  - Cache TTL: {:?}", resolver.config().cache_ttl);

    Ok(())
}

fn create_turtle_with_imports() -> OwlResult<()> {
    println!("\n6. Turtle Example with Imports");

    // Create a Turtle ontology with import statements
    let turtle_content = r#"
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex: <http://example.org/> .

# Main ontology declaration
ex:MyOntology a owl:Ontology ;
    rdfs:label "My Import Example Ontology" ;
    owl:imports <http://example.org/biomedical> ;
    owl:imports <http://example.org/geography> ;
    owl:imports <local-ontology.ttl> .

# Local classes
ex:Person a owl:Class ;
    rdfs:label "Person" ;
    rdfs:comment "A person entity" .

ex:Organization a owl:Class ;
    rdfs:label "Organization" ;
    rdfs:comment "An organization entity" .

# Properties
ex:worksFor a owl:ObjectProperty ;
    rdfs:label "works for" ;
    rdfs:domain ex:Person ;
    rdfs:range ex:Organization .

ex:hasName a owl:DatatypeProperty ;
    rdfs:label "has name" ;
    rdfs:domain ex:Person ;
    rdfs:range xsd:string .
"#;

    println!("Turtle content with import statements:");
    println!("{}", turtle_content);

    // Parse with import resolution enabled
    let config = ParserConfig {
        resolve_imports: true,
        ignore_import_errors: true,
        ..Default::default()
    };

    let parser = TurtleParser::with_config(config);
    match parser.parse_str(turtle_content) {
        Ok(ontology) => {
            println!("✓ Parsed ontology successfully");
            println!("  - Imports: {}", ontology.imports().len());
            println!("  - Classes: {}", ontology.classes().len());
            println!(
                "  - Properties: {}",
                ontology.object_properties().len() + ontology.data_properties().len()
            );
        }
        Err(e) => println!("⚠ Parsing failed (expected due to missing imports): {}", e),
    }

    Ok(())
}
