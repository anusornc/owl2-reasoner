//! Grammar rules for OWL Functional Syntax
//!
//! This module defines the grammar rules and production handlers for
//! parsing OWL Functional Syntax documents.

use crate::axioms::*;
use crate::entities::*;
use crate::iri::IRI;
use crate::parser::owl_functional::error::FunctionalSyntaxResult;
use crate::parser::owl_functional::syntax::*;
use crate::parser::owl_functional::tokenizer::{Token, TokenType};
use smallvec;
use std::collections::HashMap;
use std::sync::Arc;

/// Grammar parser for OWL Functional Syntax
pub struct GrammarParser {
    /// The tokens to parse
    tokens: Vec<Token>,
    /// Current token index
    current: usize,
    /// Prefix mappings
    prefixes: HashMap<String, String>,
}

impl GrammarParser {
    /// Create a new grammar parser
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            prefixes: HashMap::new(),
        }
    }

    /// Parse the entire document
    pub fn parse_document(&mut self) -> FunctionalSyntaxResult<FunctionalSyntaxAST> {
        let mut document = FunctionalSyntaxAST::new();

        while !self.is_at_end() {
            if self.match_token(TokenType::Prefix) {
                self.parse_prefix_declaration(&mut document)?;
            } else if self.match_token(TokenType::Ontology) {
                self.parse_ontology_declaration(&mut document)?;
            } else {
                let content = self.parse_ontology_content()?;
                document.add_content(content);
            }
        }

        Ok(document)
    }

    /// Parse a prefix declaration
    fn parse_prefix_declaration(
        &mut self,
        document: &mut FunctionalSyntaxAST,
    ) -> FunctionalSyntaxResult<()> {
        self.consume(TokenType::LeftParen, "Expected '(' after Prefix")?;

        let prefix = if let TokenType::Identifier = self.peek().token_type {
            let lexeme = self.advance().lexeme.clone();
            // Remove trailing colon if present
            lexeme.trim_end_matches(':').to_string()
        } else {
            return Err(crate::parser::owl_functional::error::grammar_error(
                "Expected prefix identifier".to_string(),
            ));
        };

        self.consume(TokenType::Equals, "Expected '=' after prefix")?;

        let namespace = if self.match_token(TokenType::IRI) {
            let iri_lexeme = self.previous().lexeme.clone();
            // Remove angle brackets
            iri_lexeme[1..iri_lexeme.len() - 1].to_string()
        } else {
            return Err(crate::parser::owl_functional::error::grammar_error(
                "Expected namespace IRI".to_string(),
            ));
        };

        self.consume(
            TokenType::RightParen,
            "Expected ')' after prefix declaration",
        )?;

        // Store the prefix mapping
        self.prefixes.insert(prefix.clone(), namespace.clone());
        document.add_prefix(prefix, namespace);

        Ok(())
    }

    /// Parse an ontology declaration
    fn parse_ontology_declaration(
        &mut self,
        document: &mut FunctionalSyntaxAST,
    ) -> FunctionalSyntaxResult<()> {
        self.consume(TokenType::LeftParen, "Expected '(' after Ontology")?;

        let ontology_iri = if self.match_token(TokenType::IRI) {
            let iri_lexeme = self.previous().lexeme.clone();
            Some(iri_lexeme[1..iri_lexeme.len() - 1].to_string())
        } else {
            None
        };

        // Update document's ontology IRI
        if let Some(iri) = ontology_iri {
            let FunctionalSyntaxAST::OntologyDocument {
                ontology_iri: doc_iri,
                ..
            } = document;
            *doc_iri = Some(iri);
        }

        self.consume(
            TokenType::RightParen,
            "Expected ')' after ontology declaration",
        )?;
        Ok(())
    }

    /// Parse ontology content (declarations and axioms)
    fn parse_ontology_content(&mut self) -> FunctionalSyntaxResult<OntologyContent> {
        if self.match_token(TokenType::Declaration) {
            self.parse_entity_declaration()
        } else if self.match_token(TokenType::Import) {
            self.parse_import_declaration()
        } else {
            // Parse various axiom types
            self.parse_axiom()
        }
    }

    /// Parse an entity declaration
    fn parse_entity_declaration(&mut self) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(TokenType::LeftParen, "Expected '(' after Declaration")?;

        let declaration = if self.match_token(TokenType::Class) {
            let class = self.parse_class()?;
            EntityDeclaration::Class(class)
        } else if self.match_token(TokenType::ObjectProperty) {
            let prop = self.parse_object_property()?;
            EntityDeclaration::ObjectProperty(prop)
        } else if self.match_token(TokenType::DataProperty) {
            let prop = self.parse_data_property()?;
            EntityDeclaration::DataProperty(prop)
        } else if self.match_token(TokenType::NamedIndividual) {
            let individual = self.parse_named_individual()?;
            EntityDeclaration::NamedIndividual(individual)
        } else if self.match_token(TokenType::AnonymousIndividual) {
            let individual = self.parse_anonymous_individual()?;
            EntityDeclaration::AnonymousIndividual(individual)
        } else if self.match_token(TokenType::AnnotationProperty) {
            let prop = self.parse_annotation_property()?;
            EntityDeclaration::AnnotationProperty(prop)
        } else {
            return Err(crate::parser::owl_functional::error::grammar_error(
                "Expected entity type in declaration".to_string(),
            ));
        };

        self.consume(TokenType::RightParen, "Expected ')' after declaration")?;
        Ok(OntologyContent::Declaration(declaration))
    }

    /// Parse an import declaration
    fn parse_import_declaration(&mut self) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(TokenType::LeftParen, "Expected '(' after Import")?;

        let import_iri = if self.match_token(TokenType::IRI) {
            let iri_lexeme = self.previous().lexeme.clone();
            let iri_str = &iri_lexeme[1..iri_lexeme.len() - 1];
            IRI::new_optimized(iri_str).map_err(|e| {
                crate::parser::owl_functional::error::invalid_iri_error(format!(
                    "Invalid import IRI: {}",
                    e
                ))
            })?
        } else {
            return Err(crate::parser::owl_functional::error::grammar_error(
                "Expected IRI in import declaration".to_string(),
            ));
        };

        self.consume(
            TokenType::RightParen,
            "Expected ')' after import declaration",
        )?;
        Ok(OntologyContent::Import(ImportDeclaration::new(
            (*import_iri).clone(),
        )))
    }

    /// Parse various axiom types
    fn parse_axiom(&mut self) -> FunctionalSyntaxResult<OntologyContent> {
        let token = self.peek();
        match token.token_type {
            TokenType::SubClassOf => self.parse_subclass_of_axiom(),
            TokenType::EquivalentClasses => self.parse_equivalent_classes_axiom(),
            TokenType::DisjointClasses => self.parse_disjoint_classes_axiom(),
            TokenType::SubObjectPropertyOf => self.parse_sub_object_property_of_axiom(),
            TokenType::ObjectPropertyDomain => self.parse_object_property_domain_axiom(),
            TokenType::ObjectPropertyRange => self.parse_object_property_range_axiom(),
            TokenType::ClassAssertion => self.parse_class_assertion_axiom(),
            TokenType::ObjectPropertyAssertion => self.parse_object_property_assertion_axiom(),
            // Property characteristics
            TokenType::TransitiveObjectProperty => self.parse_transitive_object_property_axiom(),
            TokenType::AsymmetricObjectProperty => self.parse_asymmetric_object_property_axiom(),
            TokenType::IrreflexiveObjectProperty => self.parse_irreflexive_object_property_axiom(),
            TokenType::FunctionalObjectProperty => self.parse_functional_object_property_axiom(),
            // Inverse properties
            TokenType::InverseObjectProperties => self.parse_inverse_object_properties_axiom(),
            _ => Err(crate::parser::owl_functional::error::grammar_error(
                format!("Unknown axiom type: {}", token.lexeme),
            )),
        }
    }

    /// Parse SubClassOf axiom
    fn parse_subclass_of_axiom(&mut self) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(TokenType::LeftParen, "Expected '(' after SubClassOf")?;

        let sub_class = self.parse_class_expression()?;
        let super_class = self.parse_class_expression()?;

        self.consume(TokenType::RightParen, "Expected ')' after SubClassOf axiom")?;

        let axiom = Axiom::SubClassOf(Box::new(SubClassOfAxiom::new(sub_class, super_class)));
        Ok(OntologyContent::Axiom(axiom))
    }

    /// Parse EquivalentClasses axiom
    fn parse_equivalent_classes_axiom(&mut self) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(TokenType::LeftParen, "Expected '(' after EquivalentClasses")?;

        let mut class_iris = Vec::new();

        while !self.check(TokenType::RightParen) {
            let class_expr = self.parse_class_expression()?;
            if let crate::axioms::class_expressions::ClassExpression::Class(class) = class_expr {
                class_iris.push(class.iri().clone());
            } else {
                return Err(crate::parser::owl_functional::error::grammar_error(
                    "EquivalentClasses requires simple class expressions".to_string(),
                ));
            }

            if !self.check(TokenType::RightParen) {
                self.advance(); // Skip space/comma
            }
        }

        self.consume(
            TokenType::RightParen,
            "Expected ')' after EquivalentClasses axiom",
        )?;

        if class_iris.len() >= 2 {
            let axiom = Axiom::EquivalentClasses(Box::new(EquivalentClassesAxiom::new(class_iris)));
            Ok(OntologyContent::Axiom(axiom))
        } else {
            Err(crate::parser::owl_functional::error::grammar_error(
                "EquivalentClasses requires at least 2 classes".to_string(),
            ))
        }
    }

    /// Parse DisjointClasses axiom
    fn parse_disjoint_classes_axiom(&mut self) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(TokenType::LeftParen, "Expected '(' after DisjointClasses")?;

        let mut class_iris = Vec::new();

        while !self.check(TokenType::RightParen) {
            let class_expr = self.parse_class_expression()?;
            if let crate::axioms::class_expressions::ClassExpression::Class(class) = class_expr {
                class_iris.push(class.iri().clone());
            } else {
                return Err(crate::parser::owl_functional::error::grammar_error(
                    "DisjointClasses requires simple class expressions".to_string(),
                ));
            }

            if !self.check(TokenType::RightParen) {
                self.advance(); // Skip space/comma
            }
        }

        self.consume(
            TokenType::RightParen,
            "Expected ')' after DisjointClasses axiom",
        )?;

        if class_iris.len() >= 2 {
            let axiom = Axiom::DisjointClasses(Box::new(DisjointClassesAxiom::new(class_iris)));
            Ok(OntologyContent::Axiom(axiom))
        } else {
            Err(crate::parser::owl_functional::error::grammar_error(
                "DisjointClasses requires at least 2 classes".to_string(),
            ))
        }
    }

    /// Parse SubObjectPropertyOf axiom
    fn parse_sub_object_property_of_axiom(&mut self) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(
            TokenType::LeftParen,
            "Expected '(' after SubObjectPropertyOf",
        )?;

        let sub_prop = self.parse_object_property_expression()?.to_iri()?;
        let super_prop = self.parse_object_property_expression()?.to_iri()?;

        self.consume(
            TokenType::RightParen,
            "Expected ')' after SubObjectPropertyOf axiom",
        )?;

        let axiom =
            Axiom::SubObjectProperty(Box::new(SubObjectPropertyAxiom::new(sub_prop, super_prop)));
        Ok(OntologyContent::Axiom(axiom))
    }

    /// Parse ObjectPropertyDomain axiom
    fn parse_object_property_domain_axiom(&mut self) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(
            TokenType::LeftParen,
            "Expected '(' after ObjectPropertyDomain",
        )?;

        let prop = self.parse_object_property_expression()?.to_iri()?;
        let domain = self.parse_class_expression()?;

        self.consume(
            TokenType::RightParen,
            "Expected ')' after ObjectPropertyDomain axiom",
        )?;

        let axiom = Axiom::ObjectPropertyDomain(Box::new(ObjectPropertyDomainAxiom::new(
            prop.clone(),
            domain,
        )));
        Ok(OntologyContent::Axiom(axiom))
    }

    /// Parse ObjectPropertyRange axiom
    fn parse_object_property_range_axiom(&mut self) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(
            TokenType::LeftParen,
            "Expected '(' after ObjectPropertyRange",
        )?;

        let prop = self.parse_object_property_expression()?.to_iri()?;
        let range = self.parse_class_expression()?;

        self.consume(
            TokenType::RightParen,
            "Expected ')' after ObjectPropertyRange axiom",
        )?;

        let axiom = Axiom::ObjectPropertyRange(Box::new(ObjectPropertyRangeAxiom::new(
            (*prop).clone(),
            range,
        )));
        Ok(OntologyContent::Axiom(axiom))
    }

    /// Parse ClassAssertion axiom
    fn parse_class_assertion_axiom(&mut self) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(TokenType::LeftParen, "Expected '(' after ClassAssertion")?;

        let class_expr = self.parse_class_expression()?;
        let individual = self.parse_individual()?.to_iri()?;

        self.consume(
            TokenType::RightParen,
            "Expected ')' after ClassAssertion axiom",
        )?;

        let axiom =
            Axiom::ClassAssertion(Box::new(ClassAssertionAxiom::new(individual, class_expr)));
        Ok(OntologyContent::Axiom(axiom))
    }

    /// Parse ObjectPropertyAssertion axiom
    fn parse_object_property_assertion_axiom(&mut self) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(
            TokenType::LeftParen,
            "Expected '(' after ObjectPropertyAssertion",
        )?;

        let prop = self.parse_object_property_expression()?.to_iri()?;
        let subject = self.parse_individual()?.to_iri()?;
        let object = self.parse_individual()?.to_iri()?;

        self.consume(
            TokenType::RightParen,
            "Expected ')' after ObjectPropertyAssertion axiom",
        )?;

        let axiom =
            Axiom::PropertyAssertion(Box::new(PropertyAssertionAxiom::new(subject, prop, object)));
        Ok(OntologyContent::Axiom(axiom))
    }

    /// Parse a class expression
    fn parse_class_expression(
        &mut self,
    ) -> FunctionalSyntaxResult<crate::axioms::class_expressions::ClassExpression> {
        use crate::axioms::class_expressions::ClassExpression;

        let token = self.peek();
        match token.token_type {
            TokenType::Class => {
                self.advance();
                let class = self.parse_class()?;
                Ok(ClassExpression::Class(class))
            }
            TokenType::ObjectIntersectionOf => self.parse_object_intersection_of(),
            TokenType::ObjectUnionOf => self.parse_object_union_of(),
            TokenType::ObjectComplementOf => self.parse_object_complement_of(),
            _ => Err(crate::parser::owl_functional::error::grammar_error(
                format!("Expected class expression, found: {}", token.lexeme),
            )),
        }
    }

    /// Parse ObjectIntersectionOf expression
    fn parse_object_intersection_of(
        &mut self,
    ) -> FunctionalSyntaxResult<crate::axioms::class_expressions::ClassExpression> {
        self.consume(
            TokenType::LeftParen,
            "Expected '(' after ObjectIntersectionOf",
        )?;

        let mut operands = Vec::new();

        while !self.check(TokenType::RightParen) {
            operands.push(self.parse_class_expression()?);

            if !self.check(TokenType::RightParen) {
                self.advance(); // Skip space/comma
            }
        }

        self.consume(
            TokenType::RightParen,
            "Expected ')' after ObjectIntersectionOf",
        )?;

        Ok(ClassExpression::ObjectIntersectionOf(
            smallvec::SmallVec::from_vec(operands.into_iter().map(Box::new).collect()),
        ))
    }

    /// Parse ObjectUnionOf expression
    fn parse_object_union_of(
        &mut self,
    ) -> FunctionalSyntaxResult<crate::axioms::class_expressions::ClassExpression> {
        self.consume(TokenType::LeftParen, "Expected '(' after ObjectUnionOf")?;

        let mut operands = Vec::new();

        while !self.check(TokenType::RightParen) {
            operands.push(self.parse_class_expression()?);

            if !self.check(TokenType::RightParen) {
                self.advance(); // Skip space/comma
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after ObjectUnionOf")?;

        Ok(ClassExpression::ObjectUnionOf(
            smallvec::SmallVec::from_vec(operands.into_iter().map(Box::new).collect()),
        ))
    }

    /// Parse ObjectComplementOf expression
    fn parse_object_complement_of(
        &mut self,
    ) -> FunctionalSyntaxResult<crate::axioms::class_expressions::ClassExpression> {
        self.consume(
            TokenType::LeftParen,
            "Expected '(' after ObjectComplementOf",
        )?;

        let operand = Box::new(self.parse_class_expression()?);

        self.consume(
            TokenType::RightParen,
            "Expected ')' after ObjectComplementOf",
        )?;

        Ok(ClassExpression::ObjectComplementOf(operand))
    }

    /// Parse an object property expression
    fn parse_object_property_expression(
        &mut self,
    ) -> FunctionalSyntaxResult<crate::axioms::property_expressions::ObjectPropertyExpression> {
        use crate::axioms::property_expressions::ObjectPropertyExpression;

        let token = self.peek();
        let token_type = token.token_type.clone();
        let token_lexeme = token.lexeme.clone();
        if token_type == TokenType::IRI {
            self.advance();
            let iri = self.resolve_iri(&token_lexeme)?;
            let prop = ObjectProperty::new(iri);
            Ok(ObjectPropertyExpression::ObjectProperty(Box::new(prop)))
        } else {
            Err(crate::parser::owl_functional::error::grammar_error(
                format!(
                    "Expected object property expression, found: {}",
                    token_lexeme
                ),
            ))
        }
    }

    /// Parse an individual
    fn parse_individual(&mut self) -> FunctionalSyntaxResult<crate::entities::Individual> {
        let token = self.peek();
        let token_type = token.token_type.clone();
        let token_lexeme = token.lexeme.clone();
        if token_type == TokenType::IRI {
            self.advance();
            let iri = self.resolve_iri(&token_lexeme)?;
            let individual = NamedIndividual::new(iri);
            Ok(crate::entities::Individual::Named(individual))
        } else {
            Err(crate::parser::owl_functional::error::grammar_error(
                format!("Expected individual, found: {}", token.lexeme),
            ))
        }
    }

    /// Parse a class
    fn parse_class(&mut self) -> FunctionalSyntaxResult<Class> {
        let token = self.peek().clone();
        if token.token_type == TokenType::IRI {
            self.advance();
            let iri = self.resolve_iri(&token.lexeme)?;
            Ok(Class::new((*iri).clone()))
        } else {
            Err(crate::parser::owl_functional::error::grammar_error(
                format!("Expected class IRI, found: {}", token.lexeme),
            ))
        }
    }

    /// Parse an object property
    fn parse_object_property(&mut self) -> FunctionalSyntaxResult<ObjectProperty> {
        let token = self.peek().clone();
        if token.token_type == TokenType::IRI {
            self.advance();
            let iri = self.resolve_iri(&token.lexeme)?;
            Ok(ObjectProperty::new((*iri).clone()))
        } else {
            Err(crate::parser::owl_functional::error::grammar_error(
                format!("Expected object property IRI, found: {}", token.lexeme),
            ))
        }
    }

    /// Parse a data property
    fn parse_data_property(&mut self) -> FunctionalSyntaxResult<DataProperty> {
        let token = self.peek().clone();
        if token.token_type == TokenType::IRI {
            self.advance();
            let iri = self.resolve_iri(&token.lexeme)?;
            Ok(DataProperty::new((*iri).clone()))
        } else {
            Err(crate::parser::owl_functional::error::grammar_error(
                format!("Expected data property IRI, found: {}", token.lexeme),
            ))
        }
    }

    /// Parse a named individual
    fn parse_named_individual(&mut self) -> FunctionalSyntaxResult<NamedIndividual> {
        let token = self.peek().clone();
        if token.token_type == TokenType::IRI {
            self.advance();
            let iri = self.resolve_iri(&token.lexeme)?;
            Ok(NamedIndividual::new((*iri).clone()))
        } else {
            Err(crate::parser::owl_functional::error::grammar_error(
                format!("Expected named individual IRI, found: {}", token.lexeme),
            ))
        }
    }

    /// Parse an anonymous individual
    fn parse_anonymous_individual(&mut self) -> FunctionalSyntaxResult<AnonymousIndividual> {
        let token = self.peek().clone();
        if token.token_type == TokenType::StringLiteral {
            self.advance();
            // Remove quotes
            let node_id = &token.lexeme[1..token.lexeme.len() - 1];
            Ok(AnonymousIndividual::new(node_id))
        } else {
            Err(crate::parser::owl_functional::error::grammar_error(
                format!("Expected anonymous individual, found: {}", token.lexeme),
            ))
        }
    }

    /// Parse an annotation property
    fn parse_annotation_property(&mut self) -> FunctionalSyntaxResult<AnnotationProperty> {
        let token = self.peek().clone();
        if token.token_type == TokenType::IRI {
            self.advance();
            let iri = self.resolve_iri(&token.lexeme)?;
            Ok(AnnotationProperty::new((*iri).clone()))
        } else {
            Err(crate::parser::owl_functional::error::grammar_error(
                format!("Expected annotation property IRI, found: {}", token.lexeme),
            ))
        }
    }

    /// Resolve an IRI from token lexeme
    fn resolve_iri(&self, lexeme: &str) -> FunctionalSyntaxResult<Arc<IRI>> {
        if lexeme.starts_with('<') && lexeme.ends_with('>') {
            // Full IRI
            let iri_content = &lexeme[1..lexeme.len() - 1];
            match IRI::new_optimized(iri_content) {
                Ok(iri) => Ok(iri),
                Err(e) => Err(crate::parser::owl_functional::error::invalid_iri_error(
                    format!("Invalid IRI '{}': {}", iri_content, e),
                )),
            }
        } else if lexeme.contains(':') {
            // Prefixed name
            let mut parts = lexeme.splitn(2, ':');
            let prefix = parts.next().unwrap_or("");
            let local_name = parts.next().unwrap_or("");

            let actual_prefix = if prefix.is_empty() { ":" } else { prefix };

            if let Some(namespace) = self.prefixes.get(actual_prefix) {
                let full_iri = format!("{}{}", namespace, local_name);
                match IRI::new_optimized(&full_iri) {
                    Ok(iri) => Ok(iri),
                    Err(e) => Err(crate::parser::owl_functional::error::invalid_iri_error(
                        format!("Invalid prefixed IRI '{}': {}", lexeme, e),
                    )),
                }
            } else {
                Err(crate::parser::owl_functional::error::invalid_iri_error(
                    format!("Unknown prefix: {}", actual_prefix),
                ))
            }
        } else {
            Err(crate::parser::owl_functional::error::invalid_iri_error(
                format!("Invalid IRI format: {}", lexeme),
            ))
        }
    }

    // Token manipulation methods
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> FunctionalSyntaxResult<()> {
        if self.check(token_type) {
            self.advance();
            Ok(())
        } else {
            Err(crate::parser::owl_functional::error::grammar_error(
                message.to_string(),
            ))
        }
    }

    // Property characteristic parsing functions

    /// Parse TransitiveObjectProperty axiom
    fn parse_transitive_object_property_axiom(
        &mut self,
    ) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(
            TokenType::LeftParen,
            "Expected '(' after TransitiveObjectProperty",
        )?;

        let property = self.parse_object_property_expression()?.to_iri()?;

        self.consume(
            TokenType::RightParen,
            "Expected ')' after TransitiveObjectProperty axiom",
        )?;

        let axiom = Axiom::TransitiveProperty(Box::new(TransitivePropertyAxiom::new(property)));
        Ok(OntologyContent::Axiom(axiom))
    }

    /// Parse AsymmetricObjectProperty axiom
    fn parse_asymmetric_object_property_axiom(
        &mut self,
    ) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(
            TokenType::LeftParen,
            "Expected '(' after AsymmetricObjectProperty",
        )?;

        let property = self.parse_object_property_expression()?.to_iri()?;

        self.consume(
            TokenType::RightParen,
            "Expected ')' after AsymmetricObjectProperty axiom",
        )?;

        let axiom = Axiom::AsymmetricProperty(Box::new(AsymmetricPropertyAxiom::new(property)));
        Ok(OntologyContent::Axiom(axiom))
    }

    /// Parse IrreflexiveObjectProperty axiom
    fn parse_irreflexive_object_property_axiom(
        &mut self,
    ) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(
            TokenType::LeftParen,
            "Expected '(' after IrreflexiveObjectProperty",
        )?;

        let property = self.parse_object_property_expression()?.to_iri()?;

        self.consume(
            TokenType::RightParen,
            "Expected ')' after IrreflexiveObjectProperty axiom",
        )?;

        let axiom = Axiom::IrreflexiveProperty(Box::new(IrreflexivePropertyAxiom::new(property)));
        Ok(OntologyContent::Axiom(axiom))
    }

    /// Parse FunctionalObjectProperty axiom
    fn parse_functional_object_property_axiom(
        &mut self,
    ) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(
            TokenType::LeftParen,
            "Expected '(' after FunctionalObjectProperty",
        )?;

        let property = self.parse_object_property_expression()?.to_iri()?;

        self.consume(
            TokenType::RightParen,
            "Expected ')' after FunctionalObjectProperty axiom",
        )?;

        let axiom = Axiom::FunctionalProperty(Box::new(FunctionalPropertyAxiom::new(property)));
        Ok(OntologyContent::Axiom(axiom))
    }

    /// Parse InverseObjectProperties axiom
    fn parse_inverse_object_properties_axiom(&mut self) -> FunctionalSyntaxResult<OntologyContent> {
        self.consume(
            TokenType::LeftParen,
            "Expected '(' after InverseObjectProperties",
        )?;

        let first_property = self.parse_object_property_expression()?;
        let second_property = self.parse_object_property_expression()?;

        self.consume(
            TokenType::RightParen,
            "Expected ')' after InverseObjectProperties axiom",
        )?;

        let axiom = Axiom::InverseObjectProperties(Box::new(InverseObjectPropertiesAxiom::new(
            first_property,
            second_property,
        )));
        Ok(OntologyContent::Axiom(axiom))
    }
}

