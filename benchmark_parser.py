#!/usr/bin/env python3
"""
Benchmark script for the JavaScript parser performance.
Tests parsing speed against target of 100ms per 1K LOC.
"""

import requests
import time
import json
from typing import List, Dict, Any

BASE_URL = "http://localhost:8080"

def generate_javascript_code(lines: int) -> str:
    """Generate JavaScript code with approximately the specified number of lines."""
    code_parts = []
    
    # Add imports
    code_parts.extend([
        "import React from 'react';",
        "import { useState, useEffect } from 'react';",
        "const fs = require('fs');",
        "const path = require('path');",
        ""
    ])
    
    current_lines = 5
    function_count = 0
    
    while current_lines < lines:
        if function_count % 5 == 0:
            # Add a class
            class_code = f"""
class TestClass{function_count // 5} {{
    constructor() {{
        this.value = {function_count};
        this.name = 'test{function_count}';
    }}
    
    method{function_count}(a, b, c) {{
        if (a > 0) {{
            for (let i = 0; i < b; i++) {{
                if (i % 2 === 0) {{
                    this.value += c;
                }} else if (i % 3 === 0) {{
                    this.value -= c;
                }} else {{
                    this.value *= 2;
                }}
            }}
        }} else if (a < 0) {{
            while (b > 0) {{
                this.value += a;
                b--;
            }}
        }}
        return this.value && a || b;
    }}
    
    get getValue() {{
        return this.value;
    }}
}}
"""
            code_parts.append(class_code)
            current_lines += len(class_code.strip().split('\n'))
        else:
            # Add a function
            func_code = f"""
function testFunction{function_count}(x, y, z) {{
    let result = 0;
    
    if (x > 0) {{
        result += x;
    }} else if (x < 0) {{
        result -= x;
    }} else {{
        result = 1;
    }}
    
    for (let i = 0; i < y; i++) {{
        if (i % 2 === 0) {{
            result *= 2;
        }} else {{
            result += z;
        }}
    }}
    
    switch (result % 4) {{
        case 0:
            result += 10;
            break;
        case 1:
            result += 20;
            break;
        case 2:
            result += 30;
            break;
        default:
            result += 40;
    }}
    
    return result;
}}

const arrow{function_count} = async (a, b) => {{
    const data = await fetch(`/api/data/${{a}}`);
    return data.json();
}};
"""
            code_parts.append(func_code)
            current_lines += len(func_code.strip().split('\n'))
        
        function_count += 1
        
        # Add some variable declarations
        if current_lines < lines - 5:
            code_parts.extend([
                f"const variable{function_count} = 'test value {function_count}';",
                f"let counter{function_count} = {function_count};",
                f"var legacy{function_count} = {function_count * 2};",
                ""
            ])
            current_lines += 4
    
    return '\n'.join(code_parts)

