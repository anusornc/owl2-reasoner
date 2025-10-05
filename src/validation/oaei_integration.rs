//! OAEI (Ontology Alignment Evaluation Initiative) Integration
//!
//! This module provides integration with OAEI benchmarks for ontology alignment
//! and matching validation, which is crucial for competing in ORE competitions.

use crate::{Ontology, OwlError, OwlResult, SimpleReasoner};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// OAEI Benchmark Suite implementation
pub struct OAEIBenchmarkSuite {
    configuration: OAEIConfiguration,
    test_cases: Vec<OAEITestCase>,
    alignment_engine: AlignmentEngine,
}

impl OAEIBenchmarkSuite {
    /// Create a new OAEI benchmark suite instance
    pub fn new() -> OwlResult<Self> {
        Ok(Self {
            configuration: OAEIConfiguration::default(),
            test_cases: Vec::new(),
            alignment_engine: AlignmentEngine::new()?,
        })
    }

    /// Run all OAEI benchmark tracks
    pub fn run_all_tracks(&mut self) -> OwlResult<OAEIResults> {
        info!("Starting OAEI benchmark evaluation...");
        let start_time = Instant::now();

        let mut results = OAEIResults::new();

        // Load OAEI test cases
        self.load_oaei_test_cases()?;

        // Run anatomy track
        results.anatomy_results = Some(self.run_anatomy_track()?);

        // Run bibliographic track
        results.bibliographic_results = Some(self.run_bibliographic_track()?);

        // Run large biomedical track
        results.large_biomedical_results = Some(self.run_large_biomedical_track()?);

        // Run multilingual track
        results.multilingual_results = Some(self.run_multilingual_track()?);

        // Calculate overall scores
        let total_duration = start_time.elapsed();
        results.total_duration = total_duration;
        results.calculate_overall_scores();

        info!(
            "OAEI benchmark evaluation completed in {:?}",
            total_duration
        );
        Ok(results)
    }

    /// Run anatomy track (classic OAEI benchmark)
    pub fn run_anatomy_track(&mut self) -> OwlResult<TrackResult> {
        info!("Running OAEI Anatomy track...");
        let start_time = Instant::now();

        let mut track_result = TrackResult::new("Anatomy".to_string());

        // Load anatomy test cases
        let anatomy_cases = self.get_anatomy_test_cases();

        for test_case in anatomy_cases {
            let alignment_result = self.run_alignment_test(&test_case)?;
            track_result.test_results.push(alignment_result);
        }

        track_result.duration = start_time.elapsed();
        track_result.calculate_track_metrics();

        Ok(track_result)
    }

    /// Run bibliographic track
    pub fn run_bibliographic_track(&mut self) -> OwlResult<TrackResult> {
        info!("Running OAEI Bibliographic track...");
        let start_time = Instant::now();

        let mut track_result = TrackResult::new("Bibliographic".to_string());

        // Load bibliographic test cases
        let bibliographic_cases = self.get_bibliographic_test_cases();

        for test_case in bibliographic_cases {
            let alignment_result = self.run_alignment_test(&test_case)?;
            track_result.test_results.push(alignment_result);
        }

        track_result.duration = start_time.elapsed();
        track_result.calculate_track_metrics();

        Ok(track_result)
    }

    /// Run large biomedical track
    pub fn run_large_biomedical_track(&mut self) -> OwlResult<TrackResult> {
        info!("Running OAEI Large Biomedical track...");
        let start_time = Instant::now();

        let mut track_result = TrackResult::new("Large Biomedical".to_string());

        // Load large biomedical test cases
        let biomedical_cases = self.get_large_biomedical_test_cases();

        for test_case in biomedical_cases {
            let alignment_result = self.run_alignment_test(&test_case)?;
            track_result.test_results.push(alignment_result);
        }

        track_result.duration = start_time.elapsed();
        track_result.calculate_track_metrics();

        Ok(track_result)
    }

