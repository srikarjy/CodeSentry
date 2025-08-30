use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, instrument};

use crate::{
    error::{AnalysisError, AnalysisResult},
    parser::ParserRegistry,
    types::{
        AnalysisRequest, AnalysisResponse, FileAnalysisResult, AnalysisSummary,
        Finding, FileMetrics, Language, Severity, SourceFile,
    },
};

pub struct AnalysisEngine {
    parser_registry: ParserRegistry,
}

impl AnalysisEngine {
    pub async fn new() -> AnalysisResult<Self> {
        let parser_registry = ParserRegistry::new().await?;
        
        Ok(Self {
            parser_registry,
        })
    }

    #[instrument(skip(self, request))]
    pub async fn analyze(&self, request: AnalysisRequest) -> AnalysisResult<AnalysisResponse> {
        let start_time = Instant::now();
        
        info!("Starting analysis of {} files", request.files.len());
        
        let mut results = Vec::new();
        let mut total_lines = 0u32;
        let mut total_findings = 0u32;
        let mut findings_by_severity: HashMap<String, u32> = HashMap::new();

        for file in request.files {
            let file_result = self.analyze_file(file, &request.rules).await?;
            
            total_lines += file_result.metrics.lines_of_code;
            total_findings += file_result.findings.len() as u32;
            
            // Count findings by severity
            for finding in &file_result.findings {
                let severity_str = format!("{:?}", finding.severity);
                *findings_by_severity.entry(severity_str).or_insert(0) += 1;
            }
            
            results.push(file_result);
        }

        let execution_time = start_time.elapsed();
        
        info!(
            "Analysis completed in {}ms, {} findings across {} lines",
            execution_time.as_millis(),
            total_findings,
            total_lines
        );

        Ok(AnalysisResponse {
            results,
            summary: AnalysisSummary {
                total_files: results.len() as u32,
                total_findings,
                findings_by_severity,
                total_lines_analyzed: total_lines,
            },
            execution_time_ms: execution_time.as_millis() as u64,
        })
    }

    async fn analyze_file(
        &self,
        mut file: SourceFile,
        _rule_config: &Option<crate::types::RuleConfig>,
    ) -> AnalysisResult<FileAnalysisResult> {
        // Detect language if not provided
        let language = match file.language {
            Some(lang) => lang,
            None => Language::from_filename(&file.name)
                .ok_or_else(|| AnalysisError::UnsupportedLanguage {
                    language: file.name.split('.').last().unwrap_or("unknown").to_string(),
                })?,
        };

        // For now, return basic metrics and placeholder findings
        // This will be replaced with actual parsing and analysis in later tasks
        let lines_of_code = file.content.lines().count() as u32;
        
        // Create some basic findings for demonstration
        let mut findings = Vec::new();
        
        // Simple demonstration: flag functions that might be too simple
        if file.content.contains("function") && file.content.lines().count() < 5 {
            findings.push(Finding {
                rule_id: "demo-simple-function".to_string(),
                severity: Severity::Low,
                message: "Function appears to be very simple".to_string(),
                location: crate::types::Location {
                    line: 1,
                    column: 1,
                    end_line: None,
                    end_column: None,
                },
                suggestion: Some("Consider if this function adds value".to_string()),
            });
        }

        Ok(FileAnalysisResult {
            file_name: file.name,
            language,
            findings,
            metrics: FileMetrics {
                lines_of_code,
                functions_count: count_functions(&file.content),
                classes_count: count_classes(&file.content),
                complexity_score: 1.0, // Placeholder
            },
        })
    }
}

// Simple placeholder functions for basic metrics
fn count_functions(content: &str) -> u32 {
    content.matches("function").count() as u32
        + content.matches("const ").count() as u32 // Arrow functions approximation
        + content.matches("let ").count() as u32   // Arrow functions approximation
}

fn count_classes(content: &str) -> u32 {
    content.matches("class ").count() as u32
}