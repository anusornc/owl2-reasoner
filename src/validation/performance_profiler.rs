//! Performance Profiler for Validation
//!
//! This module provides comprehensive performance profiling capabilities
//! for the validation framework, measuring and analyzing performance metrics.

use crate::{OwlError, OwlResult};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Performance profiler for validation activities
pub struct PerformanceProfiler {
    profiles: HashMap<String, PerformanceProfile>,
    active_sessions: HashMap<String, ProfilingSession>,
    configuration: ProfilerConfiguration,
    session_counter: usize,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> OwlResult<Self> {
        Ok(Self {
            profiles: HashMap::new(),
            active_sessions: HashMap::new(),
            configuration: ProfilerConfiguration::default(),
            session_counter: 0,
        })
    }

    /// Start a profiling session
    pub fn start_session(&mut self, session_name: String) -> OwlResult<String> {
        self.session_counter += 1;
        let session_id = format!("{}_{}", session_name, self.session_counter);

        let session = ProfilingSession {
            id: session_id.clone(),
            name: session_name,
            start_time: Instant::now(),
            measurements: Vec::new(),
            memory_snapshots: Vec::new(),
            total_duration: Duration::from_secs(0),
        };

        self.active_sessions.insert(session_id.clone(), session);

        info!("Started profiling session: {}", session_id);
        Ok(session_id)
    }

    /// End a profiling session
    pub fn end_session(&mut self, session_id: &str) -> OwlResult<PerformanceProfile> {
        let session = self
            .active_sessions
            .remove(session_id)
            .ok_or_else(|| OwlError::ParseError(format!("Session {} not found", session_id)))?;

        let total_duration = session.start_time.elapsed();
        let mut updated_session = session.clone();
        updated_session.total_duration = total_duration;

        // Calculate performance metrics before moving values from session
        let performance_metrics = self.calculate_performance_metrics(&session);

        let profile = PerformanceProfile {
            id: session.id.clone(),
            name: session.name,
            total_duration,
            measurements: session.measurements,
            memory_snapshots: session.memory_snapshots,
            performance_metrics,
            generated_at: chrono::Utc::now(),
        };

        self.profiles.insert(session.id.clone(), profile.clone());

        info!(
            "Ended profiling session: {} (duration: {:?})",
            session_id, total_duration
        );
        Ok(profile)
    }

    /// Record a measurement in an active session
    pub fn record_measurement(
        &mut self,
        session_id: &str,
        measurement: PerformanceMeasurement,
    ) -> OwlResult<()> {
        if let Some(session) = self.active_sessions.get_mut(session_id) {
            session.measurements.push(measurement);
            Ok(())
        } else {
            Err(OwlError::ParseError(format!(
                "Session {} not found",
                session_id
            )))
        }
    }

    /// Record a memory snapshot
    pub fn record_memory_snapshot(
        &mut self,
        session_id: &str,
        snapshot: MemorySnapshot,
    ) -> OwlResult<()> {
        if let Some(session) = self.active_sessions.get_mut(session_id) {
            session.memory_snapshots.push(snapshot);
            Ok(())
        } else {
            Err(OwlError::ParseError(format!(
                "Session {} not found",
                session_id
            )))
        }
    }

    /// Profile a specific operation
    pub fn profile_operation<F, R>(
        &mut self,
        session_id: &str,
        operation_name: &str,
        operation: F,
    ) -> OwlResult<R>
    where
        F: FnOnce() -> OwlResult<R>,
    {
        let start_time = Instant::now();
        let start_memory = self.get_current_memory_usage();

        let result = operation();

        let end_time = Instant::now();
        let end_memory = self.get_current_memory_usage();

        let start_time_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let end_time_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let measurement = PerformanceMeasurement {
            name: operation_name.to_string(),
            start_time_ms,
            end_time_ms,
            duration: end_time.duration_since(start_time),
            memory_delta: end_memory.saturating_sub(start_memory),
            success: result.is_ok(),
        };

        self.record_measurement(session_id, measurement)?;

        result
    }