    /// Run multilingual track
    pub fn run_multilingual_track(&mut self) -> OwlResult<TrackResult> {
        info!("Running OAEI Multilingual track...");
        let start_time = Instant::now();

        let mut track_result = TrackResult::new("Multilingual".to_string());

        // Load multilingual test cases
        let multilingual_cases = self.get_multilingual_test_cases();

        for test_case in multilingual_cases {
            let alignment_result = self.run_alignment_test(&test_case)?;
            track_result.test_results.push(alignment_result);
        }

        track_result.duration = start_time.elapsed();
        track_result.calculate_track_metrics();

        Ok(track_result)
    }

    /// Run a single alignment test
    fn run_alignment_test(&mut self, test_case: &OAEITestCase) -> OwlResult<AlignmentTestResult> {
        let start_time = Instant::now();

        // Load source and target ontologies
        let source_ontology = self.load_ontology(&test_case.source_ontology_path)?;
        let target_ontology = self.load_ontology(&test_case.target_ontology_path)?;

        // Perform alignment
        let alignment_result = self.alignment_engine.align_ontologies(
            &source_ontology,
            &target_ontology,
            &test_case.alignment_config,
        )?;

        // Evaluate alignment against reference
        let evaluation_metrics =
            self.evaluate_alignment(&alignment_result, &test_case.reference_alignment_path)?;

        let duration = start_time.elapsed();

        Ok(AlignmentTestResult {
            test_name: test_case.name.clone(),
            track: test_case.track.clone(),
            source_ontology: test_case.source_ontology_path.clone(),
            target_ontology: test_case.target_ontology_path.clone(),
            alignment_result,
            evaluation_metrics,
            duration,
            success: true,
        })
    }

    /// Load ontology from file path
    fn load_ontology(&self, path: &str) -> OwlResult<Ontology> {
        // For now, create a test ontology
        // In a real implementation, this would load from the actual OAEI test files
        let mut ontology = Ontology::new();

        // Add some test classes based on the path
        if path.contains("anatomy") {
            self.create_anatomy_ontology(&mut ontology)?;
        } else if path.contains("bibliographic") {
            self.create_bibliographic_ontology(&mut ontology)?;
        } else if path.contains("biomedical") {
            self.create_biomedical_ontology(&mut ontology)?;
        } else {
            self.create_generic_ontology(&mut ontology)?;
        }

        Ok(ontology)
    }

