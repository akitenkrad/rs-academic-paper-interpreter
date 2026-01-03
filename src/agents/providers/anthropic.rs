//! Anthropic Claude API provider using anthropic-tools crate

use crate::agents::traits::{LlmConfig, LlmProvider, Message, MessageRole};
use crate::shared::errors::{AppError, AppResult};
use anthropic_tools::Messages;
use async_trait::async_trait;

/// Anthropic Claude API provider
///
/// Uses the anthropic-tools crate for API communication.
/// API key is loaded from the ANTHROPIC_API_KEY environment variable.
pub struct AnthropicProvider;

impl AnthropicProvider {
    /// Create a new Anthropic provider
    ///
    /// Note: The API key parameter is accepted for API compatibility but
    /// the actual key is read from the ANTHROPIC_API_KEY environment variable
    /// by the underlying anthropic-tools crate.
    pub fn new(_api_key: impl Into<String>) -> Self {
        Self
    }

    /// Create from environment variable ANTHROPIC_API_KEY
    pub fn from_env() -> AppResult<Self> {
        // Verify the environment variable is set
        std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
            AppError::ConfigError("ANTHROPIC_API_KEY environment variable not set".to_string())
        })?;
        Ok(Self)
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn default_model(&self) -> &str {
        "claude-sonnet-4-20250514"
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
            .model(&model)
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
    }
}
