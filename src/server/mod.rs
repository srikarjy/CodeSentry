use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, instrument};

use crate::{
    analysis::AnalysisEngine,
    error::{AnalysisError, AnalysisResult},
    types::{AnalysisRequest, AnalysisResponse},
};

pub struct Server {
    engine: Arc<AnalysisEngine>,
}

impl Server {
    pub async fn new() -> AnalysisResult<Self> {
        let engine = Arc::new(AnalysisEngine::new().await?);
        Ok(Self { engine })
    }

    pub async fn run(self) -> AnalysisResult<()> {
        let app = self.create_router();
        
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
            .await
            .map_err(|e| AnalysisError::InternalError {
                message: format!("Failed to bind to port 8080: {}", e),
            })?;

        info!("Server starting on http://0.0.0.0:8080");
        
        axum::serve(listener, app)
            .await
            .map_err(|e| AnalysisError::InternalError {
                message: format!("Server error: {}", e),
            })?;

        Ok(())
    }

    fn create_router(self) -> Router {
        Router::new()
            .route("/", get(health_check))
            .route("/health", get(health_check))
            .route("/analyze", post(analyze_handler))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive()),
            )
            .with_state(self.engine)
    }
}

#[instrument]
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "rust-analysis-engine",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[instrument(skip(engine, request))]
async fn analyze_handler(
    State(engine): State<Arc<AnalysisEngine>>,
    Json(request): Json<AnalysisRequest>,
) -> AnalysisResult<Json<AnalysisResponse>> {
    // Validate request
    validate_request(&request)?;
    
    // Perform analysis
    let response = engine.analyze(request).await?;
    
    Ok(Json(response))
}

fn validate_request(request: &AnalysisRequest) -> AnalysisResult<()> {
    const MAX_FILE_SIZE: usize = 1024 * 1024; // 1MB per file
    const MAX_FILES: usize = 100;

    if request.files.is_empty() {
        return Err(AnalysisError::ValidationError {
            message: "At least one file must be provided".to_string(),
        });
    }

    if request.files.len() > MAX_FILES {
        return Err(AnalysisError::ValidationError {
            message: format!("Too many files: {} (max: {})", request.files.len(), MAX_FILES),
        });
    }

    for file in &request.files {
        if file.name.is_empty() {
            return Err(AnalysisError::ValidationError {
                message: "File name cannot be empty".to_string(),
            });
        }

        if file.content.len() > MAX_FILE_SIZE {
            return Err(AnalysisError::FileTooLarge {
                size_bytes: file.content.len(),
                limit_bytes: MAX_FILE_SIZE,
            });
        }

        // Detect language if not provided
        if file.language.is_none() {
            if crate::types::Language::from_filename(&file.name).is_none() {
                return Err(AnalysisError::UnsupportedLanguage {
                    language: file.name.split('.').last().unwrap_or("unknown").to_string(),
                });
            }
        }
    }

    Ok(())
}