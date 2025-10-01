//! Manchester Syntax Semantic Validation
//!
//! This module provides semantic validation for Manchester Syntax AST nodes,
//! ensuring that parsed constructs are semantically valid according to
//! OWL2 specifications and Manchester Syntax rules.

use super::syntax::{
    ClassExpression, DataPropertyExpression, DataRange, IndividualExpression, ManchesterAST,
    ObjectPropertyExpression, PropertyAssertion, PropertyCharacteristic,
};
use crate::error::{OwlError, OwlResult};
use crate::utils::smallvec::sizes;
use smallvec::SmallVec;
use std::collections::HashMap;

/// Semantic validator for Manchester Syntax AST
#[derive(Debug, Clone)]
pub struct SyntaxValidator {
    /// Known prefixes for IRI resolution
    prefixes: HashMap<String, String>,

    /// Whether to perform strict validation
    strict_mode: bool,

    /// Validation context stack
    context_stack: Vec<ValidationContext>,
}

/// Validation context for error reporting
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Current entity being validated
    pub entity: Option<String>,

    /// Current validation phase
    pub phase: ValidationPhase,

    /// Parent expressions for nested validation
    pub parent_expressions: Vec<String>,
}

/// Validation phases
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationPhase {
    /// Prefix declaration validation
    PrefixDeclaration,

    /// Class declaration validation
    ClassDeclaration,

    /// Property declaration validation
    PropertyDeclaration,

    /// Individual declaration validation
    IndividualDeclaration,

    /// Expression validation
    ExpressionValidation,

    /// Axiom validation
    AxiomValidation,
}

/// Validation result with detailed information
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,

    /// Validation warnings (non-critical issues)
    pub warnings: Vec<ValidationWarning>,

    /// Validation errors (critical issues)
    pub errors: Vec<ValidationError>,
}

/// Validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,

    /// Warning location (line, column if available)
    pub location: Option<(usize, usize)>,

    /// Warning code for categorization
    pub code: WarningCode,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error message
    pub message: String,

    /// Error location (line, column if available)
    pub location: Option<(usize, usize)>,

    /// Error code for categorization
    pub code: ErrorCode,
}

/// Warning codes
#[derive(Debug, Clone, PartialEq)]
pub enum WarningCode {
    /// Unused prefix declaration
    UnusedPrefix,

    /// Redundant expression (e.g., A and A)
    RedundantExpression,

    /// Complex expression that could be simplified
    ComplexExpression,

    /// Deprecated construct usage
    DeprecatedConstruct,

    /// Invalid IRI format
    InvalidIRI,

    /// Invalid cardinality value
    InvalidCardinality,

    /// Unimplemented feature
    UnimplementedFeature,
}

/// Error codes
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCode {
    /// Undefined prefix
    UndefinedPrefix,

    /// Circular dependency
    CircularDependency,

    /// Invalid property characteristic
    InvalidPropertyCharacteristic,

    /// Type mismatch in expression
    TypeMismatch,

    /// Invalid cardinality restriction
    InvalidCardinality,

    /// Missing required component
    MissingRequiredComponent,

    /// Invalid IRI reference
    InvalidIRI,

    /// Duplicate declaration
    DuplicateDeclaration,
}

impl SyntaxValidator {
    /// Create a new syntax validator
    pub fn new() -> Self {
        Self {
            prefixes: HashMap::new(),
            strict_mode: true,
            context_stack: Vec::new(),
        }
    }

    /// Create a validator with strict mode setting
    pub fn with_strict_mode(strict: bool) -> Self {
        Self {
            prefixes: HashMap::new(),
            strict_mode: strict,
            context_stack: Vec::new(),
        }
    }

    /// Add a prefix mapping
    pub fn add_prefix(&mut self, prefix: String, iri: String) {
        self.prefixes.insert(prefix, iri);
    }

    /// Set strict mode
    pub fn set_strict_mode(&mut self, strict: bool) {
        self.strict_mode = strict;
    }

