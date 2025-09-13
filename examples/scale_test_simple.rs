//! Simple Scale Testing for OWL2 Reasoner
//!
//! Quick scale test to measure performance of working components

use owl2_reasoner::*;
use std::time::Instant;

fn main() -> OwlResult<()> {
    println!("🚀 OWL2 Reasoner Simple Scale Test");
    println!("==================================");

    // Test different scales
    let scales = vec![100, 500, 1000, 2500, 5000];

    for scale in scales {
        println!("\n📊 Testing with {} entities:", scale);

        // Measure IRI creation performance
        let start = Instant::now();
        for i in 0..scale {
            let iri_str = format!("http://example.org/entity/{}", i);
            let _iri = IRI::new(&iri_str)?;
        }
        let iri_time = start.elapsed();
        println!("   IRI creation: {:?}", iri_time);

        // Measure ontology creation performance
        let start = Instant::now();
        let mut ontology = Ontology::new();

        // Add classes
        for i in 0..scale {
            let iri = IRI::new(&format!("http://example.org/Class{}", i))?;
            let class = Class::new(iri);
            ontology.add_class(class)?;
        }

        // Add properties
        for i in 0..(scale / 10).max(1) {
            let iri = IRI::new(&format!("http://example.org/hasProperty{}", i))?;
            let prop = ObjectProperty::new(iri);
            ontology.add_object_property(prop)?;
        }

        // Add subclass relationships
        for i in 1..(scale / 5).max(1) {
            let child_iri = IRI::new(&format!("http://example.org/Class{}", i))?;
            let parent_iri = IRI::new(&format!("http://example.org/Class{}", i / 2))?;

            let child = ClassExpression::Class(Class::new(child_iri));
            let parent = ClassExpression::Class(Class::new(parent_iri));
            let axiom = SubClassOfAxiom::new(child, parent);
            ontology.add_subclass_axiom(axiom)?;
        }

        let ontology_time = start.elapsed();
        println!("   Ontology creation: {:?}", ontology_time);
        println!("   Final sizes: {} classes, {} properties, {} axioms",
                 ontology.classes().len(),
                 ontology.object_properties().len(),
                 ontology.subclass_axioms().len());

        // Measure reasoning performance
        let reasoner = SimpleReasoner::new(ontology.clone());

        let start = Instant::now();
        let _is_consistent = reasoner.is_consistent()?;
        let consistency_time = start.elapsed();
        println!("   Consistency check: {:?}", consistency_time);

        // Measure subclass reasoning
        let start = Instant::now();
        let classes: Vec<_> = ontology.classes().iter().take(10).cloned().collect();
        for i in 0..classes.len().min(5) {
            for j in 0..classes.len().min(5) {
                if i != j {
                    let _ = reasoner.is_subclass_of(&classes[i].iri(), &classes[j].iri());
                }
            }
        }
        let reasoning_time = start.elapsed();
        println!("   Subclass reasoning: {:?}", reasoning_time);

        // Estimate memory usage
        let estimated_memory = estimate_ontology_memory(&ontology);
        println!("   Estimated memory: {:.2} KB", estimated_memory as f64 / 1024.0);

        println!("   Total time: {:?}", iri_time + ontology_time + consistency_time + reasoning_time);
    }

    println!("\n✅ Scale test completed!");
    Ok(())
}

fn estimate_ontology_memory(ontology: &Ontology) -> usize {
    let class_count = ontology.classes().len();
    let prop_count = ontology.object_properties().len();
    let axiom_count = ontology.subclass_axioms().len();
    let individual_count = ontology.named_individuals().len();

    // Conservative memory estimation
    (class_count * 128) +    // Classes: ~128 bytes each
    (prop_count * 96) +      // Properties: ~96 bytes each
    (axiom_count * 64) +     // Axioms: ~64 bytes each
    (individual_count * 80)  // Individuals: ~80 bytes each
}