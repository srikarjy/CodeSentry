use std::collections::HashMap;
use tracing::{debug, warn, instrument};
use tree_sitter::{Language, Node, Parser as TSParser, Tree};

use crate::{
    error::{AnalysisError, AnalysisResult},
    types::{Language as EngineLanguage, Location},
};

use super::{Parser, ParseResult, FunctionInfo, ClassInfo, ImportInfo};

#[derive(Debug)]
pub struct TypeInfo {
    pub name: String,
    pub line: u32,
    pub kind: TypeKind,
}

#[derive(Debug)]
pub enum TypeKind {
    Interface,
    TypeAlias,
    Enum,
    Generic,
}

#[derive(Debug)]
pub struct InterfaceInfo {
    pub name: String,
    pub line: u32,
    pub methods: Vec<String>,
    pub properties: Vec<String>,
    pub extends: Vec<String>,
}

pub struct TypeScriptParser {
    language: Language,
}

impl TypeScriptParser {
    pub fn new() -> AnalysisResult<Self> {
        let language = tree_sitter_typescript::language_typescript();
        Ok(Self { language })
    }

    #[instrument(skip(self, content))]
    fn parse_with_tree_sitter(&self, content: &str) -> AnalysisResult<Tree> {
        let mut parser = TSParser::new();
        parser.set_language(self.language).map_err(|e| {
            AnalysisError::ConfigError {
                message: format!("Failed to set TypeScript language: {}", e),
            }
        })?;

        // Set timeout to 7 seconds (TypeScript can be more complex)
        parser.set_timeout_micros(7_000_000);

        let tree = parser.parse(content, None).ok_or_else(|| {
            AnalysisError::ParseError {
                message: "Failed to parse TypeScript content".to_string(),
                line: 1,
            }
        })?;

        // Check for syntax errors
        if tree.root_node().has_error() {
            warn!("TypeScript parsing completed with syntax errors");
            // Continue with partial parsing rather than failing completely
        }

        debug!(
            "TypeScript parsing completed: {} nodes",
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
            "method_definition" | "method_signature" => {
                if let Some(function_info) = self.extract_method_definition(node, source) {
                    functions.push(function_info);
                }
            }
            "function_expression" => {
                if let Some(function_info) = self.extract_function_expression(node, source) {
                    functions.push(function_info);
                }
            }
            "function_signature" => {
                if let Some(function_info) = self.extract_function_signature(node, source) {
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
        let name = if let Some(name_node) = node.child_by_field_name("name") {
            self.get_node_text(&name_node, source)?
        } else {
            // Try to get name from parent context
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

    fn extract_function_signature(&self, node: &Node, source: &str) -> Option<FunctionInfo> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.get_node_text(&name_node, source)?;
        
        Some(FunctionInfo {
            name,
            line: node.start_position().row as u32 + 1,
            complexity: 1, // Function signatures have minimal complexity
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
        match node.kind() {
            "class_declaration" => {
                if let Some(class_info) = self.extract_class_declaration(node, source) {
                    classes.push(class_info);
                }
            }
            "interface_declaration" => {
                if let Some(class_info) = self.extract_interface_as_class(node, source) {
                    classes.push(class_info);
                }
            }
            _ => {}
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

    fn extract_interface_as_class(&self, node: &Node, source: &str) -> Option<ClassInfo> {
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
            "export_statement" => {
                if let Some(import_info) = self.extract_export_statement(node, source) {
                    imports.push(import_info);
                }
            }
            "call_expression" => {
                // Handle require() calls and dynamic imports
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

    fn extract_export_statement(&self, node: &Node, source: &str) -> Option<ImportInfo> {
        // Handle export ... from '...' statements
        if let Some(source_node) = node.child_by_field_name("source") {
            let module = self.get_node_text(&source_node, source)?;
            let module = module.trim_matches('"').trim_matches('\'').to_string();
            
            Some(ImportInfo {
                module,
                line: node.start_position().row as u32 + 1,
            })
        } else {
            None
        }
    }

    fn extract_require_call(&self, node: &Node, source: &str) -> Option<ImportInfo> {
        let function_node = node.child_by_field_name("function")?;
        let function_text = self.get_node_text(&function_node, source)?;
        
        if function_text == "require" || function_text == "import" {
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

    fn extract_interfaces(&self, tree: &Tree, source: &str) -> Vec<InterfaceInfo> {
        let mut interfaces = Vec::new();
        let root_node = tree.root_node();
        
        self.traverse_for_interfaces(&root_node, source, &mut interfaces);
        
        debug!("Extracted {} interfaces", interfaces.len());
        interfaces
    }

    fn traverse_for_interfaces(&self, node: &Node, source: &str, interfaces: &mut Vec<InterfaceInfo>) {
        if node.kind() == "interface_declaration" {
            if let Some(interface_info) = self.extract_interface_declaration(node, source) {
                interfaces.push(interface_info);
            }
        }

        // Recursively traverse child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_for_interfaces(&child, source, interfaces);
        }
    }

    fn extract_interface_declaration(&self, node: &Node, source: &str) -> Option<InterfaceInfo> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.get_node_text(&name_node, source)?;
        
        let mut methods = Vec::new();
        let mut properties = Vec::new();
        let mut extends = Vec::new();

        // Extract heritage clause (extends)
        if let Some(heritage_node) = node.child_by_field_name("heritage") {
            let mut cursor = heritage_node.walk();
            for child in heritage_node.children(&mut cursor) {
                if child.kind() == "extends_clause" {
                    let mut extends_cursor = child.walk();
                    for extends_child in child.children(&mut extends_cursor) {
                        if extends_child.kind() == "type_identifier" {
                            if let Some(extends_name) = self.get_node_text(&extends_child, source) {
                                extends.push(extends_name);
                            }
                        }
                    }
                }
            }
        }

        // Extract body (methods and properties)
        if let Some(body_node) = node.child_by_field_name("body") {
            let mut cursor = body_node.walk();
            for child in body_node.children(&mut cursor) {
                match child.kind() {
                    "method_signature" => {
                        if let Some(method_name_node) = child.child_by_field_name("name") {
                            if let Some(method_name) = self.get_node_text(&method_name_node, source) {
                                methods.push(method_name);
                            }
                        }
                    }
                    "property_signature" => {
                        if let Some(prop_name_node) = child.child_by_field_name("name") {
                            if let Some(prop_name) = self.get_node_text(&prop_name_node, source) {
                                properties.push(prop_name);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Some(InterfaceInfo {
            name,
            line: node.start_position().row as u32 + 1,
            methods,
            properties,
            extends,
        })
    }

    fn extract_types(&self, tree: &Tree, source: &str) -> Vec<TypeInfo> {
        let mut types = Vec::new();
        let root_node = tree.root_node();
        
        self.traverse_for_types(&root_node, source, &mut types);
        
        debug!("Extracted {} type definitions", types.len());
        types
    }

    fn traverse_for_types(&self, node: &Node, source: &str, types: &mut Vec<TypeInfo>) {
        match node.kind() {
            "type_alias_declaration" => {
                if let Some(type_info) = self.extract_type_alias(node, source) {
                    types.push(type_info);
                }
            }
            "enum_declaration" => {
                if let Some(type_info) = self.extract_enum_declaration(node, source) {
                    types.push(type_info);
                }
            }
            _ => {}
        }

        // Recursively traverse child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_for_types(&child, source, types);
        }
    }

    fn extract_type_alias(&self, node: &Node, source: &str) -> Option<TypeInfo> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.get_node_text(&name_node, source)?;
        
        Some(TypeInfo {
            name,
            line: node.start_position().row as u32 + 1,
            kind: TypeKind::TypeAlias,
        })
    }

    fn extract_enum_declaration(&self, node: &Node, source: &str) -> Option<TypeInfo> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.get_node_text(&name_node, source)?;
        
        Some(TypeInfo {
            name,
            line: node.start_position().row as u32 + 1,
            kind: TypeKind::Enum,
        })
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

impl Parser for TypeScriptParser {
    fn language(&self) -> EngineLanguage {
        EngineLanguage::TypeScript
    }

    #[instrument(skip(self, content))]
    fn parse(&self, content: &str) -> AnalysisResult<ParseResult> {
        let tree = self.parse_with_tree_sitter(content)?;
        
        let functions = self.extract_functions(&tree, content);
        let classes = self.extract_classes(&tree, content);
        let imports = self.extract_imports(&tree, content);
        
        // TypeScript-specific extractions
        let _interfaces = self.extract_interfaces(&tree, content);
        let _types = self.extract_types(&tree, content);
        
        Ok(ParseResult {
            language: EngineLanguage::TypeScript,
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
    fn test_parse_typescript_function() {
        let parser = TypeScriptParser::new().unwrap();
        let content = "function greet(name: string): string { return `Hello, ${name}!`; }";
        
        let result = parser.parse(content).unwrap();
        
        assert_eq!(result.functions.len(), 1);
        assert_eq!(result.functions[0].name, "greet");
        assert_eq!(result.functions[0].line, 1);
        assert_eq!(result.functions[0].complexity, 1);
    }

    #[test]
    fn test_parse_typescript_interface() {
        let parser = TypeScriptParser::new().unwrap();
        let content = r#"
            interface User {
                id: number;
                name: string;
                email?: string;
                getName(): string;
            }
        "#;
        
        let result = parser.parse(content).unwrap();
        
        // Interface should be counted as a class
        assert_eq!(result.classes.len(), 1);
        assert_eq!(result.classes[0].name, "User");
        
        // Method signature should be counted as a function
        assert_eq!(result.functions.len(), 1);
        assert_eq!(result.functions[0].name, "getName");
    }

    #[test]
    fn test_parse_typescript_class_with_types() {
        let parser = TypeScriptParser::new().unwrap();
        let content = r#"
            class Calculator<T extends number> {
                private value: T;
                
                constructor(initialValue: T) {
                    this.value = initialValue;
                }
                
                add(operand: T): T {
                    return this.value + operand;
                }
                
                async fetchData(): Promise<T[]> {
                    const response = await fetch('/api/data');
                    return response.json();
                }
            }
        "#;
        
        let result = parser.parse(content).unwrap();
        
        assert_eq!(result.classes.len(), 1);
        assert_eq!(result.classes[0].name, "Calculator");
        
        // Should find constructor, add, and fetchData methods
        assert_eq!(result.functions.len(), 3);
        
        let function_names: Vec<&String> = result.functions.iter().map(|f| &f.name).collect();
        assert!(function_names.contains(&&"constructor".to_string()));
        assert!(function_names.contains(&&"add".to_string()));
        assert!(function_names.contains(&&"fetchData".to_string()));
    }

    #[test]
    fn test_parse_typescript_imports() {
        let parser = TypeScriptParser::new().unwrap();
        let content = r#"
            import React, { Component } from 'react';
            import type { User } from './types';
            import * as utils from './utils';
            export { Calculator } from './calculator';
            const fs = require('fs');
        "#;
        
        let result = parser.parse(content).unwrap();
        
        assert_eq!(result.imports.len(), 5);
        
        let modules: Vec<&String> = result.imports.iter().map(|i| &i.module).collect();
        assert!(modules.contains(&&"react".to_string()));
        assert!(modules.contains(&&"./types".to_string()));
        assert!(modules.contains(&&"./utils".to_string()));
        assert!(modules.contains(&&"./calculator".to_string()));
        assert!(modules.contains(&&"fs".to_string()));
    }

    #[test]
    fn test_parse_typescript_generics() {
        let parser = TypeScriptParser::new().unwrap();
        let content = r#"
            function identity<T>(arg: T): T {
                return arg;
            }
            
            interface Repository<T> {
                findById(id: string): Promise<T | null>;
                save(entity: T): Promise<T>;
            }
            
            class GenericClass<T, U extends string> {
                process(input: T): U {
                    return input as unknown as U;
                }
            }
        "#;
        
        let result = parser.parse(content).unwrap();
        
        // Should find function, interface methods, and class method
        assert!(result.functions.len() >= 3);
        assert_eq!(result.classes.len(), 2); // Interface and class
        
        let function_names: Vec<&String> = result.functions.iter().map(|f| &f.name).collect();
        assert!(function_names.contains(&&"identity".to_string()));
        assert!(function_names.contains(&&"process".to_string()));
    }

    #[test]
    fn test_parse_typescript_enums_and_types() {
        let parser = TypeScriptParser::new().unwrap();
        let content = r#"
            enum Color {
                Red = "red",
                Green = "green",
                Blue = "blue"
            }
            
            type Status = "pending" | "completed" | "failed";
            
            type UserWithStatus = User & {
                status: Status;
                color: Color;
            };
        "#;
        
        let result = parser.parse(content).unwrap();
        
        // Enums and types don't count as classes in our current implementation
        // but they are parsed successfully
        assert!(result.language == EngineLanguage::TypeScript);
    }

    #[test]
    fn test_typescript_complexity_with_types() {
        let parser = TypeScriptParser::new().unwrap();
        let content = r#"
            function processUser<T extends User>(
                user: T, 
                options: ProcessOptions
            ): Promise<ProcessResult<T>> {
                if (user.isActive) {
                    for (const permission of user.permissions) {
                        if (permission.type === 'admin') {
                            return Promise.resolve({
                                success: true,
                                data: user
                            });
                        } else if (permission.type === 'user') {
                            return Promise.resolve({
                                success: true,
                                data: { ...user, limited: true }
                            });
                        }
                    }
                } else {
                    throw new Error('User is not active');
                }
                
                return Promise.reject(new Error('No valid permissions'));
            }
        "#;
        
        let result = parser.parse(content).unwrap();
        
        assert_eq!(result.functions.len(), 1);
        assert_eq!(result.functions[0].name, "processUser");
        // Should have complexity > 1 due to if statements and for loop
        assert!(result.functions[0].complexity > 3);
    }
}