    /// Evaluate alignment against reference
    fn evaluate_alignment(
        &self,
        alignment: &AlignmentResult,
        reference_path: &str,
    ) -> OwlResult<AlignmentMetrics> {
        // Load reference alignment
        let reference_alignment = self.load_reference_alignment(reference_path)?;

        // Calculate metrics
        let true_positives = self.count_true_positives(alignment, &reference_alignment);
        let false_positives = alignment
            .correspondences
            .len()
            .saturating_sub(true_positives);
        let false_negatives = reference_alignment
            .correspondences
            .len()
            .saturating_sub(true_positives);

        let precision = if alignment.correspondences.len() > 0 {
            true_positives as f64 / alignment.correspondences.len() as f64
        } else {
            0.0
        };

        let recall = if reference_alignment.correspondences.len() > 0 {
            true_positives as f64 / reference_alignment.correspondences.len() as f64
        } else {
            0.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        Ok(AlignmentMetrics {
            precision,
            recall,
            f1_score,
            true_positives,
            false_positives,
            false_negatives,
            total_alignments: alignment.correspondences.len(),
            reference_alignments: reference_alignment.correspondences.len(),
        })
    }

    /// Count true positives between predicted and reference alignments
    fn count_true_positives(
        &self,
        predicted: &AlignmentResult,
        reference: &ReferenceAlignment,
    ) -> usize {
        let mut true_positives = 0;

        for predicted_corr in &predicted.correspondences {
            for reference_corr in &reference.correspondences {
                if self.correspondences_match(predicted_corr, reference_corr) {
                    true_positives += 1;
                    break;
                }
            }
        }

        true_positives
    }

    /// Check if two correspondences match
    fn correspondences_match(
        &self,
        predicted: &Correspondence,
        reference: &ReferenceCorrespondence,
    ) -> bool {
        // Simple string matching for entity IRIs
        predicted.source_iri == reference.source_iri
            && predicted.target_iri == reference.target_iri
            && predicted.relation == reference.relation
    }

    /// Load reference alignment
    fn load_reference_alignment(&self, path: &str) -> OwlResult<ReferenceAlignment> {
        // For now, create a mock reference alignment
        // In a real implementation, this would load from the actual OAEI reference files
        let mut correspondences = Vec::new();

        if path.contains("anatomy") {
            correspondences.push(ReferenceCorrespondence {
                source_iri: "http://example.org/anatomy/Heart".to_string(),
                target_iri: "http://example.org/anatomy/Coer".to_string(),
                relation: "equivalent".to_string(),
                confidence: 1.0,
            });
            correspondences.push(ReferenceCorrespondence {
                source_iri: "http://example.org/anatomy/Brain".to_string(),
                target_iri: "http://example.org/anatomy/Cerebrum".to_string(),
                relation: "equivalent".to_string(),
                confidence: 1.0,
            });
        }

        Ok(ReferenceAlignment { correspondences })
    }

    // Test case loaders

    fn load_oaei_test_cases(&mut self) -> OwlResult<()> {
        self.test_cases.extend(self.get_anatomy_test_cases());
        self.test_cases.extend(self.get_bibliographic_test_cases());
        self.test_cases
            .extend(self.get_large_biomedical_test_cases());
        self.test_cases.extend(self.get_multilingual_test_cases());
        Ok(())
    }

    fn get_anatomy_test_cases(&self) -> Vec<OAEITestCase> {
        vec![
            OAEITestCase {
                name: "Anatomy Mouse-Human".to_string(),
                track: "Anatomy".to_string(),
                source_ontology_path: "oaei/anatomy/mouse_anatomy.owl".to_string(),
                target_ontology_path: "oaei/anatomy/human_anatomy.owl".to_string(),
                reference_alignment_path: "oaei/anatomy/reference.rdf".to_string(),
                alignment_config: AlignmentConfiguration::default(),
            },
            OAEITestCase {
                name: "Anatomy Adult-Mouse".to_string(),
                track: "Anatomy".to_string(),
                source_ontology_path: "oaei/anatomy/adult_anatomy.owl".to_string(),
                target_ontology_path: "oaei/anatomy/mouse_anatomy.owl".to_string(),
                reference_alignment_path: "oaei/anatomy/reference2.rdf".to_string(),
                alignment_config: AlignmentConfiguration::default(),
            },
        ]
    }

    fn get_bibliographic_test_cases(&self) -> Vec<OAEITestCase> {
        vec![OAEITestCase {
            name: "Biblio DBLP-ACM".to_string(),
            track: "Bibliographic".to_string(),
            source_ontology_path: "oaei/bibliographic/dblp.owl".to_string(),
            target_ontology_path: "oaei/bibliographic/acm.owl".to_string(),
            reference_alignment_path: "oaei/bibliographic/ref.rdf".to_string(),
            alignment_config: AlignmentConfiguration::default(),
        }]
    }

    fn get_large_biomedical_test_cases(&self) -> Vec<OAEITestCase> {
        vec![
            OAEITestCase {
                name: "LargeBio FMA-NCI".to_string(),
                track: "Large Biomedical".to_string(),
                source_ontology_path: "oaei/largebio/fma.owl".to_string(),
                target_ontology_path: "oaei/largebio/nci.owl".to_string(),
                reference_alignment_path: "oaei/largebio/ref.rdf".to_string(),
                alignment_config: AlignmentConfiguration::default(),
            },
            OAEITestCase {
                name: "LargeBio SNOMED-NCI".to_string(),
                track: "Large Biomedical".to_string(),
                source_ontology_path: "oaei/largebio/snomed.owl".to_string(),
                target_ontology_path: "oaei/largebio/nci.owl".to_string(),
                reference_alignment_path: "oaei/largebio/ref2.rdf".to_string(),
                alignment_config: AlignmentConfiguration::default(),
            },
        ]
    }

    fn get_multilingual_test_cases(&self) -> Vec<OAEITestCase> {
        vec![OAEITestCase {
            name: "Multilingual EN-DE".to_string(),
            track: "Multilingual".to_string(),
            source_ontology_path: "oaei/multilingual/english.owl".to_string(),
            target_ontology_path: "oaei/multilingual/german.owl".to_string(),
            reference_alignment_path: "oaei/multilingual/ref.rdf".to_string(),
            alignment_config: AlignmentConfiguration::default(),
        }]
    }

    // Ontology creators for testing

    fn create_anatomy_ontology(&self, ontology: &mut Ontology) -> OwlResult<()> {
        use crate::axioms::class_expressions::ClassExpression;
        use crate::axioms::SubClassOfAxiom;
        use crate::entities::Class;
        use crate::iri::IRI;

        // Create anatomy-related classes
        let heart = Class::new(IRI::new("http://example.org/anatomy/Heart")?);
        let brain = Class::new(IRI::new("http://example.org/anatomy/Brain")?);
        let organ = Class::new(IRI::new("http://example.org/anatomy/Organ")?);

        ontology.add_class(heart.clone())?;
        ontology.add_class(brain.clone())?;
        ontology.add_class(organ.clone())?;

        // Add subclass relationships
        ontology.add_subclass_axiom(SubClassOfAxiom::new(
            ClassExpression::Class(heart),
            ClassExpression::Class(organ.clone()),
        ))?;

        ontology.add_subclass_axiom(SubClassOfAxiom::new(
            ClassExpression::Class(brain),
            ClassExpression::Class(organ),
        ))?;

        Ok(())
    }

    fn create_bibliographic_ontology(&self, ontology: &mut Ontology) -> OwlResult<()> {
        use crate::axioms::class_expressions::ClassExpression;
        use crate::axioms::SubClassOfAxiom;
        use crate::entities::Class;
        use crate::iri::IRI;

        // Create bibliographic classes
        let article = Class::new(IRI::new("http://example.org/biblio/Article")?);
        let journal = Class::new(IRI::new("http://example.org/biblio/Journal")?);
        let publication = Class::new(IRI::new("http://example.org/biblio/Publication")?);

        ontology.add_class(article.clone())?;
        ontology.add_class(journal.clone())?;
        ontology.add_class(publication.clone())?;

        // Add subclass relationships
        ontology.add_subclass_axiom(SubClassOfAxiom::new(
            ClassExpression::Class(article),
            ClassExpression::Class(publication.clone()),
        ))?;

        ontology.add_subclass_axiom(SubClassOfAxiom::new(
            ClassExpression::Class(journal),
            ClassExpression::Class(publication),
        ))?;

        Ok(())
    }

    fn create_biomedical_ontology(&self, ontology: &mut Ontology) -> OwlResult<()> {
        use crate::axioms::class_expressions::ClassExpression;
        use crate::axioms::SubClassOfAxiom;
        use crate::entities::Class;
        use crate::iri::IRI;

        // Create biomedical classes
        let disease = Class::new(IRI::new("http://example.org/biomedical/Disease")?);
        let cancer = Class::new(IRI::new("http://example.org/biomedical/Cancer")?);
        let treatment = Class::new(IRI::new("http://example.org/biomedical/Treatment")?);

        ontology.add_class(disease.clone())?;
        ontology.add_class(cancer.clone())?;
        ontology.add_class(treatment)?;

        // Add subclass relationships
        ontology.add_subclass_axiom(SubClassOfAxiom::new(
            ClassExpression::Class(cancer),
            ClassExpression::Class(disease),
        ))?;

        Ok(())
    }

    fn create_generic_ontology(&self, ontology: &mut Ontology) -> OwlResult<()> {
        use crate::entities::Class;
        use crate::iri::IRI;

        // Create generic classes
        let class1 = Class::new(IRI::new("http://example.org/generic/Class1")?);
        let class2 = Class::new(IRI::new("http://example.org/generic/Class2")?);

        ontology.add_class(class1)?;
        ontology.add_class(class2)?;

        Ok(())
    }
}

/// Alignment engine for ontology matching
pub struct AlignmentEngine {
    configuration: AlignmentConfiguration,
}

impl AlignmentEngine {
    /// Create a new alignment engine
    pub fn new() -> OwlResult<Self> {
        Ok(Self {
            configuration: AlignmentConfiguration::default(),
        })
    }

