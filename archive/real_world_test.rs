//! Real-world OWL2 Ontology Test
//!
//! Tests performance with realistic biomedical ontology patterns
//! based on common structures found in research datasets

use owl2_reasoner::*;
use std::time::Instant;

fn main() -> OwlResult<()> {
    println!("🧬 Real-world OWL2 Ontology Performance Test");
    println!("===========================================");

    // Create realistic biomedical ontology with common patterns
    let mut ontology = create_biomedical_ontology();

    println!("📊 Ontology created:");
    println!("   Classes: {}", ontology.classes().len());
    println!("   Properties: {}", ontology.object_properties().len());
    println!("   Subclass axioms: {}", ontology.subclass_axioms().len());
    println!("   Individuals: {}", ontology.named_individuals().len());

    // Test parsing performance if we had real files
    println!("\n🔍 Performance Testing:");

    // Initialize reasoner
    let start = Instant::now();
    let reasoner = SimpleReasoner::new(ontology.clone());
    let reasoner_init_time = start.elapsed();
    println!("   Reasoner initialization: {:?}", reasoner_init_time);

    // Warm up caches
    reasoner.warm_up_caches()?;

    // Test consistency checking
    let start = Instant::now();
    let is_consistent = reasoner.is_consistent()?;
    let consistency_time = start.elapsed();
    println!("   Consistency check: {:?} ({})", consistency_time, is_consistent);

    // Test subclass reasoning across biomedical hierarchy
    let start = Instant::now();
    let disease_iri = IRI::new("http://purl.obolibrary.org/obo/DOID_4")?; // Disease
    let cancer_iri = IRI::new("http://purl.obolibrary.org/obo/DOID_162")?; // Cancer
    let lung_cancer_iri = IRI::new("http://purl.obolibrary.org/obo/DOID_1324")?; // Lung cancer

    let _is_cancer_disease = reasoner.is_subclass_of(&cancer_iri, &disease_iri)?;
    let _is_lung_cancer_cancer = reasoner.is_subclass_of(&lung_cancer_iri, &cancer_iri)?;
    let _is_lung_cancer_disease = reasoner.is_subclass_of(&lung_cancer_iri, &disease_iri)?;
    let reasoning_time = start.elapsed();
    println!("   Biomedical reasoning: {:?}", reasoning_time);

    // Test multiple subclass queries (realistic workload)
    let start = Instant::now();
    let classes: Vec<_> = ontology.classes().iter().take(20).cloned().collect();
    let mut subclass_checks = 0;
    for i in 0..classes.len() {
        for j in 0..classes.len() {
            if i != j {
                let _ = reasoner.is_subclass_of(&classes[i].iri(), &classes[j].iri());
                subclass_checks += 1;
            }
        }
    }
    let bulk_reasoning_time = start.elapsed();
    println!("   Bulk reasoning ({} checks): {:?}", subclass_checks, bulk_reasoning_time);

    // Test satisfiability checking
    let start = Instant::now();
    for class in classes.iter().take(10) {
        let _ = reasoner.is_class_satisfiable(&class.iri());
    }
    let satisfiability_time = start.elapsed();
    println!("   Satisfiability checking (10 classes): {:?}", satisfiability_time);

    // Memory analysis
    let start = Instant::now();
    let estimated_memory = estimate_realistic_memory_usage(&ontology);
    let memory_analysis_time = start.elapsed();
    println!("   Memory analysis: {:?} ({:.2} KB)", memory_analysis_time, estimated_memory as f64 / 1024.0);

    // Performance summary
    println!("\n📈 Performance Summary:");
    println!("   Total operations time: {:?}",
        reasoner_init_time + consistency_time + reasoning_time + bulk_reasoning_time + satisfiability_time);
    println!("   Average per subclass check: {:?}",
        bulk_reasoning_time.as_nanos() as f64 / subclass_checks as f64);
    println!("   Memory efficiency: {:.2} bytes per entity",
        estimated_memory as f64 / (ontology.classes().len() + ontology.object_properties().len() + ontology.named_individuals().len()) as f64);

    // Generate performance report
    generate_performance_report(&ontology,
        reasoner_init_time, consistency_time, reasoning_time,
        bulk_reasoning_time, satisfiability_time, estimated_memory, subclass_checks)?;

    println!("\n✅ Real-world testing completed!");
    println!("   Results show performance characteristics of realistic biomedical ontologies.");
    Ok(())
}

