//! Configuration management for the library

use crate::shared::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};

/// Type of LLM provider to use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LlmProviderType {
    /// OpenAI API (GPT-4, etc.)
    #[default]
    OpenAi,
    /// Anthropic API (Claude)
    Anthropic,
    /// Ollama (local LLMs)
    Ollama,
}

impl std::fmt::Display for LlmProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmProviderType::OpenAi => write!(f, "openai"),
            LlmProviderType::Anthropic => write!(f, "anthropic"),
            LlmProviderType::Ollama => write!(f, "ollama"),
        }
    }
}

/// Library configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Semantic Scholar API key (optional, rate limits apply without it)
    pub semantic_scholar_api_key: Option<String>,

    /// OpenAI API key
    pub openai_api_key: Option<String>,

    /// OpenAI model (default: gpt-4o)
    pub openai_model: Option<String>,

    /// Anthropic API key
    pub anthropic_api_key: Option<String>,

    /// Anthropic model (default: claude-sonnet-4-20250514)
    pub anthropic_model: Option<String>,

    /// Ollama base URL (default: http://localhost:11434)
    pub ollama_base_url: Option<String>,

    /// Default Ollama model
    pub ollama_model: Option<String>,

    /// Default LLM provider to use
    pub default_llm_provider: LlmProviderType,

    /// Default model to use (provider-specific)
    pub default_model: Option<String>,

    /// Retry count for API calls
    pub retry_count: u64,

    /// Wait time between retries (seconds)
    pub retry_wait_time: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            semantic_scholar_api_key: None,
            openai_api_key: None,
            openai_model: None,
            anthropic_api_key: None,
            anthropic_model: None,
            ollama_base_url: None,
            ollama_model: None,
            default_llm_provider: LlmProviderType::default(),
            default_model: None,
            retry_count: 3,
            retry_wait_time: 1,
        }
    }
}

impl Config {
    /// Create a new default config
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from environment variables
    pub fn from_env() -> AppResult<Self> {
        Ok(Self {
            semantic_scholar_api_key: std::env::var("SEMANTIC_SCHOLAR_API_KEY").ok(),
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            openai_model: std::env::var("OPENAI_MODEL").ok(),
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            anthropic_model: std::env::var("ANTHROPIC_MODEL").ok(),
            ollama_base_url: std::env::var("OLLAMA_BASE_URL").ok(),
            ollama_model: std::env::var("OLLAMA_MODEL").ok(),
            default_llm_provider: Self::parse_provider_from_env()?,
            default_model: std::env::var("LLM_MODEL").ok(),
            retry_count: std::env::var("API_RETRY_COUNT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3),
            retry_wait_time: std::env::var("API_RETRY_WAIT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1),
        })
    }

    /// Parse LLM provider type from environment
    fn parse_provider_from_env() -> AppResult<LlmProviderType> {
        match std::env::var("LLM_PROVIDER").as_deref() {
            Ok("openai") => Ok(LlmProviderType::OpenAi),
            Ok("anthropic") => Ok(LlmProviderType::Anthropic),
            Ok("ollama") => Ok(LlmProviderType::Ollama),
            Ok(other) => Err(AppError::ConfigError(format!(
                "Unknown LLM provider: {}. Valid options: openai, anthropic, ollama",
                other
            ))),
            Err(_) => Ok(LlmProviderType::default()),
        }
    }

    /// Check if OpenAI is configured
    pub fn has_openai(&self) -> bool {
        self.openai_api_key.is_some()
    }

    /// Check if Anthropic is configured
    pub fn has_anthropic(&self) -> bool {
        self.anthropic_api_key.is_some()
    }

    /// Check if Ollama is available (assumes local availability)
    pub fn has_ollama(&self) -> bool {
        // Ollama is assumed to be available if configured or at default location
        true
    }

    /// Get the effective Ollama base URL
    pub fn ollama_url(&self) -> String {
        self.ollama_base_url
            .clone()
            .unwrap_or_else(|| "http://localhost:11434".to_string())
    }

    /// Set OpenAI API key
    pub fn with_openai_key(mut self, key: impl Into<String>) -> Self {
        self.openai_api_key = Some(key.into());
        self
    }

    /// Set Anthropic API key
    pub fn with_anthropic_key(mut self, key: impl Into<String>) -> Self {
        self.anthropic_api_key = Some(key.into());
        self
    }

    /// Set Semantic Scholar API key
    pub fn with_ss_key(mut self, key: impl Into<String>) -> Self {
        self.semantic_scholar_api_key = Some(key.into());
        self
    }

    /// Set default LLM provider
    pub fn with_provider(mut self, provider: LlmProviderType) -> Self {
        self.default_llm_provider = provider;
        self
    }

    /// Set default model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = Some(model.into());
        self
    }

    /// Set retry configuration
    pub fn with_retry_config(mut self, count: u64, wait_time: u64) -> Self {
        self.retry_count = count;
        self.retry_wait_time = wait_time;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.default_llm_provider, LlmProviderType::OpenAi);
        assert_eq!(config.retry_count, 3);
    }

    #[test]
    fn test_config_builder() {
        let config = Config::new()
            .with_openai_key("test-key")
            .with_provider(LlmProviderType::Anthropic)
            .with_model("claude-3-opus");

        assert!(config.has_openai());
        assert_eq!(config.default_llm_provider, LlmProviderType::Anthropic);
        assert_eq!(config.default_model, Some("claude-3-opus".to_string()));
    }

    #[test]
    fn test_ollama_url() {
        let config = Config::default();
        assert_eq!(config.ollama_url(), "http://localhost:11434");

        let config = Config::default().with_provider(LlmProviderType::Ollama);
        assert!(config.has_ollama());
    }
}
