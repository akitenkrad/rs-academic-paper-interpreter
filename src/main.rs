//! CLI entry point for Academic Paper Interpreter

use academic_paper_interpreter::agents::providers::{
    AnthropicProvider, OllamaProvider, OpenAiProvider,
};
use academic_paper_interpreter::shared::config::LlmProviderType;
use academic_paper_interpreter::shared::logger::init_logger;
use academic_paper_interpreter::{
    AcademicPaper, LlmProvider, PaperAnalyzer, PaperClient, SearchParams,
};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

/// Academic Paper Interpreter - Search, fetch, and analyze academic papers with LLM
#[derive(Parser)]
#[command(name = "academic-paper-interpreter")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "warn", global = true)]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search papers by keyword, title, or author
    Search {
        /// Full-text search query
        #[arg(short, long)]
        query: Option<String>,

        /// Search in paper titles
        #[arg(short, long)]
        title: Option<String>,

        /// Search by author name
        #[arg(short, long)]
        author: Option<String>,

        /// Maximum number of results
        #[arg(short = 'n', long, default_value = "10")]
        max_results: usize,

        /// Filter by arXiv category (e.g., cs.AI, cs.CL)
        #[arg(short, long)]
        category: Option<String>,

        /// Year filter (e.g., 2023 or 2020-2023)
        #[arg(short, long)]
        year: Option<String>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        output: OutputFormat,
    },

    /// Fetch paper by arXiv ID or Semantic Scholar ID
    Fetch {
        /// arXiv paper ID (e.g., 2106.09685)
        #[arg(long)]
        arxiv: Option<String>,

        /// Semantic Scholar paper ID
        #[arg(long)]
        ss: Option<String>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        output: OutputFormat,
    },

    /// Analyze paper using LLM
    Analyze {
        /// arXiv paper ID (e.g., 2106.09685)
        #[arg(long)]
        arxiv: Option<String>,

        /// Semantic Scholar paper ID
        #[arg(long)]
        ss: Option<String>,

        /// LLM provider (openai, anthropic, ollama)
        #[arg(short, long, value_enum)]
        provider: Option<ProviderArg>,

        /// Model name (e.g., gpt-4o, claude-3-opus-20240229)
        #[arg(short, long)]
        model: Option<String>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        output: OutputFormat,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// Human-readable text format
    Text,
    /// JSON format
    Json,
    /// XML format
    Xml,
    /// TOML format
    Toml,
}

#[derive(Clone, Copy, ValueEnum)]
enum ProviderArg {
    Openai,
    Anthropic,
    Ollama,
}

impl From<ProviderArg> for LlmProviderType {
    fn from(p: ProviderArg) -> Self {
        match p {
            ProviderArg::Openai => LlmProviderType::OpenAi,
            ProviderArg::Anthropic => LlmProviderType::Anthropic,
            ProviderArg::Ollama => LlmProviderType::Ollama,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_logger(&cli.log_level)?;

    match cli.command {
        Commands::Search {
            query,
            title,
            author,
            max_results,
            category,
            year,
            output,
        } => {
            cmd_search(query, title, author, max_results, category, year, output).await?;
        }
        Commands::Fetch { arxiv, ss, output } => {
            cmd_fetch(arxiv, ss, output).await?;
        }
        Commands::Analyze {
            arxiv,
            ss,
            provider,
            model,
            output,
        } => {
            cmd_analyze(arxiv, ss, provider, model, output).await?;
        }
    }

    Ok(())
}

async fn cmd_search(
    query: Option<String>,
    title: Option<String>,
    author: Option<String>,
    max_results: usize,
    category: Option<String>,
    year: Option<String>,
    output: OutputFormat,
) -> anyhow::Result<()> {
    if query.is_none() && title.is_none() && author.is_none() {
        anyhow::bail!("At least one of --query, --title, or --author is required");
    }

    let client = PaperClient::new();
    let mut params = SearchParams::new().with_max_results(max_results);

    if let Some(q) = query {
        params = params.with_query(q);
    }
    if let Some(t) = title {
        params = params.with_title(t);
    }
    if let Some(a) = author {
        params = params.with_author(a);
    }
    if let Some(c) = category {
        params = params.with_category(c);
    }
    if let Some(y) = year {
        params = params.with_year(y);
    }

    let result = client.search(params).await?;

    match output {
        OutputFormat::Text => {
            println!("Found {} papers:\n", result.papers.len());
            for (i, paper) in result.papers.iter().enumerate() {
                print_paper_summary(i + 1, paper);
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result.papers)?);
        }
        OutputFormat::Xml => {
            let wrapper = PapersWrapper {
                papers: &result.papers,
            };
            println!("{}", to_xml(&wrapper)?);
        }
        OutputFormat::Toml => {
            // TOML requires a table at root, so wrap in a struct
            #[derive(Serialize)]
            struct TomlPapers<'a> {
                papers: &'a [AcademicPaper],
            }
            let wrapper = TomlPapers {
                papers: &result.papers,
            };
            println!("{}", to_toml(&wrapper)?);
        }
    }

