use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    JavaScript,
    TypeScript,
    Python,
    Go,
    Rust,
}

impl Language {
    pub fn from_filename(filename: &str) -> Option<Self> {
        let extension = filename.split('.').last()?;
        match extension {
            "js" | "jsx" | "mjs" => Some(Language::JavaScript),
            "ts" | "tsx" => Some(Language::TypeScript),
            "py" | "pyi" => Some(Language::Python),
            "go" => Some(Language::Go),
            "rs" => Some(Language::Rust),
            _ => None,
        }
    }

    pub fn supported_extensions(&self) -> &[&str] {
        match self {
            Language::JavaScript => &["js", "jsx", "mjs"],
            Language::TypeScript => &["ts", "tsx"],
            Language::Python => &["py", "pyi"],
            Language::Go => &["go"],
            Language::Rust => &["rs"],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    pub name: String,
    pub content: String,
    pub language: Option<Language>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub files: Vec<SourceFile>,
    pub rules: Option<RuleConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    pub complexity_threshold: Option<u32>,
    pub max_function_length: Option<u32>,
    pub enable_security_rules: Option<bool>,
    pub enable_dead_code_detection: Option<bool>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            complexity_threshold: Some(10),
            max_function_length: Some(50),
            enable_security_rules: Some(true),
            enable_dead_code_detection: Some(true),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub results: Vec<FileAnalysisResult>,
    pub summary: AnalysisSummary,
    pub execution_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileAnalysisResult {
    pub file_name: String,
    pub language: Language,
    pub findings: Vec<Finding>,
    pub metrics: FileMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub location: Location,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub line: u32,
    pub column: u32,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetrics {
    pub lines_of_code: u32,
    pub functions_count: u32,
    pub classes_count: u32,
    pub complexity_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub total_files: u32,
    pub total_findings: u32,
    pub findings_by_severity: HashMap<String, u32>,
    pub total_lines_analyzed: u32,
}

// Content hash for caching
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentHash(pub String);

impl ContentHash {
    pub fn from_content(content: &str) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        ContentHash(format!("{:x}", result))
    }
}