fn create_biomedical_ontology() -> Ontology {
    let mut ontology = Ontology::new();

    // Disease hierarchy (based on Disease Ontology patterns)
    let disease_classes = vec![
        ("http://purl.obolibrary.org/obo/DOID_4", "Disease"),
        ("http://purl.obolibrary.org/obo/DOID_14566", "Disease by infectious agent"),
        ("http://purl.obolibrary.org/obo/DOID_162", "Cancer"),
        ("http://purl.obolibrary.org/obo/DOID_1324", "Lung cancer"),
        ("http://purl.obolibrary.org/obo/DOID_0050686", "Infectious disease"),
        ("http://purl.obolibrary.org/obo/DOID_0080600", "Autoimmune disease"),
        ("http://purl.obolibrary.org/obo/DOID_0060072", "Genetic disease"),
        ("http://purl.obolibrary.org/obo/DOID_1579", "Respiratory disease"),
        ("http://purl.obolibrary.org/obo/DOID_114", "Cardiovascular disease"),
        ("http://purl.obolibrary.org/obo/DOID_150", "Diabetes"),
    ];

    for (iri, name) in disease_classes {
        let class = Class::new(IRI::new(iri).unwrap());
        ontology.add_class(class).unwrap();
    }

    // Anatomy hierarchy (based on UBERON patterns)
    let anatomy_classes = vec![
        ("http://purl.obolibrary.org/obo/UBERON_0001062", "Anatomical entity"),
        ("http://purl.obolibrary.org/obo/UBERON_0000061", "Anatomical structure"),
        ("http://purl.obolibrary.org/obo/UBERON_0000467", "Anatomical system"),
        ("http://purl.obolibrary.org/obo/UBERON_0000479", "Tissue"),
        ("http://purl.obolibrary.org/obo/UBERON_0002369", "Organ"),
        ("http://purl.obolibrary.org/obo/UBERON_0002107", "Lung"),
        ("http://purl.obolibrary.org/obo/UBERON_0000948", "Heart"),
        ("http://purl.obolibrary.org/obo/UBERON_0000955", "Brain"),
        ("http://purl.obolibrary.org/obo/UBERON_0001255", "Vascular system"),
        ("http://purl.obolibrary.org/obo/UBERON_0004535", "Cardiovascular system"),
    ];

    for (iri, name) in anatomy_classes {
        let class = Class::new(IRI::new(iri).unwrap());
        ontology.add_class(class).unwrap();
    }

    // Gene/protein hierarchy (based on GO patterns)
    let gene_classes = vec![
        ("http://purl.obolibrary.org/obo/SO_0000110", "Sequence feature"),
        ("http://purl.obolibrary.org/obo/SO_0000704", "Gene"),
        ("http://purl.obolibrary.org/obo/SO_0000352", "Regulatory region"),
        ("http://purl.obolibrary.org/obo/SO_0000316", "Coding sequence"),
        ("http://purl.obolibrary.org/obo/SO_0000336", "Pseudogene"),
        ("http://purl.obolibrary.org/obo/SO_0000234", "mRNA"),
        ("http://purl.obolibrary.org/obo/SO_0000104", "Protein"),
        ("http://purl.obolibrary.org/obo/SO_0000017", "Ribosomal RNA"),
        ("http://purl.obolibrary.org/obo/SO_0000253", "Transfer RNA"),
        ("http://purl.obolibrary.org/obo/SO_0001266", "Non-coding RNA"),
    ];

    for (iri, name) in gene_classes {
        let class = Class::new(IRI::new(iri).unwrap());
        ontology.add_class(class).unwrap();
    }

    // Create realistic subclass relationships
    let subclass_relationships = vec![
        // Disease hierarchy
        ("http://purl.obolibrary.org/obo/DOID_162", "http://purl.obolibrary.org/obo/DOID_4"), // Cancer is-a Disease
        ("http://purl.obolibrary.org/obo/DOID_1324", "http://purl.obolibrary.org/obo/DOID_162"), // Lung cancer is-a Cancer
        ("http://purl.obolibrary.org/obo/DOID_14566", "http://purl.obolibrary.org/obo/DOID_4"), // Infectious disease is-a Disease
        ("http://purl.obolibrary.org/obo/DOID_0050686", "http://purl.obolibrary.org/obo/DOID_14566"), // Infectious disease is-a Disease by infectious agent
        ("http://purl.obolibrary.org/obo/DOID_0080600", "http://purl.obolibrary.org/obo/DOID_4"), // Autoimmune disease is-a Disease
        ("http://purl.obolibrary.org/obo/DOID_0060072", "http://purl.obolibrary.org/obo/DOID_4"), // Genetic disease is-a Disease
        ("http://purl.obolibrary.org/obo/DOID_1579", "http://purl.obolibrary.org/obo/DOID_4"), // Respiratory disease is-a Disease
        ("http://purl.obolibrary.org/obo/DOID_150", "http://purl.obolibrary.org/obo/DOID_4"), // Diabetes is-a Disease

        // Anatomy hierarchy
        ("http://purl.obolibrary.org/obo/UBERON_0000061", "http://purl.obolibrary.org/obo/UBERON_0001062"), // Anatomical structure is-a Anatomical entity
        ("http://purl.obolibrary.org/obo/UBERON_0000467", "http://purl.obolibrary.org/obo/UBERON_0000061"), // Anatomical system is-a Anatomical structure
        ("http://purl.obolibrary.org/obo/UBERON_0000479", "http://purl.obolibrary.org/obo/UBERON_0000061"), // Tissue is-a Anatomical structure
        ("http://purl.obolibrary.org/obo/UBERON_0002369", "http://purl.obolibrary.org/obo/UBERON_0000479"), // Organ is-a Tissue
        ("http://purl.obolibrary.org/obo/UBERON_0002107", "http://purl.obolibrary.org/obo/UBERON_0002369"), // Lung is-a Organ
        ("http://purl.obolibrary.org/obo/UBERON_0000948", "http://purl.obolibrary.org/obo/UBERON_0002369"), // Heart is-a Organ
        ("http://purl.obolibrary.org/obo/UBERON_0000955", "http://purl.obolibrary.org/obo/UBERON_0002369"), // Brain is-a Organ
        ("http://purl.obolibrary.org/obo/UBERON_0001255", "http://purl.obolibrary.org/obo/UBERON_0000467"), // Vascular system is-a Anatomical system
        ("http://purl.obolibrary.org/obo/UBERON_0004535", "http://purl.obolibrary.org/obo/UBERON_0000467"), // Cardiovascular system is-a Anatomical system

        // Gene hierarchy
        ("http://purl.obolibrary.org/obo/SO_0000704", "http://purl.obolibrary.org/obo/SO_0000110"), // Gene is-a Sequence feature
        ("http://purl.obolibrary.org/obo/SO_0000352", "http://purl.obolibrary.org/obo/SO_0000110"), // Regulatory region is-a Sequence feature
        ("http://purl.obolibrary.org/obo/SO_0000316", "http://purl.obolibrary.org/obo/SO_0000704"), // Coding sequence is-a Gene
        ("http://purl.obolibrary.org/obo/SO_0000336", "http://purl.obolibrary.org/obo/SO_0000704"), // Pseudogene is-a Gene
        ("http://purl.obolibrary.org/obo/SO_0000234", "http://purl.obolibrary.org/obo/SO_0000253"), // mRNA is-a Transfer RNA
        ("http://purl.obolibrary.org/obo/SO_0000104", "http://purl.obolibrary.org/obo/SO_0000110"), // Protein is-a Sequence feature
        ("http://purl.obolibrary.org/obo/SO_0000017", "http://purl.obolibrary.org/obo/SO_0000234"), // Ribosomal RNA is-a mRNA
        ("http://purl.obolibrary.org/obo/SO_0000253", "http://purl.obolibrary.org/obo/SO_0000234"), // Transfer RNA is-a mRNA
        ("http://purl.obolibrary.org/obo/SO_0001266", "http://purl.obolibrary.org/obo/SO_0000110"), // Non-coding RNA is-a Sequence feature
    ];

    for (child_iri, parent_iri) in subclass_relationships {
        let child = ClassExpression::Class(Class::new(IRI::new(child_iri).unwrap()));
        let parent = ClassExpression::Class(Class::new(IRI::new(parent_iri).unwrap()));
        let axiom = SubClassOfAxiom::new(child, parent);
        ontology.add_subclass_axiom(axiom).unwrap();
    }

    // Add some object properties
    let properties = vec![
        ("http://purl.obolibrary.org/obo/BFO_0000050", "part of"),
        ("http://purl.obolibrary.org/obo/BFO_0000051", "has part"),
        ("http://purl.obolibrary.org/obo/RO_0000087", "has role"),
        ("http://purl.obolibrary.org/obo/RO_0000053", "has characteristic"),
        ("http://purl.obolibrary.org/obo/RO_0002200", "has phenotype"),
        ("http://purl.obolibrary.org/obo/RO_0000056", "participates in"),
        ("http://purl.obolibrary.org/obo/RO_0000057", "has participant"),
    ];

    for (iri, name) in properties {
        let prop = ObjectProperty::new(IRI::new(iri).unwrap());
        ontology.add_object_property(prop).unwrap();
    }

    // Add some individuals (disease instances, gene instances)
    let individuals = vec![
        ("http://example.org/individuals/patient1", "Patient 1"),
        ("http://example.org/individuals/patient2", "Patient 2"),
        ("http://example.org/individuals/gene_BRAF", "BRAF gene"),
        ("http://example.org/individuals/gene_TP53", "TP53 gene"),
        ("http://example.org/individuals/protein_p53", "p53 protein"),
    ];

    for (iri, name) in individuals {
        let individual = NamedIndividual::new(IRI::new(iri).unwrap());
        ontology.add_named_individual(individual).unwrap();
    }

    ontology
}

