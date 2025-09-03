use rust_analysis_engine::{
    parser::{typescript::TypeScriptParser, Parser},
    types::Language,
};

#[test]
fn test_typescript_parser_creation() {
    let parser = TypeScriptParser::new();
    assert!(parser.is_ok());
    
    let parser = parser.unwrap();
    assert_eq!(parser.language(), Language::TypeScript);
}

#[test]
fn test_parse_typed_function() {
    let parser = TypeScriptParser::new().unwrap();
    let content = "function greet(name: string): string { return `Hello, ${name}!`; }";
    
    let result = parser.parse(content).unwrap();
    
    assert_eq!(result.language, Language::TypeScript);
    assert_eq!(result.functions.len(), 1);
    assert_eq!(result.functions[0].name, "greet");
    assert_eq!(result.functions[0].line, 1);
    assert_eq!(result.functions[0].complexity, 1);
}

#[test]
fn test_parse_interface() {
    let parser = TypeScriptParser::new().unwrap();
    let content = r#"
        interface User {
            id: number;
            name: string;
            email?: string;
            isActive: boolean;
            
            getName(): string;
            setEmail(email: string): void;
        }
    "#;
    
    let result = parser.parse(content).unwrap();
    
    // Interface should be counted as a class
    assert_eq!(result.classes.len(), 1);
    assert_eq!(result.classes[0].name, "User");
    
    // Method signatures should be counted as functions
    assert_eq!(result.functions.len(), 2);
    
    let function_names: Vec<&String> = result.functions.iter().map(|f| &f.name).collect();
    assert!(function_names.contains(&&"getName".to_string()));
    assert!(function_names.contains(&&"setEmail".to_string()));
}

#[test]
fn test_parse_generic_class() {
    let parser = TypeScriptParser::new().unwrap();
    let content = r#"
        class Repository<T> {
            private items: T[] = [];
            
            constructor(private name: string) {}
            
            add(item: T): void {
                this.items.push(item);
            }
            
            findById<K extends keyof T>(id: K, value: T[K]): T | undefined {
                return this.items.find(item => item[id] === value);
            }
            
            async save(item: T): Promise<T> {
                // Simulate async operation
                return new Promise(resolve => {
                    setTimeout(() => resolve(item), 100);
                });
            }
        }
    "#;
    
    let result = parser.parse(content).unwrap();
    
    assert_eq!(result.classes.len(), 1);
    assert_eq!(result.classes[0].name, "Repository");
    
    // Should find constructor, add, findById, and save methods
    assert_eq!(result.functions.len(), 4);
    
    let function_names: Vec<&String> = result.functions.iter().map(|f| &f.name).collect();
    assert!(function_names.contains(&&"constructor".to_string()));
    assert!(function_names.contains(&&"add".to_string()));
    assert!(function_names.contains(&&"findById".to_string()));
    assert!(function_names.contains(&&"save".to_string()));
}

#[test]
fn test_parse_typescript_imports_exports() {
    let parser = TypeScriptParser::new().unwrap();
    let content = r#"
        import React, { Component, useState } from 'react';
        import type { User, UserRole } from './types/user';
        import * as utils from './utils';
        import { default as lodash } from 'lodash';
        
        export { Calculator } from './calculator';
        export type { ApiResponse } from './api';
        export * from './constants';
        
        const fs = require('fs');
        const path = require('path');
    "#;
    
    let result = parser.parse(content).unwrap();
    
    assert_eq!(result.imports.len(), 8);
    
    let modules: Vec<&String> = result.imports.iter().map(|i| &i.module).collect();
    assert!(modules.contains(&&"react".to_string()));
    assert!(modules.contains(&&"./types/user".to_string()));
    assert!(modules.contains(&&"./utils".to_string()));
    assert!(modules.contains(&&"lodash".to_string()));
    assert!(modules.contains(&&"./calculator".to_string()));
    assert!(modules.contains(&&"./api".to_string()));
    assert!(modules.contains(&&"./constants".to_string()));
    assert!(modules.contains(&&"fs".to_string()));
}

#[test]
fn test_parse_arrow_functions_with_types() {
    let parser = TypeScriptParser::new().unwrap();
    let content = r#"
        const add = (a: number, b: number): number => a + b;
        
        const processUser = async (user: User): Promise<ProcessedUser> => {
            const result = await validateUser(user);
            return transformUser(result);
        };
        
        const createHandler = <T>(processor: (item: T) => void) => {
            return (items: T[]) => {
                items.forEach(processor);
            };
        };
        
        const complexArrow = (
            config: Config,
            options?: Options
        ): ((input: string) => Promise<Result>) => {
            return async (input: string) => {
                if (config.validate) {
                    await validateInput(input);
                }
                return processInput(input, options);
            };
        };
    "#;
    
    let result = parser.parse(content).unwrap();
    
    assert_eq!(result.functions.len(), 5); // add, processUser, createHandler, complexArrow, and the returned function
    
    let function_names: Vec<&String> = result.functions.iter().map(|f| &f.name).collect();
    assert!(function_names.contains(&&"add".to_string()));
    assert!(function_names.contains(&&"processUser".to_string()));
    assert!(function_names.contains(&&"createHandler".to_string()));
    assert!(function_names.contains(&&"complexArrow".to_string()));
}