    /// Align two ontologies
    pub fn align_ontologies(
        &mut self,
        source_ontology: &Ontology,
        target_ontology: &Ontology,
        config: &AlignmentConfiguration,
    ) -> OwlResult<AlignmentResult> {
        let mut correspondences = Vec::new();

        // Simple string-based matching for demonstration
        // In a real implementation, this would use sophisticated matching algorithms
        for source_class in source_ontology.classes() {
            for target_class in target_ontology.classes() {
                let similarity = self
                    .calculate_similarity(source_class.iri().as_str(), target_class.iri().as_str());

                if similarity >= config.similarity_threshold {
                    correspondences.push(Correspondence {
                        source_iri: source_class.iri().to_string(),
                        target_iri: target_class.iri().to_string(),
                        relation: "equivalent".to_string(),
                        confidence: similarity,
                    });
                }
            }
        }

        Ok(AlignmentResult { correspondences })
    }

    /// Calculate similarity between two IRIs
    fn calculate_similarity(&self, iri1: &str, iri2: &str) -> f64 {
        // Simple similarity calculation based on local name matching
        let local_name1 = iri1.split('/').last().unwrap_or(iri1);
        let local_name2 = iri2.split('/').last().unwrap_or(iri2);

        if local_name1 == local_name2 {
            1.0
        } else {
            // Calculate Levenshtein distance similarity
            let distance = self.levenshtein_distance(local_name1, local_name2);
            let max_len = local_name1.len().max(local_name2.len());

            if max_len == 0 {
                1.0
            } else {
                1.0 - (distance as f64 / max_len as f64)
            }
        }
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = [
                    matrix[i - 1][j] + 1,        // deletion
                    matrix[i][j - 1] + 1,        // insertion
                    matrix[i - 1][j - 1] + cost, // substitution
                ]
                .iter()
                .min()
                .unwrap()
                .clone();
            }
        }

