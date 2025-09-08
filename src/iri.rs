//! IRI management for OWL2 entities
//! 
//! Provides efficient IRI handling with caching and namespace support.

use crate::error::{OwlError, OwlResult};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use once_cell::sync::Lazy;

/// Internationalized Resource Identifier (IRI)
/// 
/// OWL2 uses IRIs to uniquely identify all entities (classes, properties, individuals).
/// This implementation provides efficient storage and comparison.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IRI {
    /// The full IRI string
    iri: Arc<str>,
    /// Optional namespace prefix for serialization
    prefix: Option<Arc<str>>,
    /// Cache of the hash value for performance
    hash: u64,
}

impl IRI {
    /// Create a new IRI from a string
    pub fn new<S: Into<String>>(iri: S) -> OwlResult<Self> {
        let iri_str = iri.into();
        
        // Basic IRI validation
        if iri_str.is_empty() {
            return Err(OwlError::InvalidIRI("Empty IRI".to_string()));
        }
        
        // TODO: Add more thorough IRI validation according to RFC 3987
        
        let hash = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            iri_str.hash(&mut hasher);
            hasher.finish()
        };
        
        Ok(IRI {
            iri: Arc::from(iri_str),
            prefix: None,
            hash,
        })
    }
    
    /// Create a new IRI with a namespace prefix
    pub fn with_prefix<S: Into<String>, P: Into<String>>(iri: S, prefix: P) -> OwlResult<Self> {
        let mut iri = Self::new(iri)?;
        iri.prefix = Some(Arc::from(prefix.into()));
        Ok(iri)
    }
    
    /// Get the IRI as a string slice
    pub fn as_str(&self) -> &str {
        &self.iri
    }
    
    /// Get the namespace prefix if available
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
    
    /// Get the local name part (after last # or /)
    pub fn local_name(&self) -> &str {
        let iri = self.as_str();
        
        // Find the last separator
        if let Some(hash_pos) = iri.rfind('#') {
            &iri[hash_pos + 1..]
        } else if let Some(slash_pos) = iri.rfind('/') {
            &iri[slash_pos + 1..]
        } else {
            iri
        }
    }
    
    /// Get the namespace part (before last # or /)
    pub fn namespace(&self) -> &str {
        let iri = self.as_str();
        
        if let Some(hash_pos) = iri.rfind('#') {
            &iri[..hash_pos + 1]
        } else if let Some(slash_pos) = iri.rfind('/') {
            &iri[..slash_pos + 1]
        } else {
            ""
        }
    }
    
    /// Check if this IRI is in the OWL namespace
    pub fn is_owl(&self) -> bool {
        self.as_str().starts_with("http://www.w3.org/2002/07/owl#")
    }
    
    /// Check if this IRI is in the RDF namespace
    pub fn is_rdf(&self) -> bool {
        self.as_str().starts_with("http://www.w3.org/1999/02/22-rdf-syntax-ns#")
    }
    
    /// Check if this IRI is in the RDFS namespace
    pub fn is_rdfs(&self) -> bool {
        self.as_str().starts_with("http://www.w3.org/2000/01/rdf-schema#")
    }
    
    /// Check if this IRI is in the XSD namespace
    pub fn is_xsd(&self) -> bool {
        self.as_str().starts_with("http://www.w3.org/2001/XMLSchema#")
    }
}

impl fmt::Display for IRI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(prefix) = &self.prefix {
            write!(f, "{}:{}", prefix, self.local_name())
        } else {
            write!(f, "{}", self.iri)
        }
    }
}

impl Hash for IRI {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl From<&str> for IRI {
    fn from(s: &str) -> Self {
        Self::new(s).expect("Invalid IRI")
    }
}

impl From<String> for IRI {
    fn from(s: String) -> Self {
        Self::new(s).expect("Invalid IRI")
    }
}

/// Common OWL2 IRIs
pub static OWL_IRIS: Lazy<IRIRegistry> = Lazy::new(|| {
    let mut registry = IRIRegistry::new();
    
    // OWL vocabulary
    registry.register("owl", "http://www.w3.org/2002/07/owl#").unwrap();
    registry.register("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#").unwrap();
    registry.register("rdfs", "http://www.w3.org/2000/01/rdf-schema#").unwrap();
    registry.register("xsd", "http://www.w3.org/2001/XMLSchema#").unwrap();
    
    registry
});

/// Registry for managing IRI namespaces and prefixes
#[derive(Debug, Clone, Default)]
pub struct IRIRegistry {
    prefixes: indexmap::IndexMap<String, String>,
    iris: hashbrown::HashMap<String, IRI>,
}

impl IRIRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a namespace prefix
    pub fn register(&mut self, prefix: &str, namespace: &str) -> OwlResult<()> {
        self.prefixes.insert(prefix.to_string(), namespace.to_string());
        Ok(())
    }
    
