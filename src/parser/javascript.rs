use std::collections::HashMap;
use tracing::{debug, warn, instrument};
use tree_sitter::{Language, Node, Parser as TSParser, Tree};

use crate::{
    error::{AnalysisError, AnalysisResult},
    types::{Language as EngineLanguage, Location},
};

use super::{Parser, ParseResult, FunctionInfo, ClassInfo, ImportInfo};

pub struct JavaScriptParser {
    language: Language,
}

impl JavaScriptParser {
    pub fn new() -> AnalysisResult<Self> {
        let language = tree_sitter_javascript::language();
        Ok(Self { language })
    }

    #[instrument(skip(self, content))]
    fn parse_with_tree_sitter(&self, content: &str) -> AnalysisResult<Tree> {
        let mut parser = TSParser::new();
        parser.set_language(self.language).map_err(|e| {
            AnalysisError::ConfigError {
                message: format!("Failed to set JavaScript language: {}", e),
            }
        })?;

        // Set timeout to 5 seconds
        parser.set_timeout_micros(5_000_000);

        let tree = parser.parse(content, None).ok_or_else(|| {
            AnalysisError::ParseError {
                message: "Failed to parse JavaScript content".to_string(),
                line: 1,
            }
        })?;

        // Check for syntax errors
        if tree.root_node().has_error() {
            warn!("JavaScript parsing completed with syntax errors");
            // We'll continue with partial parsing rather than failing completely
        }

        debug!(
            "JavaScript parsing completed: {} nodes",
            tree.root_node().child_count()
        );

        Ok(tree)
    }

    fn extract_functions(&self, tree: &Tree, source: &str) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        let root_node = tree.root_node();
        
        self.traverse_for_functions(&root_node, source, &mut functions);
        
