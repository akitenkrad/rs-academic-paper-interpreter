//! Agents module for LLM-powered paper analysis
//!
//! This module provides:
//! - LLM provider traits and implementations (OpenAI, Anthropic, Ollama)
//! - Paper analysis agents
//! - Prompt templates for structured analysis

mod paper_analyzer;
mod prompts;
mod traits;

pub mod providers;

// Re-export main types
pub use paper_analyzer::{PaperAnalyzer, PaperAnalyzerBuilder};
pub use prompts::PromptTemplates;
pub use traits::{AnalysisAgent, LlmConfig, LlmProvider, Message, MessageRole};

// Re-export providers for convenience
pub use providers::{AnthropicProvider, OllamaProvider, OpenAiProvider};
