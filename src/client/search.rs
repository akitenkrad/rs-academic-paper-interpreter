//! Search parameters and result types for paper queries

use crate::models::AcademicPaper;
use derive_new::new;
use serde::{Deserialize, Serialize};

/// Source of paper data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperSource {
    ArXiv,
    SemanticScholar,
    Both,
}

/// Unified search parameters for paper queries
#[derive(Debug, Clone, Default, new)]
pub struct SearchParams {
    /// Full-text query string
    #[new(default)]
    pub query: Option<String>,

    /// Search in paper titles
    #[new(default)]
    pub title: Option<String>,

    /// Search by author name
    #[new(default)]
    pub author: Option<String>,

    /// Search in abstracts
    #[new(default)]
    pub abstract_contains: Option<String>,

    /// Fetch by arXiv ID
    #[new(default)]
    pub arxiv_id: Option<String>,

    /// Fetch by Semantic Scholar ID
    #[new(default)]
    pub ss_id: Option<String>,

    /// Maximum number of results
    #[new(value = "10")]
    pub max_results: usize,

    /// Filter by arXiv categories (e.g., "cs.AI", "cs.CL")
    #[new(default)]
    pub categories: Vec<String>,

    /// Minimum citation count filter
    #[new(default)]
    pub min_citations: Option<u32>,

    /// Year filter (e.g., "2023" or "2020-2023")
    #[new(default)]
    pub year: Option<String>,
}

impl SearchParams {
    /// Set the full-text query
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Set the title filter
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the author filter
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the abstract filter
    pub fn with_abstract(mut self, text: impl Into<String>) -> Self {
        self.abstract_contains = Some(text.into());
        self
    }

    /// Set the arXiv ID for direct fetch
    pub fn with_arxiv_id(mut self, id: impl Into<String>) -> Self {
        self.arxiv_id = Some(id.into());
        self
    }

    /// Set the Semantic Scholar ID for direct fetch
    pub fn with_ss_id(mut self, id: impl Into<String>) -> Self {
        self.ss_id = Some(id.into());
        self
    }

    /// Set the maximum number of results
    pub fn with_max_results(mut self, n: usize) -> Self {
        self.max_results = n;
        self
    }

    /// Add arXiv category filter
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.categories.push(category.into());
        self
    }

    /// Set minimum citation count
    pub fn with_min_citations(mut self, count: u32) -> Self {
        self.min_citations = Some(count);
        self
    }

    /// Set year filter
    pub fn with_year(mut self, year: impl Into<String>) -> Self {
        self.year = Some(year.into());
        self
    }

    /// Check if this is a direct ID lookup
    pub fn is_id_lookup(&self) -> bool {
        self.arxiv_id.is_some() || self.ss_id.is_some()
    }

    /// Check if any search criteria are set
    pub fn has_search_criteria(&self) -> bool {
        self.query.is_some()
            || self.title.is_some()
            || self.author.is_some()
            || self.abstract_contains.is_some()
    }
}

/// Search result with papers and metadata
#[derive(Debug, Clone, Default)]
pub struct SearchResult {
    /// Found papers
    pub papers: Vec<AcademicPaper>,

    /// Sources that returned results
    pub sources: Vec<PaperSource>,

    /// Total count (if available from API)
    pub total_count: Option<usize>,
}

impl SearchResult {
    /// Create a new empty search result
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if no results were found
    pub fn is_empty(&self) -> bool {
        self.papers.is_empty()
    }

    /// Get the number of papers found
    pub fn len(&self) -> usize {
        self.papers.len()
    }
}
