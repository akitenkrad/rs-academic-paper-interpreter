//! Data models for academic papers and related entities

use crate::shared::errors::AppResult;
use crate::shared::utils::datetime_from_str;
use arxiv_tools::Paper as ArxivPaper;
use chrono::{DateTime, Local};
use derive_new::new;
use serde::{Deserialize, Serialize};
use ss_tools::structs::Paper as SsPaper;

/// Author information
#[derive(Debug, Clone, Default, new, Serialize, Deserialize)]
pub struct Author {
    /// Semantic Scholar author ID
    #[new(default)]
    pub ss_id: String,

    /// Author name
    pub name: String,

    /// h-index (from Semantic Scholar)
    #[new(default)]
    pub h_index: i32,

    /// Author affiliations
    #[new(default)]
    pub affiliations: Vec<String>,

    /// Total paper count
    #[new(default)]
    pub paper_count: i32,

    /// Total citation count
    #[new(default)]
    pub citation_count: i32,
}

impl Author {
    /// Create author from Semantic Scholar author data
    pub fn from_ss_author(author: &ss_tools::structs::Author) -> Self {
        Self {
            ss_id: author.author_id.clone().unwrap_or_default(),
            name: author.name.clone().unwrap_or_default(),
            h_index: author.hindex.unwrap_or(0) as i32,
            affiliations: author.affiliations.clone().unwrap_or_default(),
            paper_count: author.paper_count.unwrap_or(0) as i32,
            citation_count: author.citation_count.unwrap_or(0) as i32,
        }
    }

    /// Create author from arXiv author name
    pub fn from_arxiv_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
}

/// LLM-generated analysis of a paper
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaperAnalysis {
    /// Concise summary of the paper (2-3 paragraphs)
    pub summary: String,

    /// Japanese translation of the summary
    pub summary_ja: Option<String>,

    /// Research background and purpose
    pub background_and_purpose: String,

    /// Methodology and technical approach
    pub methodology: String,

    /// Datasets used in the research
    pub dataset: String,

    /// Key results and findings
    pub results: String,

    /// Advantages, limitations, and future directions
    pub advantages_limitations_and_future_work: String,

    /// Key contributions as bullet points
    pub key_contributions: Vec<String>,

    /// Research task categories (e.g., "NLP", "Computer Vision")
    pub tasks: Vec<String>,

    /// When the analysis was performed
    pub analyzed_at: DateTime<Local>,

    /// LLM provider used for analysis
    pub provider: String,

    /// Model used for analysis
    pub model: String,
}

impl PaperAnalysis {
    /// Check if analysis has meaningful content
    pub fn is_complete(&self) -> bool {
        !self.summary.is_empty() && !self.methodology.is_empty()
    }
}

/// Section of an academic paper with extracted text
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaperSection {
    /// Section index (order in paper)
    pub index: i8,

    /// Section title (e.g., "Abstract", "Introduction")
    pub title: String,

    /// Section content
    pub content: String,
}

/// Extracted text from a paper PDF in multiple formats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaperText {
    /// Plain text (all sections concatenated)
    pub plain_text: String,

    /// Structured sections preserving document structure
    pub sections: Vec<PaperSection>,

    /// Markdown formatted text with headers
    pub markdown: String,

    /// When extraction was performed
    pub extracted_at: DateTime<Local>,

    /// Source PDF URL used for extraction
    pub source_url: String,
}

impl PaperText {
    /// Check if extraction has meaningful content
    pub fn is_valid(&self) -> bool {
        !self.plain_text.is_empty() && !self.sections.is_empty()
    }

    /// Get section by title (case-insensitive)
    pub fn get_section(&self, title: &str) -> Option<&PaperSection> {
        self.sections
            .iter()
            .find(|s| s.title.to_lowercase() == title.to_lowercase())
    }

    /// Get abstract section if available
    pub fn get_abstract(&self) -> Option<&PaperSection> {
        self.get_section("Abstract")
    }

    /// Get introduction section if available
    pub fn get_introduction(&self) -> Option<&PaperSection> {
        self.get_section("Introduction")
    }

    /// Convert sections to JSON string
    pub fn sections_to_json(&self) -> AppResult<String> {
        serde_json::to_string(&self.sections).map_err(|e| {
            crate::shared::errors::AppError::InternalAppError(format!(
                "JSON serialization failed: {}",
                e
            ))
        })
    }

