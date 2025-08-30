use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

pub type AnalysisResult<T> = Result<T, AnalysisError>;

#[derive(Debug, Error)]
pub enum AnalysisError {
    #[error("Parse error: {message} at line {line}")]
    ParseError { message: String, line: u32 },

    #[error("Timeout error: analysis exceeded {timeout_ms}ms")]
    TimeoutError { timeout_ms: u64 },

    #[error("Resource error: {resource} limit exceeded")]
    ResourceError { resource: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("Unsupported language: {language}")]
    UnsupportedLanguage { language: String },

    #[error("File too large: {size_bytes} bytes exceeds limit of {limit_bytes} bytes")]
    FileTooLarge { size_bytes: usize, limit_bytes: usize },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl IntoResponse for AnalysisError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AnalysisError::ValidationError { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            AnalysisError::UnsupportedLanguage { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            AnalysisError::FileTooLarge { .. } => (StatusCode::PAYLOAD_TOO_LARGE, self.to_string()),
            AnalysisError::TimeoutError { .. } => (StatusCode::REQUEST_TIMEOUT, self.to_string()),
            AnalysisError::ResourceError { .. } => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AnalysisError::JsonError(_) => (StatusCode::BAD_REQUEST, "Invalid JSON format".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "type": format!("{:?}", self).split('(').next().unwrap_or("Unknown"),
                "status": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}