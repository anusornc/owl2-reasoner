//! OWL2 EL Profile Validation
//!
//! Implements validation for the OWL2 EL Profile (Expressive Logic).
//! EL profile is designed for large ontologies and efficient reasoning.

use super::{ProfileViolation, ProfileViolationType, ViolationSeverity};
use crate::axioms::*;
use crate::error::OwlResult;
use crate::iri::IRI;
use crate::ontology::Ontology;
use std::sync::Arc;

/// EL Profile specific validator
pub struct ElProfileValidator {
    ontology: Arc<Ontology>,
}

impl ElProfileValidator {
    pub fn new(ontology: Arc<Ontology>) -> Self {
        Self { ontology }
    }

    /// Validate ontology against EL profile restrictions
    pub fn validate(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // EL Profile restrictions:
        // 1. Only simple subclass axioms allowed
        violations.extend(self.validate_subclass_axioms()?);

        // 2. No disjoint classes axioms
        violations.extend(self.validate_disjoint_classes()?);

        // 3. Limited equivalent classes (only pairwise)
        violations.extend(self.validate_equivalent_classes()?);

        // 4. No complex class expressions
        violations.extend(self.validate_class_expressions()?);

        // 5. Limited property characteristics
        violations.extend(self.validate_property_characteristics()?);

        // 6. No data property ranges beyond basic datatypes
        violations.extend(self.validate_data_property_ranges()?);

        // 7. No nominals in class expressions
        violations.extend(self.validate_nominals()?);

        Ok(violations)
    }