    /// Generate performance report
    pub fn generate_performance_report(
        &self,
        profile_ids: &[String],
    ) -> OwlResult<PerformanceReport> {
        let mut report = PerformanceReport::new();

        for profile_id in profile_ids {
            if let Some(profile) = self.profiles.get(profile_id) {
                report.profiles.push(profile.clone());
            }
        }

        report.summary = self.generate_report_summary(&report.profiles)?;
        report.recommendations = self.generate_performance_recommendations(&report.profiles);

        Ok(report)
    }

    /// Compare performance profiles
    pub fn compare_profiles(
        &self,
        profile_id1: &str,
        profile_id2: &str,
    ) -> OwlResult<ProfileComparison> {
        let profile1 = self
            .profiles
            .get(profile_id1)
            .ok_or_else(|| OwlError::ParseError(format!("Profile {} not found", profile_id1)))?;
        let profile2 = self
            .profiles
            .get(profile_id2)
            .ok_or_else(|| OwlError::ParseError(format!("Profile {} not found", profile_id2)))?;

        Ok(ProfileComparison {
            profile1_name: profile1.name.clone(),
            profile2_name: profile2.name.clone(),
            duration_comparison: self.compare_durations(&profile1, &profile2),
            memory_comparison: self.compare_memory_usage(&profile1, &profile2),
            operation_comparison: self.compare_operations(&profile1, &profile2),
            overall_improvement: self.calculate_overall_improvement(&profile1, &profile2),
        })
    }

    /// Get performance statistics
    pub fn get_performance_statistics(&self) -> PerformanceStatistics {
        let mut stats = PerformanceStatistics::new();

        for profile in self.profiles.values() {
            stats.total_profiles += 1;
            stats.total_duration += profile.total_duration;

            for measurement in &profile.measurements {
                if measurement.success {
                    stats.successful_operations += 1;
                } else {
                    stats.failed_operations += 1;
                }

                stats.total_memory_delta += measurement.memory_delta;

                if stats.fastest_operation.is_none()
                    || measurement.duration < stats.fastest_operation.unwrap()
                {
                    stats.fastest_operation = Some(measurement.duration);
                }

                if stats.slowest_operation.is_none()
                    || measurement.duration > stats.slowest_operation.unwrap()
                {
                    stats.slowest_operation = Some(measurement.duration);
                }
            }
        }

        if stats.total_profiles > 0 {
            stats.average_duration = stats.total_duration / stats.total_profiles as u32;
        }

        if stats.successful_operations + stats.failed_operations > 0 {
            stats.success_rate = stats.successful_operations as f64
                / (stats.successful_operations + stats.failed_operations) as f64;
        }

        stats
    }

    // Helper methods
    fn calculate_performance_metrics(&self, session: &ProfilingSession) -> PerformanceMetrics {
        let total_operations = session.measurements.len();
        let successful_operations = session.measurements.iter().filter(|m| m.success).count();
        let failed_operations = total_operations - successful_operations;

        let total_duration = session
            .measurements
            .iter()
            .map(|m| m.duration)
            .sum::<Duration>();

        let average_operation_time = if total_operations > 0 {
            total_duration / total_operations as u32
        } else {
            Duration::from_secs(0)
        };

        let peak_memory_usage = session
            .memory_snapshots
            .iter()
            .map(|s| s.memory_usage_mb)
            .max()
            .unwrap_or(0);

        let memory_efficiency = if session.memory_snapshots.len() > 1 {
            let initial_memory = session.memory_snapshots.first().unwrap().memory_usage_mb;
            let final_memory = session.memory_snapshots.last().unwrap().memory_usage_mb;
            if final_memory > initial_memory {
                (final_memory - initial_memory) as f64 / session.total_duration.as_secs_f64()
            } else {
                0.0
            }
        } else {
            0.0
        };

        PerformanceMetrics {
            total_operations,
            successful_operations,
            failed_operations,
            total_duration,
            average_operation_time,
            peak_memory_usage_mb: peak_memory_usage,
            memory_efficiency_mb_per_sec: memory_efficiency,
            operations_per_second: if total_duration.as_secs_f64() > 0.0 {
                total_operations as f64 / total_duration.as_secs_f64()
            } else {
                0.0
            },
        }
    }