#[test]
fn test_parse_enums_and_types() {
    let parser = TypeScriptParser::new().unwrap();
    let content = r#"
        enum UserRole {
            ADMIN = "admin",
            USER = "user",
            GUEST = "guest"
        }
        
        enum Status {
            PENDING,
            APPROVED,
            REJECTED
        }
        
        type ApiResponse<T> = {
            data: T;
            status: number;
            message: string;
        };
        
        type UserWithRole = User & {
            role: UserRole;
            permissions: string[];
        };
        
        interface ExtendedUser extends User {
            role: UserRole;
            lastLogin?: Date;
        }
    "#;
    
    let result = parser.parse(content).unwrap();
    
    // Interface should be counted as a class
    assert_eq!(result.classes.len(), 1);
    assert_eq!(result.classes[0].name, "ExtendedUser");
    
    // No functions in this example
    assert_eq!(result.functions.len(), 0);
}

#[test]
fn test_parse_complex_typescript_class() {
    let parser = TypeScriptParser::new().unwrap();
    let content = r#"
        abstract class BaseService<T extends Entity> {
            protected abstract repository: Repository<T>;
            
            constructor(protected logger: Logger) {}
            
            async findAll(options?: QueryOptions): Promise<T[]> {
                try {
                    this.logger.info('Finding all entities');
                    const entities = await this.repository.findAll(options);
                    return entities;
                } catch (error) {
                    this.logger.error('Failed to find entities', error);
                    throw error;
                }
            }
            
            async findById(id: string): Promise<T | null> {
                if (!id) {
                    throw new Error('ID is required');
                }
                
                try {
                    const entity = await this.repository.findById(id);
                    return entity;
                } catch (error) {
                    this.logger.error(`Failed to find entity with id ${id}`, error);
                    return null;
                }
            }
            
            protected validateEntity(entity: Partial<T>): boolean {
                if (!entity) {
                    return false;
                }
                
                // Complex validation logic
                for (const key in entity) {
                    if (entity.hasOwnProperty(key)) {
                        const value = entity[key];
                        if (value === null || value === undefined) {
                            continue;
                        }
                        
                        if (typeof value === 'string' && value.trim() === '') {
                            return false;
                        }
                    }
                }
                
                return true;
            }
        }
    "#;
    
    let result = parser.parse(content).unwrap();
    
    assert_eq!(result.classes.len(), 1);
    assert_eq!(result.classes[0].name, "BaseService");
    
    // Should find constructor, findAll, findById, and validateEntity
    assert_eq!(result.functions.len(), 4);
    
    let function_names: Vec<&String> = result.functions.iter().map(|f| &f.name).collect();
    assert!(function_names.contains(&&"constructor".to_string()));
    assert!(function_names.contains(&&"findAll".to_string()));
    assert!(function_names.contains(&&"findById".to_string()));
    assert!(function_names.contains(&&"validateEntity".to_string()));
    
    // Check complexity - validateEntity should have higher complexity
    let validate_fn = result.functions.iter().find(|f| f.name == "validateEntity").unwrap();
    assert!(validate_fn.complexity > 3); // Has multiple if statements and for loop
}

#[test]
fn test_parse_decorators_and_metadata() {
    let parser = TypeScriptParser::new().unwrap();
    let content = r#"
        @Entity('users')
        class User {
            @PrimaryGeneratedColumn()
            id: number;
            
            @Column({ length: 100 })
            name: string;
            
            @Column({ nullable: true })
            email?: string;
            
            @CreateDateColumn()
            createdAt: Date;
            
            @BeforeInsert()
            generateId() {
                if (!this.id) {
                    this.id = Math.random();
                }
            }
            
            @AfterLoad()
            async loadRelations() {
                // Load related data
                const relations = await this.getRelations();
                return relations;
            }
        }
        
        @Injectable()
        class UserService {
            constructor(
                @Inject('USER_REPOSITORY') private userRepo: Repository<User>
            ) {}
            
            @Transactional()
            async createUser(userData: CreateUserDto): Promise<User> {
                const user = new User();
                Object.assign(user, userData);
                return await this.userRepo.save(user);
            }
        }
    "#;
    
    let result = parser.parse(content).unwrap();
    
    assert_eq!(result.classes.len(), 2);
    
    let class_names: Vec<&String> = result.classes.iter().map(|c| &c.name).collect();
    assert!(class_names.contains(&&"User".to_string()));
    assert!(class_names.contains(&&"UserService".to_string()));
    
    // Should find generateId, loadRelations, constructor, and createUser
    assert_eq!(result.functions.len(), 4);
    
    let function_names: Vec<&String> = result.functions.iter().map(|f| &f.name).collect();
    assert!(function_names.contains(&&"generateId".to_string()));
    assert!(function_names.contains(&&"loadRelations".to_string()));
    assert!(function_names.contains(&&"constructor".to_string()));
    assert!(function_names.contains(&&"createUser".to_string()));
}

