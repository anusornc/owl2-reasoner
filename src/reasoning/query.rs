//! Query answering for OWL2 ontologies
//! 
//! Provides SPARQL-like query capabilities for OWL2 ontologies with reasoning support.

use crate::ontology::Ontology;
use crate::iri::IRI;
use crate::entities::*;
use crate::axioms::*;
use crate::reasoning::Reasoner;
use crate::error::OwlResult;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Query engine for OWL2 ontologies
pub struct QueryEngine {
    ontology: Arc<Ontology>,
    reasoner: Option<Box<dyn Reasoner>>,
    config: QueryConfig,
}

/// Query configuration
#[derive(Debug, Clone)]
pub struct QueryConfig {
    /// Enable reasoning during query answering
    pub enable_reasoning: bool,
    /// Maximum number of results
    pub max_results: Option<usize>,
    /// Timeout in milliseconds
    pub timeout: Option<u64>,
}

impl Default for QueryConfig {
    fn default() -> Self {
        QueryConfig {
            enable_reasoning: true,
            max_results: None,
            timeout: Some(10000), // 10 seconds default
        }
    }
}

/// Query result
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub bindings: Vec<QueryBinding>,
    pub variables: Vec<String>,
    pub stats: QueryStats,
}

/// Query binding (variable to value mapping)
#[derive(Debug, Clone)]
pub struct QueryBinding {
    pub variables: HashMap<String, QueryValue>,
}

