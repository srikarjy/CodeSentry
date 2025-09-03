use rust_analysis_engine::types::{AnalysisRequest, SourceFile, RuleConfig};
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
    
    // Verify metrics are populated with real parser data
    let metrics = &file_result["metrics"];
    assert_eq!(metrics["functions_count"], 1); // Should detect the function
    assert_eq!(metrics["lines_of_code"], 1);

    // Clean up
    server_handle.abort();
}

#[tokio::test]
async fn test_complex_javascript_analysis() {
    // Start the server in the background
    let server_handle = tokio::spawn(async {
        let server = rust_analysis_engine::server::Server::new().await.unwrap();
        server.run().await.unwrap();
    });

    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create a complex JavaScript file for analysis
    let complex_js = r#"
        import React from 'react';
        
        class Calculator {
            constructor() {
                this.value = 0;
            }
            
            // This function has high complexity
            complexCalculation(a, b, c) {
                if (a > 0) {
                    for (let i = 0; i < b; i++) {
                        if (i % 2 === 0) {
                            this.value += c;
                        } else if (i % 3 === 0) {
                            this.value -= c;
                        } else {
                            this.value *= 2;
                        }
                    }
                } else if (a < 0) {
                    while (b > 0) {
                        this.value += a;
                        b--;
                    }
                }
                return this.value && a || b;
            }
            
            // This is a very long function that should trigger length warning
            veryLongFunction() {
                let result = 0;
                // Line 1
                result += 1;
                // Line 2
                result += 2;
                // Line 3
                result += 3;
                // ... many more lines to make it long
                for (let i = 0; i < 100; i++) {
                    result += i;
                    if (i % 10 === 0) {
                        console.log(i);
                    }
                }
                return result;
            }
        }
        
        const arrow = () => {
            const password = "hardcoded123"; // Security issue
            return password;
        };
    "#;

    let request = AnalysisRequest {
        files: vec![SourceFile {
            name: "complex.js".to_string(),
            content: complex_js.to_string(),
            language: None,
        }],
        rules: Some(RuleConfig {
            complexity_threshold: Some(5),
            max_function_length: Some(20),
            enable_security_rules: Some(true),
            enable_dead_code_detection: Some(true),
        }),
    };

    // Make HTTP request
    let client = reqwest::Client::new();
    let response = timeout(
        Duration::from_secs(10),
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
    
    let results = response_json["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    
    let file_result = &results[0];
    let metrics = &file_result["metrics"];
    let findings = file_result["findings"].as_array().unwrap();
    
    // Should detect multiple functions and a class
    assert!(metrics["functions_count"].as_u64().unwrap() >= 4); // constructor, complexCalculation, veryLongFunction, arrow
    assert_eq!(metrics["classes_count"], 1);
    
    // Should have findings for complexity, length, and security
    assert!(findings.len() > 0);
    
    // Check for specific finding types
    let finding_rules: Vec<&str> = findings.iter()
        .map(|f| f["rule_id"].as_str().unwrap())
        .collect();
    
    // Should detect high complexity
    assert!(finding_rules.contains(&"high-complexity"));
    
    // Should detect long function
    assert!(finding_rules.contains(&"long-function"));
    
    // Should detect hardcoded secret
    assert!(finding_rules.contains(&"hardcoded-secret"));

    // Clean up
    server_handle.abort();
}

#[tokio::test]
async fn test_typescript_analysis() {
    // Start the server in the background
    let server_handle = tokio::spawn(async {
        let server = rust_analysis_engine::server::Server::new().await.unwrap();
        server.run().await.unwrap();
    });

    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create a TypeScript file for analysis
    let typescript_content = r#"
        interface User {
            id: number;
            name: string;
            email?: string;
            getDisplayName(): string;
        }
        
        class UserService<T extends User> {
            private users: T[] = [];
            
            constructor(private logger: Logger) {}
            
            async addUser(user: T): Promise<void> {
                if (!user.name || user.name.trim() === '') {
                    throw new Error('Name is required');
                }
                
                for (const existingUser of this.users) {
                    if (existingUser.id === user.id) {
                        throw new Error('User already exists');
                    }
                }
                
                this.users.push(user);
                this.logger.info(`Added user: ${user.name}`);
            }
            
            findUserById(id: number): T | undefined {
                return this.users.find(user => user.id === id);
            }
        }
        
        const processUsers = async <T extends User>(
            users: T[],
            processor: (user: T) => Promise<void>
        ): Promise<void> => {
            for (const user of users) {
                if (user.email && user.email.includes('@')) {
                    await processor(user);
                } else {
                    console.warn(`Invalid email for user: ${user.name}`);
                }
            }
        };
        
        type ApiKey = string;
        const API_KEY: ApiKey = "secret-key-12345"; // Security issue
    "#;

    let request = AnalysisRequest {
        files: vec![SourceFile {
            name: "user-service.ts".to_string(),
            content: typescript_content.to_string(),
            language: None, // Should auto-detect TypeScript
        }],
        rules: Some(RuleConfig {
            complexity_threshold: Some(3),
            max_function_length: Some(15),
            enable_security_rules: Some(true),
            enable_dead_code_detection: Some(true),
        }),
    };

    // Make HTTP request
    let client = reqwest::Client::new();
    let response = timeout(
        Duration::from_secs(10),
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
    
    let results = response_json["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    
    let file_result = &results[0];
    let metrics = &file_result["metrics"];
    let findings = file_result["findings"].as_array().unwrap();
    
    // Verify language detection
    assert_eq!(file_result["language"], "TypeScript");
    
    // Should detect functions and classes
    assert!(metrics["functions_count"].as_u64().unwrap() >= 4); // constructor, addUser, findUserById, processUsers
    assert_eq!(metrics["classes_count"], 2); // Interface and class
    
    // Should have findings
    assert!(findings.len() > 0);
    
    // Check for specific finding types
    let finding_rules: Vec<&str> = findings.iter()
        .map(|f| f["rule_id"].as_str().unwrap())
        .collect();
    
    // Should detect complexity issues
    assert!(finding_rules.contains(&"high-complexity"));
    
    // Should detect hardcoded secret
    assert!(finding_rules.contains(&"hardcoded-secret"));

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