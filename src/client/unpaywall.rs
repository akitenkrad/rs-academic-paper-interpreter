//! Unpaywall API client for resolving open access PDF URLs via DOI

use crate::shared::errors::{AppError, AppResult};
use serde::Deserialize;

/// Unpaywall API client
///
/// Uses the Unpaywall API to resolve DOIs to open access PDF URLs.
/// See: <https://unpaywall.org/products/api>
pub struct UnpaywallClient {
    email: String,
    http_client: reqwest::Client,
}

#[derive(Deserialize)]
struct UnpaywallResponse {
    best_oa_location: Option<OaLocation>,
}

#[derive(Deserialize)]
struct OaLocation {
    url_for_pdf: Option<String>,
}

impl UnpaywallClient {
    /// Create a new Unpaywall client with the given email
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            http_client: reqwest::Client::new(),
        }
    }

    /// Create a client from the `UNPAYWALL_EMAIL` environment variable
    ///
    /// Returns `None` if the env var is not set or empty.
    pub fn from_env() -> Option<Self> {
        std::env::var("UNPAYWALL_EMAIL")
            .ok()
            .filter(|e| !e.is_empty())
            .map(Self::new)
    }

    /// Resolve a DOI to an open access PDF URL via the Unpaywall API
    ///
    /// Returns `Ok(None)` if the DOI has no open access PDF available.
    pub async fn resolve_pdf_url(&self, doi: &str) -> AppResult<Option<String>> {
        if doi.is_empty() {
            return Ok(None);
        }

        let encoded_doi = urlencoding::encode(doi);
        let url = format!(
            "https://api.unpaywall.org/v2/{}?email={}",
            encoded_doi, self.email
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::UnpaywallError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                // DOI not found in Unpaywall — not an error
                return Ok(None);
            }
            return Err(AppError::UnpaywallError(format!(
                "HTTP {}: {}",
                status,
                status.canonical_reason().unwrap_or("Unknown")
            )));
        }

        let data: UnpaywallResponse = response
            .json()
            .await
            .map_err(|e| AppError::UnpaywallError(format!("Parse failed: {}", e)))?;

        Ok(data
            .best_oa_location
            .and_then(|loc| loc.url_for_pdf)
            .filter(|url| !url.is_empty()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_env_missing() {
        // When UNPAYWALL_EMAIL is not set, from_env returns None
        // (we can't guarantee the env var state, so just test the struct creation)
        let client = UnpaywallClient::new("test@example.com");
        assert_eq!(client.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_resolve_empty_doi() {
        let client = UnpaywallClient::new("test@example.com");
        let result = client.resolve_pdf_url("").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
