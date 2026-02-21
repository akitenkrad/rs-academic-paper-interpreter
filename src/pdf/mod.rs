//! PDF text extraction module
//!
//! Provides functionality to extract text from academic paper PDFs
//! using the rsrpp crate.

mod extractor;
mod resolver;

pub use extractor::{ExtractionConfig, PdfExtractor};
pub use resolver::PdfUrlResolver;
