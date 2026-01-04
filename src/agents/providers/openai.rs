//! OpenAI API provider using openai-tools crate

use crate::agents::traits::{LlmConfig, LlmProvider, Message, MessageRole};
use crate::shared::errors::{AppError, AppResult};
use async_trait::async_trait;
use openai_tools::chat::request::ChatCompletion;
use openai_tools::common::message::Message as OpenAiMessage;
use openai_tools::common::role::Role as OpenAiRole;

const DEFAULT_OPENAI_MODEL: &str = "gpt-4o";

/// OpenAI API provider
///
/// Uses the openai-tools crate for API communication.
/// API key is loaded from the OPENAI_API_KEY environment variable.
/// Model can be configured via the OPENAI_MODEL environment variable.
pub struct OpenAiProvider {
    /// Default model to use (from OPENAI_MODEL env var or fallback)
    default_model: String,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider
    ///
    /// Note: The API key parameter is accepted for API compatibility but
    /// the actual key is read from the OPENAI_API_KEY environment variable
    /// by the underlying openai-tools crate.
    pub fn new(_api_key: impl Into<String>) -> Self {
        Self {
            default_model: DEFAULT_OPENAI_MODEL.to_string(),
        }
    }

    /// Create a new OpenAI provider with a custom model
    pub fn with_model(model: impl Into<String>) -> Self {
        Self {
            default_model: model.into(),
        }
    }

    /// Create from environment variables
    ///
    /// Reads OPENAI_API_KEY (required) and OPENAI_MODEL (optional, defaults to gpt-4o)
    pub fn from_env() -> AppResult<Self> {
        // Verify the environment variable is set
        std::env::var("OPENAI_API_KEY").map_err(|_| {
            AppError::ConfigError("OPENAI_API_KEY environment variable not set".to_string())
        })?;

        let model =
            std::env::var("OPENAI_MODEL").unwrap_or_else(|_| DEFAULT_OPENAI_MODEL.to_string());

        Ok(Self {
            default_model: model,
        })
    }

    /// Convert internal Message to openai-tools Message
    fn convert_message(msg: Message) -> OpenAiMessage {
        let role = match msg.role {
            MessageRole::System => OpenAiRole::System,
            MessageRole::User => OpenAiRole::User,
            MessageRole::Assistant => OpenAiRole::Assistant,
        };
        OpenAiMessage::from_string(role, msg.content)
    }

    /// Check if the model is a reasoning model (o1, o3, etc.)
    /// Reasoning models only support temperature=1.0
    fn is_reasoning_model(model: &str) -> bool {
        model.starts_with("o1") || model.starts_with("o3")
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
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

        // Convert messages to openai-tools format
        let openai_messages: Vec<OpenAiMessage> =
            messages.into_iter().map(Self::convert_message).collect();

        // Build the request
        let mut chat = ChatCompletion::new();
        chat.model_id(&model).messages(openai_messages);

        // Reasoning models (o1, o3, etc.) only support temperature=1.0
        // Skip setting temperature for these models
        if !Self::is_reasoning_model(&model) {
            chat.temperature(config.temperature);
        }

        if let Some(max_tokens) = config.max_tokens {
            chat.max_completion_tokens(max_tokens as u64);
        }

        // Execute the request
        let response = chat
            .chat()
            .await
            .map_err(|e| AppError::LlmError(format!("OpenAI API error: {}", e)))?;

        // Extract text from response
        response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .and_then(|c| c.text)
            .ok_or_else(|| AppError::LlmError("No response from OpenAI".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = OpenAiProvider::new("test-key");
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn test_default_model() {
        let provider = OpenAiProvider::new("test-key");
        assert_eq!(provider.default_model(), "gpt-4o");

        let provider = OpenAiProvider::with_model("gpt-4-turbo");
        assert_eq!(provider.default_model(), "gpt-4-turbo");
    }

    #[test]
    fn test_is_reasoning_model() {
        // Reasoning models
        assert!(OpenAiProvider::is_reasoning_model("o1"));
        assert!(OpenAiProvider::is_reasoning_model("o1-preview"));
        assert!(OpenAiProvider::is_reasoning_model("o1-mini"));
        assert!(OpenAiProvider::is_reasoning_model("o3"));
        assert!(OpenAiProvider::is_reasoning_model("o3-mini"));

        // Non-reasoning models
        assert!(!OpenAiProvider::is_reasoning_model("gpt-4o"));
        assert!(!OpenAiProvider::is_reasoning_model("gpt-4-turbo"));
        assert!(!OpenAiProvider::is_reasoning_model("gpt-3.5-turbo"));
    }
}
