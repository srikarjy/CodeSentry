use rust_analysis_engine::{
    parser::{javascript::JavaScriptParser, Parser},
    types::Language,
};

#[test]
fn test_javascript_parser_creation() {
    let parser = JavaScriptParser::new();
    assert!(parser.is_ok());
    
    let parser = parser.unwrap();
    assert_eq!(parser.language(), Language::JavaScript);
}

#[test]
fn test_parse_simple_function() {
    let parser = JavaScriptParser::new().unwrap();
    let content = "function hello() { return 'world'; }";
    
    let result = parser.parse(content).unwrap();
    
    assert_eq!(result.language, Language::JavaScript);
    assert_eq!(result.functions.len(), 1);
    assert_eq!(result.functions[0].name, "hello");
    assert_eq!(result.functions[0].line, 1);
    assert_eq!(result.functions[0].complexity, 1);
    assert_eq!(result.classes.len(), 0);
    assert_eq!(result.imports.len(), 0);
}

#[test]
fn test_parse_arrow_function() {
    let parser = JavaScriptParser::new().unwrap();
    let content = "const add = (a, b) => a + b;";
    
    let result = parser.parse(content).unwrap();
    
    assert_eq!(result.functions.len(), 1);
    assert_eq!(result.functions[0].name, "add");
    assert_eq!(result.functions[0].complexity, 1);
}

#[test]
fn test_parse_class_with_methods() {
    let parser = JavaScriptParser::new().unwrap();
    let content = r#"
        class Calculator {
            constructor() {
                this.value = 0;
            }
            
            add(a, b) {
                return a + b;
            }
            
            multiply(x, y) {
                if (x === 0 || y === 0) {
                    return 0;
                }
                return x * y;
            }
        }
    "#;
    
    let result = parser.parse(content).unwrap();
    
    assert_eq!(result.classes.len(), 1);
    assert_eq!(result.classes[0].name, "Calculator");
    
    // Should find constructor, add, and multiply methods
    assert_eq!(result.functions.len(), 3);
    
    let function_names: Vec<&String> = result.functions.iter().map(|f| &f.name).collect();
    assert!(function_names.contains(&&"constructor".to_string()));
    assert!(function_names.contains(&&"add".to_string()));
    assert!(function_names.contains(&&"multiply".to_string()));
    
    // Check complexity - multiply should have higher complexity due to if statement
    let multiply_fn = result.functions.iter().find(|f| f.name == "multiply").unwrap();
    assert!(multiply_fn.complexity > 1);
}

#[test]
fn test_parse_imports() {
    let parser = JavaScriptParser::new().unwrap();
    let content = r#"
        import React from 'react';
        import { useState, useEffect } from 'react';
        import * as utils from './utils';
        const fs = require('fs');
        const path = require('path');
    "#;
    
    let result = parser.parse(content).unwrap();
    
    assert_eq!(result.imports.len(), 5);
    
    let modules: Vec<&String> = result.imports.iter().map(|i| &i.module).collect();
    assert!(modules.contains(&&"react".to_string()));
    assert!(modules.contains(&&"./utils".to_string()));
    assert!(modules.contains(&&"fs".to_string()));
    assert!(modules.contains(&&"path".to_string()));
}

#[test]
fn test_complexity_calculation() {
    let parser = JavaScriptParser::new().unwrap();
    let content = r#"
        function complexFunction(a, b, c) {
            if (a > 0) {                    // +1
                for (let i = 0; i < b; i++) { // +1
                    if (i % 2 === 0) {        // +1
                        console.log(c);
                    } else if (i % 3 === 0) { // +1
                        console.log('fizz');
                    }
                }
            } else if (a < 0) {             // +1
                while (b > 0) {             // +1
                    b--;
                }
            }
            return a && b || c;             // +2 (for && and ||)
        }
    "#;
    
    let result = parser.parse(content).unwrap();
    
    assert_eq!(result.functions.len(), 1);
    assert_eq!(result.functions[0].name, "complexFunction");
    // Base(1) + if(1) + for(1) + if(1) + else if(1) + else if(1) + while(1) + &&(1) + ||(1) = 9
    assert_eq!(result.functions[0].complexity, 9);
}

#[test]
fn test_nested_functions() {
    let parser = JavaScriptParser::new().unwrap();
    let content = r#"
        function outer() {
            function inner() {
                return 42;
            }
            
            const arrow = () => {
                return inner();
            };
            
            return arrow();
        }
    "#;
    
    let result = parser.parse(content).unwrap();
    
    // Should find all three functions: outer, inner, and arrow
    assert_eq!(result.functions.len(), 3);
    
    let function_names: Vec<&String> = result.functions.iter().map(|f| &f.name).collect();
    assert!(function_names.contains(&&"outer".to_string()));
    assert!(function_names.contains(&&"inner".to_string()));
    assert!(function_names.contains(&&"arrow".to_string()));
}

#[test]
fn test_async_functions() {
    let parser = JavaScriptParser::new().unwrap();
    let content = r#"
        async function fetchData() {
            const response = await fetch('/api/data');
            return response.json();
        }
        
        const asyncArrow = async () => {
            return await fetchData();
        };
    "#;
    
    let result = parser.parse(content).unwrap();
    
    assert_eq!(result.functions.len(), 2);
    
    let function_names: Vec<&String> = result.functions.iter().map(|f| &f.name).collect();
    assert!(function_names.contains(&&"fetchData".to_string()));
    assert!(function_names.contains(&&"asyncArrow".to_string()));
}

#[test]
fn test_syntax_error_handling() {
    let parser = JavaScriptParser::new().unwrap();
    
    // Test with syntax error - missing closing brace
    let content = "function broken( { return 'incomplete'; }";
    
    // Should not panic, but may have parsing errors
    let result = parser.parse(content);
    
    // We expect this to either succeed with partial parsing or fail gracefully
    match result {
        Ok(parse_result) => {
            // Partial parsing succeeded - this is acceptable
            println!("Partial parsing succeeded with {} functions", parse_result.functions.len());
        }
        Err(e) => {
            // Expected parse error - this is also acceptable
            println!("Parse error as expected: {}", e);
        }
    }
}

#[test]
fn test_empty_content() {
    let parser = JavaScriptParser::new().unwrap();
    let content = "";
    
    let result = parser.parse(content).unwrap();
    
    assert_eq!(result.functions.len(), 0);
    assert_eq!(result.classes.len(), 0);
    assert_eq!(result.imports.len(), 0);
}

#[test]
fn test_comments_and_strings() {
    let parser = JavaScriptParser::new().unwrap();
    let content = r#"
        // This is a comment with function keyword
        /* Another comment with class keyword */
        
        function realFunction() {
            const str = "This string contains function keyword";
            const template = `This template has class in it`;
            return str + template;
        }
    "#;
    
    let result = parser.parse(content).unwrap();
    
    // Should only find the real function, not the ones in comments/strings
    assert_eq!(result.functions.len(), 1);
    assert_eq!(result.functions[0].name, "realFunction");
}