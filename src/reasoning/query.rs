//! Query answering for OWL2 ontologies
//!
//! Provides SPARQL-like query capabilities for OWL2 ontologies with reasoning support.

use crate::axioms::*;
use crate::error::{OwlError, OwlResult};
use crate::iri::IRI;
use crate::ontology::Ontology;
use crate::reasoning::Reasoner;

use hashbrown::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

/// Query engine for OWL2 ontologies
pub struct QueryEngine {
    ontology: Arc<Ontology>,
    #[allow(dead_code)]
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
#[derive(Debug, Clone, PartialEq)]
pub enum QueryValue {
    IRI(IRI),
    Literal(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

impl Eq for QueryValue {}

impl std::hash::Hash for QueryValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            QueryValue::IRI(iri) => {
                state.write_u8(0);
                iri.hash(state);
            }
            QueryValue::Literal(lit) => {
                state.write_u8(1);
                lit.hash(state);
            }
            QueryValue::Boolean(b) => {
                state.write_u8(2);
                b.hash(state);
            }
            QueryValue::Integer(i) => {
                state.write_u8(3);
                i.hash(state);
            }
            QueryValue::Float(f) => {
                state.write_u8(4);
                // Convert to bits for hashing since f64 doesn't implement Hash
                f.to_bits().hash(state);
            }
        }
    }
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
    Type { term: PatternTerm, type_iri: IRI },
    /// Logical AND
    And(Vec<FilterExpression>),
    /// Logical OR
    Or(Vec<FilterExpression>),
    /// Logical NOT
    Not(Box<FilterExpression>),
}

/// RDF vocabulary constants
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// Types of triple pattern queries
#[derive(Debug, Clone, PartialEq)]
enum QueryType {
    TypeQuery,
    PropertyQuery,
    VariablePredicate,
}

impl QueryEngine {
    /// Determine the type of query based on the triple pattern
    fn determine_query_type(&self, triple: &TriplePattern) -> QueryType {
        match &triple.predicate {
            PatternTerm::IRI(pred_iri) => {
                if pred_iri.as_str() == RDF_TYPE {
                    QueryType::TypeQuery
                } else {
                    QueryType::PropertyQuery
                }
            }
            _ => QueryType::VariablePredicate,
        }
    }

    /// Collect bindings for type queries (rdf:type)
    fn collect_type_query_bindings(
        &self,
        triple: &TriplePattern,
        bindings: &mut Vec<QueryBinding>,
    ) {
        for axiom in self.ontology.class_assertions() {
            if let Some(binding) = self.match_class_assertion_optimized(triple, axiom) {
                bindings.push(binding);
            }
        }
    }

    /// Collect bindings for property queries
    fn collect_property_query_bindings(
        &self,
        triple: &TriplePattern,
        bindings: &mut Vec<QueryBinding>,
    ) {
        for axiom in self.ontology.property_assertions() {
            if let Some(binding) = self.match_property_assertion_optimized(triple, axiom) {
                bindings.push(binding);
            }
        }
    }

