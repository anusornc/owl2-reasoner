//! Biomedical Ontology Example
//! 
//! This example demonstrates how to create a biomedical ontology with
//! gene-disease associations and protein interactions, showing more
//! complex class expressions and reasoning patterns.

use owl2_reasoner::*;

fn main() -> OwlResult<()> {
    println!("=== Biomedical Ontology Example ===\n");

    // Create a new ontology
    let mut ontology = Ontology::new();
    ontology.set_iri("http://example.org/biomedical");

    // Define biomedical classes
    let gene = Class::new("http://example.org/Gene");
    let protein = Class::new("http://example.org/Protein");
    let disease = Class::new("http://example.org/Disease");
    let symptom = Class::new("http://example.org/Symptom");
    let treatment = Class::new("http://example.org/Treatment");
    let drug = Class::new("http://example.org/Drug");
    let genetic_disorder = Class::new("http://example.org/GeneticDisorder");
    let rare_disease = Class::new("http://example.org/RareDisease");

    // Add classes to ontology
    for class in &[&gene, &protein, &disease, &symptom, &treatment, &drug, &genetic_disorder, &rare_disease] {
        ontology.add_class(class.clone())?;
    }

    println!("✓ Added {} biomedical classes", ontology.classes().len());

    // Define properties
    let mut encodes = ObjectProperty::new("http://example.org/encodes");
    let mut associated_with = ObjectProperty::new("http://example.org/associatedWith");
    let mut causes = ObjectProperty::new("http://example.org/causes");
    let mut treats = ObjectProperty::new("http://example.org/treats");
    let mut has_symptom = ObjectProperty::new("http://example.org/hasSymptom");
    let mut interacts_with = ObjectProperty::new("http://example.org/interactsWith");

    // Add property characteristics
    associated_with.add_characteristic(ObjectPropertyCharacteristic::Symmetric);
    interacts_with.add_characteristic(ObjectPropertyCharacteristic::Symmetric);

    // Add properties to ontology
    for prop in &[&encodes, &associated_with, &causes, &treats, &has_symptom, &interacts_with] {
        ontology.add_object_property(prop.clone())?;
    }

    println!("✓ Added {} biomedical properties", ontology.object_properties().len());

    // Add subclass relationships
    let subclass_axioms = vec![
        // Genetic disorders are diseases
        SubClassOfAxiom::new(
            ClassExpression::from(genetic_disorder.clone()),
            ClassExpression::from(disease.clone()),
        ),
        // Rare diseases are diseases
        SubClassOfAxiom::new(
            ClassExpression::from(rare_disease.clone()),
            ClassExpression::from(disease.clone()),
        ),
        // Drugs are treatments
        SubClassOfAxiom::new(
            ClassExpression::from(drug.clone()),
            ClassExpression::from(treatment.clone()),
        ),
    ];

    for axiom in subclass_axioms {
        ontology.add_subclass_axiom(axiom)?;
    }

    println!("✓ Added {} subclass axioms", ontology.subclass_axioms().len());

    // Add equivalent classes using complex expressions
    // Genetic disorder = Disease ⊓ ∃associatedWith.Gene
    let genetic_disorder_def = ClassExpression::ObjectIntersectionOf(vec![
        ClassExpression::from(disease.clone()),
        ClassExpression::ObjectSomeValuesFrom(
            Box::new(ObjectPropertyExpression::ObjectProperty(associated_with.clone())),
            Box::new(ClassExpression::from(gene.clone())),
        ),
    ]);

    let equivalent_genetic = EquivalentClassesAxiom::new(vec![
        ClassExpression::from(genetic_disorder.clone()),
        genetic_disorder_def,
    ]);

    ontology.add_equivalent_classes_axiom(equivalent_genetic)?;

    // Create biomedical individuals
    let brca1 = NamedIndividual::new("http://example.org/BRCA1");
    let brca2 = NamedIndividual::new("http://example.org/BRCA2");
    let brca1_protein = NamedIndividual::new("http://example.org/BRCA1_protein");
    let brca2_protein = NamedIndividual::new("http://example.org/BRCA2_protein");
    let breast_cancer = NamedIndividual::new("http://example.org/BreastCancer");
    let ovarian_cancer = NamedIndividual::new("http://example.org/OvarianCancer");
    let tamoxifen = NamedIndividual::new("http://example.org/Tamoxifen");
    let fatigue = NamedIndividual::new("http://example.org/Fatigue");

    // Add individuals to ontology
    for individual in &[&brca1, &brca2, &brca1_protein, &brca2_protein, &breast_cancer, &ovarian_cancer, &tamoxifen, &fatigue] {
        ontology.add_named_individual(individual.clone())?;
    }

    println!("✓ Added {} biomedical individuals", ontology.named_individuals().len());

    // Add class assertions
    let class_assertions = vec![
        // Genes
        ClassAssertionAxiom::new(ClassExpression::from(gene.clone()), brca1.clone()),
        ClassAssertionAxiom::new(ClassExpression::from(gene.clone()), brca2.clone()),
        
        // Proteins
        ClassAssertionAxiom::new(ClassExpression::from(protein.clone()), brca1_protein.clone()),
        ClassAssertionAxiom::new(ClassExpression::from(protein.clone()), brca2_protein.clone()),
        
        // Diseases
        ClassAssertionAxiom::new(ClassExpression::from(disease.clone()), breast_cancer.clone()),
        ClassAssertionAxiom::new(ClassExpression::from(disease.clone()), ovarian_cancer.clone()),
        ClassAssertionAxiom::new(ClassExpression::from(genetic_disorder.clone()), breast_cancer.clone()),
        ClassAssertionAxiom::new(ClassExpression::from(genetic_disorder.clone()), ovarian_cancer.clone()),
        
        // Treatments
        ClassAssertionAxiom::new(ClassExpression::from(drug.clone()), tamoxifen.clone()),
        
        // Symptoms
        ClassAssertionAxiom::new(ClassExpression::from(symptom.clone()), fatigue.clone()),
    ];

    for assertion in class_assertions {
        ontology.add_class_assertion(assertion)?;
    }

    println!("✓ Added {} class assertions", ontology.class_assertions().len());

    // Add property assertions
    let property_assertions = vec![
        // Gene-protein relationships
        PropertyAssertionAxiom::new(encodes.clone(), brca1.clone(), brca1_protein.clone()),
        PropertyAssertionAxiom::new(encodes.clone(), brca2.clone(), brca2_protein.clone()),
        
        // Gene-disease associations
        PropertyAssertionAxiom::new(associated_with.clone(), brca1.clone(), breast_cancer.clone()),
        PropertyAssertionAxiom::new(associated_with.clone(), brca2.clone(), breast_cancer.clone()),
        PropertyAssertionAxiom::new(associated_with.clone(), brca1.clone(), ovarian_cancer.clone()),
        PropertyAssertionAxiom::new(associated_with.clone(), brca2.clone(), ovarian_cancer.clone()),
        
        // Protein interactions
        PropertyAssertionAxiom::new(interacts_with.clone(), brca1_protein.clone(), brca2_protein.clone()),
        PropertyAssertionAxiom::new(interacts_with.clone(), brca2_protein.clone(), brca1_protein.clone()),
        
        // Disease-symptom relationships
        PropertyAssertionAxiom::new(has_symptom.clone(), breast_cancer.clone(), fatigue.clone()),
        PropertyAssertionAxiom::new(has_symptom.clone(), ovarian_cancer.clone(), fatigue.clone()),
        
        // Treatment relationships
        PropertyAssertionAxiom::new(treats.clone(), tamoxifen.clone(), breast_cancer.clone()),
    ];

    for assertion in property_assertions {
        ontology.add_property_assertion(assertion)?;
    }

    println!("✓ Added {} property assertions", ontology.property_assertions().len());

    // Create reasoner and perform reasoning
    println!("\n=== Biomedical Reasoning Results ===");
    let reasoner = SimpleReasoner::new(ontology);

    // Check consistency
    let is_consistent = reasoner.is_consistent()?;
    println!("✓ Biomedical ontology is consistent: {}", is_consistent);

    // Check subclass relationships
    let subclass_checks = vec![
        (genetic_disorder.clone(), disease.clone(), "GeneticDisorder ⊑ Disease"),
        (rare_disease.clone(), disease.clone(), "RareDisease ⊑ Disease"),
        (drug.clone(), treatment.clone(), "Drug ⊑ Treatment"),
    ];

    for (sub, sup, desc) in subclass_checks {
        let result = reasoner.is_subclass_of(&sub, &sup)?;
        println!("✓ {}: {}", desc, result);
    }

    // Check class satisfiability
    println!("\n=== Class Satisfiability ===");
    let satisfiability_checks = vec![
        (genetic_disorder.clone(), "GeneticDisorder"),
        (breast_cancer.clone(), "BreastCancer"),
        (brca1.clone(), "BRCA1"),
    ];

    for (class, desc) in satisfiability_checks {
        let is_satisfiable = reasoner.is_class_satisfiable(&class)?;
        println!("✓ {} is satisfiable: {}", desc, is_satisfiable);
    }

    // Get instances
    println!("\n=== Instance Retrieval ===");
    let instance_checks = vec![
        (gene.clone(), "Genes"),
        (protein.clone(), "Proteins"),
        (disease.clone(), "Diseases"),
        (genetic_disorder.clone(), "Genetic Disorders"),
        (treatment.clone(), "Treatments"),
    ];

    for (class, desc) in instance_checks {
        let instances = reasoner.get_instances(&class)?;
        println!("✓ {}: {:?}", desc, instances);
    }

    // Complex queries
    println!("\n=== Complex Biomedical Queries ===");
    let mut query_engine = QueryEngine::new(&reasoner.ontology);

    // Find all genes associated with diseases
    let gene_disease_pattern = QueryPattern::And(vec![
        QueryPattern::Basic {
            subject: None,
            predicate: Some(QueryValue::IRI(IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?)),
            object: Some(QueryValue::IRI(gene.clone())),
        },
        QueryPattern::Basic {
            subject: None,
            predicate: Some(QueryValue::IRI(associated_with.clone())),
            object: None,
        },
    ]);

    let genes_with_diseases = query_engine.query_pattern(&gene_disease_pattern)?;
    println!("✓ Found {} genes associated with diseases", genes_with_diseases.len());

    // Find all disease-symptom relationships
    let symptom_pattern = QueryPattern::Basic {
        subject: None,
        predicate: Some(QueryValue::IRI(has_symptom.clone())),
        object: None,
    };

    let symptom_relationships = query_engine.query_pattern(&symptom_pattern)?;
    println!("✓ Found {} disease-symptom relationships", symptom_relationships.len());

    // Find all protein-protein interactions
    let interaction_pattern = QueryPattern::Basic {
        subject: None,
        predicate: Some(QueryValue::IRI(interacts_with.clone())),
        object: None,
    };

    let interactions = query_engine.query_pattern(&interaction_pattern)?;
    println!("✓ Found {} protein-protein interactions", interactions.len());

    // Performance statistics
    println!("\n=== Performance Statistics ===");
    println!("✓ Total biomedical entities: {}", reasoner.ontology.entity_count());
    println!("✓ Total biomedical axioms: {}", reasoner.ontology.axiom_count());
    println!("✓ Cache stats: {:?}", reasoner.cache_stats());

    // Complex class expression example
    println!("\n=== Complex Class Expression Example ===");
    
    // Create a complex class expression: Gene ⊓ ∃associatedWith.(Disease ⊓ ∃hasSymptom.Fatigue)
    let complex_class = ClassExpression::ObjectIntersectionOf(vec![
        ClassExpression::from(gene.clone()),
        ClassExpression::ObjectSomeValuesFrom(
            Box::new(ObjectPropertyExpression::ObjectProperty(associated_with.clone())),
            Box::new(ClassExpression::ObjectIntersectionOf(vec![
                ClassExpression::from(disease.clone()),
                ClassExpression::ObjectSomeValuesFrom(
                    Box::new(ObjectPropertyExpression::ObjectProperty(has_symptom.clone())),
                    Box::new(ClassExpression::from(fatigue.clone())),
                ),
            ])),
        ),
    ]);

    println!("✓ Created complex class expression for 'genes associated with diseases that have fatigue symptom'");
    println!("✓ This demonstrates the power of OWL2 class expressions for biomedical knowledge representation");

    println!("\n=== Biomedical Example Complete ===");
    Ok(())
}