    Ok(())
}

async fn cmd_fetch(
    arxiv: Option<String>,
    ss: Option<String>,
    output: OutputFormat,
) -> anyhow::Result<()> {
    if arxiv.is_none() && ss.is_none() {
        anyhow::bail!("Either --arxiv or --ss is required");
    }

    let client = PaperClient::new();
    let mut params = SearchParams::new();

    if let Some(id) = arxiv {
        params = params.with_arxiv_id(id);
    }
    if let Some(id) = ss {
        params = params.with_ss_id(id);
    }

    let result = client.search(params).await?;

    if result.papers.is_empty() {
        anyhow::bail!("Paper not found");
    }

    let paper = &result.papers[0];

    match output {
        OutputFormat::Text => {
            print_paper_detail(paper);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(paper)?);
        }
        OutputFormat::Xml => {
            println!("{}", to_xml(paper)?);
        }
        OutputFormat::Toml => {
            println!("{}", to_toml(paper)?);
        }
    }

    Ok(())
}

async fn cmd_analyze(
    arxiv: Option<String>,
    ss: Option<String>,
    provider_arg: Option<ProviderArg>,
    model: Option<String>,
    output: OutputFormat,
) -> anyhow::Result<()> {
    if arxiv.is_none() && ss.is_none() {
        anyhow::bail!("Either --arxiv or --ss is required");
    }

    // Fetch paper first
    let client = PaperClient::new();
    let mut params = SearchParams::new();

    if let Some(id) = arxiv {
        params = params.with_arxiv_id(id);
    }
    if let Some(id) = ss {
        params = params.with_ss_id(id);
    }

    let result = client.search(params).await?;

    if result.papers.is_empty() {
        anyhow::bail!("Paper not found");
    }

    let mut paper = result.papers.into_iter().next().unwrap();

    // Determine provider
    let provider_type = provider_arg.map(LlmProviderType::from).unwrap_or_else(|| {
        std::env::var("LLM_PROVIDER")
            .ok()
            .and_then(|s| match s.as_str() {
                "openai" => Some(LlmProviderType::OpenAi),
                "anthropic" => Some(LlmProviderType::Anthropic),
                "ollama" => Some(LlmProviderType::Ollama),
                _ => None,
            })
            .unwrap_or(LlmProviderType::OpenAi)
    });

    // Analyze with appropriate provider
    match provider_type {
        LlmProviderType::OpenAi => {
            let provider = OpenAiProvider::from_env()?;
            analyze_with_provider(provider, &mut paper, model.as_deref()).await?;
        }
        LlmProviderType::Anthropic => {
            let provider = AnthropicProvider::from_env()?;
            analyze_with_provider(provider, &mut paper, model.as_deref()).await?;
        }
        LlmProviderType::Ollama => {
            let provider = OllamaProvider::from_env()?;
            analyze_with_provider(provider, &mut paper, model.as_deref()).await?;
        }
    }

    match output {
        OutputFormat::Text => {
            print_paper_with_analysis(&paper);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&paper)?);
        }
        OutputFormat::Xml => {
            println!("{}", to_xml(&paper)?);
        }
        OutputFormat::Toml => {
            println!("{}", to_toml(&paper)?);
        }
    }

    Ok(())
}

async fn analyze_with_provider<P: LlmProvider>(
    provider: P,
    paper: &mut AcademicPaper,
    model: Option<&str>,
) -> anyhow::Result<()> {
    let mut analyzer = PaperAnalyzer::new(provider);
    if let Some(m) = model {
        analyzer = analyzer.with_model(m);
    }
    analyzer.analyze_and_update(paper).await?;
    Ok(())
}

