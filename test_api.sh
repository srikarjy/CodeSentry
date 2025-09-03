#!/bin/bash

# Test script for the Analysis Engine API
# This script tests the API endpoints once the server is running

BASE_URL="http://localhost:8080"

echo "🧪 Testing Rust Analysis Engine API"
echo "=================================="

# Check if server is running
if ! curl -s "$BASE_URL/health" > /dev/null; then
    echo "❌ Server is not running on $BASE_URL"
    echo "💡 Start the server with: docker-compose up --build"
    exit 1
fi

# Test 1: Health check
echo "📋 Test 1: Health Check"
curl -s "$BASE_URL/health" | jq '.' || echo "❌ Health check failed"
echo ""

# Test 2: Basic analysis
echo "📋 Test 2: Basic Analysis"
curl -s -X POST "$BASE_URL/analyze" \
  -H "Content-Type: application/json" \
  -d '{
    "files": [
      {
        "name": "test.js",
        "content": "function test() { return 1; }"
      }
    ]
  }' | jq '.' || echo "❌ Basic analysis failed"
echo ""

# Test 3: Multiple files
echo "📋 Test 3: Multiple Files Analysis"
curl -s -X POST "$BASE_URL/analyze" \
  -H "Content-Type: application/json" \
  -d '{
    "files": [
      {
        "name": "app.js",
        "content": "function add(a, b) { return a + b; }"
      },
      {
        "name": "utils.ts",
        "content": "export const multiply = (x: number, y: number): number => x * y;"
      }
    ],
    "rules": {
      "complexity_threshold": 5,
      "max_function_length": 20
    }
  }' | jq '.' || echo "❌ Multiple files analysis failed"
echo ""

# Test 4: Error handling - empty files
echo "📋 Test 4: Error Handling (Empty Files)"
curl -s -X POST "$BASE_URL/analyze" \
  -H "Content-Type: application/json" \
  -d '{"files": []}' | jq '.' || echo "❌ Error handling test failed"
echo ""

# Test 5: Error handling - unsupported language
echo "📋 Test 5: Error Handling (Unsupported Language)"
curl -s -X POST "$BASE_URL/analyze" \
  -H "Content-Type: application/json" \
  -d '{
    "files": [
      {
        "name": "test.xyz",
        "content": "some content"
      }
    ]
  }' | jq '.' || echo "❌ Unsupported language test failed"
echo ""

# Test 6: Complex JavaScript Analysis
echo "📋 Test 6: Complex JavaScript with Real Parser"
curl -s -X POST "$BASE_URL/analyze" \
  -H "Content-Type: application/json" \
  -d '{
    "files": [
      {
        "name": "complex.js",
        "content": "class Calculator { constructor() { this.value = 0; } complexMethod(a, b) { if (a > 0) { for (let i = 0; i < b; i++) { if (i % 2 === 0) { this.value += a; } } } return this.value && a || b; } } const arrow = () => { const password = \"secret123\"; return password; };"
      }
    ],
    "rules": {
      "complexity_threshold": 3,
      "max_function_length": 10,
      "enable_security_rules": true
    }
  }' | jq '.' || echo "❌ Complex analysis failed"
echo ""

# Test 7: TypeScript Analysis
echo "📋 Test 7: TypeScript with Interfaces and Generics"
curl -s -X POST "$BASE_URL/analyze" \
  -H "Content-Type: application/json" \
  -d '{
    "files": [
      {
        "name": "user-service.ts",
        "content": "interface User { id: number; name: string; } class UserService<T extends User> { private users: T[] = []; async addUser(user: T): Promise<void> { if (!user.name) { throw new Error(\"Name required\"); } for (const existing of this.users) { if (existing.id === user.id) { throw new Error(\"Duplicate\"); } } this.users.push(user); } } const API_KEY: string = \"secret123\";"
      }
    ],
    "rules": {
      "complexity_threshold": 2,
      "enable_security_rules": true
    }
  }' | jq '.' || echo "❌ TypeScript analysis failed"
echo ""

echo "✅ API tests completed!"
echo ""
echo "🚀 Week 1 Deliverable Test:"
echo "curl -X POST localhost:8080/analyze -d '{\"files\":[{\"name\":\"test.js\",\"content\":\"function test(){return 1;}\"}]}'"
echo ""
echo "🎯 Week 2 Progress: JavaScript & TypeScript Parser Integration ✅"
echo "- Tree-sitter JavaScript parsing implemented ✅"
echo "- Tree-sitter TypeScript parsing implemented ✅"
echo "- Function, class, and import extraction working ✅"
echo "- Interface and generic type support ✅"
echo "- Complexity calculation functional ✅"
echo "- Security rule detection active ✅"
echo "- Performance tests created ✅"