        matrix[len1][len2]
    }
}

// Data structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAEIConfiguration {
    pub timeout_duration: Duration,
    pub max_alignments: usize,
    pub similarity_threshold: f64,
    pub enable_multilingual_matching: bool,
}

impl Default for OAEIConfiguration {
    fn default() -> Self {
        Self {
            timeout_duration: Duration::from_secs(300),
            max_alignments: 1000,
            similarity_threshold: 0.7,
            enable_multilingual_matching: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAEITestCase {
    pub name: String,
    pub track: String,
    pub source_ontology_path: String,
    pub target_ontology_path: String,
    pub reference_alignment_path: String,
    pub alignment_config: AlignmentConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentConfiguration {
    pub similarity_threshold: f64,
    pub max_alignments: usize,
    pub enable_structural_matching: bool,
    pub enable_lexical_matching: bool,
}

impl Default for AlignmentConfiguration {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.7,
            max_alignments: 1000,
            enable_structural_matching: true,
            enable_lexical_matching: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAEIResults {
    pub anatomy_results: Option<TrackResult>,
    pub bibliographic_results: Option<TrackResult>,
    pub large_biomedical_results: Option<TrackResult>,
    pub multilingual_results: Option<TrackResult>,
    pub total_duration: Duration,
    pub overall_f1_score: f64,
    pub overall_precision: f64,
    pub overall_recall: f64,
}

impl OAEIResults {
    pub fn new() -> Self {
        Self {
            anatomy_results: None,
            bibliographic_results: None,
            large_biomedical_results: None,
            multilingual_results: None,
            total_duration: Duration::from_secs(0),
            overall_f1_score: 0.0,
            overall_precision: 0.0,
            overall_recall: 0.0,
        }
    }

    pub fn calculate_overall_scores(&mut self) {
        let mut total_precision = 0.0;
        let mut total_recall = 0.0;
        let mut total_f1 = 0.0;
        let mut track_count = 0;

        if let Some(ref result) = self.anatomy_results {
            total_precision += result.average_precision;
            total_recall += result.average_recall;
            total_f1 += result.average_f1_score;
            track_count += 1;
        }

        if let Some(ref result) = self.bibliographic_results {
            total_precision += result.average_precision;
            total_recall += result.average_recall;
            total_f1 += result.average_f1_score;
            track_count += 1;
        }

        if let Some(ref result) = self.large_biomedical_results {
            total_precision += result.average_precision;
            total_recall += result.average_recall;
            total_f1 += result.average_f1_score;
            track_count += 1;
        }

        if let Some(ref result) = self.multilingual_results {
            total_precision += result.average_precision;
            total_recall += result.average_recall;
            total_f1 += result.average_f1_score;
            track_count += 1;
        }

        if track_count > 0 {
            self.overall_precision = total_precision / track_count as f64;
            self.overall_recall = total_recall / track_count as f64;
            self.overall_f1_score = total_f1 / track_count as f64;
        }
    }
}

impl Default for OAEIResults {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackResult {
    pub track_name: String,
    pub test_results: Vec<AlignmentTestResult>,
    pub duration: Duration,
    pub average_precision: f64,
    pub average_recall: f64,
    pub average_f1_score: f64,
}

impl TrackResult {
    pub fn new(track_name: String) -> Self {
        Self {
            track_name,
            test_results: Vec::new(),
            duration: Duration::from_secs(0),
            average_precision: 0.0,
            average_recall: 0.0,
            average_f1_score: 0.0,
        }
    }

    pub fn calculate_track_metrics(&mut self) {
        if self.test_results.is_empty() {
            return;
        }

        let total_precision: f64 = self
            .test_results
            .iter()
            .map(|r| r.evaluation_metrics.precision)
            .sum();

        let total_recall: f64 = self
            .test_results
            .iter()
            .map(|r| r.evaluation_metrics.recall)
            .sum();

        let total_f1: f64 = self
            .test_results
            .iter()
            .map(|r| r.evaluation_metrics.f1_score)
            .sum();

        let count = self.test_results.len() as f64;

        self.average_precision = total_precision / count;
        self.average_recall = total_recall / count;
        self.average_f1_score = total_f1 / count;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentTestResult {
    pub test_name: String,
    pub track: String,
    pub source_ontology: String,
    pub target_ontology: String,
    pub alignment_result: AlignmentResult,
    pub evaluation_metrics: AlignmentMetrics,
    pub duration: Duration,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentResult {
    pub correspondences: Vec<Correspondence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correspondence {
    pub source_iri: String,
    pub target_iri: String,
    pub relation: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentMetrics {
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
    pub total_alignments: usize,
    pub reference_alignments: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceAlignment {
    pub correspondences: Vec<ReferenceCorrespondence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceCorrespondence {
    pub source_iri: String,
    pub target_iri: String,
    pub relation: String,
    pub confidence: f64,
}
