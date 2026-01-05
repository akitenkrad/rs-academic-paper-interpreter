//! Export data structures for comprehensive paper data output
//!
//! This module provides structures for exporting academic paper data
//! in a format optimized for LLM/AI agent consumption.

use crate::models::AcademicPaper;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Current schema version for export format
pub const EXPORT_SCHEMA_VERSION: &str = "1.0.0";

/// Comprehensive exported paper data for AI/LLM consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedPaper {
    /// Schema version for compatibility checking
    pub schema_version: String,

    /// Export metadata
    pub export_metadata: ExportMetadata,

    /// Main paper data
    pub paper: AcademicPaper,

    /// Papers citing this paper (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<CitationData>,

    /// Papers referenced by this paper (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<ReferenceData>,

    /// Extracted keywords and topics (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<KeywordsData>,

    /// Research positioning context (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub research_context: Option<ResearchContext>,
}

impl ExportedPaper {
    /// Create a new ExportedPaper with default metadata
    pub fn new(paper: AcademicPaper, options: ExportOptions) -> Self {
        Self {
            schema_version: EXPORT_SCHEMA_VERSION.to_string(),
            export_metadata: ExportMetadata {
                exported_at: Local::now(),
                tool_version: env!("CARGO_PKG_VERSION").to_string(),
                options,
                warnings: Vec::new(),
            },
            paper,
            citations: None,
            references: None,
            keywords: None,
            research_context: None,
        }
    }

    /// Add a warning message
    pub fn add_warning(&mut self, warning: String) {
        self.export_metadata.warnings.push(warning);
    }
}

/// Export operation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// When the export was performed
    pub exported_at: DateTime<Local>,

    /// Tool version
    pub tool_version: String,

    /// Options used for this export
    pub options: ExportOptions,

    /// Any warnings or notes about the export
    pub warnings: Vec<String>,
}

/// Options used during export (for reproducibility)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportOptions {
    /// Whether LLM analysis was performed
    pub analyzed: bool,

    /// Whether PDF text was extracted
    pub text_extracted: bool,

    /// Whether citations were included
    pub citations_included: bool,

    /// Whether references were included
    pub references_included: bool,

    /// Whether keywords were extracted
    pub keywords_extracted: bool,

    /// Maximum number of citations/references fetched
    pub max_citations: usize,

    /// LLM provider used (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_provider: Option<String>,

    /// LLM model used (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_model: Option<String>,
}

/// Citation network data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationData {
    /// Total citation count from API
    pub total_count: i32,

    /// Number of citations fetched
    pub fetched_count: usize,

    /// Citing papers (simplified)
    pub papers: Vec<PaperSummary>,

    /// Citation statistics
    pub statistics: CitationStatistics,
}

/// Reference network data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceData {
    /// Total reference count from API
    pub total_count: i32,

    /// Number of references fetched
    pub fetched_count: usize,

    /// Referenced papers (simplified)
    pub papers: Vec<PaperSummary>,

    /// Reference statistics
    pub statistics: ReferenceStatistics,
}

/// Simplified paper representation for citations/references
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaperSummary {
    /// Semantic Scholar paper ID
    pub ss_id: String,

    /// arXiv paper ID
    pub arxiv_id: String,

    /// Digital Object Identifier
    pub doi: String,

    /// Paper title
    pub title: String,

    /// Author names (just names for brevity)
    pub authors: Vec<String>,

    /// Publication year
    pub year: i32,

    /// Journal or venue name
    pub venue: String,

    /// Citation count
    pub citation_count: i32,

    /// Influential citation count
    pub influential_citation_count: i32,

    /// Brief abstract (truncated if too long)
    pub abstract_snippet: String,

    /// Paper URL
    pub url: String,
}

impl PaperSummary {
    /// Create a PaperSummary from AcademicPaper
    pub fn from_academic_paper(paper: &AcademicPaper) -> Self {
        let abstract_snippet = if paper.abstract_text.len() > 500 {
            format!("{}...", &paper.abstract_text[..500])
        } else {
            paper.abstract_text.clone()
        };

        Self {
            ss_id: paper.ss_id.clone(),
            arxiv_id: paper.arxiv_id.clone(),
            doi: paper.doi.clone(),
            title: paper.title.clone(),
            authors: paper.authors.iter().map(|a| a.name.clone()).collect(),
            year: paper
                .published_date
                .format("%Y")
                .to_string()
                .parse()
                .unwrap_or(0),
            venue: paper.journal.clone(),
            citation_count: paper.citations_count,
            influential_citation_count: paper.influential_citation_count,
            abstract_snippet,
            url: paper.url.clone(),
        }
    }
}

/// Statistics about citations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CitationStatistics {
    /// Year distribution of citations
    pub by_year: HashMap<i32, usize>,

    /// Top venues citing this paper
    pub top_venues: Vec<(String, usize)>,

    /// Average citations of citing papers
    pub avg_citation_count: f64,

    /// Most influential citing papers (top 5 by citation count)
    pub most_influential: Vec<String>,
}