    fn get_current_memory_usage(&self) -> usize {
        // In a real implementation, this would measure actual memory usage
        // For now, return a placeholder value
        50 // MB
    }

    fn generate_report_summary(&self, profiles: &[PerformanceProfile]) -> OwlResult<ReportSummary> {
        let total_profiles = profiles.len();
        let total_duration: Duration = profiles.iter().map(|p| p.total_duration).sum();
        let total_operations: usize = profiles
            .iter()
            .map(|p| p.performance_metrics.total_operations)
            .sum();

        let average_duration = if total_profiles > 0 {
            total_duration / total_profiles as u32
        } else {
            Duration::from_secs(0)
        };

        Ok(ReportSummary {
            total_profiles,
            total_duration,
            average_duration,
            total_operations,
            overall_performance_rating: self.calculate_overall_performance_rating(profiles),
        })
    }

    fn calculate_overall_performance_rating(
        &self,
        profiles: &[PerformanceProfile],
    ) -> PerformanceRating {
        if profiles.is_empty() {
            return PerformanceRating::NoData;
        }

        let avg_duration = profiles
            .iter()
            .map(|p| p.total_duration.as_secs_f64())
            .sum::<f64>()
            / profiles.len() as f64;

        let avg_memory = profiles
            .iter()
            .map(|p| p.performance_metrics.peak_memory_usage_mb)
            .sum::<usize>()
            / profiles.len();

        let success_rate = profiles
            .iter()
            .map(|p| {
                let total = p.performance_metrics.total_operations;
                let successful = p.performance_metrics.successful_operations;
                if total > 0 {
                    successful as f64 / total as f64
                } else {
                    1.0
                }
            })
            .sum::<f64>()
            / profiles.len() as f64;

        // Calculate overall rating based on multiple factors
        let duration_score = if avg_duration < 1.0 {
            1.0
        } else {
            1.0 / avg_duration
        };
        let memory_score = if avg_memory < 100 {
            1.0
        } else {
            100.0 / avg_memory as f64
        };
        let success_score = success_rate;

        let overall_score =
            (duration_score * 0.4 + memory_score * 0.3 + success_score * 0.3).min(1.0);

        match overall_score {
            s if s >= 0.9 => PerformanceRating::Excellent,
            s if s >= 0.8 => PerformanceRating::Good,
            s if s >= 0.7 => PerformanceRating::Fair,
            s if s >= 0.6 => PerformanceRating::Poor,
            _ => PerformanceRating::VeryPoor,
        }
    }

