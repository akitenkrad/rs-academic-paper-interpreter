//! OpenAI API provider using openai-tools crate

use crate::agents::traits::{LlmConfig, LlmProvider, Message, MessageRole};
use crate::shared::errors::{AppError, AppResult};
use async_trait::async_trait;
use openai_tools::chat::request::ChatCompletion;
use openai_tools::common::message::Message as OpenAiMessage;
use openai_tools::common::role::Role as OpenAiRole;

/// OpenAI API provider
///
/// Uses the openai-tools crate for API communication.
/// API key is loaded from the OPENAI_API_KEY environment variable.
pub struct OpenAiProvider;

impl OpenAiProvider {
    /// Create a new OpenAI provider
    ///
    /// Note: The API key parameter is accepted for API compatibility but
    /// the actual key is read from the OPENAI_API_KEY environment variable
    /// by the underlying openai-tools crate.
    pub fn new(_api_key: impl Into<String>) -> Self {
        Self
    }

    /// Create from environment variable OPENAI_API_KEY
    pub fn from_env() -> AppResult<Self> {
        // Verify the environment variable is set
        std::env::var("OPENAI_API_KEY").map_err(|_| {
            AppError::ConfigError("OPENAI_API_KEY environment variable not set".to_string())
        })?;
        Ok(Self)
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
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn default_model(&self) -> &str {
        "gpt-4o"
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
        chat.model_id(&model)
            .messages(openai_messages)
            .temperature(config.temperature);

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
    }
}
