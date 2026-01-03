//! Client module for paper search and retrieval
//!
//! This module provides a unified interface for searching and fetching papers
//! from multiple sources (arXiv and Semantic Scholar).

mod arxiv;
mod search;
mod semantic;

pub use arxiv::ArxivClient;
pub use search::{PaperSource, SearchParams, SearchResult};
pub use semantic::SemanticScholarClient;

use crate::models::AcademicPaper;
use crate::pdf::PdfExtractor;
use crate::shared::errors::{AppError, AppResult};

/// Unified client for paper search and retrieval across multiple sources
pub struct PaperClient {
    arxiv: ArxivClient,
    semantic_scholar: SemanticScholarClient,
}

impl Default for PaperClient {
    fn default() -> Self {
        Self::new()
    }
}

impl PaperClient {
    /// Create a new paper client with default configuration
    pub fn new() -> Self {
        Self {
            arxiv: ArxivClient::new(),
            semantic_scholar: SemanticScholarClient::new(),
        }
    }

    /// Create a client with custom Semantic Scholar retry configuration
    pub fn with_ss_retry_config(mut self, retry_count: u64, wait_time: u64) -> Self {
        self.semantic_scholar = self
            .semantic_scholar
            .with_retry_config(retry_count, wait_time);
        self
    }

    /// Search papers across all sources
    ///
    /// Searches both arXiv and Semantic Scholar in parallel and merges results.
    pub async fn search(&self, params: SearchParams) -> AppResult<SearchResult> {
        // If it's an ID lookup, use the specific fetch methods
        if params.is_id_lookup() {
            return self.fetch_by_id(&params).await;
        }

        // Search both sources in parallel
        let arxiv_future = self.arxiv.search(&params);
        let ss_future = self.semantic_scholar.search(&params);

        let (arxiv_result, ss_result) = tokio::join!(arxiv_future, ss_future);

        let mut result = SearchResult::new();

        // Process arXiv results
        if let Ok(arxiv_papers) = arxiv_result {
            for paper in arxiv_papers {
                let academic_paper = AcademicPaper::from_arxiv(paper);
                result.papers.push(academic_paper);
            }
            result.sources.push(PaperSource::ArXiv);
        }

        // Process Semantic Scholar results
        if let Ok(ss_papers) = ss_result {
            for paper in ss_papers {
                let academic_paper = AcademicPaper::from_semantic_scholar(paper);
                result.papers.push(academic_paper);
            }
            result.sources.push(PaperSource::SemanticScholar);
        }

        // Deduplicate papers (by title similarity)
        result.papers = self.deduplicate_papers(result.papers);

        if result.papers.is_empty() {
            return Err(AppError::PaperNotFound(
                "No papers found matching the search criteria".to_string(),
            ));
        }

        Ok(result)
    }

    /// Fetch a paper by arXiv ID
    ///
    /// This method also attempts to extract PDF text automatically.
    /// If PDF extraction fails, the paper is still returned with `extracted_text` as `None`.
    pub async fn fetch_by_arxiv_id(&self, arxiv_id: &str) -> AppResult<AcademicPaper> {
        let arxiv_paper = self.arxiv.fetch_by_id(arxiv_id).await?;
        let mut paper = AcademicPaper::from_arxiv(arxiv_paper);

        // Try to enrich with Semantic Scholar data
        if let Ok(ss_paper) = self.semantic_scholar.search_exact_title(&paper.title).await {
            paper.enrich_from_semantic_scholar(ss_paper);
        }

        // Try to extract PDF text (non-fatal on failure)
        self.try_extract_text(&mut paper).await;

        Ok(paper)
    }

    /// Fetch a paper by Semantic Scholar ID
    ///
    /// This method also attempts to extract PDF text automatically.
    /// If PDF extraction fails, the paper is still returned with `extracted_text` as `None`.
    pub async fn fetch_by_ss_id(&self, ss_id: &str) -> AppResult<AcademicPaper> {
        let ss_paper = self.semantic_scholar.fetch_details(ss_id).await?;
        let mut paper = AcademicPaper::from_semantic_scholar(ss_paper);

        // Try to extract PDF text (non-fatal on failure)
        self.try_extract_text(&mut paper).await;

        Ok(paper)
    }