impl CitationStatistics {
    /// Calculate statistics from a list of papers
    pub fn from_papers(papers: &[PaperSummary]) -> Self {
        let mut by_year: HashMap<i32, usize> = HashMap::new();
        let mut venues: HashMap<String, usize> = HashMap::new();
        let mut total_citations = 0i64;

        for paper in papers {
            if paper.year > 0 {
                *by_year.entry(paper.year).or_insert(0) += 1;
            }
            if !paper.venue.is_empty() {
                *venues.entry(paper.venue.clone()).or_insert(0) += 1;
            }
            total_citations += paper.citation_count as i64;
        }

        // Sort venues by count and take top 10
        let mut venue_vec: Vec<_> = venues.into_iter().collect();
        venue_vec.sort_by(|a, b| b.1.cmp(&a.1));
        let top_venues: Vec<_> = venue_vec.into_iter().take(10).collect();

        // Get most influential papers (top 5 by citation count)
        let mut sorted_papers: Vec<_> = papers.iter().collect();
        sorted_papers.sort_by(|a, b| b.citation_count.cmp(&a.citation_count));
        let most_influential: Vec<_> = sorted_papers
            .iter()
            .take(5)
            .map(|p| p.title.clone())
            .collect();

        let avg_citation_count = if papers.is_empty() {
            0.0
        } else {
            total_citations as f64 / papers.len() as f64
        };

        Self {
            by_year,
            top_venues,
            avg_citation_count,
            most_influential,
        }
    }
}

/// Statistics about references
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReferenceStatistics {
    /// Year distribution of references
    pub by_year: HashMap<i32, usize>,

    /// Oldest and newest reference years
    pub year_range: Option<(i32, i32)>,

    /// Top venues referenced
    pub top_venues: Vec<(String, usize)>,
}

impl ReferenceStatistics {
    /// Calculate statistics from a list of papers
    pub fn from_papers(papers: &[PaperSummary]) -> Self {
        let mut by_year: HashMap<i32, usize> = HashMap::new();
        let mut venues: HashMap<String, usize> = HashMap::new();
        let mut min_year: Option<i32> = None;
        let mut max_year: Option<i32> = None;

        for paper in papers {
            if paper.year > 0 {
                *by_year.entry(paper.year).or_insert(0) += 1;
                min_year = Some(min_year.map_or(paper.year, |m| m.min(paper.year)));
                max_year = Some(max_year.map_or(paper.year, |m| m.max(paper.year)));
            }
            if !paper.venue.is_empty() {
                *venues.entry(paper.venue.clone()).or_insert(0) += 1;
            }
        }

        // Sort venues by count and take top 10
        let mut venue_vec: Vec<_> = venues.into_iter().collect();
        venue_vec.sort_by(|a, b| b.1.cmp(&a.1));
        let top_venues: Vec<_> = venue_vec.into_iter().take(10).collect();

        let year_range = match (min_year, max_year) {
            (Some(min), Some(max)) => Some((min, max)),
            _ => None,
        };

        Self {
            by_year,
            year_range,
            top_venues,
        }
    }
}

/// Extracted keywords and topics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeywordsData {
    /// Primary research keywords
    pub keywords: Vec<String>,

    /// Research topics/domains
    pub topics: Vec<String>,

    /// Technical terms with definitions
    pub technical_terms: Vec<TechnicalTerm>,

    /// Methods/techniques mentioned
    pub methods: Vec<String>,

    /// Datasets mentioned
    pub datasets: Vec<String>,
}

/// Technical term with optional definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalTerm {
    /// The term
    pub term: String,

    /// Definition (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
}

/// Research field positioning
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResearchContext {
    /// Primary research field
    pub primary_field: String,

    /// Sub-fields
    pub sub_fields: Vec<String>,

    /// Research type (empirical, theoretical, survey, etc.)
    pub research_type: String,

    /// Positioning statement (LLM-generated)
    pub positioning: String,

    /// Related research directions
    pub related_directions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_paper() -> AcademicPaper {
        let mut paper = AcademicPaper::new();
        paper.ss_id = "123".to_string();
        paper.arxiv_id = "2106.09685".to_string();
        paper.title = "Test Paper".to_string();
        paper.abstract_text = "This is a test abstract".to_string();
        paper.citations_count = 100;
        paper
    }

    #[test]
    fn test_paper_summary_from_academic_paper() {
        let paper = create_test_paper();

        let summary = PaperSummary::from_academic_paper(&paper);
        assert_eq!(summary.ss_id, "123");
        assert_eq!(summary.arxiv_id, "2106.09685");
        assert_eq!(summary.title, "Test Paper");
        assert_eq!(summary.citation_count, 100);
    }

    #[test]
    fn test_abstract_truncation() {
        let long_abstract = "a".repeat(600);
        let mut paper = AcademicPaper::new();
        paper.abstract_text = long_abstract;

        let summary = PaperSummary::from_academic_paper(&paper);
        assert!(summary.abstract_snippet.len() < 510);
        assert!(summary.abstract_snippet.ends_with("..."));
    }

    #[test]
    fn test_citation_statistics() {
        let papers = vec![
            PaperSummary {
                year: 2020,
                venue: "NeurIPS".to_string(),
                citation_count: 100,
                title: "Paper 1".to_string(),
                ..Default::default()
            },
            PaperSummary {
                year: 2021,
                venue: "NeurIPS".to_string(),
                citation_count: 50,
                title: "Paper 2".to_string(),
                ..Default::default()
            },
            PaperSummary {
                year: 2020,
                venue: "ICML".to_string(),
                citation_count: 200,
                title: "Paper 3".to_string(),
                ..Default::default()
            },
        ];

        let stats = CitationStatistics::from_papers(&papers);
        assert_eq!(stats.by_year.get(&2020), Some(&2));
        assert_eq!(stats.by_year.get(&2021), Some(&1));
        assert!(!stats.top_venues.is_empty());
        assert_eq!(stats.most_influential[0], "Paper 3");
    }

    #[test]
    fn test_exported_paper_new() {
        let paper = AcademicPaper::new();
        let options = ExportOptions::default();
        let exported = ExportedPaper::new(paper, options);

        assert_eq!(exported.schema_version, EXPORT_SCHEMA_VERSION);
        assert!(exported.citations.is_none());
        assert!(exported.references.is_none());
    }
}
