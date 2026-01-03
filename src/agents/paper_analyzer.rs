//! Paper analysis agent implementation

use super::prompts::PromptTemplates;
use super::traits::{AnalysisAgent, LlmConfig, LlmProvider, Message};
use crate::models::{AcademicPaper, PaperAnalysis};
use crate::shared::errors::AppResult;
use async_trait::async_trait;
use chrono::Local;
use serde::Deserialize;

/// Response structure for full paper analysis
#[derive(Debug, Deserialize)]
struct AnalysisResponse {
    summary: String,
    background_and_purpose: String,
    methodology: String,
    dataset: String,
    results: String,
    advantages_limitations_and_future_work: String,
    key_contributions: Vec<String>,
    tasks: Vec<String>,
}

/// Paper analysis agent that uses LLM for analysis
pub struct PaperAnalyzer<P: LlmProvider> {
    provider: P,
    config: LlmConfig,
}

impl<P: LlmProvider> PaperAnalyzer<P> {
    /// Create a new paper analyzer with the given LLM provider
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            config: LlmConfig::default(),
        }
    }

    /// Configure the analyzer with custom LLM settings
    pub fn with_config(mut self, config: LlmConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the temperature for analysis
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.config.temperature = temp;
        self
    }

    /// Set the model for analysis
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    /// Get the effective config (with provider defaults applied)
    fn effective_config(&self) -> LlmConfig {
        let mut config = self.config.clone();
        if config.model.is_empty() {
            config.model = self.provider.default_model().to_string();
        }
        config
    }

    /// Analyze a paper and update it with the analysis
    pub async fn analyze_and_update(&self, paper: &mut AcademicPaper) -> AppResult<()> {
        let analysis = self.analyze(paper).await?;
        paper.set_analysis(analysis);
        Ok(())
    }
}

#[async_trait]
impl<P: LlmProvider> AnalysisAgent for PaperAnalyzer<P> {
    async fn analyze(&self, paper: &AcademicPaper) -> AppResult<PaperAnalysis> {
        let messages = vec![
            Message::system(PromptTemplates::system_prompt()),
            Message::user(PromptTemplates::full_analysis_prompt(
                &paper.title,
                &paper.abstract_text,
            )),
        ];

        let config = self.effective_config();
        let response: AnalysisResponse = self.provider.complete_json(messages, &config).await?;

        Ok(PaperAnalysis {
            summary: response.summary,
            summary_ja: None,
            background_and_purpose: response.background_and_purpose,
            methodology: response.methodology,
            dataset: response.dataset,
            results: response.results,
            advantages_limitations_and_future_work: response.advantages_limitations_and_future_work,
            key_contributions: response.key_contributions,
            tasks: response.tasks,
            analyzed_at: Local::now(),
            provider: self.provider.name().to_string(),
            model: config.model,
        })
    }

    async fn generate_summary(&self, paper: &AcademicPaper) -> AppResult<String> {
        let messages = vec![
            Message::system(PromptTemplates::system_prompt()),
            Message::user(PromptTemplates::summary_prompt(
                &paper.title,
                &paper.abstract_text,
            )),
        ];

        let config = self.effective_config();
        self.provider.complete(messages, &config).await
    }

    async fn generate_methodology(&self, paper: &AcademicPaper) -> AppResult<String> {
        let messages = vec![
            Message::system(PromptTemplates::system_prompt()),
            Message::user(PromptTemplates::methodology_prompt(
                &paper.title,
                &paper.abstract_text,
            )),
        ];

        let config = self.effective_config();
        self.provider.complete(messages, &config).await
    }

    async fn translate_to_japanese(&self, text: &str) -> AppResult<String> {
        let messages = vec![
            Message::system(PromptTemplates::japanese_translation_system()),
            Message::user(PromptTemplates::translation_prompt(text, "Japanese")),
        ];

        let config = self.effective_config();
        self.provider.complete(messages, &config).await
    }
}

/// Builder for PaperAnalyzer with fluent API
pub struct PaperAnalyzerBuilder<P: LlmProvider> {
    provider: P,
    config: LlmConfig,
}

impl<P: LlmProvider> PaperAnalyzerBuilder<P> {
    /// Create a new builder
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            config: LlmConfig::default(),
        }
    }

    /// Set temperature
    pub fn temperature(mut self, temp: f32) -> Self {
        self.config.temperature = temp;
        self
    }

    /// Set max tokens
    pub fn max_tokens(mut self, tokens: u32) -> Self {
        self.config.max_tokens = Some(tokens);
        self
    }

    /// Set model
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    /// Build the analyzer
    pub fn build(self) -> PaperAnalyzer<P> {
        PaperAnalyzer {
            provider: self.provider,
            config: self.config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Mock provider for testing
    struct MockProvider;

    #[async_trait]
    impl LlmProvider for MockProvider {
        fn name(&self) -> &str {
            "mock"
        }

        fn default_model(&self) -> &str {
            "mock-model"
        }

        async fn complete(
            &self,
            _messages: Vec<Message>,
            _config: &LlmConfig,
        ) -> AppResult<String> {
            Ok(r#"{
                "summary": "Test summary",
                "background_and_purpose": "Test background",
                "methodology": "Test methodology",
                "dataset": "Test dataset",
                "results": "Test results",
                "advantages_limitations_and_future_work": "Test advantages",
                "key_contributions": ["contribution 1"],
                "tasks": ["task 1"]
            }"#
            .to_string())
        }
    }

    #[tokio::test]
    async fn test_paper_analyzer_creation() {
        let provider = MockProvider;
        let analyzer = PaperAnalyzer::new(provider);
        assert_eq!(analyzer.config.temperature, 0.3);
    }

    #[tokio::test]
    async fn test_analyze_paper() {
        let provider = MockProvider;
        let analyzer = PaperAnalyzer::new(provider);

        let mut paper = AcademicPaper::new();
        paper.title = "Test Paper".to_string();
        paper.abstract_text = "Test abstract".to_string();

        let result = analyzer.analyze(&paper).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.summary, "Test summary");
        assert_eq!(analysis.provider, "mock");
    }
}