    /// Convert sections to pretty-printed JSON string
    pub fn sections_to_json_pretty(&self) -> AppResult<String> {
        serde_json::to_string_pretty(&self.sections).map_err(|e| {
            crate::shared::errors::AppError::InternalAppError(format!(
                "JSON serialization failed: {}",
                e
            ))
        })
    }
}

/// Unified academic paper representation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AcademicPaper {
    // Source data (internal)
    #[serde(skip)]
    arxiv_paper: Option<ArxivPaper>,
    #[serde(skip)]
    ss_paper: Option<SsPaper>,

    // Identifiers
    /// Semantic Scholar paper ID
    pub ss_id: String,

    /// arXiv paper ID (e.g., "2106.09685")
    pub arxiv_id: String,

    /// Digital Object Identifier
    pub doi: String,

    // Metadata
    /// Paper title
    pub title: String,

    /// List of authors
    pub authors: Vec<Author>,

    /// Paper abstract
    pub abstract_text: String,

    /// Japanese translation of abstract
    pub abstract_text_ja: String,

    /// Full paper text (if available)
    pub text: String,

    /// Paper URL
    pub url: String,

    /// Journal or venue name
    pub journal: String,

    /// Primary arXiv category (e.g., "cs.CL")
    pub primary_category: String,

    /// All arXiv categories
    pub categories: Vec<String>,

    /// Publication date
    pub published_date: DateTime<Local>,

    /// BibTeX citation
    pub bibtex: String,

    // Metrics
    /// Number of citations
    pub citations_count: i32,

    /// Number of references
    pub references_count: i32,

    /// Influential citation count (Semantic Scholar metric)
    pub influential_citation_count: i32,

    /// Whether open access PDF is available
    pub is_open_access: bool,

    /// Open access PDF URL
    pub open_access_pdf_url: Option<String>,

    // LLM Analysis
    /// LLM-generated analysis (populated by agents)
    pub analysis: Option<PaperAnalysis>,

    // Extracted Text
    /// Extracted full text from PDF (populated by pdf extractor)
    pub extracted_text: Option<PaperText>,

    // Timestamps
    /// When this record was created
    pub created_at: DateTime<Local>,

    /// When this record was last updated
    pub updated_at: DateTime<Local>,
}

