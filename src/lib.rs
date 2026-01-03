//! # Academic Paper Interpreter
//!
//! A Rust library for searching, fetching, and analyzing academic papers
//! from arXiv and Semantic Scholar with LLM-powered analysis.
//!
//! ## Features
//!
//! - **Paper Search**: Search papers by keyword, title, or author across arXiv and Semantic Scholar
//! - **Paper Fetch**: Retrieve detailed paper information by arXiv ID or Semantic Scholar ID
//! - **LLM Analysis**: Analyze papers using LLM providers (OpenAI, Anthropic, Ollama)
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use academic_paper_interpreter::{PaperClient, SearchParams, PaperAnalyzer};
//! use academic_paper_interpreter::agents::providers::OpenAiProvider;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Search for papers
//!     let client = PaperClient::new();
//!     let params = SearchParams::new()
//!         .with_query("attention is all you need")
//!         .with_max_results(5);
//!     let results = client.search(params).await?;
//!
//!     // Analyze with LLM
//!     let provider = OpenAiProvider::new(std::env::var("OPENAI_API_KEY")?);
//!     let analyzer = PaperAnalyzer::new(provider);
//!
//!     let mut paper = results.papers.into_iter().next().unwrap();
//!     analyzer.analyze(&mut paper).await?;
//!
//!     println!("{}", paper.analysis.unwrap().summary);
//!     Ok(())
//! }
//! ```

pub mod agents;
pub mod client;
pub mod models;
pub mod pdf;
pub mod shared;

// Re-export main types at crate root
pub use client::{PaperClient, PaperSource, SearchParams, SearchResult};
pub use models::{AcademicPaper, Author, PaperAnalysis, PaperSection, PaperText};
pub use pdf::{ExtractionConfig, PdfExtractor};
pub use shared::config::Config;
pub use shared::errors::{AppError, AppResult};

// Re-export agent types
pub use agents::{AnalysisAgent, LlmConfig, LlmProvider, Message, MessageRole, PaperAnalyzer};

/// Prelude module for convenient imports
pub mod prelude {
    pub use super::{
        AcademicPaper, AnalysisAgent, AppError, AppResult, Author, ExtractionConfig, LlmProvider,
        PaperAnalysis, PaperAnalyzer, PaperClient, PaperSection, PaperText, PdfExtractor,
        SearchParams, SearchResult,
    };
}