        debug!("Extracted {} functions", functions.len());
        functions
    }

    fn traverse_for_functions(&self, node: &Node, source: &str, functions: &mut Vec<FunctionInfo>) {
        match node.kind() {
            "function_declaration" => {
                if let Some(function_info) = self.extract_function_declaration(node, source) {
                    functions.push(function_info);
                }
            }
            "arrow_function" => {
                if let Some(function_info) = self.extract_arrow_function(node, source) {
                    functions.push(function_info);
                }
            }
            "method_definition" => {
                if let Some(function_info) = self.extract_method_definition(node, source) {
                    functions.push(function_info);
                }
            }
            "function_expression" => {
                if let Some(function_info) = self.extract_function_expression(node, source) {
                    functions.push(function_info);
                }
            }
            _ => {}
        }

        // Recursively traverse child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_for_functions(&child, source, functions);
        }
    }

    fn extract_function_declaration(&self, node: &Node, source: &str) -> Option<FunctionInfo> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.get_node_text(&name_node, source)?;
        
        Some(FunctionInfo {
            name,
            line: node.start_position().row as u32 + 1,
            complexity: self.calculate_complexity(node),
        })
    }

    fn extract_arrow_function(&self, node: &Node, source: &str) -> Option<FunctionInfo> {
        // For arrow functions, try to find the identifier they're assigned to
        let parent = node.parent()?;
        let name = match parent.kind() {
            "variable_declarator" => {
                let name_node = parent.child_by_field_name("name")?;
                self.get_node_text(&name_node, source)?
            }
            "assignment_expression" => {
                let left_node = parent.child_by_field_name("left")?;
                self.get_node_text(&left_node, source)?
            }
            "property" => {
                let key_node = parent.child_by_field_name("key")?;
                self.get_node_text(&key_node, source)?
            }
            _ => "anonymous".to_string(),
        };

        Some(FunctionInfo {
            name,
            line: node.start_position().row as u32 + 1,
            complexity: self.calculate_complexity(node),
        })
    }

    fn extract_method_definition(&self, node: &Node, source: &str) -> Option<FunctionInfo> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.get_node_text(&name_node, source)?;
        
        Some(FunctionInfo {
            name,
            line: node.start_position().row as u32 + 1,
            complexity: self.calculate_complexity(node),
        })
    }

    fn extract_function_expression(&self, node: &Node, source: &str) -> Option<FunctionInfo> {
        // Try to get the name from the function expression itself
        let name = if let Some(name_node) = node.child_by_field_name("name") {
            self.get_node_text(&name_node, source)?
        } else {
            // If no name, try to get it from the parent context
            let parent = node.parent()?;
            match parent.kind() {
                "variable_declarator" => {
                    let name_node = parent.child_by_field_name("name")?;
                    self.get_node_text(&name_node, source)?
                }
                "assignment_expression" => {
                    let left_node = parent.child_by_field_name("left")?;
                    self.get_node_text(&left_node, source)?
                }
                _ => "anonymous".to_string(),
            }
        };

        Some(FunctionInfo {
            name,
            line: node.start_position().row as u32 + 1,
            complexity: self.calculate_complexity(node),
        })
    }

    fn extract_classes(&self, tree: &Tree, source: &str) -> Vec<ClassInfo> {
        let mut classes = Vec::new();
        let root_node = tree.root_node();
        
        self.traverse_for_classes(&root_node, source, &mut classes);
        
        debug!("Extracted {} classes", classes.len());
        classes
    }

    fn traverse_for_classes(&self, node: &Node, source: &str, classes: &mut Vec<ClassInfo>) {
        if node.kind() == "class_declaration" {
            if let Some(class_info) = self.extract_class_declaration(node, source) {
                classes.push(class_info);
            }
        }

        // Recursively traverse child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_for_classes(&child, source, classes);
        }
    }

    fn extract_class_declaration(&self, node: &Node, source: &str) -> Option<ClassInfo> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.get_node_text(&name_node, source)?;
        
        Some(ClassInfo {
            name,
            line: node.start_position().row as u32 + 1,
        })
    }

    fn extract_imports(&self, tree: &Tree, source: &str) -> Vec<ImportInfo> {
        let mut imports = Vec::new();
        let root_node = tree.root_node();
        
        self.traverse_for_imports(&root_node, source, &mut imports);
        
        debug!("Extracted {} imports", imports.len());
        imports
    }

    fn traverse_for_imports(&self, node: &Node, source: &str, imports: &mut Vec<ImportInfo>) {
        match node.kind() {
            "import_statement" => {
                if let Some(import_info) = self.extract_import_statement(node, source) {
                    imports.push(import_info);
                }
            }
            "call_expression" => {
                // Handle require() calls
                if let Some(import_info) = self.extract_require_call(node, source) {
                    imports.push(import_info);
                }
            }
            _ => {}
        }

        // Recursively traverse child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_for_imports(&child, source, imports);
        }
    }

    fn extract_import_statement(&self, node: &Node, source: &str) -> Option<ImportInfo> {
        let source_node = node.child_by_field_name("source")?;
        let module = self.get_node_text(&source_node, source)?;
        
        // Remove quotes from the module name
        let module = module.trim_matches('"').trim_matches('\'').to_string();
        
        Some(ImportInfo {
            module,
            line: node.start_position().row as u32 + 1,
        })
    }

    fn extract_require_call(&self, node: &Node, source: &str) -> Option<ImportInfo> {
        let function_node = node.child_by_field_name("function")?;
        let function_text = self.get_node_text(&function_node, source)?;
        
        if function_text == "require" {
            let arguments_node = node.child_by_field_name("arguments")?;
            let mut cursor = arguments_node.walk();
            
            // Get the first argument (the module path)
            for child in arguments_node.children(&mut cursor) {
                if child.kind() == "string" {
                    let module = self.get_node_text(&child, source)?;
                    let module = module.trim_matches('"').trim_matches('\'').to_string();
                    
                    return Some(ImportInfo {
                        module,
                        line: node.start_position().row as u32 + 1,
                    });
                }
            }
        }
        
        None
    }

    fn calculate_complexity(&self, node: &Node) -> u32 {
        let mut complexity = 1; // Base complexity
        
        self.traverse_for_complexity(node, &mut complexity);
        
        complexity
    }

    fn traverse_for_complexity(&self, node: &Node, complexity: &mut u32) {
        match node.kind() {
            // Decision points that increase complexity
            "if_statement" | "while_statement" | "for_statement" | "for_in_statement" 
            | "for_of_statement" | "do_statement" | "switch_statement" | "catch_clause"
            | "conditional_expression" => {
                *complexity += 1;
            }
            // Logical operators
            "binary_expression" => {
                if let Some(operator) = node.child_by_field_name("operator") {
                    let mut cursor = operator.walk();
                    if let Some(op_node) = operator.children(&mut cursor).next() {
                        if matches!(op_node.kind(), "&&" | "||") {
                            *complexity += 1;
                        }
                    }
                }
            }
            _ => {}
        }

        // Recursively traverse child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_for_complexity(&child, complexity);
        }
    }

    fn get_node_text(&self, node: &Node, source: &str) -> Option<String> {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        
        if start_byte < source.len() && end_byte <= source.len() {
            Some(source[start_byte..end_byte].to_string())
        } else {
            None
        }
    }
}

