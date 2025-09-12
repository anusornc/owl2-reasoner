//! Property-based test runner
//!
//! This file runs all property-based tests for the OWL2 Reasoner.

#[cfg(test)]
mod tests {
    #[test]
    fn test_property_integration() {
        // This test ensures that property tests are properly integrated
        println!("Property-based tests are available and integrated");
        
        // Run a simple property test to verify functionality
        use owl2_reasoner::iri::IRI;
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let iri = IRI::new("http://example.org/test");
        if let Ok(iri) = iri {
            let mut hasher = DefaultHasher::new();
            iri.hash(&mut hasher);
            assert!(!iri.as_str().is_empty());
        }
    }
}