def generate_typescript_code(lines: int) -> str:
    """Generate TypeScript code with approximately the specified number of lines."""
    code_parts = []
    
    # Add imports with types
    code_parts.extend([
        "import React, { Component } from 'react';",
        "import type { User, ApiResponse } from './types';",
        "import * as utils from './utils';",
        "const fs = require('fs');",
        ""
    ])
    
    current_lines = 5
    function_count = 0
    
    while current_lines < lines:
        if function_count % 4 == 0:
            # Add an interface
            interface_code = f"""
interface Entity{function_count // 4} {{
    id: number;
    name: string;
    createdAt: Date;
    updatedAt?: Date;
    
    getName(): string;
    setName(name: string): void;
    validate(): boolean;
}}
"""
            code_parts.append(interface_code)
            current_lines += len(interface_code.strip().split('\n'))
        elif function_count % 4 == 1:
            # Add a generic class
            class_code = f"""
class Repository<T extends Entity{function_count // 4}> {{
    private items: T[] = [];
    
    constructor(private name: string) {{}}
    
    async add(item: T): Promise<T> {{
        if (!item.name || item.name.trim() === '') {{
            throw new Error('Name is required');
        }}
        
        for (const existing of this.items) {{
            if (existing.id === item.id) {{
                throw new Error('Item already exists');
            }}
        }}
        
        this.items.push(item);
        return item;
    }}
    
    findById<K extends keyof T>(id: K, value: T[K]): T | undefined {{
        return this.items.find(item => item[id] === value);
    }}
    
    async update(id: number, updates: Partial<T>): Promise<T | null> {{
        const index = this.items.findIndex(item => item.id === id);
        if (index === -1) {{
            return null;
        }}
        
        const updated = {{ ...this.items[index], ...updates }};
        this.items[index] = updated;
        return updated;
    }}
}}
"""
            code_parts.append(class_code)
            current_lines += len(class_code.strip().split('\n'))
        else:
            # Add typed functions
            func_code = f"""
async function processEntity{function_count}<T extends Entity{function_count // 4}>(
    entity: T,
    options: ProcessOptions = {{}}
): Promise<ProcessResult<T>> {{
    try {{
        if (!entity.validate()) {{
            throw new Error('Invalid entity');
        }}
        
        const result: ProcessResult<T> = {{
            success: false,
            data: entity,
            errors: []
        }};
        
        if (options.strict) {{
            for (const key in entity) {{
                if (entity.hasOwnProperty(key)) {{
                    const value = entity[key];
                    if (value === null || value === undefined) {{
                        result.errors.push(`Missing value for ${{key}}`);
                    }}
                }}
            }}
        }}
        
        if (result.errors.length === 0) {{
            result.success = true;
        }}
        
        return result;
    }} catch (error) {{
        return {{
            success: false,
            data: entity,
            errors: [error.message]
        }};
    }}
}}

const createValidator{function_count} = <T>(
    validator: (item: T) => boolean
): ((items: T[]) => T[]) => {{
    return (items: T[]): T[] => {{
        return items.filter(validator);
    }};
}};
"""
            code_parts.append(func_code)
            current_lines += len(func_code.strip().split('\n'))
        
        function_count += 1
        
        # Add type definitions
        if current_lines < lines - 10:
            code_parts.extend([
                f"type ProcessOptions = {{ strict?: boolean; timeout?: number; }};",
                f"type ProcessResult<T> = {{ success: boolean; data: T; errors: string[]; }};",
                f"enum Status{function_count} {{ PENDING = 'pending', COMPLETED = 'completed' }}",
                ""
            ])
            current_lines += 4
    
    return '\n'.join(code_parts)

