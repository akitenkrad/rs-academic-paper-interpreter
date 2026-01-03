# Academic Paper Interpreter

arXivおよびSemantic Scholarから論文を検索・取得し、LLM（OpenAI、Anthropic、Ollama）を使って論文の要約・分析を行うRustライブラリ・CLIツール。

<img src="LOGO.png" alt="LOGO" width="150" height="150">

## Features

- **論文検索**: arXivとSemantic Scholarの統合検索クライアント
- **LLM分析**: OpenAI、Anthropic、Ollamaに対応した論文分析エージェント
- **日本語対応**: 分析プロンプトは日本語で出力

## Quick Start

### Prerequisites

- Rust 2024 edition
- [cargo-make](https://github.com/sagiegurari/cargo-make): `cargo install cargo-make`
- [cargo-nextest](https://nexte.st/): `cargo install cargo-nextest`

### Installation

```bash
git clone https://github.com/akitenkrad/rs-academic-paper-interpreter.git
cd rs-academic-paper-interpreter
cargo build --release
```

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `OPENAI_API_KEY` | OpenAI API key | OpenAI使用時 |
| `ANTHROPIC_API_KEY` | Anthropic API key | Anthropic使用時 |
| `SEMANTIC_SCHOLAR_API_KEY` | Semantic Scholar API key | Optional |
| `OLLAMA_BASE_URL` | Ollama server URL (default: http://localhost:11434) | Ollama使用時 |
| `OLLAMA_MODEL` | Default Ollama model | Ollama使用時 |
| `LLM_PROVIDER` | Default provider: openai, anthropic, ollama | Optional |

### CLI Usage

```bash
# Search papers by keyword
academic-paper-interpreter search --query "transformer attention"

# Search by author
academic-paper-interpreter search --author "Vaswani" --max-results 5

# Search with category filter
academic-paper-interpreter search --query "large language model" --category cs.CL

# Fetch paper by arXiv ID
academic-paper-interpreter fetch --arxiv 1706.03762

# Analyze paper with OpenAI (default)
academic-paper-interpreter analyze --arxiv 1706.03762

# Analyze with specific provider and model
academic-paper-interpreter analyze --arxiv 1706.03762 --provider anthropic --model claude-sonnet-4-20250514

# Output as JSON
academic-paper-interpreter search --query "BERT" --output json
```

### Library Usage

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

## Development

### Build

```bash
cargo make dev-build          # Debug build
cargo make release-build      # Release build
```

### Test

```bash
cargo make nextest-run        # Unit tests
cargo make integration-test   # Integration tests (requires API keys)
cargo make test-all           # All tests
```

### Format & Lint

```bash
cargo make format-all         # Run all formatters
cargo make clippy             # Run clippy
cargo make reformat           # Run rustfmt
```

## Architecture

```
src/
  lib.rs              # Library root
  main.rs             # CLI entry point

  client/             # Paper search and retrieval
    mod.rs            # PaperClient (unified client)
    arxiv.rs          # ArxivClient wrapper
    semantic.rs       # SemanticScholarClient wrapper
    search.rs         # SearchParams, SearchResult

  models.rs           # AcademicPaper, Author, PaperAnalysis

  agents/             # LLM-powered paper analysis
    mod.rs            # Agent exports
    traits.rs         # LlmProvider, AnalysisAgent traits
    prompts.rs        # Prompt templates (Japanese)
    paper_analyzer.rs # PaperAnalyzer implementation
    providers/
      openai.rs       # OpenAI API (via openai-tools)
      anthropic.rs    # Anthropic API (via anthropic-tools)
      ollama.rs       # Local LLM via Ollama

  shared/             # Cross-cutting utilities
    config.rs         # Config, LlmProviderType
    errors.rs         # AppError, AppResult<T>
    logger.rs         # Tracing-based logging
    utils.rs          # Progress bar, date parsing
```

## License

Apache-2.0