fn print_paper_summary(index: usize, paper: &AcademicPaper) {
    println!("{}. {}", index, paper.title);
    println!(
        "   Authors: {}",
        paper
            .authors
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    if !paper.arxiv_id.is_empty() {
        println!("   arXiv: {}", paper.arxiv_id);
    }
    if !paper.ss_id.is_empty() {
        println!("   SS ID: {}", paper.ss_id);
    }
    println!("   Published: {}", paper.published_date.format("%Y-%m-%d"));
    if paper.citations_count > 0 {
        println!("   Citations: {}", paper.citations_count);
    }
    println!();
}

fn print_paper_detail(paper: &AcademicPaper) {
    println!("Title: {}", paper.title);
    println!();
    println!(
        "Authors: {}",
        paper
            .authors
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();

    if !paper.arxiv_id.is_empty() {
        println!("arXiv ID: {}", paper.arxiv_id);
    }
    if !paper.ss_id.is_empty() {
        println!("Semantic Scholar ID: {}", paper.ss_id);
    }
    if !paper.doi.is_empty() {
        println!("DOI: {}", paper.doi);
    }
    println!("Published: {}", paper.published_date.format("%Y-%m-%d"));
    if !paper.journal.is_empty() {
        println!("Journal: {}", paper.journal);
    }
    if !paper.categories.is_empty() {
        println!("Categories: {}", paper.categories.join(", "));
    }
    if paper.citations_count > 0 {
        println!("Citations: {}", paper.citations_count);
    }
    println!("URL: {}", paper.url);
    println!();
    println!("Abstract:");
    println!("{}", paper.abstract_text);
}

fn print_paper_with_analysis(paper: &AcademicPaper) {
    print_paper_detail(paper);

    if let Some(analysis) = &paper.analysis {
        println!();
        println!("=== LLM Analysis ===");
        println!();
        println!("Summary:");
        println!("{}", analysis.summary);
        println!();

        println!("Background and Purpose:");
        println!("{}", analysis.background_and_purpose);
        println!();

        println!("Methodology:");
        println!("{}", analysis.methodology);
        println!();

        if !analysis.datasets.is_empty() {
            println!("Datasets:");
            for dataset in &analysis.datasets {
                println!("  - {}", dataset.name);
                if !dataset.url.is_empty() {
                    println!("    URL: {}", dataset.url);
                }
                if !dataset.description.is_empty() {
                    println!("    Description: {}", dataset.description);
                }
                if !dataset.domain.is_empty() {
                    println!("    Domain: {}", dataset.domain);
                }
                if !dataset.size.is_empty() {
                    println!("    Size: {}", dataset.size);
                }
                if !dataset.paper_title.is_empty() {
                    println!("    Original Paper: {}", dataset.paper_title);
                    if !dataset.paper_authors.is_empty() {
                        println!("    Paper Authors: {}", dataset.paper_authors);
                    }
                    if !dataset.paper_url.is_empty() {
                        println!("    Paper URL: {}", dataset.paper_url);
                    }
                }
            }
            println!();
        }

        println!("Results:");
        println!("{}", analysis.results);
        println!();

        println!("Advantages, Limitations and Future Work:");
        println!("{}", analysis.advantages_limitations_and_future_work);
        println!();

        if !analysis.key_contributions.is_empty() {
            println!("Key Contributions:");
            for contribution in &analysis.key_contributions {
                println!("  - {}", contribution);
            }
            println!();
        }

        if !analysis.tasks.is_empty() {
            println!("Tasks: {}", analysis.tasks.join(", "));
            println!();
        }

        println!(
            "Analyzed by: {} ({}) at {}",
            analysis.provider,
            analysis.model,
            analysis.analyzed_at.format("%Y-%m-%d %H:%M:%S")
        );
    }
}

// =============================================================================
// Output formatters
// =============================================================================

/// Serialize data to XML format
fn to_xml<T: Serialize>(data: &T) -> anyhow::Result<String> {
    let mut buffer = String::new();
    buffer.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let xml_content = quick_xml::se::to_string(data)?;
    buffer.push_str(&xml_content);
    Ok(buffer)
}

/// Serialize data to TOML format
fn to_toml<T: Serialize>(data: &T) -> anyhow::Result<String> {
    Ok(toml::to_string_pretty(data)?)
}

/// Wrapper for multiple papers (for XML root element)
#[derive(Serialize)]
struct PapersWrapper<'a> {
    #[serde(rename = "paper")]
    papers: &'a [AcademicPaper],
}