/// Query value
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QueryValue {
    IRI(IRI),
    Literal(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

/// Query statistics
#[derive(Debug, Clone)]
pub struct QueryStats {
    pub results_count: usize,
    pub time_ms: u64,
    pub reasoning_used: bool,
}

/// Query pattern
#[derive(Debug, Clone)]
pub enum QueryPattern {
    /// Basic graph pattern
    BasicGraphPattern(Vec<TriplePattern>),
    /// Optional pattern
    OptionalPattern(Box<QueryPattern>),
    /// Union pattern
    UnionPattern(Vec<QueryPattern>),
    /// Filter pattern
    FilterPattern {
        pattern: Box<QueryPattern>,
        expression: FilterExpression,
    },
}

/// Triple pattern for SPARQL-like queries
#[derive(Debug, Clone)]
pub struct TriplePattern {
    pub subject: PatternTerm,
    pub predicate: PatternTerm,
    pub object: PatternTerm,
}

/// Pattern term (can be variable or constant)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternTerm {
    Variable(String),
    IRI(IRI),
    Literal(String),
}

/// Filter expression
#[derive(Debug, Clone)]
pub enum FilterExpression {
    /// Equality comparison
    Equals {
        left: PatternTerm,
        right: PatternTerm,
    },
    /// Type check
    Type {
        term: PatternTerm,
        type_iri: IRI,
    },
    /// Logical AND
    And(Vec<FilterExpression>),
    /// Logical OR
    Or(Vec<FilterExpression>),
    /// Logical NOT
    Not(Box<FilterExpression>),
}

impl QueryEngine {
    /// Create a new query engine
    pub fn new(ontology: Ontology) -> Self {
        Self::with_config(ontology, QueryConfig::default())
    }
    
    /// Create a new query engine with custom configuration
    pub fn with_config(ontology: Ontology, config: QueryConfig) -> Self {
        let ontology = Arc::new(ontology);
        let reasoner = if config.enable_reasoning {
            // This would be initialized with a proper reasoner
            None
        } else {
            None
        };
        
        QueryEngine {
            ontology,
            reasoner,
            config,
        }
    }
    
    /// Execute a query
    pub fn execute_query(&mut self, pattern: &QueryPattern) -> OwlResult<QueryResult> {
        let start_time = std::time::Instant::now();
        let mut bindings = Vec::new();
        
        match pattern {
            QueryPattern::BasicGraphPattern(triples) => {
                bindings = self.evaluate_basic_graph_pattern(triples)?;
            }
            QueryPattern::OptionalPattern(pattern) => {
                bindings = self.evaluate_optional_pattern(pattern.as_ref())?;
            }
            QueryPattern::UnionPattern(patterns) => {
                bindings = self.evaluate_union_pattern(patterns)?;
            }
            QueryPattern::FilterPattern { pattern, expression } => {
                let mut result_bindings = self.evaluate_basic_graph_pattern(
                    if let QueryPattern::BasicGraphPattern(triples) = pattern.as_ref() {
                        triples
                    } else {
                        return Err(crate::error::OwlError::QueryError(
                            "Filter pattern can only be applied to basic graph patterns".to_string()
                        ));
                    }
                )?;
                
                bindings = self.apply_filter(&result_bindings, expression)?;
            }
        }
        
        // Apply result limit
        if let Some(max_results) = self.config.max_results {
            bindings.truncate(max_results);
        }
        
        let variables = self.extract_variables(pattern);
        let time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(QueryResult {
            bindings: result_bindings,
            variables,
            stats: QueryStats {
                results_count: result_bindings.len(),
                time_ms,
                reasoning_used: self.config.enable_reasoning,
            },
        })
    }
    
    /// Evaluate a basic graph pattern
    fn evaluate_basic_graph_pattern(&self, triples: &[TriplePattern]) -> OwlResult<Vec<QueryBinding>> {
        let mut bindings = Vec::new();
        
        if triples.is_empty() {
            return Ok(bindings);
        }
        
        // Start with the first triple pattern
        let first_bindings = self.match_triple_pattern(&triples[0])?;
        bindings = first_bindings;
        
        // Join with remaining triple patterns
        for (_i, triple) in triples.iter().enumerate().skip(1) {
            let triple_bindings = self.match_triple_pattern(triple)?;
            let mut new_bindings = Vec::new();
            
            for existing_binding in &bindings {
                for triple_binding in &triple_bindings {
                    if let Some(joined_binding) = self.join_bindings(existing_binding, triple_binding) {
                        new_bindings.push(joined_binding);
                    }
                }
            }
            
            bindings = new_bindings;
            
            if bindings.is_empty() {
                break; // No more matches possible
            }
        }
        
        Ok(bindings)
    }
    
    /// Match a single triple pattern against the ontology
    fn match_triple_pattern(&self, triple: &TriplePattern) -> OwlResult<Vec<QueryBinding>> {
        let mut bindings = Vec::new();
        
        // Match against class assertions
        for axiom in self.ontology.class_assertions() {
            if let Some(binding) = self.match_class_assertion(triple, axiom) {
                bindings.push(binding);
            }
        }
        
        // Match against property assertions
        for axiom in self.ontology.property_assertions() {
            if let Some(binding) = self.match_property_assertion(triple, axiom) {
                bindings.push(binding);
            }
        }
        
        // Match against subclass axioms
        for axiom in self.ontology.subclass_axioms() {
            if let Some(binding) = self.match_subclass_axiom(triple, axiom) {
                bindings.push(binding);
            }
        }
        
        // Add more pattern matching as needed
        Ok(bindings)
    }
    
    /// Match triple pattern against class assertion
    fn match_class_assertion(&self, triple: &TriplePattern, axiom: &crate::axioms::ClassAssertion) -> Option<QueryBinding> {
        // Try to match: individual rdf:type class
        let type_iri = IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap();
        
        let subject_match = self.match_term(&triple.subject, &PatternTerm::IRI(axiom.individual().clone()));
        let predicate_match = self.match_term(&triple.predicate, &PatternTerm::IRI(type_iri));
        let object_match = self.match_class_expr_term(&triple.object, axiom.class_expr());
        
        if subject_match && predicate_match && object_match {
            let mut binding = QueryBinding {
                variables: HashMap::new(),
            };
            
            self.add_binding(&mut binding, &triple.subject, &PatternTerm::IRI(axiom.individual().clone()));
            self.add_class_expr_binding(&mut binding, &triple.object, axiom.class_expr());
            
            Some(binding)
        } else {
            None
        }
    }
    
    /// Match triple pattern against property assertion
    fn match_property_assertion(&self, triple: &TriplePattern, axiom: &crate::axioms::PropertyAssertion) -> Option<QueryBinding> {
        let subject_match = self.match_term(&triple.subject, &PatternTerm::IRI(axiom.subject().clone()));
        let predicate_match = self.match_term(&triple.predicate, &PatternTerm::IRI(axiom.property().clone()));
        let object_match = self.match_term(&triple.object, &PatternTerm::IRI(axiom.object().clone()));
        
        if subject_match && predicate_match && object_match {
            let mut binding = QueryBinding {
                variables: HashMap::new(),
            };
            
            self.add_binding(&mut binding, &triple.subject, &PatternTerm::IRI(axiom.subject().clone()));
            self.add_binding(&mut binding, &triple.predicate, &PatternTerm::IRI(axiom.property().clone()));
            self.add_binding(&mut binding, &triple.object, &PatternTerm::IRI(axiom.object().clone()));
            
            Some(binding)
        } else {
            None
        }
    }
    
    /// Match triple pattern against subclass axiom
    fn match_subclass_axiom(&self, triple: &TriplePattern, axiom: &crate::axioms::SubClassOfAxiom) -> Option<QueryBinding> {
        let sub_iri = if let ClassExpression::Class(iri) = axiom.sub_class() {
            iri
        } else {
            return None;
        };
        
        let super_iri = if let ClassExpression::Class(iri) = axiom.super_class() {
            iri
        } else {
            return None;
        };
        
        let rdfs_subclassof = IRI::new("http://www.w3.org/2000/01/rdf-schema#subClassOf").unwrap();
        
        let subject_match = self.match_term(&triple.subject, &PatternTerm::IRI(sub_iri.clone()));
        let predicate_match = self.match_term(&triple.predicate, &PatternTerm::IRI(rdfs_subclassof));
        let object_match = self.match_term(&triple.object, &PatternTerm::IRI(super_iri.clone()));
        
        if subject_match && predicate_match && object_match {
            let mut binding = QueryBinding {
                variables: HashMap::new(),
            };
            
            self.add_binding(&mut binding, &triple.subject, &PatternTerm::IRI(sub_iri.clone()));
            self.add_binding(&mut binding, &triple.predicate, &PatternTerm::IRI(rdfs_subclassof));
            self.add_binding(&mut binding, &triple.object, &PatternTerm::IRI(super_iri.clone()));
            
            Some(binding)
        } else {
            None
        }
    }
    
    /// Match two pattern terms
    fn match_term(&self, pattern: &PatternTerm, value: &PatternTerm) -> bool {
        match (pattern, value) {
            (PatternTerm::Variable(_), _) => true,
            (PatternTerm::IRI(pattern_iri), PatternTerm::IRI(value_iri)) => pattern_iri == value_iri,
            (PatternTerm::Literal(pattern_lit), PatternTerm::Literal(value_lit)) => pattern_lit == value_lit,
            _ => false,
        }
    }
    
    /// Match pattern term against class expression
    fn match_class_expr_term(&self, pattern: &PatternTerm, class_expr: &ClassExpression) -> bool {
        match pattern {
            PatternTerm::Variable(_) => true,
            PatternTerm::IRI(iri) => class_expr.contains_class(iri),
            _ => false,
        }
    }
    
    /// Add binding from pattern term to value
    fn add_binding(&self, binding: &mut QueryBinding, pattern: &PatternTerm, value: &PatternTerm) {
        if let PatternTerm::Variable(var_name) = pattern {
            let query_value = match value {
                PatternTerm::IRI(iri) => QueryValue::IRI(iri.clone()),
                PatternTerm::Literal(lit) => QueryValue::Literal(lit.clone()),
                PatternTerm::Variable(_) => return, // Can't bind variable to variable
            };
            
            binding.variables.insert(var_name.clone(), query_value);
        }
    }
    
    /// Add binding from pattern term to class expression
    fn add_class_expr_binding(&self, binding: &mut QueryBinding, pattern: &PatternTerm, class_expr: &ClassExpression) {
        if let PatternTerm::Variable(var_name) = pattern {
            if let ClassExpression::Class(iri) = class_expr {
                binding.variables.insert(var_name.clone(), QueryValue::IRI(iri.clone()));
            }
        }
    }
    
    /// Join two bindings
    fn join_bindings(&self, binding1: &QueryBinding, binding2: &QueryBinding) -> Option<QueryBinding> {
        let mut joined = binding1.clone();
        
        for (var, value) in &binding2.variables {
            if let Some(existing_value) = joined.variables.get(var) {
                if existing_value != value {
                    return None; // Variable conflict
                }
            } else {
                joined.variables.insert(var.clone(), value.clone());
            }
        }
        
        Some(joined)
    }
    
    /// Evaluate optional pattern
    fn evaluate_optional_pattern(&mut self, pattern: &QueryPattern) -> OwlResult<Vec<QueryBinding>> {
        // For optional patterns, we need to handle cases where the pattern may not match
        // This is a simplified implementation
        self.execute_query(pattern).map(|result| result.bindings)
    }
    
    /// Evaluate union pattern
    fn evaluate_union_pattern(&mut self, patterns: &[QueryPattern]) -> OwlResult<Vec<QueryBinding>> {
        let mut all_bindings = Vec::new();
        
        for pattern in patterns {
            let pattern_bindings = self.execute_query(pattern)?;
            all_bindings.extend(pattern_bindings.bindings);
        }
        
        Ok(all_bindings)
    }
    
    /// Apply filter expression to bindings
    fn apply_filter(&self, bindings: &[QueryBinding], expression: &FilterExpression) -> OwlResult<Vec<QueryBinding>> {
        let mut filtered_bindings = Vec::new();
        
        for binding in bindings {
            if self.evaluate_filter_expression(binding, expression) {
                filtered_bindings.push(binding.clone());
            }
        }
        
        Ok(filtered_bindings)
    }
    
    /// Evaluate filter expression for a binding
    fn evaluate_filter_expression(&self, binding: &QueryBinding, expression: &FilterExpression) -> bool {
        match expression {
            FilterExpression::Equals { left, right } => {
                let left_value = self.evaluate_term(binding, left);
                let right_value = self.evaluate_term(binding, right);
                left_value == right_value
            }
            FilterExpression::Type { term, type_iri } => {
                if let Some(value) = self.evaluate_term_opt(binding, term) {
                    match value {
                        QueryValue::IRI(iri) => {
                            // Check if the IRI has the specified type
                            // This is simplified - in practice, we'd need to reason about types
                            false // Placeholder implementation
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }
            FilterExpression::And(expressions) => {
                expressions.iter().all(|expr| self.evaluate_filter_expression(binding, expr))
            }
            FilterExpression::Or(expressions) => {
                expressions.iter().any(|expr| self.evaluate_filter_expression(binding, expr))
            }
            FilterExpression::Not(expr) => {
                !self.evaluate_filter_expression(binding, expr)
            }
        }
    }
    
    /// Evaluate pattern term to query value
    fn evaluate_term(&self, binding: &QueryBinding, term: &PatternTerm) -> QueryValue {
        self.evaluate_term_opt(binding, term).unwrap_or(QueryValue::Literal("".to_string()))
    }
    
    /// Evaluate pattern term to query value (optional)
    fn evaluate_term_opt(&self, binding: &QueryBinding, term: &PatternTerm) -> Option<QueryValue> {
        match term {
            PatternTerm::Variable(var_name) => binding.variables.get(var_name).cloned(),
            PatternTerm::IRI(iri) => Some(QueryValue::IRI(iri.clone())),
            PatternTerm::Literal(lit) => Some(QueryValue::Literal(lit.clone())),
        }
    }
    
    /// Extract variables from query pattern
    fn extract_variables(&self, pattern: &QueryPattern) -> Vec<String> {
        let mut variables = HashSet::new();
        
        match pattern {
            QueryPattern::BasicGraphPattern(triples) => {
                for triple in triples {
                    self.extract_variables_from_term(&triple.subject, &mut variables);
                    self.extract_variables_from_term(&triple.predicate, &mut variables);
                    self.extract_variables_from_term(&triple.object, &mut variables);
                }
            }
            QueryPattern::OptionalPattern(pattern) => {
                variables.extend(self.extract_variables(pattern.as_ref()));
            }
            QueryPattern::UnionPattern(patterns) => {
                for pattern in patterns {
                    variables.extend(self.extract_variables(pattern));
                }
            }
            QueryPattern::FilterPattern { pattern, expression } => {
                variables.extend(self.extract_variables(pattern.as_ref()));
                self.extract_variables_from_expression(expression, &mut variables);
            }
        }
        
        let mut sorted_vars: Vec<_> = variables.into_iter().collect();
        sorted_vars.sort();
        sorted_vars
    }
    
    /// Extract variables from pattern term
    fn extract_variables_from_term(&self, term: &PatternTerm, variables: &mut HashSet<String>) {
        if let PatternTerm::Variable(var_name) = term {
            variables.insert(var_name.clone());
        }
    }
    
    /// Extract variables from filter expression
    fn extract_variables_from_expression(&self, expression: &FilterExpression, variables: &mut HashSet<String>) {
        match expression {
            FilterExpression::Equals { left, right } => {
                self.extract_variables_from_term(left, variables);
                self.extract_variables_from_term(right, variables);
            }
            FilterExpression::Type { term, .. } => {
                self.extract_variables_from_term(term, variables);
            }
            FilterExpression::And(expressions) | FilterExpression::Or(expressions) => {
                for expr in expressions {
                    self.extract_variables_from_expression(expr, variables);
                }
            }
            FilterExpression::Not(expr) => {
                self.extract_variables_from_expression(expr, variables);
            }
        }
    }
    
    /// Get all classes in the ontology
    pub fn get_all_classes(&self) -> Vec<IRI> {
        self.ontology.classes().iter().map(|c| c.iri().clone()).collect()
    }
    
    /// Get all properties in the ontology
    pub fn get_all_properties(&self) -> Vec<IRI> {
        let mut properties = Vec::new();
        
        for prop in self.ontology.object_properties() {
            properties.push(prop.iri().clone());
        }
        
        for prop in self.ontology.data_properties() {
            properties.push(prop.iri().clone());
        }
        
        properties
    }
    
    /// Get all individuals in the ontology
    pub fn get_all_individuals(&self) -> Vec<IRI> {
        self.ontology.named_individuals().iter().map(|i| i.iri().clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::Ontology;
    
    #[test]
    fn test_query_engine_creation() {
        let ontology = Ontology::new();
        let engine = QueryEngine::new(ontology);
        
        assert!(engine.get_all_classes().is_empty());
        assert!(engine.get_all_properties().is_empty());
        assert!(engine.get_all_individuals().is_empty());
    }
    
    #[test]
    fn test_simple_query() {
        let mut ontology = Ontology::new();
        
        // Add test data
        let person_iri = IRI::new("http://example.org/Person").unwrap();
        let john_iri = IRI::new("http://example.org/john").unwrap();
        
        let person_class = Class::new(person_iri.clone());
        let john_individual = NamedIndividual::new(john_iri.clone());
        
        ontology.add_class(person_class).unwrap();
        ontology.add_named_individual(john_individual).unwrap();
        
        // Add class assertion
        let class_assertion = ClassAssertion::new(
            john_iri.clone(),
            ClassExpression::Class(person_iri.clone()),
        );
        ontology.add_class_assertion(class_assertion).unwrap();
        
        let mut engine = QueryEngine::new(ontology);
        
        // Create query: ?x rdf:type Person
        let type_iri = IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap();
        let pattern = QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("?x".to_string()),
            predicate: PatternTerm::IRI(type_iri),
            object: PatternTerm::IRI(person_iri),
        }]);
        
        let result = engine.execute_query(&pattern).unwrap();
        
        assert_eq!(result.bindings.len(), 1);
        assert_eq!(result.variables, vec!["?x"]);
        
        let binding = &result.bindings[0];
        assert_eq!(binding.variables.get("?x"), Some(&QueryValue::IRI(john_iri)));
    }
    
    #[test]
    fn test_filter_expression() {
        let ontology = Ontology::new();
        let engine = QueryEngine::new(ontology);
        
        let binding = QueryBinding {
            variables: {
                let mut vars = HashMap::new();
                vars.insert("?x".to_string(), QueryValue::IRI(
                    IRI::new("http://example.org/test").unwrap()
                ));
                vars
            },
        };
        
        let expression = FilterExpression::Equals {
            left: PatternTerm::Variable("?x".to_string()),
            right: PatternTerm::IRI(IRI::new("http://example.org/test").unwrap()),
        };
        
        assert!(engine.evaluate_filter_expression(&binding, &expression));
    }
}