    fn generate_performance_recommendations(
        &self,
        profiles: &[PerformanceProfile],
    ) -> Vec<PerformanceRecommendation> {
        let mut recommendations = Vec::new();

        // Analyze duration patterns
        let avg_duration = profiles
            .iter()
            .map(|p| p.total_duration.as_secs_f64())
            .sum::<f64>()
            / profiles.len() as f64;

        if avg_duration > 10.0 {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::Performance,
                title: "High Average Duration Detected".to_string(),
                description: format!(
                    "Average duration is {:.1}s, consider optimization",
                    avg_duration
                ),
                priority: RecommendationPriority::High,
                suggested_actions: vec![
                    "Profile individual operations to identify bottlenecks".to_string(),
                    "Consider algorithmic optimizations".to_string(),
                    "Review memory allocation patterns".to_string(),
                ],
            });
        }

        // Analyze memory usage
        let avg_peak_memory = profiles
            .iter()
            .map(|p| p.performance_metrics.peak_memory_usage_mb)
            .sum::<usize>()
            / profiles.len();

        if avg_peak_memory > 500 {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::Memory,
                title: "High Memory Usage Detected".to_string(),
                description: format!(
                    "Average peak memory usage is {}MB, consider optimization",
                    avg_peak_memory
                ),
                priority: RecommendationPriority::Medium,
                suggested_actions: vec![
                    "Implement memory pooling".to_string(),
                    "Review data structure sizes".to_string(),
                    "Consider lazy loading strategies".to_string(),
                ],
            });
        }

        // Analyze success rates
        let avg_success_rate = profiles
            .iter()
            .map(|p| {
                let total = p.performance_metrics.total_operations;
                let successful = p.performance_metrics.successful_operations;
                if total > 0 {
                    successful as f64 / total as f64
                } else {
                    1.0
                }
            })
            .sum::<f64>()
            / profiles.len() as f64;

        if avg_success_rate < 0.95 {
            recommendations.push(PerformanceRecommendation {
                category: RecommendationCategory::Reliability,
                title: "Low Success Rate Detected".to_string(),
                description: format!(
                    "Success rate is {:.1}%, investigate failures",
                    avg_success_rate * 100.0
                ),
                priority: RecommendationPriority::High,
                suggested_actions: vec![
                    "Review error handling and logging".to_string(),
                    "Add input validation".to_string(),
                    "Implement retry mechanisms for transient failures".to_string(),
                ],
            });
        }

        recommendations
    }

    fn compare_durations(
        &self,
        profile1: &PerformanceProfile,
        profile2: &PerformanceProfile,
    ) -> DurationComparison {
        let improvement = profile1
            .total_duration
            .saturating_sub(profile2.total_duration);
        let percentage_change = if profile1.total_duration.as_secs_f64() > 0.0 {
            (improvement.as_secs_f64() / profile1.total_duration.as_secs_f64()) * 100.0
        } else {
            0.0
        };

        DurationComparison {
            profile1_duration: profile1.total_duration,
            profile2_duration: profile2.total_duration,
            improvement,
            percentage_change,
        }
    }

    fn compare_memory_usage(
        &self,
        profile1: &PerformanceProfile,
        profile2: &PerformanceProfile,
    ) -> MemoryComparison {
        let memory1 = profile1.performance_metrics.peak_memory_usage_mb;
        let memory2 = profile2.performance_metrics.peak_memory_usage_mb;
        let improvement = memory1.saturating_sub(memory2);
        let percentage_change = if memory1 > 0 {
            (improvement as f64 / memory1 as f64) * 100.0
        } else {
            0.0
        };

        MemoryComparison {
            profile1_memory_mb: memory1,
            profile2_memory_mb: memory2,
            improvement_mb: improvement,
            percentage_change,
        }
    }

    fn compare_operations(
        &self,
        profile1: &PerformanceProfile,
        profile2: &PerformanceProfile,
    ) -> OperationComparison {
        let ops1 = profile1.performance_metrics.total_operations;
        let ops2 = profile2.performance_metrics.total_operations;
        let throughput1 = if profile1.total_duration.as_secs_f64() > 0.0 {
            ops1 as f64 / profile1.total_duration.as_secs_f64()
        } else {
            0.0
        };
        let throughput2 = if profile2.total_duration.as_secs_f64() > 0.0 {
            ops2 as f64 / profile2.total_duration.as_secs_f64()
        } else {
            0.0
        };

        OperationComparison {
            profile1_operations: ops1,
            profile2_operations: ops2,
            profile1_throughput: throughput1,
            profile2_throughput: throughput2,
            throughput_improvement: throughput2 - throughput1,
        }
    }

    fn calculate_overall_improvement(
        &self,
        profile1: &PerformanceProfile,
        profile2: &PerformanceProfile,
    ) -> f64 {
        let duration_improvement = if profile1.total_duration.as_secs_f64() > 0.0 {
            (profile1
                .total_duration
                .saturating_sub(profile2.total_duration)
                .as_secs_f64()
                / profile1.total_duration.as_secs_f64())
                * 100.0
        } else {
            0.0
        };

        let memory_improvement = if profile1.performance_metrics.peak_memory_usage_mb > 0 {
            (profile1
                .performance_metrics
                .peak_memory_usage_mb
                .saturating_sub(profile2.performance_metrics.peak_memory_usage_mb)
                as f64
                / profile1.performance_metrics.peak_memory_usage_mb as f64)
                * 100.0
        } else {
            0.0
        };

        (duration_improvement + memory_improvement) / 2.0
    }
}

