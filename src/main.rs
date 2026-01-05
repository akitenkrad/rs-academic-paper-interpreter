//! CLI entry point for Academic Paper Interpreter

use academic_paper_interpreter::agents::providers::{
    AnthropicProvider, OllamaProvider, OpenAiProvider,
};
use academic_paper_interpreter::shared::config::LlmProviderType;
use academic_paper_interpreter::shared::logger::init_logger;
use academic_paper_interpreter::{
    AcademicPaper, CitationData, CitationStatistics, ExportOptions, ExportedPaper, KeywordsData,
    LlmProvider, PaperAnalyzer, PaperClient, PaperSummary, ReferenceData, ReferenceStatistics,
    ResearchContext, SearchParams,
};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::path::PathBuf;

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

    /// Export comprehensive paper data as JSON for AI/LLM consumption
    Export {
        /// arXiv paper ID (e.g., 2106.09685)
        #[arg(long)]
        arxiv: Option<String>,

        /// Semantic Scholar paper ID
        #[arg(long)]
        ss: Option<String>,

        /// Search by paper title (uses fuzzy matching with Levenshtein distance)
        #[arg(short = 't', long)]
        title: Option<String>,

        /// Similarity threshold for title matching (0.0 = exact match, 1.0 = no match required)
        #[arg(long, default_value = "0.3")]
        threshold: f64,

        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Run LLM analysis on the paper
        #[arg(short, long)]
        analyze: bool,

        /// Extract full text from PDF
        #[arg(short, long)]
        extract_text: bool,

        /// Include papers that cite this paper
        #[arg(short = 'c', long)]
        include_citations: bool,

        /// Include papers referenced by this paper
        #[arg(short = 'r', long)]
        include_references: bool,

        /// Maximum number of citations/references to fetch
        #[arg(long, default_value = "50")]
        max_citations: usize,

        /// LLM provider (openai, anthropic, ollama)
        #[arg(short, long, value_enum)]
        provider: Option<ProviderArg>,

        /// Model name for LLM analysis
        #[arg(short, long)]
        model: Option<String>,

        /// Extract keywords and topics via LLM
        #[arg(short = 'k', long)]
        extract_keywords: bool,

        /// Compact JSON output (no pretty printing)
        #[arg(long)]
        compact: bool,
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
        Commands::Export {
            arxiv,
            ss,
            title,
            threshold,
            output,
            analyze,
            extract_text,
            include_citations,
            include_references,
            max_citations,
            provider,
            model,
            extract_keywords,
            compact,
        } => {
            cmd_export(
                arxiv,
                ss,
                title,
                threshold,
                output,
                analyze,
                extract_text,
                include_citations,
                include_references,
                max_citations,
                provider,
                model,
                extract_keywords,
                compact,
            )
            .await?;
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

#[allow(clippy::too_many_arguments)]
async fn cmd_export(
    arxiv: Option<String>,
    ss: Option<String>,
    title: Option<String>,
    threshold: f64,
    output_path: Option<PathBuf>,
    analyze: bool,
    extract_text: bool,
    include_citations: bool,
    include_references: bool,
    max_citations: usize,
    provider_arg: Option<ProviderArg>,
    model: Option<String>,
    extract_keywords: bool,
    compact: bool,
) -> anyhow::Result<()> {
    if arxiv.is_none() && ss.is_none() && title.is_none() {
        anyhow::bail!("Either --arxiv, --ss, or --title is required");
    }

    // Build export options
    let mut export_options = ExportOptions {
        analyzed: analyze,
        text_extracted: extract_text,
        citations_included: include_citations,
        references_included: include_references,
        keywords_extracted: extract_keywords,
        max_citations,
        llm_provider: None,
        llm_model: None,
    };

    // Fetch paper
    let client = PaperClient::new();

    let mut paper = if let Some(ref title_query) = title {
        // Search by title using fuzzy matching
        eprintln!("Searching for paper: \"{}\" (threshold: {:.2})", title_query, threshold);
        let found_paper = client.search_by_title_fuzzy(title_query, threshold).await?;
        eprintln!("Found: \"{}\"", found_paper.title);
        found_paper
    } else {
        // Search by ID
        let mut params = SearchParams::new();
        if let Some(id) = &arxiv {
            params = params.with_arxiv_id(id.clone());
        }
        if let Some(id) = &ss {
            params = params.with_ss_id(id.clone());
        }

        let result = client.search(params).await?;

        if result.papers.is_empty() {
            anyhow::bail!("Paper not found");
        }

        result.papers.into_iter().next().unwrap()
    };

    let mut exported = ExportedPaper::new(paper.clone(), export_options.clone());

    // Determine provider type for LLM operations
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

    // Extract text if requested
    if extract_text && !paper.has_extracted_text() {
        match client.extract_text(&mut paper).await {
            Ok(_) => {}
            Err(e) => {
                exported.add_warning(format!("Text extraction failed: {}", e));
            }
        }
    }

    // Run LLM analysis if requested
    if analyze && !paper.is_analyzed() {
        let analyze_result = match provider_type {
            LlmProviderType::OpenAi => {
                let provider = OpenAiProvider::from_env()?;
                export_options.llm_provider = Some("openai".to_string());
                analyze_with_provider(provider, &mut paper, model.as_deref()).await
            }
            LlmProviderType::Anthropic => {
                let provider = AnthropicProvider::from_env()?;
                export_options.llm_provider = Some("anthropic".to_string());
                analyze_with_provider(provider, &mut paper, model.as_deref()).await
            }
            LlmProviderType::Ollama => {
                let provider = OllamaProvider::from_env()?;
                export_options.llm_provider = Some("ollama".to_string());
                analyze_with_provider(provider, &mut paper, model.as_deref()).await
            }
        };

        if let Err(e) = analyze_result {
            exported.add_warning(format!("LLM analysis failed: {}", e));
        }
        export_options.llm_model = model.clone();
    }

    // Fetch citations and references in parallel
    let (citations_result, references_result) = if include_citations || include_references {
        let citations_future = async {
            if include_citations {
                fetch_citations(&client, &paper, max_citations).await
            } else {
                Ok(None)
            }
        };

        let references_future = async {
            if include_references {
                fetch_references(&client, &paper, max_citations).await
            } else {
                Ok(None)
            }
        };

        tokio::join!(citations_future, references_future)
    } else {
        (Ok(None), Ok(None))
    };

    // Handle citations result
    match citations_result {
        Ok(Some(citations)) => {
            exported.citations = Some(citations);
        }
        Ok(None) => {}
        Err(e) => {
            exported.add_warning(format!("Citations fetch failed: {}", e));
        }
    }

    // Handle references result
    match references_result {
        Ok(Some(references)) => {
            exported.references = Some(references);
        }
        Ok(None) => {}
        Err(e) => {
            exported.add_warning(format!("References fetch failed: {}", e));
        }
    }

    // Extract keywords if requested
    if extract_keywords {
        let keywords_result = match provider_type {
            LlmProviderType::OpenAi => {
                let provider = OpenAiProvider::from_env()?;
                extract_keywords_with_provider(provider, &paper, model.as_deref()).await
            }
            LlmProviderType::Anthropic => {
                let provider = AnthropicProvider::from_env()?;
                extract_keywords_with_provider(provider, &paper, model.as_deref()).await
            }
            LlmProviderType::Ollama => {
                let provider = OllamaProvider::from_env()?;
                extract_keywords_with_provider(provider, &paper, model.as_deref()).await
            }
        };

        match keywords_result {
            Ok((keywords, context)) => {
                exported.keywords = Some(keywords);
                exported.research_context = Some(context);
            }
            Err(e) => {
                exported.add_warning(format!("Keyword extraction failed: {}", e));
            }
        }
    }

    // Update paper in exported
    exported.paper = paper;
    exported.export_metadata.options = export_options;

    // Output JSON
    let json_output = if compact {
        serde_json::to_string(&exported)?
    } else {
        serde_json::to_string_pretty(&exported)?
    };

    match output_path {
        Some(path) => {
            std::fs::write(&path, &json_output)?;
            eprintln!("Exported to: {}", path.display());
        }
        None => {
            println!("{}", json_output);
        }
    }

    Ok(())
}

async fn fetch_citations(
    client: &PaperClient,
    paper: &AcademicPaper,
    max_citations: usize,
) -> anyhow::Result<Option<CitationData>> {
    let citations = client.fetch_citations(paper).await?;
    let limited: Vec<_> = citations.into_iter().take(max_citations).collect();

    if limited.is_empty() {
        return Ok(None);
    }

    let summaries: Vec<PaperSummary> = limited
        .iter()
        .map(PaperSummary::from_academic_paper)
        .collect();
    let statistics = CitationStatistics::from_papers(&summaries);

    Ok(Some(CitationData {
        total_count: paper.citations_count,
        fetched_count: summaries.len(),
        papers: summaries,
        statistics,
    }))
}

async fn fetch_references(
    client: &PaperClient,
    paper: &AcademicPaper,
    max_citations: usize,
) -> anyhow::Result<Option<ReferenceData>> {
    let references = client.fetch_references(paper).await?;
    let limited: Vec<_> = references.into_iter().take(max_citations).collect();

    if limited.is_empty() {
        return Ok(None);
    }

    let summaries: Vec<PaperSummary> = limited
        .iter()
        .map(PaperSummary::from_academic_paper)
        .collect();
    let statistics = ReferenceStatistics::from_papers(&summaries);

    Ok(Some(ReferenceData {
        total_count: paper.references_count,
        fetched_count: summaries.len(),
        papers: summaries,
        statistics,
    }))
}

async fn extract_keywords_with_provider<P: LlmProvider>(
    provider: P,
    paper: &AcademicPaper,
    model: Option<&str>,
) -> anyhow::Result<(KeywordsData, ResearchContext)> {
    let mut analyzer = PaperAnalyzer::new(provider);
    if let Some(m) = model {
        analyzer = analyzer.with_model(m);
    }

    let keywords = analyzer.extract_keywords(paper).await?;
    let context = analyzer
        .extract_research_context(paper, &keywords.keywords)
        .await?;

    Ok((keywords, context))
}
