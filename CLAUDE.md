# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

This project uses `cargo-make` for task automation. Install with `cargo install cargo-make`.

```bash
# Build
cargo make dev-build          # Debug build
cargo make release-build      # Release build

# Test (uses cargo-nextest)
cargo make nextest-run        # Run all tests with nextest

# Format & Lint
cargo make format-all         # Run all formatters (taplo + clippy + rustfmt)
cargo make clippy             # Run clippy
cargo make reformat           # Run rustfmt
cargo make sort-cargo-toml    # Sort Cargo.toml with taplo
```

**Run a single test:**
```bash
cargo nextest run <test_name>
```

## Architecture

**Library:** `academic_paper_interpreter` (src/lib.rs)
**Binary:** `academic-paper-interpreter` (src/main.rs) - thin CLI wrapper

### Core Modules

```
src/
  lib.rs              # Library root, public exports
  main.rs             # CLI entry point

  client/             # Paper search and retrieval
    mod.rs            # PaperClient (unified client)
    arxiv.rs          # ArxivClient wrapper
    semantic.rs       # SemanticScholarClient wrapper
    search.rs         # SearchParams, SearchResult, PaperSource

  models.rs           # AcademicPaper, Author, PaperAnalysis structs

  agents/             # LLM-powered paper analysis
    mod.rs            # Agent exports
    traits.rs         # LlmProvider, AnalysisAgent traits
    prompts.rs        # Prompt templates for analysis
    paper_analyzer.rs # PaperAnalyzer implementation
    providers/
      openai.rs       # OpenAI API
      anthropic.rs    # Claude API
      ollama.rs       # Local LLM via Ollama

  shared/             # Cross-cutting utilities
    config.rs         # Config, LlmProviderType
    errors.rs         # AppError, AppResult<T>
    logger.rs         # Tracing-based logging
    utils.rs          # Progress bar, date parsing
```

### Key Types

- `PaperClient` - Unified client for searching/fetching papers from arXiv and Semantic Scholar
- `SearchParams` - Builder-pattern query parameters
- `AcademicPaper` - Unified paper representation with `from_arxiv()`, `from_semantic_scholar()` constructors
- `PaperAnalysis` - LLM-generated analysis (summary, methodology, etc.)
- `PaperAnalyzer<P: LlmProvider>` - Generic analyzer over LLM providers
- `LlmProvider` trait - Abstraction for LLM APIs (OpenAI, Anthropic, Ollama)

### External Crates
- `arxiv-tools` (git) - arXiv API client
- `ss-tools` (git) - Semantic Scholar API client

### Usage Example

```rust
use academic_paper_interpreter::{PaperClient, SearchParams, PaperAnalyzer};
use academic_paper_interpreter::agents::providers::OpenAiProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Search papers
    let client = PaperClient::new();
    let params = SearchParams::new()
        .with_query("attention is all you need")
        .with_max_results(5);
    let results = client.search(params).await?;

    // Analyze with LLM
    let provider = OpenAiProvider::from_env()?;
    let analyzer = PaperAnalyzer::new(provider);
    let analysis = analyzer.analyze(&results.papers[0]).await?;

    println!("{}", analysis.summary);
    Ok(())
}
```

## Testing

Tests use `test-log` for tracing integration:
```rust
#[test_log::test]
fn my_test() {
    tracing::info!("logs captured during test");
}
```

Async tests use tokio:
```rust
#[tokio::test]
async fn my_async_test() {
    // ...
}
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `OPENAI_API_KEY` | OpenAI API key |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `SEMANTIC_SCHOLAR_API_KEY` | Semantic Scholar API key (optional) |
| `OLLAMA_BASE_URL` | Ollama server URL (default: http://localhost:11434) |
| `OLLAMA_MODEL` | Default Ollama model |
| `LLM_PROVIDER` | Default provider: openai, anthropic, ollama |