/// Profiling configuration
#[derive(Debug, Clone)]
pub struct ProfilerConfiguration {
    pub enable_memory_profiling: bool,
    pub enable_cpu_profiling: bool,
    pub max_profiles: usize,
    pub auto_cleanup_threshold: Duration,
}

impl Default for ProfilerConfiguration {
    fn default() -> Self {
        Self {
            enable_memory_profiling: true,
            enable_cpu_profiling: false, // Requires additional dependencies
            max_profiles: 1000,
            auto_cleanup_threshold: Duration::from_secs(24 * 60 * 60), // 24 hours
        }
    }
}

/// Active profiling session
#[derive(Debug, Clone)]
pub struct ProfilingSession {
    pub id: String,
    pub name: String,
    pub start_time: Instant,
    pub measurements: Vec<PerformanceMeasurement>,
    pub memory_snapshots: Vec<MemorySnapshot>,
    pub total_duration: Duration,
}

/// Performance measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMeasurement {
    pub name: String,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
    pub duration: Duration,
    pub memory_delta: usize,
    pub success: bool,
}

/// Memory snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub timestamp_ms: u64,
    pub memory_usage_mb: usize,
    pub heap_size_mb: usize,
    pub stack_size_mb: usize,
}

/// Performance profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub id: String,
    pub name: String,
    pub total_duration: Duration,
    pub measurements: Vec<PerformanceMeasurement>,
    pub memory_snapshots: Vec<MemorySnapshot>,
    pub performance_metrics: PerformanceMetrics,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub total_duration: Duration,
    pub average_operation_time: Duration,
    pub peak_memory_usage_mb: usize,
    pub memory_efficiency_mb_per_sec: f64,
    pub operations_per_second: f64,
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub profiles: Vec<PerformanceProfile>,
    pub summary: ReportSummary,
    pub recommendations: Vec<PerformanceRecommendation>,
}

impl PerformanceReport {
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            summary: ReportSummary::new(),
            recommendations: Vec::new(),
        }
    }
}

/// Report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_profiles: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub total_operations: usize,
    pub overall_performance_rating: PerformanceRating,
}

impl ReportSummary {
    pub fn new() -> Self {
        Self {
            total_profiles: 0,
            total_duration: Duration::from_secs(0),
            average_duration: Duration::from_secs(0),
            total_operations: 0,
            overall_performance_rating: PerformanceRating::NoData,
        }
    }
}

/// Performance rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceRating {
    Excellent,
    Good,
    Fair,
    Poor,
    VeryPoor,
    NoData,
}

/// Performance recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendation {
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub suggested_actions: Vec<String>,
}

/// Recommendation category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Performance,
    Memory,
    Reliability,
    Scalability,
}

/// Recommendation priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Profile comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileComparison {
    pub profile1_name: String,
    pub profile2_name: String,
    pub duration_comparison: DurationComparison,
    pub memory_comparison: MemoryComparison,
    pub operation_comparison: OperationComparison,
    pub overall_improvement: f64,
}

/// Duration comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationComparison {
    pub profile1_duration: Duration,
    pub profile2_duration: Duration,
    pub improvement: Duration,
    pub percentage_change: f64,
}

/// Memory comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryComparison {
    pub profile1_memory_mb: usize,
    pub profile2_memory_mb: usize,
    pub improvement_mb: usize,
    pub percentage_change: f64,
}

/// Operation comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationComparison {
    pub profile1_operations: usize,
    pub profile2_operations: usize,
    pub profile1_throughput: f64,
    pub profile2_throughput: f64,
    pub throughput_improvement: f64,
}

/// Performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStatistics {
    pub total_profiles: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub success_rate: f64,
    pub total_memory_delta: usize,
    pub fastest_operation: Option<Duration>,
    pub slowest_operation: Option<Duration>,
}

impl PerformanceStatistics {
    pub fn new() -> Self {
        Self {
            total_profiles: 0,
            total_duration: Duration::from_secs(0),
            average_duration: Duration::from_secs(0),
            successful_operations: 0,
            failed_operations: 0,
            success_rate: 0.0,
            total_memory_delta: 0,
            fastest_operation: None,
            slowest_operation: None,
        }
    }
}
