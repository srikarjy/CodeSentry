#!/usr/bin/env python3
"""
Validation script to check the Rust Analysis Engine project structure
and verify that all required files are present for Week 1 deliverable.
"""

import os
import json
from pathlib import Path

def check_file_exists(path, description):
    """Check if a file exists and print status."""
    if os.path.exists(path):
        print(f"✅ {description}: {path}")
        return True
    else:
        print(f"❌ {description}: {path} (MISSING)")
        return False

def check_file_contains(path, content, description):
    """Check if a file contains specific content."""
    try:
        with open(path, 'r') as f:
            file_content = f.read()
            if content in file_content:
                print(f"✅ {description}")
                return True
            else:
                print(f"❌ {description} (MISSING)")
                return False
    except FileNotFoundError:
        print(f"❌ {description} - File not found: {path}")
        return False

def validate_json_structure(path, expected_keys, description):
    """Validate JSON file structure."""
    try:
        with open(path, 'r') as f:
            data = json.load(f)
            missing_keys = [key for key in expected_keys if key not in data]
            if not missing_keys:
                print(f"✅ {description}")
                return True
            else:
                print(f"❌ {description} - Missing keys: {missing_keys}")
                return False
    except (FileNotFoundError, json.JSONDecodeError) as e:
        print(f"❌ {description} - Error: {e}")
        return False

def main():
    print("🔍 Validating Rust Analysis Engine Project Structure")
    print("=" * 55)
    
    all_checks_passed = True
    
    # Core project files
    checks = [
        ("Cargo.toml", "Cargo configuration"),
        ("src/main.rs", "Main entry point"),
        ("src/lib.rs", "Library root"),
        ("src/error.rs", "Error handling"),
        ("src/types.rs", "Type definitions"),
        ("src/server/mod.rs", "HTTP server"),
        ("src/analysis/mod.rs", "Analysis engine"),
        ("src/parser/mod.rs", "Parser registry"),
        ("tests/integration_test.rs", "Integration tests"),
        ("README.md", "Documentation"),
        ("Dockerfile", "Container configuration"),
        ("docker-compose.yml", "Docker Compose"),
        ("test_api.sh", "API test script"),
    ]
    
    print("\n📁 File Structure Check:")
    for file_path, description in checks:
        if not check_file_exists(file_path, description):
            all_checks_passed = False
    
    # Content checks
    print("\n📝 Content Validation:")
    content_checks = [
        ("src/server/mod.rs", "/analyze", "Analyze endpoint"),
        ("src/server/mod.rs", "/health", "Health endpoint"),
        ("src/server/mod.rs", "axum", "Axum framework"),
        ("src/types.rs", "AnalysisRequest", "Request type"),
        ("src/types.rs", "AnalysisResponse", "Response type"),
        ("src/error.rs", "AnalysisError", "Error type"),
        ("tests/integration_test.rs", "test_analyze_endpoint", "Analysis test"),
        ("Dockerfile", "rust:1.75", "Rust base image"),
    ]
    
    for file_path, content, description in content_checks:
        if not check_file_contains(file_path, content, description):
            all_checks_passed = False
    
    # Cargo.toml validation
    print("\n📦 Dependency Check:")
    cargo_deps = ["axum", "tokio", "serde", "tracing", "thiserror"]
    for dep in cargo_deps:
        if not check_file_contains("Cargo.toml", dep, f"Dependency: {dep}"):
            all_checks_passed = False
    
    # API structure validation
    print("\n🌐 API Structure Check:")
    api_checks = [
        ("src/types.rs", "pub struct AnalysisRequest", "Request structure"),
        ("src/types.rs", "pub struct AnalysisResponse", "Response structure"),
        ("src/types.rs", "pub struct Finding", "Finding structure"),
        ("src/types.rs", "pub enum Severity", "Severity enum"),
        ("src/types.rs", "pub enum Language", "Language enum"),
    ]
    
    for file_path, content, description in api_checks:
        if not check_file_contains(file_path, content, description):
            all_checks_passed = False
    
    print("\n" + "=" * 55)
    if all_checks_passed:
        print("🎉 All checks passed! Week 1 structure is complete.")
        print("\n📋 Next Steps:")
        print("1. Build and run: docker-compose up --build")
        print("2. Test API: ./test_api.sh")
        print("3. Verify deliverable: curl -X POST localhost:8080/analyze -d '{\"files\":[{\"name\":\"test.js\",\"content\":\"function test(){return 1;}\"}]}'")
    else:
        print("❌ Some checks failed. Please review the missing components.")
    
    return all_checks_passed

if __name__ == "__main__":
    success = main()
    exit(0 if success else 1)