// Helper trait extension for ObjectPropertyExpression
trait ObjectPropertyExpressionExt {
    fn to_iri(&self) -> FunctionalSyntaxResult<Arc<IRI>>;
}

impl ObjectPropertyExpressionExt for crate::axioms::property_expressions::ObjectPropertyExpression {
    fn to_iri(&self) -> FunctionalSyntaxResult<Arc<IRI>> {
        match self {
            crate::axioms::property_expressions::ObjectPropertyExpression::ObjectProperty(prop) => {
                Ok((*prop).iri().clone())
            }
            _ => Err(crate::parser::owl_functional::error::grammar_error(
                "Complex property expressions not supported in this context".to_string(),
            )),
        }
    }
}

// Helper trait extension for Individual
trait IndividualExt {
    fn to_iri(&self) -> FunctionalSyntaxResult<Arc<IRI>>;
}

impl IndividualExt for crate::entities::Individual {
    fn to_iri(&self) -> FunctionalSyntaxResult<Arc<IRI>> {
        match self {
            crate::entities::Individual::Named(individual) => Ok(individual.iri().clone()),
            crate::entities::Individual::Anonymous(_) => {
                Err(crate::parser::owl_functional::error::grammar_error(
                    "Anonymous individuals not supported in this context".to_string(),
                ))
            }
        }
    }
}
