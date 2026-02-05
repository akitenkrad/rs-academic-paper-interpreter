//! Anthropic Claude API provider using anthropic-tools crate

use crate::agents::traits::{LlmConfig, LlmProvider, Message, MessageRole};
use crate::shared::errors::{AppError, AppResult};
use anthropic_tools::Messages;
use async_trait::async_trait;

/// Anthropic Claude API provider
///
/// Uses the anthropic-tools crate for API communication.
/// API key is loaded from the ANTHROPIC_API_KEY environment variable.
/// Model can be configured via the ANTHROPIC_MODEL environment variable.
pub struct AnthropicProvider {
    /// Default model to use (from ANTHROPIC_MODEL env var or fallback)
    default_model: String,
}

const DEFAULT_ANTHROPIC_MODEL: &str = "claude-sonnet-4-20250514";

impl AnthropicProvider {
    /// Create a new Anthropic provider
    ///
    /// Note: The API key parameter is accepted for API compatibility but
    /// the actual key is read from the ANTHROPIC_API_KEY environment variable
    /// by the underlying anthropic-tools crate.
    pub fn new(_api_key: impl Into<String>) -> Self {
        Self {
            default_model: DEFAULT_ANTHROPIC_MODEL.to_string(),
        }
    }

    /// Create a new Anthropic provider with a custom model
    pub fn with_model(model: impl Into<String>) -> Self {
        Self {
            default_model: model.into(),
        }
    }

    /// Create from environment variables
    ///
    /// Reads ANTHROPIC_API_KEY (required) and ANTHROPIC_MODEL (optional, defaults to claude-sonnet-4-20250514)
    pub fn from_env() -> AppResult<Self> {
        // Verify the environment variable is set
        std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
            AppError::ConfigError("ANTHROPIC_API_KEY environment variable not set".to_string())
        })?;

        let model = std::env::var("ANTHROPIC_MODEL")
            .unwrap_or_else(|_| DEFAULT_ANTHROPIC_MODEL.to_string());

        Ok(Self {
            default_model: model,
        })
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    async fn complete(&self, messages: Vec<Message>, config: &LlmConfig) -> AppResult<String> {
        let model = if config.model.is_empty() {
            self.default_model().to_string()
        } else {
            config.model.clone()
        };

        // Build the request
        let mut client = Messages::new();
        client
            .model(model.as_str())
            .max_tokens(config.max_tokens.unwrap_or(4096) as usize)
            .temperature(config.temperature);

        // Add messages
        for msg in messages {
            match msg.role {
                MessageRole::System => {
                    client.system(&msg.content);
                }
                MessageRole::User => {
                    client.user(&msg.content);
                }
                MessageRole::Assistant => {
                    client.assistant(&msg.content);
                }
            }
        }

        // Execute the request
        let response = client
            .post()
            .await
            .map_err(|e| AppError::LlmError(format!("Anthropic API error: {}", e)))?;

        // Extract text from response
        let text = response.get_text();
        if text.is_empty() {
            Err(AppError::LlmError(
                "No text response from Anthropic".to_string(),
            ))
        } else {
            Ok(text)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = AnthropicProvider::new("test-key");
        assert_eq!(provider.name(), "anthropic");
    }

    #[test]
    fn test_default_model() {
        let provider = AnthropicProvider::new("test-key");
        assert!(provider.default_model().contains("claude"));

        let provider = AnthropicProvider::with_model("claude-3-opus-20240229");
        assert_eq!(provider.default_model(), "claude-3-opus-20240229");
    }
}
