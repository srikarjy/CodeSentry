#!/bin/bash

# Test script for the Analysis Engine API
# This script tests the API endpoints once the server is running

BASE_URL="http://localhost:8080"

echo "🧪 Testing Rust Analysis Engine API"
echo "=================================="

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

echo "✅ API tests completed!"
echo ""
echo "🚀 Week 1 Deliverable Test:"
echo "curl -X POST localhost:8080/analyze -d '{\"files\":[{\"name\":\"test.js\",\"content\":\"function test(){return 1;}\"}]}'"