impl AcademicPaper {
    /// Create a new empty paper
    pub fn new() -> Self {
        let now = Local::now();
        Self {
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    /// Create paper from arXiv data
    pub fn from_arxiv(paper: ArxivPaper) -> Self {
        let now = Local::now();
        let published_date = datetime_from_str(&paper.published);

        let authors = paper
            .authors
            .iter()
            .map(|name| Author::from_arxiv_name(name))
            .collect();

        // Extract clean arXiv ID from URL (e.g., "http://arxiv.org/abs/1706.03762v7" -> "1706.03762")
        let arxiv_id = Self::extract_arxiv_id(&paper.id);

        Self {
            arxiv_paper: Some(paper.clone()),
            arxiv_id: arxiv_id.clone(),
            title: paper.title.clone(),
            abstract_text: paper.abstract_text.clone(),
            authors,
            url: format!("https://arxiv.org/abs/{}", arxiv_id),
            primary_category: paper.primary_category.clone(),
            categories: paper.categories.clone(),
            journal: if paper.journal_ref.is_empty() {
                "arXiv".to_string()
            } else {
                paper.journal_ref.clone()
            },
            doi: paper.doi.clone(),
            published_date,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    /// Create paper from Semantic Scholar data
    pub fn from_semantic_scholar(paper: SsPaper) -> Self {
        let now = Local::now();
        let published_date = paper
            .publication_date
            .as_ref()
            .map(|d| datetime_from_str(d))
            .unwrap_or(now);

        let authors = paper
            .authors
            .as_ref()
            .map(|authors| authors.iter().map(Author::from_ss_author).collect())
            .unwrap_or_default();

        let (is_open_access, open_access_pdf_url) = paper
            .open_access_pdf
            .as_ref()
            .map(|pdf| (true, pdf.url.clone()))
            .unwrap_or((paper.is_open_access.unwrap_or(false), None));

        let bibtex = paper
            .citation_styles
            .as_ref()
            .and_then(|cs| cs.bibtex.clone())
            .unwrap_or_default();

        let journal = paper
            .journal
            .as_ref()
            .and_then(|j| j.name.clone())
            .or_else(|| paper.venue.clone())
            .unwrap_or_default();

        Self {
            ss_paper: Some(paper.clone()),
            ss_id: paper.paper_id.clone().unwrap_or_default(),
            title: paper.title.clone().unwrap_or_default(),
            abstract_text: paper.abstract_text.clone().unwrap_or_default(),
            authors,
            url: paper.url.clone().unwrap_or_default(),
            journal,
            citations_count: paper.citation_count.unwrap_or(0) as i32,
            references_count: paper.reference_count.unwrap_or(0) as i32,
            influential_citation_count: paper.influential_citation_count.unwrap_or(0) as i32,
            is_open_access,
            open_access_pdf_url,
            bibtex,
            published_date,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    /// Enrich paper data from Semantic Scholar
    pub fn enrich_from_semantic_scholar(&mut self, paper: SsPaper) {
        self.ss_paper = Some(paper.clone());
        self.ss_id = paper.paper_id.clone().unwrap_or_default();
        self.citations_count = paper.citation_count.unwrap_or(0) as i32;
        self.references_count = paper.reference_count.unwrap_or(0) as i32;
        self.influential_citation_count = paper.influential_citation_count.unwrap_or(0) as i32;

        // Update authors with h-index if available
        if let Some(ss_authors) = &paper.authors {
            for ss_author in ss_authors {
                if let Some(existing) = self
                    .authors
                    .iter_mut()
                    .find(|a| a.name == ss_author.name.clone().unwrap_or_default())
                {
                    existing.ss_id = ss_author.author_id.clone().unwrap_or_default();
                    existing.h_index = ss_author.hindex.unwrap_or(0) as i32;
                    existing.paper_count = ss_author.paper_count.unwrap_or(0) as i32;
                    existing.citation_count = ss_author.citation_count.unwrap_or(0) as i32;
                }
            }
        }

        // Update bibtex if not already set
        if self.bibtex.is_empty() {
            if let Some(cs) = &paper.citation_styles {
                if let Some(bibtex) = &cs.bibtex {
                    self.bibtex = bibtex.clone();
                }
            }
        }

        // Update open access info
        if let Some(pdf) = &paper.open_access_pdf {
            self.is_open_access = true;
            self.open_access_pdf_url = pdf.url.clone();
        }

        self.updated_at = Local::now();
    }

    /// Get arXiv ID (returns error if not available)
    pub fn arxiv_id(&self) -> AppResult<String> {
        if !self.arxiv_id.is_empty() {
            Ok(self.arxiv_id.clone())
        } else if let Some(arxiv_paper) = &self.arxiv_paper {
            Ok(arxiv_paper.id.clone())
        } else {
            Err("arXiv paper data is not available.".into())
        }
    }

    /// Get Semantic Scholar ID (returns error if not available)
    pub fn ss_id(&self) -> AppResult<String> {
        if !self.ss_id.is_empty() {
            Ok(self.ss_id.clone())
        } else if let Some(ss_paper) = &self.ss_paper {
            ss_paper
                .paper_id
                .clone()
                .ok_or_else(|| "Semantic Scholar paper ID is not available.".into())
        } else {
            Err("Semantic Scholar paper data is not available.".into())
        }
    }

    /// Get journal name (returns error if not available)
    pub fn journal(&self) -> AppResult<String> {
        if !self.journal.is_empty() {
            return Ok(self.journal.clone());
        }

        let mut journal_name = String::new();

        if self.arxiv_paper.is_some() {
            journal_name = "arXiv".to_string();
        }

        if let Some(ss_paper) = &self.ss_paper {
            if let Some(journal) = &ss_paper.journal {
                if let Some(name) = &journal.name {
                    if !name.is_empty() {
                        journal_name = name.clone();
                    }
                }
            }
        }

        if journal_name.is_empty() {
            Err("Journal information is not available.".into())
        } else {
            Ok(journal_name)
        }
    }

    /// Check if paper has been analyzed by LLM
    pub fn is_analyzed(&self) -> bool {
        self.analysis
            .as_ref()
            .map(|a| a.is_complete())
            .unwrap_or(false)
    }

    /// Set analysis results
    pub fn set_analysis(&mut self, analysis: PaperAnalysis) {
        self.analysis = Some(analysis);
        self.updated_at = Local::now();
    }

    /// Generate a simple text citation
    pub fn to_citation(&self) -> String {
        let authors_str = if self.authors.len() > 3 {
            format!("{} et al.", self.authors[0].name)
        } else {
            self.authors
                .iter()
                .map(|a| a.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        };

        let year = self.published_date.format("%Y");
        format!("{} ({}). {}", authors_str, year, self.title)
    }

    /// Check if paper has extracted text
    pub fn has_extracted_text(&self) -> bool {
        self.extracted_text
            .as_ref()
            .map(|t| t.is_valid())
            .unwrap_or(false)
    }

    /// Get full text (extracted or empty)
    pub fn get_full_text(&self) -> &str {
        self.extracted_text
            .as_ref()
            .map(|t| t.plain_text.as_str())
            .unwrap_or("")
    }

    /// Get PDF URL for extraction (prefers open_access, falls back to arXiv)
    pub fn pdf_url(&self) -> Option<String> {
        if let Some(ref url) = self.open_access_pdf_url {
            return Some(url.clone());
        }
        if !self.arxiv_id.is_empty() {
            return Some(format!("https://arxiv.org/pdf/{}", self.arxiv_id));
        }
        None
    }

    /// Set extracted text
    pub fn set_extracted_text(&mut self, text: PaperText) {
        self.extracted_text = Some(text);
        self.updated_at = Local::now();
    }

    /// Extract clean arXiv ID from various formats
    ///
    /// Handles:
    /// - Full URL: "http://arxiv.org/abs/1706.03762v7" -> "1706.03762"
    /// - With version: "1706.03762v7" -> "1706.03762"
    /// - Clean ID: "1706.03762" -> "1706.03762"
    fn extract_arxiv_id(raw_id: &str) -> String {
        let id = raw_id
            // Remove URL prefix
            .trim_start_matches("http://arxiv.org/abs/")
            .trim_start_matches("https://arxiv.org/abs/")
            // Remove trailing whitespace
            .trim();

        // Remove version suffix (e.g., "v7")
        if let Some(pos) = id.rfind('v') {
            if id[pos + 1..].chars().all(|c| c.is_ascii_digit()) {
                return id[..pos].to_string();
            }
        }

        id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_analysis_is_complete() {
        let mut analysis = PaperAnalysis::default();
        assert!(!analysis.is_complete());

        analysis.summary = "Test summary".to_string();
        assert!(!analysis.is_complete());

        analysis.methodology = "Test methodology".to_string();
        assert!(analysis.is_complete());
    }

    #[test]
    fn test_academic_paper_is_analyzed() {
        let mut paper = AcademicPaper::new();
        assert!(!paper.is_analyzed());

        paper.analysis = Some(PaperAnalysis {
            summary: "Test".to_string(),
            methodology: "Test".to_string(),
            ..Default::default()
        });
        assert!(paper.is_analyzed());
    }

    #[test]
    fn test_to_citation() {
        let paper = AcademicPaper {
            title: "Attention Is All You Need".to_string(),
            authors: vec![
                Author::from_arxiv_name("Vaswani"),
                Author::from_arxiv_name("Shazeer"),
            ],
            published_date: Local::now(),
            ..Default::default()
        };

        let citation = paper.to_citation();
        assert!(citation.contains("Vaswani"));
        assert!(citation.contains("Attention Is All You Need"));
    }

    #[test]
    fn test_extract_arxiv_id() {
        // Full URL with version
        assert_eq!(
            AcademicPaper::extract_arxiv_id("http://arxiv.org/abs/1706.03762v7"),
            "1706.03762"
        );

        // HTTPS URL with version
        assert_eq!(
            AcademicPaper::extract_arxiv_id("https://arxiv.org/abs/2301.00001v2"),
            "2301.00001"
        );

        // ID with version (no URL)
        assert_eq!(
            AcademicPaper::extract_arxiv_id("1706.03762v7"),
            "1706.03762"
        );

        // Clean ID (no version)
        assert_eq!(
            AcademicPaper::extract_arxiv_id("1706.03762"),
            "1706.03762"
        );

        // Old-style arXiv ID
        assert_eq!(
            AcademicPaper::extract_arxiv_id("cs.CL/0001001v1"),
            "cs.CL/0001001"
        );
    }
}