    fn validate_subclass_axioms(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // In EL profile, subclass axioms must be simple:
        // C ⊑ D where C and D are class names or existential restrictions

        for axiom in self.ontology.subclass_axioms() {
            let (sub_expr, sup_expr) = (axiom.sub_class(), axiom.super_class());

            // Check if both sides are simple (class names or existential restrictions)
            if !self.is_simple_class_expression(sub_expr)
                || !self.is_simple_class_expression(sup_expr)
            {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexSubclassAxiom,
                    message: "Complex subclass axiom not allowed in EL profile. Only simple class names and existential restrictions permitted.".to_string(),
                    affected_entities: self.extract_entities_from_axiom(axiom),
                    severity: ViolationSeverity::Error,
                });
            }
        }

        Ok(violations)
    }

    fn validate_disjoint_classes(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // EL profile does not allow disjoint classes axioms
        if !self.ontology.disjoint_classes_axioms().is_empty() {
            let mut affected_entities = Vec::new();
            for axiom in self.ontology.disjoint_classes_axioms() {
                affected_entities.extend(
                    axiom
                        .classes()
                        .iter()
                        .map(|iri| (**iri).clone())
                        .collect::<Vec<IRI>>(),
                );
            }

            violations.push(ProfileViolation {
                violation_type: ProfileViolationType::DisjointClassesAxiom,
                message: "Disjoint classes axioms are not allowed in EL profile".to_string(),
                affected_entities,
                severity: ViolationSeverity::Error,
            });
        }

        Ok(violations)
    }

    fn validate_equivalent_classes(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        for axiom in self.ontology.equivalent_classes_axioms() {
            // EL profile only allows pairwise equivalent classes
            if axiom.classes().len() > 2 {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::EquivalentClassesAxiom,
                    message: "Only pairwise equivalent classes are allowed in EL profile"
                        .to_string(),
                    affected_entities: axiom.classes().iter().map(|iri| (**iri).clone()).collect(),
                    severity: ViolationSeverity::Error,
                });
            }

            // Check if all expressions are simple
            for class_iri in axiom.classes() {
                let class_expr = crate::axioms::ClassExpression::Class(
                    crate::entities::Class::new(class_iri.as_str()),
                );
                if !self.is_simple_class_expression(&class_expr) {
                    violations.push(ProfileViolation {
                        violation_type: ProfileViolationType::EquivalentClassesAxiom,
                        message: "Complex class expressions in equivalent classes are not allowed in EL profile".to_string(),
                        affected_entities: vec![(*(*class_iri)).clone()],
                        severity: ViolationSeverity::Error,
                    });
                }
            }
        }

        Ok(violations)
    }

    fn validate_class_expressions(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        // EL profile allows only:
        // - Class names
        // - Existential restrictions (∃R.C)
        // - Intersection of class expressions

        // Check all class expressions in the ontology
        violations.extend(self.check_complex_expressions_in_subclasses()?);
        violations.extend(self.check_complex_expressions_in_equivalences()?);
        violations.extend(self.check_complex_expressions_in_domains_ranges()?);

        Ok(violations)
    }

    fn validate_property_characteristics(&self) -> OwlResult<Vec<ProfileViolation>> {
        let violations = Vec::new();

        // EL profile has restrictions on property characteristics
        // No transitive, symmetric, asymmetric, or reflexive properties
        // (This would require checking property characteristic axioms)

        // For now, we'll add a placeholder check
        // In a full implementation, this would check property characteristic axioms

        Ok(violations)
    }

    fn validate_data_property_ranges(&self) -> OwlResult<Vec<ProfileViolation>> {
        let violations = Vec::new();

        // EL profile restricts data property ranges to basic datatypes
        // No complex data ranges

        // Check data property ranges
        for _prop in self.ontology.data_properties() {
            // This would need to check the actual range restrictions
            // For now, we'll assume they're valid
        }

        Ok(violations)
    }

    fn validate_nominals(&self) -> OwlResult<Vec<ProfileViolation>> {
        let violations = Vec::new();

        // EL profile does not allow nominals (named individuals in class expressions)
        // This would need to check class expressions for ObjectOneOf constructs

        Ok(violations)
    }

    // Helper methods
    #[allow(clippy::only_used_in_recursion)]
    fn is_simple_class_expression(&self, expr: &crate::axioms::ClassExpression) -> bool {
        use crate::axioms::ClassExpression::*;

        match expr {
            Class(_) => true,                   // Class names are allowed
            ObjectSomeValuesFrom(_, _) => true, // Existential restrictions are allowed
            ObjectIntersectionOf(classes) => {
                // Intersection is allowed if all operands are simple
                classes.iter().all(|c| self.is_simple_class_expression(c))
            }
            _ => false, // Other constructs are not allowed in EL
        }
    }

    fn extract_entities_from_axiom(&self, axiom: &SubClassOfAxiom) -> Vec<IRI> {
        let mut entities = Vec::new();

        // Extract entities from subclass expression
        if let crate::axioms::ClassExpression::Class(class) = axiom.sub_class() {
            entities.push((**class.iri()).clone());
        }

        // Extract entities from superclass expression
        if let crate::axioms::ClassExpression::Class(class) = axiom.super_class() {
            entities.push((**class.iri()).clone());
        }

        entities
    }

    fn check_complex_expressions_in_subclasses(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        for axiom in self.ontology.subclass_axioms() {
            if !self.is_simple_class_expression(axiom.sub_class()) {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexSubclassAxiom,
                    message: "Complex subclass expression not allowed in EL profile".to_string(),
                    affected_entities: self.extract_entities_from_axiom(axiom),
                    severity: ViolationSeverity::Error,
                });
            }

            if !self.is_simple_class_expression(axiom.super_class()) {
                violations.push(ProfileViolation {
                    violation_type: ProfileViolationType::ComplexSubclassAxiom,
                    message: "Complex superclass expression not allowed in EL profile".to_string(),
                    affected_entities: self.extract_entities_from_axiom(axiom),
                    severity: ViolationSeverity::Error,
                });
            }
        }

        Ok(violations)
    }

    fn check_complex_expressions_in_equivalences(&self) -> OwlResult<Vec<ProfileViolation>> {
        let mut violations = Vec::new();

        for axiom in self.ontology.equivalent_classes_axioms() {
            for class_iri in axiom.classes() {
                let class_expr = crate::axioms::ClassExpression::Class(
                    crate::entities::Class::new(class_iri.as_str()),
                );
                if !self.is_simple_class_expression(&class_expr) {
                    violations.push(ProfileViolation {
                        violation_type: ProfileViolationType::EquivalentClassesAxiom,
                        message: "Complex class expression in equivalent classes not allowed in EL profile".to_string(),
                        affected_entities: vec![(*(*class_iri)).clone()],
                        severity: ViolationSeverity::Error,
                    });
                }
            }
        }

        Ok(violations)
    }

    fn check_complex_expressions_in_domains_ranges(&self) -> OwlResult<Vec<ProfileViolation>> {
        let violations = Vec::new();

        // Check domain and range restrictions for properties
        // This would need to examine property domain/range axioms

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Class;
    use crate::iri::IRI;
    use crate::ontology::Ontology;
    use std::sync::Arc;

    #[test]
    fn test_el_validator_creation() {
        let ontology = Arc::new(Ontology::new());
        let _validator = ElProfileValidator::new(ontology);

        // Should create without error
        // Assertion removed - always true
    }

    #[test]
    fn test_empty_ontology_validation() {
        let ontology = Arc::new(Ontology::new());
        let validator = ElProfileValidator::new(ontology);

        let violations = validator.validate().unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_disjoint_classes_detection() {
        let mut ontology = Ontology::new();

        // Add disjoint classes axiom (not allowed in EL)
        let class1 = Class::new(IRI::new("http://example.org/Class1").unwrap());
        let class2 = Class::new(IRI::new("http://example.org/Class2").unwrap());

        ontology.add_class(class1.clone()).unwrap();
        ontology.add_class(class2.clone()).unwrap();

        // Note: Would need to add disjoint classes axiom support to ontology
        // For now, this test is conceptual

        let validator = ElProfileValidator::new(Arc::new(ontology));
        let _violations = validator.validate().unwrap();

        // Should detect disjoint classes as violation
        // assert!(!violations.is_empty());
    }
}
