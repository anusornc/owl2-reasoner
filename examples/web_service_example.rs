//! Web Service Example for OWL2 Reasoner
//!
//! This example demonstrates how to start and test the web service API.

#[cfg(feature = "web-service")]
use owl2_reasoner::web_service::start_web_service;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "web-service")]
    {
        println!("🚀 OWL2 Reasoner Web Service Example");
        println!("====================================");

        // Start web service in background
        let service_handle = tokio::spawn(async { start_web_service(8080) });

        // Give the service time to start
        sleep(Duration::from_secs(1)).await;

        println!("✅ Web service is running on http://localhost:8080");
        println!("📊 Available endpoints:");
        println!("   GET  /health - Health check");
        println!("   POST /epcis - Upload EPCIS data");
        println!("   POST /reasoning - Perform reasoning operations");
        println!("   POST /analysis - Analyze EPCIS data");
        println!("   GET  /statistics - Get ontology statistics");

        println!("\n🧪 Testing endpoints...");

        // Test health endpoint
        test_health_endpoint().await?;

        // Keep service running
        println!("\n⏳ Press Ctrl+C to stop the service");

        // Wait for service completion (or Ctrl+C)
        match service_handle.await {
            Ok(result) => result?,
            Err(e) => return Err(Box::new(e)),
        }
    }

    #[cfg(not(feature = "web-service"))]
    {
        println!("❌ Web service feature not enabled");
        println!("Please run with: cargo run --example web_service_example --features web-service");
        return Err("Web service feature not enabled".into());
    }

    Ok(())
}

#[cfg(feature = "web-service")]
async fn test_health_endpoint() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing GET /health...");

    let client = reqwest::Client::new();
    let response = client.get("http://localhost:8080/health").send().await?;

    if response.status().is_success() {
        println!("✅ Health check successful");
        let body: serde_json::Value = response.json().await?;
        println!("📋 Response: {}", serde_json::to_string_pretty(&body)?);
    } else {
        println!("❌ Health check failed: {}", response.status());
    }

    Ok(())
}
