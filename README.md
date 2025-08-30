# Rust Analysis Engine

A high-performance static code analysis engine built in Rust.

## Quick Start

### Running the Server

#### Option 1: With Docker (Recommended)
```bash
# Build and run with Docker
docker-compose up --build

# Or for development with hot reload
docker-compose --profile dev up --build

# The server will start on http://localhost:8080
```

#### Option 2: With Rust installed locally
```bash
# Install dependencies and run
cargo run

# The server will start on http://localhost:8080
```

### Testing the API

```bash
# Run the automated test script
./test_api.sh

# Or test manually:

# Health check
curl http://localhost:8080/health

# Analyze a JavaScript file
curl -X POST http://localhost:8080/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "files": [
      {
        "name": "test.js",
        "content": "function test() { return 1; }"
      }
    ]
  }'

# Week 1 Deliverable Test
curl -X POST localhost:8080/analyze -d '{"files":[{"name":"test.js","content":"function test(){return 1;}"}]}'
```

### Running Tests

```bash
# Run all tests
cargo test

# Run integration tests specifically
cargo test --test integration_test
```

## API Endpoints

### `GET /health`
Returns server health status.

### `POST /analyze`
Analyzes code files and returns findings.

**Request Body:**
```json
{
  "files": [
    {
      "name": "example.js",
      "content": "function example() { return 42; }",
      "language": "JavaScript" // Optional, auto-detected from filename
    }
  ],
  "rules": {
    "complexity_threshold": 10,
    "max_function_length": 50,
    "enable_security_rules": true,
    "enable_dead_code_detection": true
  }
}
```

**Response:**
```json
{
  "results": [
    {
      "file_name": "example.js",
      "language": "JavaScript",
      "findings": [
        {
          "rule_id": "complexity",
          "severity": "Medium",
          "message": "Function complexity is too high",
          "location": {
            "line": 1,
            "column": 1
          },
          "suggestion": "Consider breaking this function into smaller parts"
        }
      ],
      "metrics": {
        "lines_of_code": 1,
        "functions_count": 1,
        "classes_count": 0,
        "complexity_score": 1.0
      }
    }
  ],
  "summary": {
    "total_files": 1,
    "total_findings": 1,
    "findings_by_severity": {
      "Medium": 1
    },
    "total_lines_analyzed": 1
  },
  "execution_time_ms": 5
}
```

## Development Status

### Week 1: HTTP API + Basic Integration ✅
- [x] HTTP Server with Axum
- [x] `/analyze` endpoint with request validation
- [x] JSON response formatting
- [x] Basic error handling middleware
- [x] Integration tests

### Week 2: Real JavaScript Parser (Coming Next)
- [ ] Tree-sitter JavaScript integration
- [ ] TypeScript support
- [ ] Parser performance testing

### Week 3: Working Analysis Rules (Coming Soon)
- [ ] Cyclomatic complexity rule
- [ ] Function length rule
- [ ] Hardcoded secrets rule
- [ ] Dead code detection

### Week 4: Performance + Documentation (Coming Soon)
- [ ] Performance optimization
- [ ] Benchmarking
- [ ] API polish
- [ ] Docker container

## Supported Languages

- JavaScript (.js, .jsx, .mjs)
- TypeScript (.ts, .tsx) 
- Python (.py, .pyi) - Coming in Week 2+
- Go (.go) - Coming in Week 2+
- Rust (.rs) - Coming in Week 2+

## Configuration

Set log level with environment variable:
```bash
RUST_LOG=debug cargo run
```