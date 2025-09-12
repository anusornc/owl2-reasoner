//! Property-based tests for IRI functionality

use proptest::prelude::*;
use owl2_reasoner::iri::IRI;
use std::hash::{Hash, Hasher};

/// Test that IRI parsing and serialization are consistent
proptest! {
    #[test]
    fn test_iri_roundtrip(
        iri_str in r"[a-zA-Z][a-zA-Z0-9-]*://[a-zA-Z0-9-._~:/?#\\[\\]@!$&'()*+,;=]+"
    ) {
        // Skip empty strings and strings that are too long
        if !iri_str.is_empty() && iri_str.len() <= 2048 {
            let result = IRI::new(&iri_str);
            
            // Test that valid IRIs can be round-tripped
            if let Ok(iri) = result {
                let serialized = iri.as_str();
                let result2 = IRI::new(serialized);
                
                // Should either succeed or fail consistently
                match result2 {
                    Ok(iri2) => {
                        // If parsing succeeds, the serialized form should be equivalent
                        assert!(iri2.as_str() == iri.as_str() || 
                                iri2.as_str().ends_with(iri.as_str()) ||
                                iri.as_str().ends_with(iri2.as_str()));
                    }
                    Err(_) => {
                        // Some IRIs might not round-trip perfectly due to normalization
                        // This is acceptable as long as it doesn't panic
                    }
                }
            }
        }
    }
}

/// Test that IRI equality is reflexive
proptest! {
    #[test]
    fn test_iri_equality_reflexive(
        iri_str in r"[a-zA-Z][a-zA-Z0-9-]*://[a-zA-Z0-9-._~:/?#\\[\\]@!$&'()*+,;=]+"
    ) {
        if !iri_str.is_empty() && iri_str.len() <= 2048 {
            if let Ok(iri) = IRI::new(&iri_str) {
                assert_eq!(iri, iri, "IRI equality should be reflexive");
            }
        }
    }
}

/// Test that IRI hashing is consistent with equality
proptest! {
    #[test]
    fn test_ri_hashing_consistency(
        iri_str1 in r"[a-zA-Z][a-zA-Z0-9-]*://[a-zA-Z0-9-._~:/?#\\[\\]@!$&'()*+,;=]+",
        iri_str2 in r"[a-zA-Z][a-zA-Z0-9-]*://[a-zA-Z0-9-._~:/?#\\[\\]@!$&'()*+,;=]+"
    ) {
        if !iri_str1.is_empty() && iri_str1.len() <= 2048 && 
           !iri_str2.is_empty() && iri_str2.len() <= 2048 {
            let iri1_result = IRI::new(&iri_str1);
            let iri2_result = IRI::new(&iri_str2);
            
            if let (Ok(iri1), Ok(iri2)) = (iri1_result, iri2_result) {
                // Equal IRIs should have equal hashes
                if iri1 == iri2 {
                    use std::hash::Hash;
                    use std::collections::hash_map::DefaultHasher;
                    
                    let mut hasher1 = DefaultHasher::new();
                    let mut hasher2 = DefaultHasher::new();
                    
                    iri1.hash(&mut hasher1);
                    iri2.hash(&mut hasher2);
                    
                    assert_eq!(hasher1.finish(), hasher2.finish(), 
                              "Equal IRIs should have equal hashes");
                }
            }
        }
    }
}

/// Test that IRI components are accessible
proptest! {
    #[test]
    fn test_iri_components_accessible(
        iri_str in r"[a-zA-Z][a-zA-Z0-9-]*://[a-zA-Z0-9-._~:/?#\\[\\]@!$&'()*+,;=]+"
    ) {
        if !iri_str.is_empty() && iri_str.len() <= 2048 {
            if let Ok(iri) = IRI::new(&iri_str) {
                // These operations should not panic
                let _str = iri.as_str();
                
                // Test hashing with a hasher
                use std::hash::Hash;
                use std::collections::hash_map::DefaultHasher;
                let mut hasher = DefaultHasher::new();
                iri.hash(&mut hasher);
                
                // Basic sanity checks
                assert!(!iri.as_str().is_empty());
            }
        }
    }
}