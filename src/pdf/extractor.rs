//! PDF text extraction implementation using rsrpp

use crate::models::{AcademicPaper, PaperSection, PaperText};
use crate::shared::errors::{AppError, AppResult};
use chrono::Local;
use futures::FutureExt;
use rsrpp::config::ParserConfig;
use rsrpp::models::Section;
use rsrpp::parser::parse;
use std::panic::AssertUnwindSafe;

/// Configuration for PDF extraction
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    /// Enable verbose logging during extraction
    pub verbose: bool,
    /// Cleanup temporary files after extraction
    pub cleanup: bool,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            cleanup: true,
        }
    }
}

impl ExtractionConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set verbose logging
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set cleanup behavior
    pub fn with_cleanup(mut self, cleanup: bool) -> Self {
        self.cleanup = cleanup;
        self
    }
}

/// PDF text extractor using rsrpp
pub struct PdfExtractor {
    config: ExtractionConfig,
}

impl PdfExtractor {
    /// Create a new extractor with default configuration
    pub fn new() -> Self {
        Self {
            config: ExtractionConfig::default(),
        }
    }

    /// Create a new extractor with custom configuration
    pub fn with_config(config: ExtractionConfig) -> Self {
        Self { config }
    }

    /// Extract text from a PDF URL
    pub async fn extract_from_url(&self, url: &str) -> AppResult<PaperText> {
        tracing::info!("Extracting text from PDF: {}", url);

        let mut parser_config = ParserConfig::new();

        // Wrap parse call in catch_unwind to handle panics from rsrpp gracefully
        let parse_result = AssertUnwindSafe(parse(url, &mut parser_config, self.config.verbose))
            .catch_unwind()
            .await;

        let pages = match parse_result {
            Ok(Ok(pages)) => pages,
            Ok(Err(e)) => {
                return Err(AppError::PdfExtractionError(format!(
                    "PDF parse failed: {}",
                    e
                )));
            }
            Err(panic_info) => {
                let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic during PDF extraction".to_string()
                };
                tracing::error!("PDF extraction panicked: {}", panic_msg);
                return Err(AppError::PdfExtractionError(format!(
                    "PDF extraction panicked: {}",
                    panic_msg
                )));
            }
        };

        let sections = Section::from_pages(&pages);

        // Build PaperText from sections
        let paper_text = self.build_paper_text(&sections, url);

        // Cleanup temp files
        if self.config.cleanup && parser_config.clean_files().is_err() {
            tracing::warn!("Failed to cleanup temp files");
        }

        tracing::info!(
            "Extracted {} sections, {} chars total",
            paper_text.sections.len(),
            paper_text.plain_text.len()
        );

        Ok(paper_text)
    }

    /// Extract text from a paper, using available PDF URL
    pub async fn extract_for_paper(&self, paper: &AcademicPaper) -> AppResult<PaperText> {
        let pdf_url = self.get_pdf_url(paper)?;
        self.extract_from_url(&pdf_url).await
    }

    /// Get PDF URL from paper (prefers open_access_pdf_url, falls back to arXiv)
    fn get_pdf_url(&self, paper: &AcademicPaper) -> AppResult<String> {
        // Try open access PDF first
        if let Some(ref url) = paper.open_access_pdf_url {
            return Ok(url.clone());
        }

        // Fall back to arXiv PDF if arxiv_id is available
        if !paper.arxiv_id.is_empty() {
            return Ok(format!("https://arxiv.org/pdf/{}", paper.arxiv_id));
        }

        Err(AppError::PdfExtractionError(
            "No PDF URL available for this paper".to_string(),
        ))
    }

    /// Build PaperText from rsrpp sections
    fn build_paper_text(&self, sections: &[Section], source_url: &str) -> PaperText {
        let paper_sections: Vec<PaperSection> = sections
            .iter()
            .map(|s| PaperSection {
                index: s.index,
                title: s.title.clone(),
                content: s.get_text(),
            })
            .collect();

        let plain_text = self.build_plain_text(&paper_sections);
        let markdown = self.build_markdown(&paper_sections);

        PaperText {
            plain_text,
            sections: paper_sections,
            markdown,
            extracted_at: Local::now(),
            source_url: source_url.to_string(),
        }
    }

    /// Build plain text from sections
    fn build_plain_text(&self, sections: &[PaperSection]) -> String {
        sections
            .iter()
            .map(|s| s.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Build markdown from sections
    fn build_markdown(&self, sections: &[PaperSection]) -> String {
        sections
            .iter()
            .map(|s| format!("## {}\n\n{}", s.title, s.content))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

impl Default for PdfExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_config_default() {
        let config = ExtractionConfig::default();
        assert!(!config.verbose);
        assert!(config.cleanup);
    }

    #[test]
    fn test_extraction_config_builder() {
        let config = ExtractionConfig::new()
            .with_verbose(true)
            .with_cleanup(false);
        assert!(config.verbose);
        assert!(!config.cleanup);
    }

    #[test]
    fn test_build_plain_text() {
        let extractor = PdfExtractor::new();
        let sections = vec![
            PaperSection {
                index: 0,
                title: "Abstract".to_string(),
                content: "This is the abstract.".to_string(),
            },
            PaperSection {
                index: 1,
                title: "Introduction".to_string(),
                content: "This is the introduction.".to_string(),
            },
        ];
        let plain = extractor.build_plain_text(&sections);
        assert!(plain.contains("This is the abstract."));
        assert!(plain.contains("This is the introduction."));
        assert!(plain.contains("\n\n"));
    }

    #[test]
    fn test_build_markdown() {
        let extractor = PdfExtractor::new();
        let sections = vec![PaperSection {
            index: 0,
            title: "Abstract".to_string(),
            content: "This is the abstract.".to_string(),
        }];
        let md = extractor.build_markdown(&sections);
        assert!(md.contains("## Abstract"));
        assert!(md.contains("This is the abstract."));
    }

    #[test]
    fn test_get_pdf_url_open_access() {
        let extractor = PdfExtractor::new();
        let mut paper = AcademicPaper::new();
        paper.open_access_pdf_url = Some("https://example.com/paper.pdf".to_string());

        let url = extractor.get_pdf_url(&paper);
        assert!(url.is_ok());
        assert_eq!(url.unwrap(), "https://example.com/paper.pdf");
    }

    #[test]
    fn test_get_pdf_url_arxiv_fallback() {
        let extractor = PdfExtractor::new();
        let mut paper = AcademicPaper::new();
        paper.arxiv_id = "1706.03762".to_string();

        let url = extractor.get_pdf_url(&paper);
        assert!(url.is_ok());
        assert_eq!(url.unwrap(), "https://arxiv.org/pdf/1706.03762");
    }

    #[test]
    fn test_get_pdf_url_no_url() {
        let extractor = PdfExtractor::new();
        let paper = AcademicPaper::new();

        let url = extractor.get_pdf_url(&paper);
        assert!(url.is_err());
    }
}