    /// Collect bindings for variable predicate queries
    fn collect_variable_predicate_bindings(
        &self,
        triple: &TriplePattern,
        bindings: &mut Vec<QueryBinding>,
    ) {
        for axiom in self.ontology.class_assertions() {
            if let Some(binding) = self.match_class_assertion_optimized(triple, axiom) {
                bindings.push(binding);
            }
        }

        for axiom in self.ontology.property_assertions() {
            if let Some(binding) = self.match_property_assertion_optimized(triple, axiom) {
                bindings.push(binding);
            }
        }
    }
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
        let mut bindings;

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
            QueryPattern::FilterPattern {
                pattern,
                expression,
            } => {
                let result_bindings = self.evaluate_basic_graph_pattern(
                    if let QueryPattern::BasicGraphPattern(triples) = pattern.as_ref() {
                        triples
                    } else {
                        return Err(OwlError::QueryError(
                            "Filter pattern can only be applied to basic graph patterns"
                                .to_string(),
                        ));
                    },
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

        let results_count = bindings.len();
        Ok(QueryResult {
            bindings,
            variables,
            stats: QueryStats {
                results_count,
                time_ms,
                reasoning_used: self.config.enable_reasoning,
            },
        })
    }

    /// Evaluate a basic graph pattern using hash joins for optimization
    fn evaluate_basic_graph_pattern(
        &self,
        triples: &[TriplePattern],
    ) -> OwlResult<Vec<QueryBinding>> {
        let mut bindings = Vec::new();

        if triples.is_empty() {
            return Ok(bindings);
        }

        // Start with the first triple pattern
        let first_bindings = self.match_triple_pattern_optimized(&triples[0])?;
        bindings = first_bindings;

        // Join with remaining triple patterns using hash joins
        for triple in triples.iter().skip(1) {
            let triple_bindings = self.match_triple_pattern_optimized(triple)?;
            bindings = self.hash_join_bindings(&bindings, &triple_bindings)?;

            if bindings.is_empty() {
                break; // No more matches possible
            }
        }

        Ok(bindings)
    }

    /// Match a single triple pattern against the ontology using indexed storage
    fn match_triple_pattern_optimized(
        &self,
        triple: &TriplePattern,
    ) -> OwlResult<Vec<QueryBinding>> {
        let mut bindings = Vec::new();

        match self.determine_query_type(triple) {
            QueryType::TypeQuery => {
                self.collect_type_query_bindings(triple, &mut bindings);
            }
            QueryType::PropertyQuery => {
                self.collect_property_query_bindings(triple, &mut bindings);
            }
            QueryType::VariablePredicate => {
                self.collect_variable_predicate_bindings(triple, &mut bindings);
            }
        }

        Ok(bindings)
    }

    /// Match triple pattern against class assertion (optimized)
    fn match_class_assertion_optimized(
        &self,
        triple: &TriplePattern,
        axiom: &crate::axioms::ClassAssertionAxiom,
    ) -> Option<QueryBinding> {
        let type_iri = IRI::new(RDF_TYPE)
            .map_err(|e| OwlError::IriParseError {
                iri: RDF_TYPE.to_string(),
                context: format!("Failed to create rdf:type IRI: {}", e),
            })
            .expect("Failed to create rdf:type IRI");

        let individual_iri = axiom.individual();
        let individual_term = PatternTerm::IRI(individual_iri.clone());

        if self.is_class_assertion_match(triple, &individual_term, &type_iri, axiom.class_expr()) {
            Some(self.create_class_assertion_binding(triple, &individual_term, axiom.class_expr()))
        } else {
            None
        }
    }

    /// Check if a class assertion matches the triple pattern
    fn is_class_assertion_match(
        &self,
        triple: &TriplePattern,
        individual_term: &PatternTerm,
        type_iri: &IRI,
        class_expr: &crate::axioms::ClassExpression,
    ) -> bool {
        let subject_match = self.match_term(&triple.subject, individual_term);
        let predicate_match =
            self.match_term(&triple.predicate, &PatternTerm::IRI(type_iri.clone()));
        let object_match = self.match_class_expr_term(&triple.object, class_expr);

        subject_match && predicate_match && object_match
    }

    /// Create a binding for a class assertion match
    fn create_class_assertion_binding(
        &self,
        triple: &TriplePattern,
        individual_term: &PatternTerm,
        class_expr: &crate::axioms::ClassExpression,
    ) -> QueryBinding {
        let mut binding = QueryBinding {
            variables: HashMap::new(),
        };

        self.add_binding(&mut binding, &triple.subject, individual_term);
        self.add_class_expr_binding(&mut binding, &triple.object, class_expr);

        binding
    }

    /// Match triple pattern against property assertion (optimized)
    fn match_property_assertion_optimized(
        &self,
        triple: &TriplePattern,
        axiom: &crate::axioms::PropertyAssertionAxiom,
    ) -> Option<QueryBinding> {
        let subject_iri = axiom.subject();
        let property_iri = axiom.property();

        let subject_term = PatternTerm::IRI(subject_iri.clone());
        let property_term = PatternTerm::IRI(property_iri.clone());

        if self.is_property_assertion_match(triple, &subject_term, &property_term, axiom) {
            Some(self.create_property_assertion_binding(
                triple,
                &subject_term,
                &property_term,
                axiom,
            ))
        } else {
            None
        }
    }

    /// Check if a property assertion matches the triple pattern
    fn is_property_assertion_match(
        &self,
        triple: &TriplePattern,
        subject_term: &PatternTerm,
        property_term: &PatternTerm,
        axiom: &crate::axioms::PropertyAssertionAxiom,
    ) -> bool {
        let subject_match = self.match_term(&triple.subject, subject_term);
        let predicate_match = self.match_term(&triple.predicate, property_term);
        let object_match = self.match_property_object(&triple.object, axiom);

        subject_match && predicate_match && object_match
    }

    /// Match property object term
    fn match_property_object(
        &self,
        object_term: &PatternTerm,
        axiom: &crate::axioms::PropertyAssertionAxiom,
    ) -> bool {
        if let Some(object_iri) = axiom.object_iri() {
            self.match_term(object_term, &PatternTerm::IRI(object_iri.clone()))
        } else {
            // Skip anonymous individuals in query matching for now
            false
        }
    }

    /// Create a binding for a property assertion match
    fn create_property_assertion_binding(
        &self,
        triple: &TriplePattern,
        subject_term: &PatternTerm,
        property_term: &PatternTerm,
        axiom: &crate::axioms::PropertyAssertionAxiom,
    ) -> QueryBinding {
        let mut binding = QueryBinding {
            variables: HashMap::new(),
        };

        self.add_binding(&mut binding, &triple.subject, subject_term);
        self.add_binding(&mut binding, &triple.predicate, property_term);

        if let Some(object_iri) = axiom.object_iri() {
            self.add_binding(
                &mut binding,
                &triple.object,
                &PatternTerm::IRI(object_iri.clone()),
            );
        }

        binding
    }

    /// Perform hash join between two sets of bindings
    fn hash_join_bindings(
        &self,
        left_bindings: &[QueryBinding],
        right_bindings: &[QueryBinding],
    ) -> OwlResult<Vec<QueryBinding>> {
        if left_bindings.is_empty() || right_bindings.is_empty() {
            return Ok(Vec::new());
        }

        // Find common variables between left and right bindings
        let left_vars: HashSet<String> = left_bindings
            .first()
            .map(|b| b.variables.keys().cloned().collect())
            .unwrap_or_default();
        let right_vars: HashSet<String> = right_bindings
            .first()
            .map(|b| b.variables.keys().cloned().collect())
            .unwrap_or_default();

        let common_vars: Vec<String> = left_vars.intersection(&right_vars).cloned().collect();

        if common_vars.is_empty() {
            // No common variables - return cartesian product
            let mut result = Vec::new();
            for left in left_bindings {
                for right in right_bindings {
                    let mut combined = left.clone();
                    combined.variables.extend(right.variables.clone());
                    result.push(combined);
                }
            }
            return Ok(result);
        }

        // Use hash join for common variables
        let mut hash_table: HashMap<Vec<QueryValue>, Vec<&QueryBinding>> = HashMap::new();

        // Build hash table from right bindings
        for right_binding in right_bindings {
            let key: Vec<QueryValue> = common_vars
                .iter()
                .map(|var| right_binding.variables.get(var).cloned().unwrap())
                .collect();

            hash_table.entry(key).or_default().push(right_binding);
        }

        // Probe with left bindings
        let mut result = Vec::new();
        for left_binding in left_bindings {
            let key: Vec<QueryValue> = common_vars
                .iter()
                .map(|var| left_binding.variables.get(var).cloned().unwrap())
                .collect();

            if let Some(matching_rights) = hash_table.get(&key) {
                for right_binding in matching_rights {
                    let mut combined = left_binding.clone();
                    combined.variables.extend(right_binding.variables.clone());
                    result.push(combined);
                }
            }
        }

        Ok(result)
    }

    /// Match triple pattern against class assertion
    #[allow(dead_code)]
    fn match_class_assertion(
        &self,
        triple: &TriplePattern,
        axiom: &crate::axioms::ClassAssertionAxiom,
    ) -> Option<QueryBinding> {
        // Try to match: individual rdf:type class
        let type_iri = IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
            .map_err(|e| OwlError::IriParseError {
                iri: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
                context: format!("Failed to create rdf:type IRI: {}", e),
            })
            .expect("Failed to create rdf:type IRI");

        let subject_match = self.match_term(
            &triple.subject,
            &PatternTerm::IRI(axiom.individual().clone()),
        );
        let predicate_match = self.match_term(&triple.predicate, &PatternTerm::IRI(type_iri));
        let object_match = self.match_class_expr_term(&triple.object, axiom.class_expr());

        if subject_match && predicate_match && object_match {
            let mut binding = QueryBinding {
                variables: HashMap::new(),
            };

            self.add_binding(
                &mut binding,
                &triple.subject,
                &PatternTerm::IRI(axiom.individual().clone()),
            );
            self.add_class_expr_binding(&mut binding, &triple.object, axiom.class_expr());

            Some(binding)
        } else {
            None
        }
    }

    /// Match triple pattern against property assertion
    #[allow(dead_code)]
    fn match_property_assertion(
        &self,
        triple: &TriplePattern,
        axiom: &crate::axioms::PropertyAssertionAxiom,
    ) -> Option<QueryBinding> {
        let subject_match =
            self.match_term(&triple.subject, &PatternTerm::IRI(axiom.subject().clone()));
        let predicate_match = self.match_term(
            &triple.predicate,
            &PatternTerm::IRI(axiom.property().clone()),
        );
        let object_match = if let Some(object_iri) = axiom.object_iri() {
            self.match_term(&triple.object, &PatternTerm::IRI(object_iri.clone()))
        } else {
            // Skip anonymous individuals in query matching for now
            false
        };

        if subject_match && predicate_match && object_match {
            let mut binding = QueryBinding {
                variables: HashMap::new(),
            };

            self.add_binding(
                &mut binding,
                &triple.subject,
                &PatternTerm::IRI(axiom.subject().clone()),
            );
            self.add_binding(
                &mut binding,
                &triple.predicate,
                &PatternTerm::IRI(axiom.property().clone()),
            );
            if let Some(object_iri) = axiom.object_iri() {
                self.add_binding(
                    &mut binding,
                    &triple.object,
                    &PatternTerm::IRI(object_iri.clone()),
                );
            }

            Some(binding)
        } else {
            None
        }
    }

    /// Match triple pattern against subclass axiom
    #[allow(dead_code)]
    fn match_subclass_axiom(
        &self,
        triple: &TriplePattern,
        axiom: &crate::axioms::SubClassOfAxiom,
    ) -> Option<QueryBinding> {
        let sub_iri = if let ClassExpression::Class(class) = axiom.sub_class() {
            class.iri()
        } else {
            return None;
        };

        let super_iri = if let ClassExpression::Class(class) = axiom.super_class() {
            class.iri()
        } else {
            return None;
        };

        let rdfs_subclassof = IRI::new("http://www.w3.org/2000/01/rdf-schema#subClassOf")
            .map_err(|e| OwlError::IriParseError {
                iri: "http://www.w3.org/2000/01/rdf-schema#subClassOf".to_string(),
                context: format!("Failed to create rdfs:subClassOf IRI: {}", e),
            })
            .ok()?;

        // Clone once and reuse terms
        let sub_term = PatternTerm::IRI(sub_iri.clone());
        let super_term = PatternTerm::IRI(super_iri.clone());
        let subclass_term = PatternTerm::IRI(rdfs_subclassof.clone());

        let subject_match = self.match_term(&triple.subject, &sub_term);
        let predicate_match = self.match_term(&triple.predicate, &subclass_term);
        let object_match = self.match_term(&triple.object, &super_term);

        if subject_match && predicate_match && object_match {
            let mut binding = QueryBinding {
                variables: HashMap::new(),
            };

            self.add_binding(&mut binding, &triple.subject, &sub_term);
            self.add_binding(&mut binding, &triple.predicate, &subclass_term);
            self.add_binding(&mut binding, &triple.object, &super_term);

            Some(binding)
        } else {
            None
        }
    }

    /// Match two pattern terms
    fn match_term(&self, pattern: &PatternTerm, value: &PatternTerm) -> bool {
        match (pattern, value) {
            (PatternTerm::Variable(_), _) => true,
            (PatternTerm::IRI(pattern_iri), PatternTerm::IRI(value_iri)) => {
                pattern_iri == value_iri
            }
            (PatternTerm::Literal(pattern_lit), PatternTerm::Literal(value_lit)) => {
                pattern_lit == value_lit
            }
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
    fn add_class_expr_binding(
        &self,
        binding: &mut QueryBinding,
        pattern: &PatternTerm,
        class_expr: &ClassExpression,
    ) {
        if let PatternTerm::Variable(var_name) = pattern {
            if let ClassExpression::Class(class) = class_expr {
                binding
                    .variables
                    .insert(var_name.clone(), QueryValue::IRI(class.iri().clone()));
            }
        }
    }

    /// Join two bindings
    #[allow(dead_code)]
    fn join_bindings(
        &self,
        binding1: &QueryBinding,
        binding2: &QueryBinding,
    ) -> Option<QueryBinding> {
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
    fn evaluate_optional_pattern(
        &mut self,
        pattern: &QueryPattern,
    ) -> OwlResult<Vec<QueryBinding>> {
        // For optional patterns, we need to handle cases where the pattern may not match
        // This is a simplified implementation
        self.execute_query(pattern).map(|result| result.bindings)
    }

    /// Evaluate union pattern
    fn evaluate_union_pattern(
        &mut self,
        patterns: &[QueryPattern],
    ) -> OwlResult<Vec<QueryBinding>> {
        let mut all_bindings = Vec::new();

        for pattern in patterns {
            let pattern_bindings = self.execute_query(pattern)?;
            all_bindings.extend(pattern_bindings.bindings);
        }

        Ok(all_bindings)
    }

    /// Apply filter expression to bindings
    fn apply_filter(
        &self,
        bindings: &[QueryBinding],
        expression: &FilterExpression,
    ) -> OwlResult<Vec<QueryBinding>> {
        let mut filtered_bindings = Vec::new();

        for binding in bindings {
            if self.evaluate_filter_expression(binding, expression) {
                filtered_bindings.push(binding.clone());
            }
        }

        Ok(filtered_bindings)
    }

    /// Evaluate filter expression for a binding
    fn evaluate_filter_expression(
        &self,
        binding: &QueryBinding,
        expression: &FilterExpression,
    ) -> bool {
        match expression {
            FilterExpression::Equals { left, right } => {
                let left_value = self.evaluate_term(binding, left);
                let right_value = self.evaluate_term(binding, right);
                left_value == right_value
            }
            FilterExpression::Type { term, type_iri: _ } => {
                if let Some(value) = self.evaluate_term_opt(binding, term) {
                    match value {
                        QueryValue::IRI(_iri) => {
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
            FilterExpression::And(expressions) => expressions
                .iter()
                .all(|expr| self.evaluate_filter_expression(binding, expr)),
            FilterExpression::Or(expressions) => expressions
                .iter()
                .any(|expr| self.evaluate_filter_expression(binding, expr)),
            FilterExpression::Not(expr) => !self.evaluate_filter_expression(binding, expr),
        }
    }

    /// Evaluate pattern term to query value
    fn evaluate_term(&self, binding: &QueryBinding, term: &PatternTerm) -> QueryValue {
        self.evaluate_term_opt(binding, term)
            .unwrap_or(QueryValue::Literal("".to_string()))
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
            QueryPattern::FilterPattern {
                pattern,
                expression,
            } => {
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
    fn extract_variables_from_expression(
        &self,
        expression: &FilterExpression,
        variables: &mut HashSet<String>,
    ) {
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
        self.ontology
            .classes()
            .iter()
            .map(|c| c.iri().clone())
            .collect()
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
        self.ontology
            .named_individuals()
            .iter()
            .map(|i| i.iri().clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::Ontology;
    use crate::Class;
    use crate::NamedIndividual;

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
        let person_iri =
            IRI::new("http://example.org/Person").expect("Failed to create Person IRI for testing");
        let john_iri =
            IRI::new("http://example.org/john").expect("Failed to create john IRI for testing");

        let person_class = Class::new(person_iri.clone());
        let john_individual = NamedIndividual::new(john_iri.clone());

        ontology
            .add_class(person_class.clone())
            .expect("Failed to add Person class");
        ontology
            .add_named_individual(john_individual)
            .expect("Failed to add john individual");

        // Add class assertion
        let class_assertion =
            ClassAssertionAxiom::new(john_iri.clone(), ClassExpression::Class(person_class));
        ontology
            .add_class_assertion(class_assertion)
            .expect("Failed to add class assertion");

        let mut engine = QueryEngine::new(ontology);

        // Create query: ?x rdf:type Person
        let type_iri = IRI::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
            .map_err(|e| OwlError::IriParseError {
                iri: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
                context: format!("Failed to create rdf:type IRI: {}", e),
            })
            .expect("Failed to create rdf:type IRI");
        let pattern = QueryPattern::BasicGraphPattern(vec![TriplePattern {
            subject: PatternTerm::Variable("?x".to_string()),
            predicate: PatternTerm::IRI(type_iri),
            object: PatternTerm::IRI(person_iri),
        }]);

        let result = engine
            .execute_query(&pattern)
            .expect("Failed to execute query");

        assert_eq!(result.bindings.len(), 1);
        assert_eq!(result.variables, vec!["?x"]);

        let binding = &result.bindings[0];
        assert_eq!(
            binding.variables.get("?x"),
            Some(&QueryValue::IRI(john_iri))
        );
    }

    #[test]
    fn test_filter_expression() {
        let ontology = Ontology::new();
        let engine = QueryEngine::new(ontology);

        let binding = QueryBinding {
            variables: {
                let mut vars = HashMap::new();
                vars.insert(
                    "?x".to_string(),
                    QueryValue::IRI(
                        IRI::new("http://example.org/test").expect("Failed to create test IRI"),
                    ),
                );
                vars
            },
        };

        let expression = FilterExpression::Equals {
            left: PatternTerm::Variable("?x".to_string()),
            right: PatternTerm::IRI(
                IRI::new("http://example.org/test").expect("Failed to create test IRI"),
            ),
        };

        assert!(engine.evaluate_filter_expression(&binding, &expression));
    }
}