impl Parser for JavaScriptParser {
    fn language(&self) -> EngineLanguage {
        EngineLanguage::JavaScript
    }

    #[instrument(skip(self, content))]
    fn parse(&self, content: &str) -> AnalysisResult<ParseResult> {
        let tree = self.parse_with_tree_sitter(content)?;
        
        let functions = self.extract_functions(&tree, content);
        let classes = self.extract_classes(&tree, content);
        let imports = self.extract_imports(&tree, content);
        
        Ok(ParseResult {
            language: EngineLanguage::JavaScript,
            functions,
            classes,
            imports,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let parser = JavaScriptParser::new().unwrap();
        let content = "function hello() { return 'world'; }";
        
        let result = parser.parse(content).unwrap();
        
        assert_eq!(result.functions.len(), 1);
        assert_eq!(result.functions[0].name, "hello");
        assert_eq!(result.functions[0].line, 1);
        assert_eq!(result.functions[0].complexity, 1);
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
    fn test_parse_class() {
        let parser = JavaScriptParser::new().unwrap();
        let content = r#"
            class Calculator {
                add(a, b) {
                    return a + b;
                }
            }
        "#;
        
        let result = parser.parse(content).unwrap();
        
        assert_eq!(result.classes.len(), 1);
        assert_eq!(result.classes[0].name, "Calculator");
        assert_eq!(result.functions.len(), 1);
        assert_eq!(result.functions[0].name, "add");
    }

    #[test]
    fn test_parse_imports() {
        let parser = JavaScriptParser::new().unwrap();
        let content = r#"
            import React from 'react';
            import { useState } from 'react';
            const fs = require('fs');
        "#;
        
        let result = parser.parse(content).unwrap();
        
        assert_eq!(result.imports.len(), 3);
        assert!(result.imports.iter().any(|i| i.module == "react"));
        assert!(result.imports.iter().any(|i| i.module == "fs"));
    }

    #[test]
    fn test_complexity_calculation() {
        let parser = JavaScriptParser::new().unwrap();
        let content = r#"
            function complexFunction(a, b, c) {
                if (a > 0) {
                    for (let i = 0; i < b; i++) {
                        if (i % 2 === 0) {
                            console.log(c);
                        }
                    }
                }
                return a && b || c;
            }
        "#;
        
        let result = parser.parse(content).unwrap();
        
        assert_eq!(result.functions.len(), 1);
        assert_eq!(result.functions[0].name, "complexFunction");
        // Base(1) + if(1) + for(1) + if(1) + &&(1) + ||(1) = 6
        assert_eq!(result.functions[0].complexity, 6);
    }

    #[test]
    fn test_syntax_error_handling() {
        let parser = JavaScriptParser::new().unwrap();
        let content = "function broken( { return 'incomplete'; }";
        
        // Should not panic, but may have parsing errors
        let result = parser.parse(content);
        
        // We expect this to either succeed with partial parsing or fail gracefully
        match result {
            Ok(_) => {
                // Partial parsing succeeded
            }
            Err(AnalysisError::ParseError { .. }) => {
                // Expected parse error
            }
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }
}