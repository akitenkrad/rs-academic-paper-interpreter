//! Ollama local LLM provider

use crate::agents::traits::{LlmConfig, LlmProvider, Message};
use crate::shared::errors::{AppError, AppResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Ollama local LLM provider
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    default_model: String,
}

impl OllamaProvider {
    /// Create a new Ollama provider with the given model
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: "http://localhost:11434".to_string(),
            default_model: model.into(),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> AppResult<Self> {
        let base_url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string());

        Ok(Self {
            client: Client::new(),
            base_url,
            default_model: model,
        })
    }

    /// Set custom base URL
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop: Vec<String>,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    async fn complete(&self, messages: Vec<Message>, config: &LlmConfig) -> AppResult<String> {
        let model = if config.model.is_empty() {
            self.default_model.clone()
        } else {
            config.model.clone()
        };

        let ollama_messages: Vec<OllamaMessage> = messages
            .into_iter()
            .map(|m| OllamaMessage {
                role: m.role.as_str().to_string(),
                content: m.content,
            })
            .collect();

        let options = OllamaOptions {
            temperature: config.temperature,
            num_predict: config.max_tokens,
            top_p: config.top_p,
            stop: config.stop_sequences.clone(),
        };

        let request = ChatRequest {
            model,
            messages: ollama_messages,
            stream: false,
            options: Some(options),
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                AppError::LlmError(format!(
                    "Failed to connect to Ollama at {}: {}",
                    self.base_url, e
                ))
            })?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| AppError::LlmError(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(AppError::LlmError(format!(
                "Ollama API error ({}): {}",
                status, body
            )));
        }

        let chat_response: ChatResponse = serde_json::from_str(&body)
            .map_err(|e| AppError::LlmError(format!("Failed to parse response: {}", e)))?;

        Ok(chat_response.message.content)
    }
}

impl Default for OllamaProvider {
    /// Create with default model (llama3.2)
    fn default() -> Self {
        Self::new("llama3.2")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = OllamaProvider::new("llama3.2");
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn test_default_model() {
        let provider = OllamaProvider::new("mistral");
        assert_eq!(provider.default_model(), "mistral");
    }

    #[test]
    fn test_custom_base_url() {
        let provider = OllamaProvider::new("llama3.2").with_base_url("http://remote:11434");
        assert_eq!(provider.base_url, "http://remote:11434");
    }
}