def benchmark_parsing(file_sizes: List[int]) -> Dict[str, Any]:
    """Benchmark parsing performance for different file sizes."""
    results = {
        'benchmarks': [],
        'summary': {}
    }
    
    print("🚀 Starting JavaScript Parser Benchmarks")
    print("=" * 50)
    
    for size in file_sizes:
        # Test both JavaScript and TypeScript
        for lang, generator, extension in [
            ("JavaScript", generate_javascript_code, "js"),
            ("TypeScript", generate_typescript_code, "ts")
        ]:
            print(f"\n📊 Testing {size} lines of {lang}...")
            
            # Generate test code
            code = generator(size)
            actual_lines = len(code.split('\n'))
            
            # Prepare request
            request_data = {
                "files": [
                    {
                        "name": f"test_{size}_lines.{extension}",
                        "content": code
                    }
                ],
            "rules": {
                "complexity_threshold": 10,
                "max_function_length": 50,
                "enable_security_rules": True
            }
        }
        
        # Make request and measure time
        start_time = time.time()
        
        try:
            response = requests.post(
                f"{BASE_URL}/analyze",
                json=request_data,
                timeout=30
            )
            
            end_time = time.time()
            duration_ms = (end_time - start_time) * 1000
            
            if response.status_code == 200:
                result_data = response.json()
                
                # Extract metrics
                file_result = result_data['results'][0]
                metrics = file_result['metrics']
                
                benchmark_result = {
                    'language': lang,
                    'target_lines': size,
                    'actual_lines': actual_lines,
                    'duration_ms': duration_ms,
                    'lines_per_second': actual_lines / (duration_ms / 1000),
                    'ms_per_1k_lines': (duration_ms / actual_lines) * 1000,
                    'functions_found': metrics['functions_count'],
                    'classes_found': metrics['classes_count'],
                    'complexity_score': metrics['complexity_score'],
                    'findings_count': len(file_result['findings']),
                    'success': True
                }
                
                print(f"  ✅ Parsed {actual_lines} {lang} lines in {duration_ms:.1f}ms")
                print(f"  📈 {benchmark_result['lines_per_second']:.0f} lines/second")
                print(f"  🎯 {benchmark_result['ms_per_1k_lines']:.1f}ms per 1K lines")
                print(f"  🔍 Found: {metrics['functions_count']} functions, {metrics['classes_count']} classes")
                print(f"  ⚠️  {len(file_result['findings'])} findings")
                
                # Check if meets performance target (100ms per 1K LOC)
                if benchmark_result['ms_per_1k_lines'] <= 100:
                    print(f"  🎉 MEETS TARGET (≤100ms per 1K lines)")
                else:
                    print(f"  ❌ EXCEEDS TARGET (>100ms per 1K lines)")
                
            else:
                benchmark_result = {
                    'language': lang,
                    'target_lines': size,
                    'actual_lines': actual_lines,
                    'duration_ms': duration_ms,
                    'error': f"HTTP {response.status_code}: {response.text}",
                    'success': False
                }
                print(f"  ❌ {lang} Failed: HTTP {response.status_code}")
                
            except Exception as e:
                benchmark_result = {
                    'language': lang,
                    'target_lines': size,
                    'actual_lines': actual_lines,
                    'error': str(e),
                    'success': False
                }
                print(f"  ❌ {lang} Error: {e}")
            
            results['benchmarks'].append(benchmark_result)
    
    # Calculate summary statistics
    successful_benchmarks = [b for b in results['benchmarks'] if b['success']]
    
    if successful_benchmarks:
        avg_ms_per_1k = sum(b['ms_per_1k_lines'] for b in successful_benchmarks) / len(successful_benchmarks)
        avg_lines_per_sec = sum(b['lines_per_second'] for b in successful_benchmarks) / len(successful_benchmarks)
        max_lines_tested = max(b['actual_lines'] for b in successful_benchmarks)
        
        results['summary'] = {
            'total_tests': len(file_sizes),
            'successful_tests': len(successful_benchmarks),
            'average_ms_per_1k_lines': avg_ms_per_1k,
            'average_lines_per_second': avg_lines_per_sec,
            'max_lines_tested': max_lines_tested,
            'meets_target': avg_ms_per_1k <= 100
        }
        
        print(f"\n📊 BENCHMARK SUMMARY")
        print(f"=" * 30)
        print(f"Tests completed: {len(successful_benchmarks)}/{len(file_sizes)}")
        print(f"Average performance: {avg_ms_per_1k:.1f}ms per 1K lines")
        print(f"Average throughput: {avg_lines_per_sec:.0f} lines/second")
        print(f"Largest file tested: {max_lines_tested:,} lines")
        
        if results['summary']['meets_target']:
            print(f"🎉 PERFORMANCE TARGET MET! (≤100ms per 1K lines)")
        else:
            print(f"❌ Performance target not met (>100ms per 1K lines)")
    
    return results

def main():
    """Run the benchmark suite."""
    # Check if server is running
    try:
        response = requests.get(f"{BASE_URL}/health", timeout=5)
        if response.status_code != 200:
            print(f"❌ Server health check failed: HTTP {response.status_code}")
            return
    except Exception as e:
        print(f"❌ Cannot connect to server at {BASE_URL}")
        print(f"💡 Make sure the server is running: docker-compose up --build")
        return
    
    # Run benchmarks with different file sizes
    file_sizes = [100, 500, 1000, 2000, 5000]  # Lines of code
    
    results = benchmark_parsing(file_sizes)
    
    # Save results to file
    with open('benchmark_results.json', 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"\n💾 Results saved to benchmark_results.json")
    
    # Week 2 deliverable check
    successful_tests = [b for b in results['benchmarks'] if b['success']]
    if len(successful_tests) >= 3 and all(b['actual_lines'] >= 1000 for b in successful_tests[-3:]):
        print(f"\n🎯 Week 2 Deliverable Status: ✅ READY")
        print(f"   Successfully parsed files with 1K+ lines")
        print(f"   Performance metrics collected")
        print(f"   Ready to test against real GitHub repositories")
    else:
        print(f"\n🎯 Week 2 Deliverable Status: ⚠️  IN PROGRESS")
        print(f"   Need more successful tests with larger files")

if __name__ == "__main__":
    main()