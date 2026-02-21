//! PDF URL resolution with multi-stage fallback chain
//!
//! Fallback order:
//! 1. `open_access_pdf_url` (from paper metadata)
//! 2. arXiv PDF URL (from `arxiv_id`)
//! 3. Semantic Scholar re-fetch (by ss_id or DOI)
//! 4. Unpaywall API (by DOI)
//! 5. Error

use crate::client::SemanticScholarClient;
use crate::client::UnpaywallClient;
use crate::models::AcademicPaper;
use crate::shared::errors::{AppError, AppResult};

/// Resolves PDF URLs for academic papers using a multi-stage fallback chain
pub struct PdfUrlResolver<'a> {
    semantic_scholar: &'a SemanticScholarClient,
    unpaywall: Option<&'a UnpaywallClient>,
}

impl<'a> PdfUrlResolver<'a> {
    /// Create a new resolver with the given clients
    pub fn new(
        semantic_scholar: &'a SemanticScholarClient,
        unpaywall: Option<&'a UnpaywallClient>,
    ) -> Self {
        Self {
            semantic_scholar,
            unpaywall,
        }
    }

    /// Resolve a PDF URL for the given paper using the fallback chain
    pub async fn resolve(&self, paper: &AcademicPaper) -> AppResult<String> {
        // 1. Try open_access_pdf_url
        if let Some(ref url) = paper.open_access_pdf_url
            && !url.is_empty()
        {
            tracing::debug!("PDF URL resolved via open_access_pdf_url");
            return Ok(url.clone());
        }

        // 2. Try arXiv PDF URL
        if !paper.arxiv_id.is_empty() {
            tracing::debug!("PDF URL resolved via arXiv ID: {}", paper.arxiv_id);
            return Ok(format!("https://arxiv.org/pdf/{}", paper.arxiv_id));
        }

        // 3. Try Semantic Scholar re-fetch
        if let Some(url) = self.try_ss_refetch(paper).await {
            tracing::debug!("PDF URL resolved via Semantic Scholar re-fetch");
            return Ok(url);
        }

        // 4. Try Unpaywall API
        if let Some(url) = self.try_unpaywall(paper).await {
            tracing::debug!("PDF URL resolved via Unpaywall API");
            return Ok(url);
        }

        // 5. All fallbacks exhausted
        Err(AppError::PdfExtractionError(
            "No PDF URL available for this paper (tried: open_access_pdf_url, arXiv, Semantic Scholar re-fetch, Unpaywall)".to_string(),
        ))
    }

    /// Try to resolve PDF URL by re-fetching from Semantic Scholar
    ///
    /// Uses the paper's ss_id directly, or constructs a `DOI:{doi}` identifier.
    async fn try_ss_refetch(&self, paper: &AcademicPaper) -> Option<String> {
        // Determine paper_id for SS API
        let paper_id = if !paper.ss_id.is_empty() {
            paper.ss_id.clone()
        } else if !paper.doi.is_empty() {
            format!("DOI:{}", paper.doi)
        } else {
            tracing::debug!("SS re-fetch skipped: no ss_id or DOI");
            return None;
        };

        tracing::debug!("Attempting SS re-fetch with ID: {}", paper_id);

        match self.semantic_scholar.fetch_details(&paper_id).await {
            Ok(ss_paper) => {
                // Check for open access PDF in the response
                let url = ss_paper
                    .open_access_pdf
                    .as_ref()
                    .and_then(|pdf| pdf.url.as_ref())
                    .filter(|u| !u.is_empty())
                    .cloned();

                if url.is_none() {
                    // Also check for arXiv ID in external_ids
                    if let Some(arxiv_id) = ss_paper
                        .external_ids
                        .as_ref()
                        .and_then(|ids| ids.arxiv.as_ref())
                        .filter(|id| !id.is_empty())
                    {
                        tracing::debug!(
                            "SS re-fetch found arXiv ID: {}",
                            arxiv_id
                        );
                        return Some(format!("https://arxiv.org/pdf/{}", arxiv_id));
                    }
                    tracing::debug!("SS re-fetch: no PDF URL in response");
                }

                url
            }
            Err(e) => {
                tracing::debug!("SS re-fetch failed: {}", e);
                None
            }
        }
    }

    /// Try to resolve PDF URL via the Unpaywall API
    async fn try_unpaywall(&self, paper: &AcademicPaper) -> Option<String> {
        let unpaywall = match self.unpaywall {
            Some(client) => client,
            None => {
                tracing::debug!("Unpaywall skipped: client not configured (set UNPAYWALL_EMAIL)");
                return None;
            }
        };

        if paper.doi.is_empty() {
            tracing::debug!("Unpaywall skipped: no DOI available");
            return None;
        }

        tracing::debug!("Attempting Unpaywall lookup for DOI: {}", paper.doi);

        match unpaywall.resolve_pdf_url(&paper.doi).await {
            Ok(Some(url)) => Some(url),
            Ok(None) => {
                tracing::debug!("Unpaywall: no open access PDF for this DOI");
                None
            }
            Err(e) => {
                tracing::debug!("Unpaywall lookup failed: {}", e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_open_access_pdf_url() {
        let ss = SemanticScholarClient::new();
        let resolver = PdfUrlResolver::new(&ss, None);

        let mut paper = AcademicPaper::new();
        paper.open_access_pdf_url = Some("https://example.com/paper.pdf".to_string());

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(resolver.resolve(&paper));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/paper.pdf");
    }

    #[test]
    fn test_resolve_arxiv_fallback() {
        let ss = SemanticScholarClient::new();
        let resolver = PdfUrlResolver::new(&ss, None);

        let mut paper = AcademicPaper::new();
        paper.arxiv_id = "1706.03762".to_string();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(resolver.resolve(&paper));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://arxiv.org/pdf/1706.03762");
    }

    #[test]
    fn test_resolve_empty_open_access_url_falls_through() {
        let ss = SemanticScholarClient::new();
        let resolver = PdfUrlResolver::new(&ss, None);

        let mut paper = AcademicPaper::new();
        paper.open_access_pdf_url = Some("".to_string());
        paper.arxiv_id = "1706.03762".to_string();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(resolver.resolve(&paper));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://arxiv.org/pdf/1706.03762");
    }

    #[test]
    fn test_resolve_all_fallbacks_fail() {
        let ss = SemanticScholarClient::new();
        let resolver = PdfUrlResolver::new(&ss, None);

        let paper = AcademicPaper::new();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(resolver.resolve(&paper));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("No PDF URL available"));
    }
}