    /// Validate a complete AST
    pub fn validate_ast(&mut self, ast: &ManchesterAST) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
        };

        match ast {
            ManchesterAST::PrefixDeclaration { prefix, iri } => {
                self.validate_prefix_declaration(prefix, iri, &mut result);
            }
            ManchesterAST::ClassDeclaration {
                name,
                sub_class_of,
                equivalent_to,
                disjoint_with,
                ..
            } => {
                self.validate_class_declaration(
                    name,
                    sub_class_of,
                    equivalent_to,
                    disjoint_with,
                    &mut result,
                );
            }
            ManchesterAST::ObjectPropertyDeclaration {
                name,
                domain,
                range,
                characteristics,
                ..
            } => {
                self.validate_object_property_declaration(
                    name,
                    domain,
                    range,
                    characteristics,
                    &mut result,
                );
            }
            ManchesterAST::DataPropertyDeclaration {
                name,
                domain,
                range,
                characteristics,
                ..
            } => {
                self.validate_data_property_declaration(
                    name,
                    domain,
                    range,
                    characteristics,
                    &mut result,
                );
            }
            ManchesterAST::IndividualDeclaration {
                name, types, facts, ..
            } => {
                self.validate_individual_declaration(name, types, facts, &mut result);
            }
            ManchesterAST::DisjointClasses {
                classes,
                annotations: _,
            } => {
                self.validate_disjoint_classes(classes, &mut result);
            }
            ManchesterAST::EquivalentClasses {
                classes,
                annotations: _,
            } => {
                self.validate_equivalent_classes(classes, &mut result);
            }
            ManchesterAST::DifferentIndividuals {
                individuals,
                annotations: _,
            } => {
                self.validate_different_individuals(individuals, &mut result);
            }
            ManchesterAST::SameIndividual {
                individuals,
                annotations: _,
            } => {
                self.validate_same_individuals(individuals, &mut result);
            }
            ManchesterAST::AnnotationDeclaration { .. } => {
                // TODO: Implement annotation validation
                result.warnings.push(ValidationWarning {
                    message: "Annotation validation not yet implemented".to_string(),
                    location: None,
                    code: WarningCode::UnimplementedFeature,
                });
            }
            ManchesterAST::RuleDeclaration { .. } => {
                // TODO: Implement rule validation
                result.warnings.push(ValidationWarning {
                    message: "Rule validation not yet implemented".to_string(),
                    location: None,
                    code: WarningCode::UnimplementedFeature,
                });
            }
        }

        result.is_valid = result.errors.is_empty();
        result
    }

    /// Validate prefix declaration
    fn validate_prefix_declaration(&self, prefix: &str, iri: &str, result: &mut ValidationResult) {
        // Check prefix format
        if prefix.is_empty() {
            result.errors.push(ValidationError {
                message: "Prefix cannot be empty".to_string(),
                location: None,
                code: ErrorCode::InvalidIRI,
            });
            return;
        }

        // Check for invalid characters in prefix
        if prefix
            .chars()
            .any(|c| !c.is_alphanumeric() && c != '_' && c != '-')
        {
            result.errors.push(ValidationError {
                message: format!("Invalid characters in prefix: {}", prefix),
                location: None,
                code: ErrorCode::InvalidIRI,
            });
        }

        // Check IRI format (basic validation)
        if iri.is_empty() {
            result.errors.push(ValidationError {
                message: "IRI cannot be empty".to_string(),
                location: None,
                code: ErrorCode::InvalidIRI,
            });
            return;
        }

        if !iri.starts_with("http://") && !iri.starts_with("https://") && !iri.starts_with("urn:") {
            result.warnings.push(ValidationWarning {
                message: format!("IRI doesn't start with common scheme: {}", iri),
                location: None,
                code: WarningCode::InvalidIRI,
            });
        }

        // Check if IRI ends with separator
        if !iri.ends_with('#') && !iri.ends_with('/') && !iri.ends_with(':') {
            result.warnings.push(ValidationWarning {
                message: format!("IRI should end with separator (#, /, or :): {}", iri),
                location: None,
                code: WarningCode::InvalidIRI,
            });
        }
    }

    /// Validate class declaration
    fn validate_class_declaration(
        &self,
        name: &str,
        sub_class_of: &SmallVec<[Box<ClassExpression>; sizes::CLASS_EXPRESSIONS]>,
        equivalent_to: &SmallVec<[Box<ClassExpression>; sizes::CLASS_EXPRESSIONS]>,
        disjoint_with: &SmallVec<[Box<ClassExpression>; sizes::CLASS_EXPRESSIONS]>,
        result: &mut ValidationResult,
    ) {
        // Validate class name
        self.validate_entity_name(name, "Class", result);

        // Validate subclass expressions
        for expr in sub_class_of {
            self.validate_class_expression(expr, result);
        }

        // Validate equivalent expressions
        for expr in equivalent_to {
            self.validate_class_expression(expr, result);
        }

        // Validate disjoint expressions
        for expr in disjoint_with {
            self.validate_class_expression(expr, result);
        }

        // Check for circular dependencies in strict mode
        if self.strict_mode {
            self.check_circular_dependencies(name, sub_class_of, result);
        }

        // Check for redundant expressions
        if self.strict_mode {
            self.check_redundant_expressions(sub_class_of, result);
            self.check_redundant_expressions(equivalent_to, result);
        }
    }

    /// Validate object property declaration
    fn validate_object_property_declaration(
        &self,
        name: &str,
        domain: &SmallVec<[Box<ClassExpression>; sizes::CLASS_EXPRESSIONS]>,
        range: &SmallVec<[Box<ClassExpression>; sizes::CLASS_EXPRESSIONS]>,
        characteristics: &SmallVec<[PropertyCharacteristic; 4]>,
        result: &mut ValidationResult,
    ) {
        // Validate property name
        self.validate_entity_name(name, "ObjectProperty", result);

        // Validate domain
        for domain_expr in domain {
            self.validate_class_expression(domain_expr, result);
        }

        // Validate range
        for range_expr in range {
            self.validate_class_expression(range_expr, result);
        }

        // Validate characteristics
        for characteristic in characteristics {
            self.validate_property_characteristic(characteristic, result);
        }

        // Check for incompatible characteristics
        if self.strict_mode {
            self.check_incompatible_characteristics(characteristics, result);
        }
    }

    /// Validate data property declaration
    fn validate_data_property_declaration(
        &self,
        name: &str,
        domain: &SmallVec<[Box<ClassExpression>; sizes::CLASS_EXPRESSIONS]>,
        range: &SmallVec<[DataRange; 4]>,
        characteristics: &SmallVec<[PropertyCharacteristic; 4]>,
        result: &mut ValidationResult,
    ) {
        // Validate property name
        self.validate_entity_name(name, "DataProperty", result);

        // Validate domain
        for domain_expr in domain {
            self.validate_class_expression(domain_expr, result);
        }

        // Validate range
        for range_expr in range {
            self.validate_data_range(range_expr, result);
        }

        // Validate characteristics
        for characteristic in characteristics {
            self.validate_property_characteristic(characteristic, result);
        }

        // Check that data properties only have valid characteristics
        if self.strict_mode {
            self.check_data_property_characteristics(characteristics, result);
        }
    }

    /// Validate individual declaration
    fn validate_individual_declaration(
        &self,
        name: &str,
        types: &SmallVec<[Box<ClassExpression>; sizes::CLASS_EXPRESSIONS]>,
        facts: &SmallVec<[PropertyAssertion; 8]>,
        result: &mut ValidationResult,
    ) {
        // Validate individual name
        self.validate_entity_name(name, "Individual", result);

        // Validate type expressions
        for expr in types {
            self.validate_class_expression(expr, result);
        }

        // Validate property assertions
        for fact in facts {
            self.validate_property_assertion(fact, result);
        }
    }

    /// Validate disjoint classes axiom
    fn validate_disjoint_classes(
        &self,
        classes: &SmallVec<[Box<ClassExpression>; sizes::CLASS_EXPRESSIONS]>,
        result: &mut ValidationResult,
    ) {
        if classes.len() < 2 {
            result.errors.push(ValidationError {
                message: "DisjointClasses requires at least 2 classes".to_string(),
                location: None,
                code: ErrorCode::MissingRequiredComponent,
            });
            return;
        }

        for expr in classes {
            self.validate_class_expression(expr, result);
        }
    }

    /// Validate equivalent classes axiom
    fn validate_equivalent_classes(
        &self,
        classes: &SmallVec<[Box<ClassExpression>; sizes::CLASS_EXPRESSIONS]>,
        result: &mut ValidationResult,
    ) {
        if classes.len() < 2 {
            result.errors.push(ValidationError {
                message: "EquivalentClasses requires at least 2 classes".to_string(),
                location: None,
                code: ErrorCode::MissingRequiredComponent,
            });
            return;
        }

        for expr in classes {
            self.validate_class_expression(expr, result);
        }
    }

    /// Validate different individuals axiom
    fn validate_different_individuals(
        &self,
        individuals: &SmallVec<[IndividualExpression; 6]>,
        result: &mut ValidationResult,
    ) {
        if individuals.len() < 2 {
            result.errors.push(ValidationError {
                message: "DifferentIndividuals requires at least 2 individuals".to_string(),
                location: None,
                code: ErrorCode::MissingRequiredComponent,
            });
            return;
        }

        // Check for duplicate individuals
        let mut seen = std::collections::HashSet::new();
        for individual in individuals {
            if seen.contains(individual) {
                result.errors.push(ValidationError {
                    message: format!(
                        "Duplicate individual in DifferentIndividuals: {:?}",
                        individual
                    ),
                    location: None,
                    code: ErrorCode::DuplicateDeclaration,
                });
            }
            seen.insert(individual.clone());
        }
    }

    /// Validate same individual axiom
    fn validate_same_individuals(
        &self,
        individuals: &SmallVec<[IndividualExpression; 6]>,
        result: &mut ValidationResult,
    ) {
        if individuals.len() < 2 {
            result.errors.push(ValidationError {
                message: "SameIndividual requires at least 2 individuals".to_string(),
                location: None,
                code: ErrorCode::MissingRequiredComponent,
            });
            return;
        }

        // Check for duplicate individuals
        let mut seen = std::collections::HashSet::new();
        for individual in individuals {
            if seen.contains(individual) {
                result.errors.push(ValidationError {
                    message: format!("Duplicate individual in SameIndividual: {:?}", individual),
                    location: None,
                    code: ErrorCode::DuplicateDeclaration,
                });
            }
            seen.insert(individual.clone());
        }
    }

    /// Validate class expression
    fn validate_class_expression(
        &self,
        expr: &Box<ClassExpression>,
        result: &mut ValidationResult,
    ) {
        match &**expr {
            ClassExpression::NamedClass(class_name) => {
                self.validate_iri_reference(class_name, result);
            }
            ClassExpression::ObjectIntersection(operands) => {
                if operands.is_empty() {
                    result.errors.push(ValidationError {
                        message: "ObjectIntersection requires at least 1 operand".to_string(),
                        location: None,
                        code: ErrorCode::MissingRequiredComponent,
                    });
                    return;
                }
                for operand in operands {
                    self.validate_class_expression(operand, result);
                }
            }
            ClassExpression::ObjectUnion(operands) => {
                if operands.is_empty() {
                    result.errors.push(ValidationError {
                        message: "ObjectUnion requires at least 1 operand".to_string(),
                        location: None,
                        code: ErrorCode::MissingRequiredComponent,
                    });
                    return;
                }
                for operand in operands {
                    self.validate_class_expression(operand, result);
                }
            }
            ClassExpression::ObjectComplement(expr) => {
                self.validate_class_expression(expr, result);
            }
            ClassExpression::ObjectOneOf(individuals) => {
                if individuals.is_empty() {
                    result.errors.push(ValidationError {
                        message: "ObjectOneOf requires at least 1 individual".to_string(),
                        location: None,
                        code: ErrorCode::MissingRequiredComponent,
                    });
                    return;
                }
                for individual in individuals {
                    self.validate_iri_reference(individual, result);
                }
            }
            ClassExpression::ObjectSomeValuesFrom(prop, expr) => {
                self.validate_object_property_expression(prop, result);
                self.validate_class_expression(expr, result);
            }
            ClassExpression::ObjectAllValuesFrom(prop, expr) => {
                self.validate_object_property_expression(prop, result);
                self.validate_class_expression(expr, result);
            }
            ClassExpression::ObjectHasValue(prop, individual) => {
                self.validate_object_property_expression(prop, result);
                self.validate_iri_reference(individual, result);
            }
            ClassExpression::ObjectHasSelf(prop) => {
                self.validate_object_property_expression(prop, result);
            }
            ClassExpression::ObjectMinCardinality(prop, cardinality) => {
                self.validate_object_property_expression(prop, result);
                self.validate_cardinality(*cardinality, result);
            }
            ClassExpression::ObjectMaxCardinality(prop, cardinality) => {
                self.validate_object_property_expression(prop, result);
                self.validate_cardinality(*cardinality, result);
            }
            ClassExpression::ObjectExactCardinality(prop, cardinality) => {
                self.validate_object_property_expression(prop, result);
                self.validate_cardinality(*cardinality, result);
            }
            // Data property restrictions
            ClassExpression::DataSomeValuesFrom(prop, range) => {
                self.validate_data_property_expression(prop, result);
                self.validate_data_range(range, result);
            }
            ClassExpression::DataAllValuesFrom(prop, range) => {
                self.validate_data_property_expression(prop, result);
                self.validate_data_range(range, result);
            }
            ClassExpression::DataHasValue(prop, literal) => {
                self.validate_data_property_expression(prop, result);
                // Validate literal format
                if literal.is_empty() {
                    result.errors.push(ValidationError {
                        message: "DataHasValue literal cannot be empty".to_string(),
                        location: None,
                        code: ErrorCode::MissingRequiredComponent,
                    });
                }
            }
            ClassExpression::DataMinCardinality(prop, cardinality) => {
                self.validate_data_property_expression(prop, result);
                self.validate_cardinality(*cardinality, result);
            }
            ClassExpression::DataMaxCardinality(prop, cardinality) => {
                self.validate_data_property_expression(prop, result);
                self.validate_cardinality(*cardinality, result);
            }
            ClassExpression::DataExactCardinality(prop, cardinality) => {
                self.validate_data_property_expression(prop, result);
                self.validate_cardinality(*cardinality, result);
            }
        }
    }

    /// Validate object property expression
    fn validate_object_property_expression(
        &self,
        expr: &ObjectPropertyExpression,
        result: &mut ValidationResult,
    ) {
        match expr {
            ObjectPropertyExpression::NamedProperty(name) => {
                self.validate_iri_reference(name, result);
            }
            ObjectPropertyExpression::InverseProperty(prop) => {
                self.validate_object_property_expression(prop, result);
            }
        }
    }

    /// Validate data property expression
    fn validate_data_property_expression(
        &self,
        expr: &DataPropertyExpression,
        result: &mut ValidationResult,
    ) {
        match expr {
            DataPropertyExpression::NamedProperty(name) => {
                self.validate_iri_reference(name, result);
            }
        }
    }

    /// Validate data range
    fn validate_data_range(&self, range: &DataRange, result: &mut ValidationResult) {
        match range {
            DataRange::Datatype(iri) => {
                self.validate_iri_reference(iri, result);
            }
            DataRange::DataIntersection(ranges) => {
                if ranges.is_empty() {
                    result.errors.push(ValidationError {
                        message: "DataIntersection requires at least 1 operand".to_string(),
                        location: None,
                        code: ErrorCode::MissingRequiredComponent,
                    });
                    return;
                }
                for r in ranges {
                    self.validate_data_range(r, result);
                }
            }
            DataRange::DataUnion(ranges) => {
                if ranges.is_empty() {
                    result.errors.push(ValidationError {
                        message: "DataUnion requires at least 1 operand".to_string(),
                        location: None,
                        code: ErrorCode::MissingRequiredComponent,
                    });
                    return;
                }
                for r in ranges {
                    self.validate_data_range(r, result);
                }
            }
            DataRange::DataComplement(range) => {
                self.validate_data_range(range, result);
            }
            DataRange::DataOneOf(literals) => {
                if literals.is_empty() {
                    result.errors.push(ValidationError {
                        message: "DataOneOf requires at least 1 literal".to_string(),
                        location: None,
                        code: ErrorCode::MissingRequiredComponent,
                    });
                    return;
                }
                for literal in literals {
                    if literal.is_empty() {
                        result.errors.push(ValidationError {
                            message: "DataOneOf literal cannot be empty".to_string(),
                            location: None,
                            code: ErrorCode::MissingRequiredComponent,
                        });
                    }
                }
            }
            DataRange::DatatypeRestriction {
                datatype,
                restrictions,
            } => {
                self.validate_iri_reference(datatype, result);
                if restrictions.is_empty() {
                    result.errors.push(ValidationError {
                        message: "DatatypeRestriction requires at least 1 restriction".to_string(),
                        location: None,
                        code: ErrorCode::MissingRequiredComponent,
                    });
                }
            }
        }
    }

    /// Validate property assertion
    fn validate_property_assertion(
        &self,
        assertion: &PropertyAssertion,
        result: &mut ValidationResult,
    ) {
        match assertion {
            PropertyAssertion::ObjectPropertyAssertion {
                subject,
                property,
                object,
            } => {
                self.validate_iri_reference(subject, result);
                self.validate_object_property_expression(property, result);
                self.validate_iri_reference(object, result);
            }
            PropertyAssertion::DataPropertyAssertion {
                subject,
                property,
                object,
            } => {
                self.validate_iri_reference(subject, result);
                self.validate_data_property_expression(property, result);
                if object.is_empty() {
                    result.errors.push(ValidationError {
                        message: "DataPropertyAssertion literal cannot be empty".to_string(),
                        location: None,
                        code: ErrorCode::MissingRequiredComponent,
                    });
                }
            }
            PropertyAssertion::NegativeObjectPropertyAssertion {
                subject,
                property,
                object,
            } => {
                self.validate_iri_reference(subject, result);
                self.validate_object_property_expression(property, result);
                self.validate_iri_reference(object, result);
            }
            PropertyAssertion::NegativeDataPropertyAssertion {
                subject,
                property,
                object,
            } => {
                self.validate_iri_reference(subject, result);
                self.validate_data_property_expression(property, result);
                if object.is_empty() {
                    result.errors.push(ValidationError {
                        message: "NegativeDataPropertyAssertion literal cannot be empty"
                            .to_string(),
                        location: None,
                        code: ErrorCode::MissingRequiredComponent,
                    });
                }
            }
        }
    }

    /// Validate property characteristic
    fn validate_property_characteristic(
        &self,
        _characteristic: &PropertyCharacteristic,
        _result: &mut ValidationResult,
    ) {
        // All property characteristics are valid enum variants, so no validation needed here
        // This could be extended with specific rules for characteristic combinations
    }

    /// Validate entity name
    fn validate_entity_name(&self, name: &str, entity_type: &str, result: &mut ValidationResult) {
        if name.is_empty() {
            result.errors.push(ValidationError {
                message: format!("{} name cannot be empty", entity_type),
                location: None,
                code: ErrorCode::MissingRequiredComponent,
            });
            return;
        }

        // Check for invalid characters
        if name.chars().any(|c| c.is_control()) {
            result.errors.push(ValidationError {
                message: format!("{} name contains control characters: {}", entity_type, name),
                location: None,
                code: ErrorCode::InvalidIRI,
            });
        }

        // Validate as IRI reference
        self.validate_iri_reference(name, result);
    }

    /// Validate IRI reference
    fn validate_iri_reference(&self, iri_ref: &str, result: &mut ValidationResult) {
        if iri_ref.is_empty() {
            result.errors.push(ValidationError {
                message: "IRI reference cannot be empty".to_string(),
                location: None,
                code: ErrorCode::InvalidIRI,
            });
            return;
        }

        // Check for prefixed names
        if let Some((prefix, local_name)) = iri_ref.split_once(':') {
            if !self.prefixes.contains_key(prefix) {
                result.errors.push(ValidationError {
                    message: format!("Undefined prefix: {}", prefix),
                    location: None,
                    code: ErrorCode::UndefinedPrefix,
                });
            }

            // Validate local name
            if local_name.is_empty() {
                result.errors.push(ValidationError {
                    message: format!("Local name cannot be empty in prefixed name: {}", iri_ref),
                    location: None,
                    code: ErrorCode::InvalidIRI,
                });
            }
        } else {
            // Full IRI validation (basic check)
            if !(iri_ref.starts_with("http://")
                || iri_ref.starts_with("https://")
                || iri_ref.starts_with("urn:"))
            {
                result.warnings.push(ValidationWarning {
                    message: format!("IRI reference doesn't use standard scheme: {}", iri_ref),
                    location: None,
                    code: WarningCode::InvalidIRI,
                });
            }
        }
    }

    /// Validate cardinality value
    fn validate_cardinality(&self, cardinality: u32, result: &mut ValidationResult) {
        // Cardinality is always valid as u32, but could add range constraints
        if cardinality == 0 {
            result.warnings.push(ValidationWarning {
                message: format!("Cardinality of 0 may not be meaningful: {}", cardinality),
                location: None,
                code: WarningCode::InvalidCardinality,
            });
        }
    }

    /// Check for circular dependencies
    fn check_circular_dependencies(
        &self,
        class_name: &str,
        sub_class_of: &SmallVec<[Box<ClassExpression>; sizes::CLASS_EXPRESSIONS]>,
        result: &mut ValidationResult,
    ) {
        // Simple circular dependency detection
        for expr in sub_class_of {
            if let ClassExpression::NamedClass(name) = &**expr {
                if name == class_name {
                    result.errors.push(ValidationError {
                        message: format!(
                            "Circular dependency detected: {} SubClassOf {}",
                            class_name, name
                        ),
                        location: None,
                        code: ErrorCode::CircularDependency,
                    });
                }
            }
        }
    }

    /// Check for redundant expressions
    fn check_redundant_expressions(
        &self,
        expressions: &SmallVec<[Box<ClassExpression>; sizes::CLASS_EXPRESSIONS]>,
        result: &mut ValidationResult,
    ) {
        if expressions.len() < 2 {
            return;
        }

        // Simple duplicate detection
        for i in 0..expressions.len() {
            for j in i + 1..expressions.len() {
                if *expressions[i] == *expressions[j] {
                    result.warnings.push(ValidationWarning {
                        message: format!("Duplicate expression detected: {:?}", expressions[i]),
                        location: None,
                        code: WarningCode::RedundantExpression,
                    });
                }
            }
        }
    }

    /// Check for incompatible property characteristics
    fn check_incompatible_characteristics(
        &self,
        characteristics: &[PropertyCharacteristic],
        result: &mut ValidationResult,
    ) {
        // Check for incompatible combinations
        let has_transitive = characteristics.contains(&PropertyCharacteristic::Transitive);
        let has_asymmetric = characteristics.contains(&PropertyCharacteristic::Asymmetric);
        let has_reflexive = characteristics.contains(&PropertyCharacteristic::Reflexive);
        let has_irreflexive = characteristics.contains(&PropertyCharacteristic::Irreflexive);

        if has_transitive && has_asymmetric {
            result.errors.push(ValidationError {
                message: "Property cannot be both Transitive and Asymmetric".to_string(),
                location: None,
                code: ErrorCode::InvalidPropertyCharacteristic,
            });
        }

        if has_reflexive && has_irreflexive {
            result.errors.push(ValidationError {
                message: "Property cannot be both Reflexive and Irreflexive".to_string(),
                location: None,
                code: ErrorCode::InvalidPropertyCharacteristic,
            });
        }
    }

    /// Check data property characteristics
    fn check_data_property_characteristics(
        &self,
        characteristics: &[PropertyCharacteristic],
        result: &mut ValidationResult,
    ) {
        for characteristic in characteristics {
            match characteristic {
                PropertyCharacteristic::Transitive
                | PropertyCharacteristic::Symmetric
                | PropertyCharacteristic::Asymmetric
                | PropertyCharacteristic::Reflexive
                | PropertyCharacteristic::Irreflexive => {
                    result.errors.push(ValidationError {
                        message: format!(
                            "Data properties cannot have {} characteristic",
                            format!("{:?}", characteristic)
                        ),
                        location: None,
                        code: ErrorCode::InvalidPropertyCharacteristic,
                    });
                }
                PropertyCharacteristic::Functional
                | PropertyCharacteristic::InverseFunctional
                | PropertyCharacteristic::Annotation
                | PropertyCharacteristic::Ontology
                | PropertyCharacteristic::Data
                | PropertyCharacteristic::Object => {
                    // These are valid for data properties
                }
            }
        }
    }

    /// Get current validation context
    pub fn current_context(&self) -> Option<&ValidationContext> {
        self.context_stack.last()
    }

    /// Get all defined prefixes
    pub fn prefixes(&self) -> &HashMap<String, String> {
        &self.prefixes
    }

    /// Check if validation is in strict mode
    pub fn is_strict(&self) -> bool {
        self.strict_mode
    }

    /// Validate and convert to OwlResult
    pub fn validate_to_result(&mut self, ast: &ManchesterAST) -> OwlResult<()> {
        let result = self.validate_ast(ast);
        if result.is_valid {
            Ok(())
        } else {
            let error_messages: Vec<String> =
                result.errors.iter().map(|e| e.message.clone()).collect();
            Err(OwlError::ParseError(error_messages.join("\n")))
        }
    }
}