    /// Get the namespace for a prefix
    pub fn namespace(&self, prefix: &str) -> Option<&str> {
        self.prefixes.get(prefix).map(|s| s.as_str())
    }
    
    /// Create an IRI with a registered prefix
    pub fn iri_with_prefix(&mut self, prefix: &str, local_name: &str) -> OwlResult<IRI> {
        let namespace = self.namespace(prefix)
            .ok_or_else(|| OwlError::UnknownPrefix(prefix.to_string()))?;
        
        let full_iri = format!("{}{}", namespace, local_name);
        let iri = IRI::with_prefix(full_iri, prefix)?;
        
        // Cache the IRI
        self.iris.insert(iri.as_str().to_string(), iri.clone());
        
        Ok(iri)
    }
    
    /// Get or create an IRI
    pub fn get_or_create_iri(&mut self, iri_str: &str) -> OwlResult<IRI> {
        if let Some(cached) = self.iris.get(iri_str) {
            return Ok(cached.clone());
        }
        
        let iri = IRI::new(iri_str)?;
        self.iris.insert(iri_str.to_string(), iri.clone());
        Ok(iri)
    }
    
    /// Get all registered prefixes
    pub fn prefixes(&self) -> impl Iterator<Item = (&str, &str)> {
        self.prefixes.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

/// A reference to an IRI, used in various OWL constructs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IRIRef {
    /// Direct IRI reference
    IRI(IRI),
    /// Abbreviated IRI (prefix:local)
    Abbreviated { prefix: String, local: String },
}

impl IRIRef {
    /// Create a direct IRI reference
    pub fn iri<S: Into<IRI>>(iri: S) -> Self {
        IRIRef::IRI(iri.into())
    }
    
    /// Create an abbreviated IRI reference
    pub fn abbreviated<P: Into<String>, L: Into<String>>(prefix: P, local: L) -> Self {
        IRIRef::Abbreviated {
            prefix: prefix.into(),
            local: local.into(),
        }
    }
    
    /// Resolve to a full IRI using the given registry
    pub fn resolve(&self, registry: &mut IRIRegistry) -> OwlResult<IRI> {
        match self {
            IRIRef::IRI(iri) => Ok(iri.clone()),
            IRIRef::Abbreviated { prefix, local } => {
                registry.iri_with_prefix(prefix, local)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iri_creation() {
        let iri = IRI::new("http://example.org/Person").unwrap();
        assert_eq!(iri.as_str(), "http://example.org/Person");
        assert_eq!(iri.local_name(), "Person");
        assert_eq!(iri.namespace(), "http://example.org/");
    }
    
    #[test]
    fn test_iri_with_prefix() {
        let iri = IRI::with_prefix("http://example.org/Person", "ex").unwrap();
        assert_eq!(iri.as_str(), "http://example.org/Person");
        assert_eq!(iri.prefix(), Some("ex"));
    }
    
    #[test]
    fn test_iri_namespaces() {
        let owl_iri = IRI::new("http://www.w3.org/2002/07/owl#Class").unwrap();
        assert!(owl_iri.is_owl());
        assert!(!owl_iri.is_rdf());
        
        let rdf_iri = IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap();
        assert!(rdf_iri.is_rdf());
        assert!(!rdf_iri.is_owl());
    }
    
    #[test]
    fn test_iri_registry() {
        let mut registry = IRIRegistry::new();
        registry.register("ex", "http://example.org/").unwrap();
        
        let iri = registry.iri_with_prefix("ex", "Person").unwrap();
        assert_eq!(iri.as_str(), "http://example.org/Person");
        assert_eq!(iri.prefix(), Some("ex"));
    }
}