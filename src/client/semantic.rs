//! Semantic Scholar API client wrapper

use crate::shared::errors::{AppError, AppResult};
use ss_tools::structs::{AuthorField, Paper as SsPaper, PaperField};
use ss_tools::{QueryParams as SsQueryParams, SemanticScholar};

use super::search::SearchParams;

/// Client for Semantic Scholar API operations
pub struct SemanticScholarClient {
    client: SemanticScholar,
    retry_count: u64,
    wait_time: u64,
}

impl Default for SemanticScholarClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticScholarClient {
    /// Create a new Semantic Scholar client
    pub fn new() -> Self {
        Self {
            client: SemanticScholar::new(),
            retry_count: 3,
            wait_time: 1,
        }
    }

    /// Set retry configuration
    pub fn with_retry_config(mut self, retry_count: u64, wait_time: u64) -> Self {
        self.retry_count = retry_count;
        self.wait_time = wait_time;
        self
    }

    /// Default paper fields to request from the Semantic Scholar API.
    ///
    /// Includes all commonly used fields plus ExternalIds for cross-referencing
    /// arXiv IDs and DOIs from Semantic Scholar responses.
    fn default_paper_fields() -> Vec<PaperField> {
        vec![
            PaperField::PaperId,
            PaperField::Title,
            PaperField::Abstract,
            PaperField::Url,
            PaperField::Venue,
            PaperField::Year,
            PaperField::ReferenceCount,
            PaperField::CitationCount,
            PaperField::InfluentialCitationCount,
            PaperField::IsOpenAccess,
            PaperField::OpenAccessPdf,
            PaperField::PublicationDate,
            PaperField::Journal,
            PaperField::CitationStyles,
            PaperField::ExternalIds,
            PaperField::Authors(vec![
                AuthorField::AuthorId,
                AuthorField::Name,
                AuthorField::Affiliations,
                AuthorField::PaperCount,
                AuthorField::CitationCount,
                AuthorField::HIndex,
            ]),
        ]
    }

    /// Search papers by title or query
    pub async fn search(&self, params: &SearchParams) -> AppResult<Vec<SsPaper>> {
        let query_text = self.build_query_text(params)?;
        let mut query_params = SsQueryParams::default();
        query_params.query_text(&query_text);
        query_params.fields(Self::default_paper_fields());
        query_params.limit(params.max_results as u64);

        if let Some(ref year) = params.year {
            query_params.year(year);
        }

        if let Some(min_citations) = params.min_citations {
            query_params.min_citation_count(min_citations);
        }

        let mut client = self.client.clone();
        let papers = client
            .query_papers_by_title(query_params, self.retry_count, self.wait_time)
            .await
            .map_err(|e| AppError::SemanticScholarError(format!("Search failed: {}", e)))?;

        Ok(papers)
    }

    /// Search for a single paper by exact title match
    pub async fn search_exact_title(&self, title: &str) -> AppResult<SsPaper> {
        let mut query_params = SsQueryParams::default();
        query_params.query_text(title);
        query_params.fields(Self::default_paper_fields());

        let mut client = self.client.clone();
        let paper = client
            .query_a_paper_by_title(query_params, self.retry_count, self.wait_time)
            .await
            .map_err(|e| {
                AppError::SemanticScholarError(format!("Exact title search failed: {}", e))
            })?;

        Ok(paper)
    }

    /// Fetch paper details by Semantic Scholar paper ID
    pub async fn fetch_details(&self, paper_id: &str) -> AppResult<SsPaper> {
        let mut query_params = SsQueryParams::default();
        query_params.paper_id(paper_id);
        query_params.fields(Self::default_paper_fields());

        let mut client = self.client.clone();
        let paper = client
            .query_paper_details(query_params, self.retry_count, self.wait_time)
            .await
            .map_err(|e| AppError::SemanticScholarError(format!("Fetch details failed: {}", e)))?;

        Ok(paper)
    }

    /// Fetch papers that cite the given paper
    pub async fn fetch_citations(&self, paper_id: &str) -> AppResult<Vec<SsPaper>> {
        let mut query_params = SsQueryParams::default();
        query_params.paper_id(paper_id);

        let mut client = self.client.clone();
        let response = client
            .query_paper_citations(query_params, self.retry_count, self.wait_time)
            .await
            .map_err(|e| {
                AppError::SemanticScholarError(format!("Fetch citations failed: {}", e))
            })?;

        // Extract papers from response data
        let papers = response
            .data
            .into_iter()
            .filter_map(|rd| rd.citing_paper)
            .collect();

        Ok(papers)
    }

    /// Fetch papers referenced by the given paper
    pub async fn fetch_references(&self, paper_id: &str) -> AppResult<Vec<SsPaper>> {
        let mut query_params = SsQueryParams::default();
        query_params.paper_id(paper_id);

        let mut client = self.client.clone();
        let response = client
            .query_paper_references(query_params, self.retry_count, self.wait_time)
            .await
            .map_err(|e| {
                AppError::SemanticScholarError(format!("Fetch references failed: {}", e))
            })?;

        // Extract papers from response data
        let papers = response
            .data
            .into_iter()
            .filter_map(|rd| rd.citing_paper)
            .collect();

        Ok(papers)
    }

    /// Build query text from search params
    fn build_query_text(&self, params: &SearchParams) -> AppResult<String> {
        // Prefer query, then title, then author
        if let Some(ref query) = params.query {
            Ok(query.clone())
        } else if let Some(ref title) = params.title {
            Ok(title.clone())
        } else if let Some(ref author) = params.author {
            Ok(author.clone())
        } else {
            Err(AppError::SemanticScholarError(
                "No search criteria provided".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_query_text_with_query() {
        let client = SemanticScholarClient::new();
        let params = SearchParams::new().with_query("attention mechanism".to_string());

        let query = client.build_query_text(&params);
        assert!(query.is_ok());
        assert_eq!(query.unwrap(), "attention mechanism");
    }

    #[test]
    fn test_build_query_text_with_title() {
        let client = SemanticScholarClient::new();
        let params = SearchParams::new().with_title("Attention Is All You Need".to_string());

        let query = client.build_query_text(&params);
        assert!(query.is_ok());
        assert_eq!(query.unwrap(), "Attention Is All You Need");
    }

    #[test]
    fn test_build_query_text_empty_params() {
        let client = SemanticScholarClient::new();
        let params = SearchParams::new();

        let query = client.build_query_text(&params);
        assert!(query.is_err());
    }
}
