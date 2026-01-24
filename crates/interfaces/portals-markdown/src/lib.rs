//! Markdown interfaces.
//!
//! Parse and render Markdown text.

use std::fmt;

/// Markdown renderer options.
#[derive(Debug, Clone, Default)]
pub struct MarkdownOptions {
    /// Enable GitHub Flavored Markdown extensions.
    pub gfm: bool,
    /// Enable strikethrough (`~~text~~`).
    pub strikethrough: bool,
    /// Enable tables.
    pub tables: bool,
    /// Enable task lists (`- [x] item`).
    pub task_lists: bool,
    /// Enable autolinks (URLs without `<>`).
    pub autolinks: bool,
    /// Enable smart punctuation (quotes, dashes).
    pub smart_punctuation: bool,
    /// Enable heading IDs.
    pub heading_ids: bool,
    /// Enable footnotes.
    pub footnotes: bool,
}

impl MarkdownOptions {
    /// Create options with all GFM extensions enabled.
    pub fn gfm() -> Self {
        Self {
            gfm: true,
            strikethrough: true,
            tables: true,
            task_lists: true,
            autolinks: true,
            smart_punctuation: false,
            heading_ids: true,
            footnotes: false,
        }
    }

    /// Create minimal options (standard CommonMark only).
    pub fn standard() -> Self {
        Self::default()
    }

    /// Create options with all extensions enabled.
    pub fn full() -> Self {
        Self {
            gfm: true,
            strikethrough: true,
            tables: true,
            task_lists: true,
            autolinks: true,
            smart_punctuation: true,
            heading_ids: true,
            footnotes: true,
        }
    }
}

/// Render Markdown to HTML.
pub trait MarkdownRenderer {
    /// Render Markdown text to HTML.
    fn render(&self, markdown: &str) -> String;

    /// Render Markdown text to HTML with custom options.
    fn render_with_options(&self, markdown: &str, options: &MarkdownOptions) -> String;
}

/// A Markdown document (parsed AST).
pub trait MarkdownDocument {
    /// Get the original source text.
    fn source(&self) -> &str;

    /// Render to HTML.
    fn to_html(&self) -> String;

    /// Extract plain text content (no formatting).
    fn to_text(&self) -> String;

    /// Get all headings with their levels and text.
    fn headings(&self) -> Vec<(u8, String)>;

    /// Get all links (text, url).
    fn links(&self) -> Vec<(String, String)>;

    /// Get all code blocks (language, code).
    fn code_blocks(&self) -> Vec<(Option<String>, String)>;
}

/// Parse Markdown into a document.
pub trait MarkdownParser {
    /// The document type returned by parsing.
    type Document: MarkdownDocument;

    /// Parse Markdown text.
    fn parse(&self, markdown: &str) -> Self::Document;

    /// Parse Markdown text with custom options.
    fn parse_with_options(&self, markdown: &str, options: &MarkdownOptions) -> Self::Document;
}

/// Error type for Markdown operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownError {
    /// Invalid input.
    InvalidInput(String),
    /// Other error.
    Other(String),
}

impl fmt::Display for MarkdownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(msg) => write!(f, "invalid input: {}", msg),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for MarkdownError {}
