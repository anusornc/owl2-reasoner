//! Web Service Integration for OWL2 Reasoner with EPCIS
//!
//! This module provides REST API endpoints for exposing OWL2 reasoning
//! and EPCIS processing capabilities through web services.

#[cfg(feature = "web-service")]
mod web_service_impl {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use uuid::Uuid;
    use warp::{Filter, Rejection, Reply};

    use crate::epcis::*;
    use crate::epcis_parser::*;
    use crate::reasoning::SimpleReasoner;
    use crate::Ontology;

    /// Web service state
    #[derive(Clone)]
    pub struct WebServiceState {
        pub reasoner: Arc<RwLock<Option<SimpleReasoner>>>,
        pub parser: EPCISDocumentParser,
        pub start_time: std::time::Instant,
        pub loaded_events: Arc<std::sync::atomic::AtomicUsize>,
    }

    impl WebServiceState {
        pub fn new() -> Self {
            Self {
                reasoner: Arc::new(RwLock::new(None)),
                parser: EPCISDocumentParser::default(),
                start_time: std::time::Instant::now(),
                loaded_events: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            }
        }

        /// Update the count of loaded events
        pub fn update_event_count(&self, count: usize) {
            self.loaded_events
                .store(count, std::sync::atomic::Ordering::Relaxed);
        }

        /// Get the current count of loaded events
        pub fn get_event_count(&self) -> usize {
            self.loaded_events
                .load(std::sync::atomic::Ordering::Relaxed)
        }
    }

    /// API Error response
    #[derive(Debug, Serialize)]
    pub struct ApiError {
        pub error: String,
        pub error_id: String,
        pub timestamp: String,
    }

    impl ApiError {
        pub fn new(message: &str) -> Self {
            Self {
                error: message.to_string(),
                error_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }
        }
    }

    /// EPCIS event upload request
    #[derive(Debug, Deserialize)]
    pub struct EPCISUploadRequest {
        pub xml_content: Option<String>,
        pub file_path: Option<String>,
        pub events: Option<Vec<EPCISSimpleEvent>>,
    }

    /// Reasoning request
    #[derive(Debug, Deserialize)]
    pub struct ReasoningRequest {
        pub check_consistency: Option<bool>,
        pub validate_profiles: Option<Vec<String>>,
        pub get_statistics: Option<bool>,
    }

    /// Reasoning response
    #[derive(Debug, Serialize)]
    pub struct ReasoningResponse {
        pub request_id: String,
        pub timestamp: String,
        pub consistency: Option<bool>,
        pub profile_validation: Option<HashMap<String, bool>>,
        pub statistics: Option<HashMap<String, usize>>,
        pub execution_time_ms: u64,
    }

    /// EPCIS analysis request
    #[derive(Debug, Deserialize)]
    pub struct EPCISAnalysisRequest {
        pub extract_epcs: Option<bool>,
        pub event_type_counts: Option<bool>,
        pub business_steps: Option<bool>,
        pub traceability_analysis: Option<bool>,
    }

    /// EPCIS analysis response
    #[derive(Debug, Serialize)]
    pub struct EPCISAnalysisResponse {
        pub request_id: String,
        pub timestamp: String,
        pub total_events: usize,
        pub unique_epcs: Option<Vec<String>>,
        pub event_type_counts: Option<HashMap<String, usize>>,
        pub business_steps: Option<Vec<String>>,
        pub traceability_summary: Option<String>,
        pub execution_time_ms: u64,
    }

    /// Health check response
    #[derive(Debug, Serialize)]
    pub struct HealthResponse {
        pub status: String,
        pub service: String,
        pub version: String,
        pub timestamp: String,
        pub uptime_seconds: u64,
    }

    /// Web service API routes
    pub fn api_routes(
        state: WebServiceState,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        // CORS headers
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type"])
            .allow_methods(vec!["GET", "POST", "OPTIONS"]);

        // Health check endpoint
        let health = warp::path("health")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(health_check);

        // EPCIS upload endpoint
        let upload_epcis = warp::path("epcis")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(upload_epcis_handler);

        // Reasoning endpoint
        let reasoning = warp::path("reasoning")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(reasoning_handler);

        // EPCIS analysis endpoint
        let analysis = warp::path("analysis")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_state(state.clone()))
            .and_then(analysis_handler);

        // Statistics endpoint
        let stats = warp::path("statistics")
            .and(warp::get())
            .and(with_state(state.clone()))
            .and_then(statistics_handler);

