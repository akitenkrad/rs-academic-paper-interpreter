//! Error types for the library

use thiserror::Error;

/// Main error type for the library
#[derive(Error, Debug)]
pub enum AppError {
    /// Generic internal error
    #[error("{0}")]
    InternalAppError(String),

    /// arXiv API error
    #[error("arXiv API error: {0}")]
    ArxivError(String),

    /// Semantic Scholar API error
    #[error("Semantic Scholar API error: {0}")]
    SemanticScholarError(String),

    /// LLM provider error
    #[error("LLM error: {0}")]
    LlmError(String),

    /// Paper not found
    #[error("Paper not found: {0}")]
    PaperNotFound(String),

    /// Analysis failed
    #[error("Analysis failed: {0}")]
    AnalysisError(String),

    /// PDF extraction error
    #[error("PDF extraction failed: {0}")]
    PdfExtractionError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Anyhow error (for compatibility)
    #[error("Error: {0}")]
    AnyhowError(#[from] anyhow::Error),

    /// Tracing initialization error
    #[error("Tracing Error: {0}")]
    TracingTryInitError(#[from] tracing_subscriber::util::TryInitError),
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::InternalAppError(s.to_string())
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::InternalAppError(s)
    }
}

/// Result type alias using AppError
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_from_str() {
        let error: AppError = "test error".into();
        assert!(matches!(error, AppError::InternalAppError(_)));
        assert_eq!(error.to_string(), "test error");
    }

    #[test]
    fn test_error_from_string() {
        let error: AppError = String::from("test error").into();
        assert!(matches!(error, AppError::InternalAppError(_)));
    }

    #[test]
    fn test_specific_errors() {
        let arxiv_error = AppError::ArxivError("connection failed".to_string());
        assert!(arxiv_error.to_string().contains("arXiv"));

        let llm_error = AppError::LlmError("rate limit exceeded".to_string());
        assert!(llm_error.to_string().contains("LLM"));
    }
}
