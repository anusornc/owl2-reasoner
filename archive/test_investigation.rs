use owl2_reasoner::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 INVESTIGATION: Testing EntitySizeCalculator Reality");
    println!("===================================================");

    // Create a simple class and measure its size
    let class_iri = IRI::new("http://example.org/TestClass")?;
    let class = Class::new(class_iri);

    // Calculate size using EntitySizeCalculator
    let size = validation::memory_profiler::EntitySizeCalculator::estimate_class_size(&class);
    let size_kb = size as f64 / 1024.0;

    println!("📊 Class Size Analysis:");
    println!("   • Class: {:?}", class.iri().as_str());
    println!("   • Calculated size: {} bytes", size);
    println!("   • Size in KB: {:.6} KB", size_kb);

    // Let's also check what's actually being calculated
    println!("\n🔍 BREAKDOWN:");
    println!("   • std::mem::size_of_val(&class): {} bytes", std::mem::size_of_val(&class));
    println!("   • IRI string length: {} bytes", class.iri().as_str().len());
    println!("   • Arc overhead (estimated): 16 bytes");
    println!("   • Annotations: {} bytes", class.annotations().len() * 24); // Estimate

    // Test multiple classes to see if there's variation
    println!("\n📊 Testing Multiple Classes:");
    for i in 0..5 {
        let iri = IRI::new(&format!("http://example.org/Class{}", i))?;
        let class = Class::new(iri);
        let size = validation::memory_profiler::EntitySizeCalculator::estimate_class_size(&class);
        println!("   • Class{}: {} bytes ({:.3} KB)", i, size, size as f64 / 1024.0);
    }

    // Test the "breakthrough" claims
    println!("\n🎯 BREAKTHROUGH CLAIM VALIDATION:");
    println!("   • Claim: ~0.23KB per entity");
    println!("   • Actual result: {:.3} KB", size_kb);
    println!("   • Claim: 43x better than 10KB target");
    let improvement = 10.0 / size_kb;
    println!("   • Actual improvement: {:.1}x", improvement);

    // Test SimpleReasoner cache
    println!("\n🧠 SimpleReasoner Cache Test:");
    let mut ontology = Ontology::new();
    for i in 0..10 {
        let iri = IRI::new(&format!("http://example.org/Class{}", i))?;
        let class = Class::new(iri);
        ontology.add_class(class)?;
    }

    let reasoner = reasoning::SimpleReasoner::new(ontology);
    reasoner.warm_up_caches()?;
    let stats = reasoner.get_cache_stats();
    println!("   • Cache hits: {}", stats.hits);
    println!("   • Cache misses: {}", stats.misses);
    println!("   • Hit rate: {:.1}%", stats.hit_rate() * 100.0);

    // Check what "consistency checking" actually does
    println!("\n✅ Consistency Check:");
    let start = std::time::Instant::now();
    let is_consistent = reasoner.is_consistent()?;
    let duration = start.elapsed();
    println!("   • Result: {}", is_consistent);
    println!("   • Time: {:?}", duration);
    println!("   • Sub-millisecond? {}", duration.as_millis() < 1);

    Ok(())
}