#[test]
fn test_typescript_complexity_calculation() {
    let parser = TypeScriptParser::new().unwrap();
    let content = r#"
        function complexTypeScriptFunction<T extends User>(
            users: T[],
            filter: (user: T) => boolean,
            options: ProcessOptions = {}
        ): Promise<ProcessResult<T[]>> {
            return new Promise((resolve, reject) => {
                try {
                    const results: T[] = [];
                    
                    for (const user of users) {
                        if (filter(user)) {
                            if (user.isActive) {
                                if (options.includeInactive || user.lastLogin) {
                                    results.push(user);
                                } else if (options.strict) {
                                    continue;
                                } else {
                                    results.push({ ...user, filtered: true });
                                }
                            } else if (options.includeInactive) {
                                results.push(user);
                            }
                        } else if (options.includeFiltered) {
                            results.push({ ...user, excluded: true });
                        }
                    }
                    
                    const finalResults = results.length > 0 ? results : [];
                    resolve({
                        success: true,
                        data: finalResults,
                        count: finalResults.length
                    });
                } catch (error) {
                    reject(error);
                }
            });
        }
    "#;
    
    let result = parser.parse(content).unwrap();
    
    assert_eq!(result.functions.len(), 1);
    assert_eq!(result.functions[0].name, "complexTypeScriptFunction");
    
    // Should have high complexity due to multiple nested if statements
    // Base(1) + for(1) + if(1) + if(1) + if(1) + else if(1) + else(1) + else if(1) + else if(1) + ternary(1) + try/catch(1) = 11
    assert!(result.functions[0].complexity >= 8);
}

#[test]
fn test_typescript_syntax_error_handling() {
    let parser = TypeScriptParser::new().unwrap();
    
    // Test with TypeScript syntax error
    let content = "function broken<T extends { return 'incomplete'; }";
    
    // Should not panic, but may have parsing errors
    let result = parser.parse(content);
    
    // We expect this to either succeed with partial parsing or fail gracefully
    match result {
        Ok(parse_result) => {
            // Partial parsing succeeded - this is acceptable
            println!("Partial TypeScript parsing succeeded with {} functions", parse_result.functions.len());
        }
        Err(e) => {
            // Expected parse error - this is also acceptable
            println!("TypeScript parse error as expected: {}", e);
        }
    }
}

#[test]
fn test_mixed_javascript_typescript_features() {
    let parser = TypeScriptParser::new().unwrap();
    let content = r#"
        // Regular JavaScript function
        function regularFunction(a, b) {
            return a + b;
        }
        
        // TypeScript typed function
        function typedFunction(a: number, b: number): number {
            return a + b;
        }
        
        // JavaScript class
        class JSClass {
            constructor(value) {
                this.value = value;
            }
        }
        
        // TypeScript class with types
        class TSClass {
            private value: number;
            
            constructor(value: number) {
                this.value = value;
            }
            
            getValue(): number {
                return this.value;
            }
        }
        
        // Arrow functions
        const jsArrow = (x) => x * 2;
        const tsArrow = (x: number): number => x * 2;
    "#;
    
    let result = parser.parse(content).unwrap();
    
    // Should parse both JavaScript and TypeScript syntax
    assert_eq!(result.classes.len(), 2);
    assert_eq!(result.functions.len(), 6); // regularFunction, typedFunction, 2 constructors, getValue, jsArrow, tsArrow
    
    let class_names: Vec<&String> = result.classes.iter().map(|c| &c.name).collect();
    assert!(class_names.contains(&&"JSClass".to_string()));
    assert!(class_names.contains(&&"TSClass".to_string()));
    
    let function_names: Vec<&String> = result.functions.iter().map(|f| &f.name).collect();
    assert!(function_names.contains(&&"regularFunction".to_string()));
    assert!(function_names.contains(&&"typedFunction".to_string()));
    assert!(function_names.contains(&&"getValue".to_string()));
    assert!(function_names.contains(&&"jsArrow".to_string()));
    assert!(function_names.contains(&&"tsArrow".to_string()));
}