    /// Try to extract PDF text for a paper (non-fatal on failure)
    ///
    /// If extraction fails, a warning is logged and `extracted_text` remains `None`.
    async fn try_extract_text(&self, paper: &mut AcademicPaper) {
        let extractor = PdfExtractor::new();
        match extractor.extract_for_paper(paper).await {
            Ok(text) => {
                paper.set_extracted_text(text);
            }
            Err(e) => {
                tracing::warn!("PDF extraction failed for '{}': {}", paper.title, e);
            }
        }
    }

    /// Extract PDF text for a paper, returning an error on failure
    ///
    /// Use this method when you need to ensure text extraction succeeds.
    pub async fn extract_text(&self, paper: &mut AcademicPaper) -> AppResult<()> {
        let extractor = PdfExtractor::new();
        let text = extractor.extract_for_paper(paper).await?;
        paper.set_extracted_text(text);
        Ok(())
    }

    /// Fetch papers that cite the given paper
    pub async fn fetch_citations(&self, paper: &AcademicPaper) -> AppResult<Vec<AcademicPaper>> {
        let ss_id = paper.ss_id()?;
        let citations = self.semantic_scholar.fetch_citations(&ss_id).await?;

        Ok(citations
            .into_iter()
            .map(AcademicPaper::from_semantic_scholar)
            .collect())
    }

    /// Fetch papers referenced by the given paper
    pub async fn fetch_references(&self, paper: &AcademicPaper) -> AppResult<Vec<AcademicPaper>> {
        let ss_id = paper.ss_id()?;
        let references = self.semantic_scholar.fetch_references(&ss_id).await?;

        Ok(references
            .into_iter()
            .map(AcademicPaper::from_semantic_scholar)
            .collect())
    }

    /// Handle ID-based lookups
    async fn fetch_by_id(&self, params: &SearchParams) -> AppResult<SearchResult> {
        let mut result = SearchResult::new();

        if let Some(ref arxiv_id) = params.arxiv_id {
            let paper = self.fetch_by_arxiv_id(arxiv_id).await?;
            result.papers.push(paper);
            result.sources.push(PaperSource::ArXiv);
        }

        if let Some(ref ss_id) = params.ss_id {
            let paper = self.fetch_by_ss_id(ss_id).await?;
            result.papers.push(paper);
            result.sources.push(PaperSource::SemanticScholar);
        }

        Ok(result)
    }

    /// Deduplicate papers by title similarity
    fn deduplicate_papers(&self, papers: Vec<AcademicPaper>) -> Vec<AcademicPaper> {
        let mut unique_papers: Vec<AcademicPaper> = Vec::new();

        for paper in papers {
            let normalized_title = self.normalize_title(&paper.title);
            let is_duplicate = unique_papers.iter().any(|p| {
                let existing_normalized = self.normalize_title(&p.title);
                self.titles_match(&normalized_title, &existing_normalized)
            });

            if !is_duplicate {
                unique_papers.push(paper);
            }
        }

        unique_papers
    }

    /// Normalize title for comparison
    fn normalize_title(&self, title: &str) -> String {
        title
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Check if two normalized titles match
    fn titles_match(&self, title1: &str, title2: &str) -> bool {
        // Simple exact match after normalization
        // Could be enhanced with fuzzy matching
        title1 == title2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title() {
        let client = PaperClient::new();
        let title = "Attention Is All You Need!";
        let normalized = client.normalize_title(title);
        assert_eq!(normalized, "attention is all you need");
    }

    #[test]
    fn test_titles_match() {
        let client = PaperClient::new();
        assert!(client.titles_match("attention is all you need", "attention is all you need"));
        assert!(!client.titles_match("attention is all you need", "bert pretraining"));
    }
}
