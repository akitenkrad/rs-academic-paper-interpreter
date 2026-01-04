//! arXiv API client wrapper

use crate::shared::errors::{AppError, AppResult};
use arxiv_tools::{ArXiv, Paper as ArxivPaper, QueryParams, SortBy, SortOrder};

use super::search::SearchParams;

/// Client for arXiv API operations
#[derive(Debug, Clone, Default)]
pub struct ArxivClient;

impl ArxivClient {
    /// Create a new arXiv client
    pub fn new() -> Self {
        Self
    }

    /// Search papers on arXiv
    pub async fn search(&self, params: &SearchParams) -> AppResult<Vec<ArxivPaper>> {
        let query = self.build_query(params)?;

        let papers = ArXiv::from_args(query)
            .max_results(params.max_results as u64)
            .sort_by(SortBy::SubmittedDate)
            .sort_order(SortOrder::Descending)
            .query()
            .await;

        Ok(papers)
    }

    /// Fetch a single paper by arXiv ID
    pub async fn fetch_by_id(&self, arxiv_id: &str) -> AppResult<ArxivPaper> {
        let papers = ArXiv::from_id_list(vec![arxiv_id]).query().await;

        papers
            .into_iter()
            .next()
            .ok_or_else(|| AppError::PaperNotFound(format!("arXiv paper not found: {}", arxiv_id)))
    }

    /// Build QueryParams from SearchParams
    fn build_query(&self, params: &SearchParams) -> AppResult<QueryParams> {
        let mut conditions: Vec<QueryParams> = Vec::new();

        // Add title filter
        if let Some(ref title) = params.title {
            conditions.push(QueryParams::title(title));
        }

        // Add author filter
        if let Some(ref author) = params.author {
            conditions.push(QueryParams::author(author));
        }

        // Add abstract filter
        if let Some(ref abstract_text) = params.abstract_contains {
            conditions.push(QueryParams::abstract_text(abstract_text));
        }

        // Note: category filter requires Category enum, skipping for now
        // as it requires mapping from string to Category

        // Build the final query
        if conditions.is_empty() {
            // If no specific conditions, use the general query
            if let Some(ref query) = params.query {
                Ok(QueryParams::all(query))
            } else {
                Err(AppError::ArxivError(
                    "No search criteria provided".to_string(),
                ))
            }
        } else if conditions.len() == 1 {
            Ok(conditions.pop().unwrap())
        } else {
            Ok(QueryParams::and(conditions))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_params_build() {
        let client = ArxivClient::new();
        let params = SearchParams::new().with_title("attention".to_string());

        let query = client.build_query(&params);
        assert!(query.is_ok());
    }

    #[test]
    fn test_empty_params_with_query() {
        let client = ArxivClient::new();
        let params = SearchParams::new().with_query("transformer".to_string());

        let query = client.build_query(&params);
        assert!(query.is_ok());
    }

    #[test]
    fn test_empty_params_error() {
        let client = ArxivClient::new();
        let params = SearchParams::new();

        let query = client.build_query(&params);
        assert!(query.is_err());
    }
}
