use rust_analysis_engine::{
    parser::{javascript::JavaScriptParser, Parser},
    analysis::AnalysisEngine,
    types::{AnalysisRequest, SourceFile},
};
use std::time::Instant;

#[tokio::test]
async fn test_parsing_performance() {
    let parser = JavaScriptParser::new().unwrap();
    
    // Create a moderately complex JavaScript file
    let content = generate_test_javascript(1000); // ~1000 lines
    
    let start = Instant::now();
    let result = parser.parse(&content).unwrap();
    let duration = start.elapsed();
    
    println!("Parsed {} lines in {}ms", content.lines().count(), duration.as_millis());
    println!("Found {} functions, {} classes", result.functions.len(), result.classes.len());
    
    // Target: 100ms per 1K LOC, so 1K lines should be under 100ms
    assert!(duration.as_millis() < 100, "Parsing took too long: {}ms", duration.as_millis());
    
    // Verify we found the expected structures
    assert!(result.functions.len() > 0);
}

#[tokio::test]
async fn test_analysis_engine_performance() {
    let engine = AnalysisEngine::new().await.unwrap();
    
    // Create multiple files for batch analysis
    let files = vec![
        SourceFile {
            name: "file1.js".to_string(),
            content: generate_test_javascript(500),
            language: None,
        },
        SourceFile {
            name: "file2.js".to_string(),
            content: generate_test_javascript(500),
            language: None,
        },
        SourceFile {
            name: "file3.js".to_string(),
            content: generate_test_javascript(500),
            language: None,
        },
    ];
    
    let request = AnalysisRequest {
        files,
        rules: None,
    };
    
    let start = Instant::now();
    let response = engine.analyze(request).await.unwrap();
    let duration = start.elapsed();
    
    println!("Analyzed {} files ({} total lines) in {}ms", 
        response.summary.total_files,
        response.summary.total_lines_analyzed,
        duration.as_millis()
    );
    
    // Should be well under 1 second for 1500 lines
    assert!(duration.as_millis() < 1000, "Analysis took too long: {}ms", duration.as_millis());
    
    // Verify results
    assert_eq!(response.results.len(), 3);
    assert!(response.summary.total_lines_analyzed > 1400); // Should be around 1500
}

#[test]
fn test_large_file_parsing() {
    let parser = JavaScriptParser::new().unwrap();
    
    // Create a large JavaScript file (5K lines)
    let content = generate_test_javascript(5000);
    
    let start = Instant::now();
    let result = parser.parse(&content).unwrap();
    let duration = start.elapsed();
    
    println!("Parsed large file ({} lines) in {}ms", 
        content.lines().count(), 
        duration.as_millis()
    );
    
    // Target: 100ms per 1K LOC, so 5K lines should be under 500ms
    assert!(duration.as_millis() < 500, "Large file parsing took too long: {}ms", duration.as_millis());
    
    // Verify we found structures
    assert!(result.functions.len() > 40); // Should find many functions
}

fn generate_test_javascript(target_lines: usize) -> String {
    let mut content = String::new();
    
    // Add some imports
    content.push_str("import React from 'react';\n");
    content.push_str("import { useState, useEffect } from 'react';\n");
    content.push_str("const fs = require('fs');\n");
    content.push_str("\n");
    
    let mut current_lines = 4;
    let mut function_count = 0;
    
    // Generate classes and functions to reach target line count
    while current_lines < target_lines {
        if function_count % 10 == 0 {
            // Add a class every 10 functions
            let class_content = format!(r#"
class TestClass{} {{
    constructor() {{
        this.value = {};
    }}
    
    method{}() {{
        if (this.value > 0) {{
            for (let i = 0; i < this.value; i++) {{
                if (i % 2 === 0) {{
                    console.log(i);
                }} else {{
                    console.log('odd');
                }}
            }}
        }}
        return this.value;
    }}
}}
"#, function_count / 10, function_count, function_count);
            
            content.push_str(&class_content);
            current_lines += class_content.lines().count();
        } else {
            // Add a regular function
            let func_content = format!(r#"
function testFunction{}(a, b, c) {{
    let result = 0;
    
    if (a > 0) {{
        result += a;
    }} else if (a < 0) {{
        result -= a;
    }} else {{
        result = 1;
    }}
    
    for (let i = 0; i < b; i++) {{
        if (i % 2 === 0) {{
            result *= 2;
        }} else {{
            result += c;
        }}
    }}
    
    while (result > 1000) {{
        result /= 2;
    }}
    
    return result && a || b;
}}

const arrow{} = (x, y) => {{
    return x + y + {};
}};
"#, function_count, function_count, function_count);
            
            content.push_str(&func_content);
            current_lines += func_content.lines().count();
        }
        
        function_count += 1;
        
        // Add some variable declarations and other statements
        if current_lines < target_lines - 10 {
            content.push_str(&format!("const variable{} = 'test value {}';\n", function_count, function_count));
            content.push_str(&format!("let counter{} = {};\n", function_count, function_count));
            content.push_str("\n");
            current_lines += 3;
        }
    }
    
    content
}

#[test]
fn test_complexity_performance() {
    let parser = JavaScriptParser::new().unwrap();
    
    // Create a file with many complex functions
    let mut content = String::new();
    
    for i in 0..100 {
        let func = format!(r#"
function complexFunction{}(a, b, c, d, e) {{
    if (a > 0) {{
        if (b > 0) {{
            if (c > 0) {{
                if (d > 0) {{
                    if (e > 0) {{
                        return a + b + c + d + e;
                    }} else {{
                        return a + b + c + d;
                    }}
                }} else {{
                    return a + b + c;
                }}
            }} else {{
                return a + b;
            }}
        }} else {{
            return a;
        }}
    }} else {{
        return 0;
    }}
}}
"#, i);
        content.push_str(&func);
    }
    
    let start = Instant::now();
    let result = parser.parse(&content).unwrap();
    let duration = start.elapsed();
    
    println!("Parsed {} complex functions in {}ms", result.functions.len(), duration.as_millis());
    
    // Should handle complexity calculation efficiently
    assert!(duration.as_millis() < 200, "Complexity calculation took too long: {}ms", duration.as_millis());
    
    // Verify complexity was calculated
    for function in &result.functions {
        assert!(function.complexity > 1, "Function {} should have complexity > 1", function.name);
    }
}