fn estimate_realistic_memory_usage(ontology: &Ontology) -> usize {
    let class_count = ontology.classes().len();
    let prop_count = ontology.object_properties().len();
    let axiom_count = ontology.subclass_axioms().len();
    let individual_count = ontology.named_individuals().len();

    // More realistic memory estimation for complex biomedical ontologies
    // Classes have longer IRIs and more complex relationships
    (class_count * 256) +     // Classes: longer IRIs, ~256 bytes each
    (prop_count * 128) +     // Properties: ~128 bytes each
    (axiom_count * 96) +     // Axioms: complex relationships, ~96 bytes each
    (individual_count * 160)   // Individuals: with properties, ~160 bytes each
}

fn generate_performance_report(
    ontology: &Ontology,
    reasoner_init: std::time::Duration,
    consistency_check: std::time::Duration,
    reasoning: std::time::Duration,
    bulk_reasoning: std::time::Duration,
    satisfiability: std::time::Duration,
    memory_usage: usize,
    subclass_checks: usize
) -> OwlResult<()> {

    let report = format!(
        "Real-world OWL2 Ontology Performance Report
============================================

Test Configuration:
- Ontology Type: Biomedical (Disease + Anatomy + Gene)
- Total Classes: {}
- Object Properties: {}
- Subclass Axioms: {}
- Named Individuals: {}
- Subclass Checks Performed: {}

Performance Results:
- Reasoner Initialization: {:?}
- Consistency Checking: {:?}
- Reasoning Operations: {:?}
- Bulk Reasoning ({} checks): {:?}
- Satisfiability Checking: {:?}
- Total Time: {:?}

Memory Usage:
- Estimated Total Memory: {:.2} KB
- Memory per Entity: {:.1} bytes
- Entity Count: {}

Performance Metrics:
- Average Time per Subclass Check: {:.2} ns
- Checks per Second: {:.0}
- Memory Efficiency: {:.2} bytes/entity

Test Notes:
- Uses realistic biomedical ontology structure
- Based on common patterns from Disease Ontology, UBERON, and Gene Ontology
- Performance reflects actual working components, not theoretical claims
- Results may vary with different ontology structures and sizes

Generated by: OWL2 Reasoner Performance Testing
",
        ontology.classes().len(),
        ontology.object_properties().len(),
        ontology.subclass_axioms().len(),
        ontology.named_individuals().len(),
        subclass_checks,
        reasoner_init,
        consistency_check,
        reasoning,
        subclass_checks,
        bulk_reasoning,
        satisfiability,
        reasoner_init + consistency_check + reasoning + bulk_reasoning + satisfiability,
        memory_usage as f64 / 1024.0,
        memory_usage as f64 / (ontology.classes().len() + ontology.object_properties().len() + ontology.named_individuals().len()) as f64,
        ontology.classes().len() + ontology.object_properties().len() + ontology.named_individuals().len(),
        bulk_reasoning.as_nanos() as f64 / subclass_checks as f64,
        subclass_checks as f64 / bulk_reasoning.as_secs_f64(),
        memory_usage as f64 / (ontology.classes().len() + ontology.object_properties().len() + ontology.named_individuals().len()) as f64
    );

    std::fs::write("real_world_performance_report.txt", report)?;
    println!("\n📄 Performance report saved to: real_world_performance_report.txt");

    Ok(())
}