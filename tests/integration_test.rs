use rust_analysis_engine::types::{AnalysisRequest, SourceFile};
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_analyze_endpoint() {
    // Start the server in the background
    let server_handle = tokio::spawn(async {
        let server = rust_analysis_engine::server::Server::new().await.unwrap();
        server.run().await.unwrap();
    });

    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create a test request
    let request = AnalysisRequest {
        files: vec![SourceFile {
            name: "test.js".to_string(),
            content: "function test() { return 1; }".to_string(),
            language: None,
        }],
        rules: None,
    };

    // Make HTTP request
    let client = reqwest::Client::new();
    let response = timeout(
        Duration::from_secs(5),
        client
            .post("http://localhost:8080/analyze")
            .json(&request)
            .send(),
    )
    .await
    .expect("Request timed out")
    .expect("Request failed");

    // Verify response
    assert_eq!(response.status(), 200);
    
    let response_json: serde_json::Value = response.json().await.expect("Invalid JSON response");
    
    // Verify response structure
    assert!(response_json.get("results").is_some());
    assert!(response_json.get("summary").is_some());
    assert!(response_json.get("execution_time_ms").is_some());

    // Verify we got results for our file
    let results = response_json["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    
    let file_result = &results[0];
    assert_eq!(file_result["file_name"], "test.js");
    assert_eq!(file_result["language"], "JavaScript");

    // Clean up
    server_handle.abort();
}

#[tokio::test]
async fn test_health_endpoint() {
    // Start the server in the background
    let server_handle = tokio::spawn(async {
        let server = rust_analysis_engine::server::Server::new().await.unwrap();
        server.run().await.unwrap();
    });

    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Make health check request
    let client = reqwest::Client::new();
    let response = timeout(
        Duration::from_secs(5),
        client.get("http://localhost:8080/health").send(),
    )
    .await
    .expect("Request timed out")
    .expect("Request failed");

    // Verify response
    assert_eq!(response.status(), 200);
    
    let response_json: serde_json::Value = response.json().await.expect("Invalid JSON response");
    
    assert_eq!(response_json["status"], "healthy");
    assert_eq!(response_json["service"], "rust-analysis-engine");

    // Clean up
    server_handle.abort();
}

#[tokio::test]
async fn test_validation_errors() {
    // Start the server in the background
    let server_handle = tokio::spawn(async {
        let server = rust_analysis_engine::server::Server::new().await.unwrap();
        server.run().await.unwrap();
    });

    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Test empty files array
    let empty_request = json!({
        "files": []
    });

    let response = client
        .post("http://localhost:8080/analyze")
        .json(&empty_request)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 400);

    // Test unsupported file extension
    let unsupported_request = json!({
        "files": [{
            "name": "test.xyz",
            "content": "some content"
        }]
    });

    let response = client
        .post("http://localhost:8080/analyze")
        .json(&unsupported_request)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 400);

    // Clean up
    server_handle.abort();
}