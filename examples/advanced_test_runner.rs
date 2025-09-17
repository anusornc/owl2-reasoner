//! Advanced OWL2 Test Suite Runner
//!
//! This example demonstrates comprehensive test suite functionality using
//! advanced reasoning capabilities including tableaux-based reasoning.

use owl2_reasoner::OwlResult;
use owl2_reasoner::test_suite_advanced::{
    AdvancedTestSuiteConfig, AdvancedTestSuiteRunner, ReasoningMode,
};
use std::path::PathBuf;

fn main() -> OwlResult<()> {
    println!("🚀 Advanced OWL2 Test Suite Runner");
    println!("===================================");

    // Create advanced test suite configuration
    let config = AdvancedTestSuiteConfig {
        test_files: vec![
            PathBuf::from("test_suite/simple_test.ofn"),
            PathBuf::from("test_suite/family_test.ttl"),
            PathBuf::from("test_suite/property_test.rdf"),
            PathBuf::from("test_suite/complex_expressions.ttl"),
            PathBuf::from("test_suite/biomedical_test.ttl"),
            PathBuf::from("test_suite/classification_test.rdf"),
            PathBuf::from("test_suite/inconsistent_test.ofn"),
        ],
        reasoning_modes: vec![
            ReasoningMode::Simple,
            ReasoningMode::AdvancedTableaux,
            ReasoningMode::Hybrid,
        ],
        enable_comprehensive_testing: true,
        timeout_seconds: 60,
        tableaux_config: owl2_reasoner::reasoning::tableaux::ReasoningConfig {
            max_depth: 2000,
            debug: false,
            incremental: true,
            timeout: Some(45000),
        },
    };

    println!("Configuration:");
    println!("  Test files: {}", config.test_files.len());
    println!("  Reasoning modes: {:?}", config.reasoning_modes);
    println!(
        "  Comprehensive testing: {}",
        config.enable_comprehensive_testing
    );
    println!("  Timeout: {} seconds", config.timeout_seconds);
    println!();

    // Create and run advanced test suite
    let runner = AdvancedTestSuiteRunner::new(config.clone());

    println!("🔬 Running advanced test suite...");
    println!("This will test each ontology with multiple reasoning modes...");
    println!();

    match runner.run_tests() {
        Ok(result) => {
            println!("✅ Advanced test suite completed successfully!");
            println!();
            println!("{}", runner.get_summary(&result));

            // Show detailed breakdown
            println!("\n📋 Detailed Results Breakdown:");
            println!("=================================");

            let mut mode_results = std::collections::HashMap::new();
            for detail in &result.details {
                mode_results
                    .entry(detail.reasoning_mode.clone())
                    .or_insert_with(Vec::new)
                    .push(detail);
            }

            for (mode, tests) in mode_results {
                let passed = tests.iter().filter(|t| t.passed).count();
                let total = tests.len();
                let avg_time = if total > 0 {
                    tests
                        .iter()
                        .map(|t| t.execution_time)
                        .sum::<std::time::Duration>()
                        / total as u32
                } else {
                    std::time::Duration::from_secs(0)
                };

                println!("\n🎯 {:?} Mode Results:", mode);
                println!(
                    "  Tests: {}/{} passed ({:.1}%)",
                    passed,
                    total,
                    if total > 0 {
                        (passed as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    }
                );
                println!("  Average execution time: {:?}", avg_time);

                // Show individual test results for this mode
                for test in tests {
                    let status = if test.passed { "✅ PASS" } else { "❌ FAIL" };
                    println!(
                        "    {} {} ({})",
                        status,
                        test.test_name,
                        test.execution_time.as_millis()
                    );

                    if let Some(consistency) = test.consistency_check {
                        println!(
                            "      Consistency: {}",
                            if consistency {
                                "✅ Consistent"
                            } else {
                                "❌ Inconsistent"
                            }
                        );
                    }

                    if test.satisfiability_checks > 0 {
                        println!(
                            "      Satisfiability checks: {}",
                            test.satisfiability_checks
                        );
                    }

                    if let Some(classification) = &test.classification_result {
                        println!(
                            "      Classification: {} classes classified",
                            classification.classes_classified
                        );
                    }

                    if let Some(error) = &test.error_message {
                        println!("      Error: {}", error);
                    }
                }
            }

            // Show reasoning metrics
            println!("\n🔬 Advanced Reasoning Metrics:");
            println!("================================");
            println!(
                "  Total consistency checks: {}",
                result.reasoning_metrics.total_consistency_checks
            );
            println!(
                "  Total satisfiability checks: {}",
                result.reasoning_metrics.total_satisfiability_checks
            );
            println!(
                "  Total classification operations: {}",
                result.reasoning_metrics.total_classification_operations
            );
            println!(
                "  Advanced reasoning used: {}",
                result.reasoning_metrics.advanced_reasoning_used
            );
            println!(
                "  Average reasoning time: {:?}",
                result.reasoning_metrics.average_reasoning_time
            );

            // Overall assessment
            println!("\n🎯 Advanced Compliance Assessment:");
            let pass_rate = if result.total_tests > 0 {
                (result.passed_tests as f64 / result.total_tests as f64) * 100.0
            } else {
                0.0
            };

            if pass_rate >= 90.0 {
                println!(
                    "✅ EXCELLENT: {:.1}% pass rate - Advanced reasoning is working perfectly!",
                    pass_rate
                );
            } else if pass_rate >= 75.0 {
                println!(
                    "✅ GOOD: {:.1}% pass rate - Advanced reasoning is performing well",
                    pass_rate
                );
            } else if pass_rate >= 60.0 {
                println!(
                    "⚠️  FAIR: {:.1}% pass rate - Some advanced reasoning issues detected",
                    pass_rate
                );
            } else {
                println!(
                    "❌ POOR: {:.1}% pass rate - Advanced reasoning needs significant work",
                    pass_rate
                );
            }

            // Additional advanced insights
            if result.reasoning_metrics.advanced_reasoning_used {
                println!("🚀 Advanced reasoning features are active and functional");
                if result.reasoning_metrics.total_satisfiability_checks > 0 {
                    println!("🔍 Tableaux-based satisfiability checking is working");
                }
                if result.reasoning_metrics.total_classification_operations > 0 {
                    println!("📊 Classification reasoning is operational");
                }
            } else {
                println!("⚠️  Advanced reasoning was not used - check configuration");
            }

            // Generate detailed report
            println!("\n📄 Generating detailed report...");
            let detailed_report = runner.generate_detailed_report(&result);
            println!("Report generated successfully!");

            // Save report to file
            if let Err(e) = std::fs::write("advanced_test_report.txt", detailed_report) {
                println!("Warning: Could not save report: {}", e);
            } else {
                println!("📄 Detailed report saved to: advanced_test_report.txt");
            }
        }
        Err(e) => {
            println!("❌ Advanced test suite execution failed: {}", e);
        }
    }

    Ok(())
}