impl Default for SyntaxValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            is_valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Add an error
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Get warnings
    pub fn warnings(&self) -> &[ValidationWarning] {
        &self.warnings
    }

    /// Get errors
    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    /// Get formatted error messages
    pub fn error_messages(&self) -> Vec<String> {
        self.errors.iter().map(|e| e.message.clone()).collect()
    }

    /// Get formatted warning messages
    pub fn warning_messages(&self) -> Vec<String> {
        self.warnings.iter().map(|w| w.message.clone()).collect()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationContext {
    /// Create a new validation context
    pub fn new(entity: Option<String>, phase: ValidationPhase) -> Self {
        Self {
            entity,
            phase,
            parent_expressions: Vec::new(),
        }
    }

    /// Create a context for entity validation
    pub fn for_entity(entity: String, phase: ValidationPhase) -> Self {
        Self::new(Some(entity), phase)
    }

    /// Create a context for phase validation
    pub fn for_phase(phase: ValidationPhase) -> Self {
        Self::new(None, phase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::manchester::syntax::*;

    #[test]
    fn test_validator_creation() {
        let validator = SyntaxValidator::new();
        assert!(validator.prefixes().is_empty());
        assert!(validator.is_strict());
    }

    #[test]
    fn test_prefix_validation() {
        let mut validator = SyntaxValidator::new();
        let ast = ManchesterAST::PrefixDeclaration {
            prefix: "test".to_string(),
            iri: "http://example.org/".to_string(),
        };

        let result = validator.validate_ast(&ast);
        assert!(result.is_valid());
    }

    #[test]
    fn test_invalid_prefix_validation() {
        let mut validator = SyntaxValidator::new();
        let ast = ManchesterAST::PrefixDeclaration {
            prefix: "".to_string(),
            iri: "http://example.org/".to_string(),
        };

        let result = validator.validate_ast(&ast);
        assert!(!result.is_valid());
        assert!(result
            .errors()
            .iter()
            .any(|e| e.message.contains("Prefix cannot be empty")));
    }

    #[test]
    fn test_class_declaration_validation() {
        let mut validator = SyntaxValidator::new();
        let ast = ManchesterAST::ClassDeclaration {
            name: "Person".to_string(),
            sub_class_of: vec![].into(),
            equivalent_to: vec![].into(),
            disjoint_with: vec![].into(),
            annotations: vec![].into(),
        };

        let result = validator.validate_ast(&ast);
        assert!(result.is_valid());
    }

    #[test]
    fn test_empty_class_name_validation() {
        let mut validator = SyntaxValidator::new();
        let ast = ManchesterAST::ClassDeclaration {
            name: "".to_string(),
            sub_class_of: vec![].into(),
            equivalent_to: vec![].into(),
            disjoint_with: vec![].into(),
            annotations: vec![].into(),
        };

        let result = validator.validate_ast(&ast);
        assert!(!result.is_valid());
        assert!(result
            .errors()
            .iter()
            .any(|e| e.message.contains("Class name cannot be empty")));
    }

    #[test]
    fn test_strict_mode() {
        let mut validator = SyntaxValidator::with_strict_mode(false);
        assert!(!validator.is_strict());

        validator.set_strict_mode(true);
        assert!(validator.is_strict());
    }

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid());
        assert!(result.warnings().is_empty());
        assert!(result.errors().is_empty());

        result.add_warning(ValidationWarning {
            message: "Test warning".to_string(),
            location: None,
            code: WarningCode::UnusedPrefix,
        });

        assert_eq!(result.warnings().len(), 1);
        assert_eq!(result.warning_messages()[0], "Test warning");
    }
}