        // Combine all routes with CORS
        health
            .or(upload_epcis)
            .or(reasoning)
            .or(analysis)
            .or(stats)
            .with(cors)
            .with(warp::log("owl2_reasoner_web"))
    }

    /// Helper to inject state into handlers
    fn with_state(
        state: WebServiceState,
    ) -> impl Filter<Extract = (WebServiceState,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || state.clone())
    }

    /// Health check handler
    async fn health_check(state: WebServiceState) -> Result<impl Reply, Rejection> {
        let response = HealthResponse {
            status: "healthy".to_string(),
            service: "OWL2 Reasoner Web Service".to_string(),
            version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            uptime_seconds: state.start_time.elapsed().as_secs(),
        };

        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            warp::http::StatusCode::OK,
        ))
    }

    /// EPCIS upload handler
    async fn upload_epcis_handler(
        request: EPCISUploadRequest,
        state: WebServiceState,
    ) -> Result<impl Reply, Rejection> {
        let start_time = std::time::Instant::now();

        // Parse EPCIS data
        let events = if let Some(xml_content) = request.xml_content {
            match state.parser.parse_xml_str(&xml_content) {
                Ok(events) => events,
                Err(e) => {
                    return Ok(error_response(
                        warp::http::StatusCode::BAD_REQUEST,
                        &format!("Failed to parse XML: {}", e),
                    ))
                }
            }
        } else if let Some(file_path) = request.file_path {
            match state.parser.parse_xml_file(&file_path) {
                Ok(events) => events,
                Err(e) => {
                    return Ok(error_response(
                        warp::http::StatusCode::BAD_REQUEST,
                        &format!("Failed to parse file: {}", e),
                    ))
                }
            }
        } else if let Some(events) = request.events {
            events
        } else {
            return Ok(error_response(
                warp::http::StatusCode::BAD_REQUEST,
                "No EPCIS data provided",
            ));
        };

        // Convert to ontology
        let ontology = match state.parser.to_ontology(&events) {
            Ok(ontology) => ontology,
            Err(e) => {
                return Ok(error_response(
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Failed to create ontology: {}", e),
                ))
            }
        };

        // Update reasoner
        let mut reasoner_guard = state.reasoner.write().await;
        *reasoner_guard = Some(SimpleReasoner::new(ontology));
        drop(reasoner_guard);

        // Track the number of loaded events
        state.update_event_count(events.len());

        let execution_time = start_time.elapsed().as_millis() as u64;

        let response = serde_json::json!({
            "status": "success",
            "message": "EPCIS data uploaded and processed successfully",
            "events_processed": events.len(),
            "execution_time_ms": execution_time,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            warp::http::StatusCode::OK,
        ))
    }

    /// Reasoning handler
    async fn reasoning_handler(
        request: ReasoningRequest,
        state: WebServiceState,
    ) -> Result<impl Reply, Rejection> {
        let start_time = std::time::Instant::now();

        let reasoner_guard = state.reasoner.read().await;
        let reasoner = match &*reasoner_guard {
            Some(reasoner) => reasoner,
            None => {
                return Ok(error_response(
                    warp::http::StatusCode::BAD_REQUEST,
                    "No ontology loaded. Please upload EPCIS data first.",
                ))
            }
        };

        let mut response = ReasoningResponse {
            request_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            consistency: None,
            profile_validation: None,
            statistics: None,
            execution_time_ms: 0,
        };

        // Consistency check
        if request.check_consistency.unwrap_or(false) {
            match reasoner.is_consistent() {
                Ok(is_consistent) => response.consistency = Some(is_consistent),
                Err(e) => {
                    return Ok(error_response(
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        &format!("Consistency check failed: {}", e),
                    ))
                }
            }
        }

        // Profile validation
        if let Some(profiles) = request.validate_profiles {
            let mut validation_results = HashMap::new();
            for profile in profiles {
                let result = match profile.as_str() {
                    "EL" => reasoner.validate_profile(crate::profiles::Owl2Profile::EL),
                    "QL" => reasoner.validate_profile(crate::profiles::Owl2Profile::QL),
                    "RL" => reasoner.validate_profile(crate::profiles::Owl2Profile::RL),
                    _ => {
                        return Ok(error_response(
                            warp::http::StatusCode::BAD_REQUEST,
                            &format!("Unknown profile: {}", profile),
                        ))
                    }
                };

                match result {
                    Ok(profile_result) => {
                        validation_results.insert(profile, profile_result.is_valid);
                    }
                    Err(e) => {
                        return Ok(error_response(
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                            &format!("Profile validation failed for {}: {}", profile, e),
                        ))
                    }
                }
            }
            response.profile_validation = Some(validation_results);
        }

        // Statistics
        if request.get_statistics.unwrap_or(false) {
            let mut stats = HashMap::new();
            stats.insert("classes", reasoner.ontology().classes().len());
            stats.insert(
                "object_properties",
                reasoner.ontology().object_properties().len(),
            );
            stats.insert(
                "data_properties",
                reasoner.ontology().data_properties().len(),
            );
            stats.insert("individuals", reasoner.ontology().named_individuals().len());
            stats.insert("axioms", reasoner.ontology().axioms().len());
            response.statistics = Some(stats);
        }

        response.execution_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            warp::http::StatusCode::OK,
        ))
    }

    /// Analysis handler
    async fn analysis_handler(
        request: EPCISAnalysisRequest,
        state: WebServiceState,
    ) -> Result<impl Reply, Rejection> {
        let start_time = std::time::Instant::now();

        let reasoner_guard = state.reasoner.read().await;
        let reasoner = match &*reasoner_guard {
            Some(reasoner) => reasoner,
            None => {
                return Ok(error_response(
                    warp::http::StatusCode::BAD_REQUEST,
                    "No ontology loaded. Please upload EPCIS data first.",
                ))
            }
        };

        let mut response = EPCISAnalysisResponse {
            request_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_events: state.get_event_count(),
            unique_epcs: None,
            event_type_counts: None,
            business_steps: None,
            traceability_summary: None,
            execution_time_ms: 0,
        };

        // Extract basic statistics from ontology
        if request.extract_epcs.unwrap_or(false) {
            // Extract individuals (representing EPCs)
            let epcs: Vec<String> = reasoner
                .ontology()
                .named_individuals()
                .iter()
                .filter(|iri| iri.as_str().contains("epcs/"))
                .map(|iri| iri.as_str().to_string())
                .collect();
            response.unique_epcs = Some(epcs);
        }

        // Traceability analysis
        if request.traceability_analysis.unwrap_or(false) {
            let summary = analyze_traceability(reasoner);
            response.traceability_summary = Some(summary);
        }

        response.execution_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            warp::http::StatusCode::OK,
        ))
    }

    /// Statistics handler
    async fn statistics_handler(state: WebServiceState) -> Result<impl Reply, Rejection> {
        let reasoner_guard = state.reasoner.read().await;
        let reasoner = match &*reasoner_guard {
            Some(reasoner) => reasoner,
            None => {
                return Ok(error_response(
                    warp::http::StatusCode::BAD_REQUEST,
                    "No ontology loaded. Please upload EPCIS data first.",
                ))
            }
        };

        let mut stats = HashMap::new();
        stats.insert("classes", reasoner.ontology().classes().len());
        stats.insert(
            "object_properties",
            reasoner.ontology().object_properties().len(),
        );
        stats.insert(
            "data_properties",
            reasoner.ontology().data_properties().len(),
        );
        stats.insert("individuals", reasoner.ontology().named_individuals().len());
        stats.insert("axioms", reasoner.ontology().axioms().len());

        let response = serde_json::json!({
            "status": "success",
            "statistics": stats,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            warp::http::StatusCode::OK,
        ))
    }

    /// Error response helper
    fn error_response(status: warp::http::StatusCode, message: &str) -> impl Reply {
        let error = ApiError::new(message);
        warp::reply::with_status(warp::reply::json(&error), status)
    }

    /// Traceability analysis helper
    fn analyze_traceability(reasoner: &SimpleReasoner) -> String {
        let ontology = reasoner.ontology();
        let individual_count = ontology.named_individuals().len();
        let class_count = ontology.classes().len();
        let axiom_count = ontology.axioms().len();

        format!(
            "Traceability Analysis: {} entities across {} classes with {} logical relationships",
            individual_count, class_count, axiom_count
        )
    }
}

// Public API exports
#[cfg(feature = "web-service")]
pub use web_service_impl::{WebServiceState, *};

/// Start the web service
#[cfg(feature = "web-service")]
pub async fn start_web_service(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    use web_service_impl::api_routes;
    use web_service_impl::WebServiceState;

    let state = WebServiceState::new();
    let routes = api_routes(state);

    println!("🚀 Starting OWL2 Reasoner Web Service on port {}", port);
    println!("📊 API Endpoints:");
    println!("   GET  /health - Health check");
    println!("   POST /epcis - Upload EPCIS data");
    println!("   POST /reasoning - Perform reasoning operations");
    println!("   POST /analysis - Analyze EPCIS data");
    println!("   GET  /statistics - Get ontology statistics");

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
    Ok(())
}

#[cfg(not(feature = "web-service"))]
pub async fn start_web_service(_port: u16) -> Result<(), Box<dyn std::error::Error>> {
    Err("Web service feature not enabled. Compile with --features web-service".into())
}
