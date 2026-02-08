//! Core traits for LLM providers and analysis agents

use crate::models::{AcademicPaper, PaperAnalysis};
use crate::shared::errors::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// Role of a message in a conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message (instructions for the LLM)
    System,
    /// User message (input)
    User,
    /// Assistant message (LLM response)
    Assistant,
}

impl MessageRole {
    /// Convert to string for API calls
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        }
    }
}

/// A single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: MessageRole,
    /// Content of the message
    pub content: String,
}

impl Message {
    /// Create a new message
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(MessageRole::System, content)
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(MessageRole::User, content)
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content)
    }
}

/// Configuration for LLM requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Temperature for sampling (0.0 = deterministic, 1.0 = creative)
    pub temperature: Option<f32>,

    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,

    /// Model identifier (provider-specific)
    pub model: String,

    /// Top-p sampling parameter
    pub top_p: Option<f32>,

    /// Stop sequences
    pub stop_sequences: Vec<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            temperature: None,
            max_tokens: Some(4096),
            model: String::new(), // Provider-specific default
            top_p: None,
            stop_sequences: Vec::new(),
        }
    }
}

impl LlmConfig {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    /// Set model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set top-p
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Add stop sequence
    pub fn with_stop_sequence(mut self, seq: impl Into<String>) -> Self {
        self.stop_sequences.push(seq.into());
        self
    }
}

/// Trait for LLM providers (OpenAI, Anthropic, Ollama, etc.)
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get the provider name for logging/tracking
    fn name(&self) -> &str;

    /// Get the default model for this provider
    fn default_model(&self) -> &str;

    /// Send a completion request and get a text response
    async fn complete(&self, messages: Vec<Message>, config: &LlmConfig) -> AppResult<String>;

    /// Send a completion request expecting JSON response
    async fn complete_json<T: DeserializeOwned + Send>(
        &self,
        messages: Vec<Message>,
        config: &LlmConfig,
    ) -> AppResult<T> {
        let response = self.complete(messages, config).await?;
        self.parse_json_response(&response)
    }

    /// Parse JSON from response text (handles markdown code blocks)
    fn parse_json_response<T: DeserializeOwned>(&self, response: &str) -> AppResult<T> {
        // Try to extract JSON from markdown code blocks
        let json_str = if response.contains("```json") {
            response
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .map(|s| s.trim())
                .unwrap_or(response)
        } else if response.contains("```") {
            response
                .split("```")
                .nth(1)
                .map(|s| s.trim())
                .unwrap_or(response)
        } else {
            response.trim()
        };

        serde_json::from_str(json_str).map_err(|e| {
            crate::shared::errors::AppError::LlmError(format!(
                "Failed to parse JSON response: {}. Response: {}",
                e,
                &response[..response.len().min(500)]
            ))
        })
    }
}

/// Trait for paper analysis agents
#[async_trait]
pub trait AnalysisAgent: Send + Sync {
    /// Analyze a paper and return the analysis
    async fn analyze(&self, paper: &AcademicPaper) -> AppResult<PaperAnalysis>;

    /// Generate just the summary
    async fn generate_summary(&self, paper: &AcademicPaper) -> AppResult<String>;

    /// Generate methodology description
    async fn generate_methodology(&self, paper: &AcademicPaper) -> AppResult<String>;

    /// Translate text to Japanese
    async fn translate_to_japanese(&self, text: &str) -> AppResult<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::system("You are a helpful assistant");
        assert_eq!(msg.role, MessageRole::System);
        assert_eq!(msg.content, "You are a helpful assistant");
    }

    #[test]
    fn test_llm_config_builder() {
        let config = LlmConfig::new()
            .with_temperature(0.7)
            .with_max_tokens(2048)
            .with_model("gpt-4");

        assert_eq!(config.temperature, Some(0.7));
        assert_eq!(config.max_tokens, Some(2048));
        assert_eq!(config.model, "gpt-4");
    }
}
