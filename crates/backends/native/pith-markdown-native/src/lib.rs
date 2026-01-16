//! Native Markdown implementation using pulldown-cmark.

use rhizome_pith_markdown::{MarkdownDocument, MarkdownOptions, MarkdownParser, MarkdownRenderer};
use pulldown_cmark::{html, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// Markdown renderer using pulldown-cmark.
#[derive(Debug, Default, Clone, Copy)]
pub struct Markdown;

impl Markdown {
    pub fn new() -> Self {
        Self
    }

    fn options_to_pulldown(options: &MarkdownOptions) -> Options {
        let mut opts = Options::empty();

        if options.tables || options.gfm {
            opts.insert(Options::ENABLE_TABLES);
        }
        if options.strikethrough || options.gfm {
            opts.insert(Options::ENABLE_STRIKETHROUGH);
        }
        if options.task_lists || options.gfm {
            opts.insert(Options::ENABLE_TASKLISTS);
        }
        if options.smart_punctuation {
            opts.insert(Options::ENABLE_SMART_PUNCTUATION);
        }
        if options.heading_ids || options.gfm {
            opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        }
        if options.footnotes {
            opts.insert(Options::ENABLE_FOOTNOTES);
        }

        opts
    }
}

impl MarkdownRenderer for Markdown {
    fn render(&self, markdown: &str) -> String {
        self.render_with_options(markdown, &MarkdownOptions::default())
    }

    fn render_with_options(&self, markdown: &str, options: &MarkdownOptions) -> String {
        let opts = Self::options_to_pulldown(options);
        let parser = Parser::new_ext(markdown, opts);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }
}

impl MarkdownParser for Markdown {
    type Document = Document;

    fn parse(&self, markdown: &str) -> Self::Document {
        self.parse_with_options(markdown, &MarkdownOptions::default())
    }

    fn parse_with_options(&self, markdown: &str, options: &MarkdownOptions) -> Self::Document {
        Document {
            source: markdown.to_string(),
            options: options.clone(),
        }
    }
}

/// A parsed Markdown document.
#[derive(Debug, Clone)]
pub struct Document {
    source: String,
    options: MarkdownOptions,
}

impl MarkdownDocument for Document {
    fn source(&self) -> &str {
        &self.source
    }

    fn to_html(&self) -> String {
        let opts = Markdown::options_to_pulldown(&self.options);
        let parser = Parser::new_ext(&self.source, opts);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }

    fn to_text(&self) -> String {
        let opts = Markdown::options_to_pulldown(&self.options);
        let parser = Parser::new_ext(&self.source, opts);
        let mut text = String::new();

        for event in parser {
            if let Event::Text(t) | Event::Code(t) = event {
                text.push_str(&t);
            }
        }

        text
    }

    fn headings(&self) -> Vec<(u8, String)> {
        let opts = Markdown::options_to_pulldown(&self.options);
        let parser = Parser::new_ext(&self.source, opts);
        let mut headings = Vec::new();
        let mut current_level: Option<u8> = None;
        let mut current_text = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    current_level = Some(heading_level_to_u8(level));
                    current_text.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some(level) = current_level.take() {
                        headings.push((level, std::mem::take(&mut current_text)));
                    }
                }
                Event::Text(t) | Event::Code(t) if current_level.is_some() => {
                    current_text.push_str(&t);
                }
                _ => {}
            }
        }

        headings
    }

    fn links(&self) -> Vec<(String, String)> {
        let opts = Markdown::options_to_pulldown(&self.options);
        let parser = Parser::new_ext(&self.source, opts);
        let mut links = Vec::new();
        let mut current_url: Option<String> = None;
        let mut current_text = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Link { dest_url, .. }) => {
                    current_url = Some(dest_url.to_string());
                    current_text.clear();
                }
                Event::End(TagEnd::Link) => {
                    if let Some(url) = current_url.take() {
                        links.push((std::mem::take(&mut current_text), url));
                    }
                }
                Event::Text(t) | Event::Code(t) if current_url.is_some() => {
                    current_text.push_str(&t);
                }
                _ => {}
            }
        }

        links
    }

    fn code_blocks(&self) -> Vec<(Option<String>, String)> {
        let opts = Markdown::options_to_pulldown(&self.options);
        let parser = Parser::new_ext(&self.source, opts);
        let mut blocks = Vec::new();
        let mut current_lang: Option<Option<String>> = None;
        let mut current_code = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                            let lang = lang.to_string();
                            if lang.is_empty() {
                                None
                            } else {
                                Some(lang)
                            }
                        }
                        pulldown_cmark::CodeBlockKind::Indented => None,
                    };
                    current_lang = Some(lang);
                    current_code.clear();
                }
                Event::End(TagEnd::CodeBlock) => {
                    if let Some(lang) = current_lang.take() {
                        blocks.push((lang, std::mem::take(&mut current_code)));
                    }
                }
                Event::Text(t) if current_lang.is_some() => {
                    current_code.push_str(&t);
                }
                _ => {}
            }
        }

        blocks
    }
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_basic() {
        let md = Markdown::new();
        let html = md.render("# Hello\n\nWorld");
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<p>World</p>"));
    }

    #[test]
    fn render_emphasis() {
        let md = Markdown::new();
        let html = md.render("*italic* and **bold**");
        assert!(html.contains("<em>italic</em>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn render_code() {
        let md = Markdown::new();
        let html = md.render("Inline `code` here");
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn render_code_block() {
        let md = Markdown::new();
        let html = md.render("```rust\nfn main() {}\n```");
        assert!(html.contains("<pre>"));
        assert!(html.contains("<code"));
        assert!(html.contains("fn main()"));
    }

    #[test]
    fn render_link() {
        let md = Markdown::new();
        let html = md.render("[link](https://example.com)");
        assert!(html.contains("<a href=\"https://example.com\">link</a>"));
    }

    #[test]
    fn render_list() {
        let md = Markdown::new();
        let html = md.render("- one\n- two\n- three");
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>one</li>"));
    }

    #[test]
    fn render_table_with_gfm() {
        let md = Markdown::new();
        let options = MarkdownOptions::gfm();
        let html = md.render_with_options("| A | B |\n|---|---|\n| 1 | 2 |", &options);
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>A</th>"));
    }

    #[test]
    fn render_strikethrough_with_gfm() {
        let md = Markdown::new();
        let options = MarkdownOptions::gfm();
        let html = md.render_with_options("~~deleted~~", &options);
        assert!(html.contains("<del>deleted</del>"));
    }

    #[test]
    fn document_to_text() {
        let md = Markdown::new();
        let doc = md.parse("# Hello\n\n**World** and `code`");
        assert_eq!(doc.to_text(), "HelloWorld and code");
    }

    #[test]
    fn document_headings() {
        let md = Markdown::new();
        let doc = md.parse("# H1\n## H2\n### H3\ntext\n## Another H2");
        let headings = doc.headings();
        assert_eq!(headings.len(), 4);
        assert_eq!(headings[0], (1, "H1".to_string()));
        assert_eq!(headings[1], (2, "H2".to_string()));
        assert_eq!(headings[2], (3, "H3".to_string()));
        assert_eq!(headings[3], (2, "Another H2".to_string()));
    }

    #[test]
    fn document_links() {
        let md = Markdown::new();
        let doc = md.parse("[Google](https://google.com) and [Rust](https://rust-lang.org)");
        let links = doc.links();
        assert_eq!(links.len(), 2);
        assert_eq!(links[0], ("Google".to_string(), "https://google.com".to_string()));
        assert_eq!(links[1], ("Rust".to_string(), "https://rust-lang.org".to_string()));
    }

    #[test]
    fn document_code_blocks() {
        let md = Markdown::new();
        let doc = md.parse("```rust\nfn main() {}\n```\n\n```\nplain\n```");
        let blocks = doc.code_blocks();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].0, Some("rust".to_string()));
        assert!(blocks[0].1.contains("fn main()"));
        assert_eq!(blocks[1].0, None);
        assert!(blocks[1].1.contains("plain"));
    }

    #[test]
    fn options_presets() {
        let standard = MarkdownOptions::standard();
        assert!(!standard.gfm);
        assert!(!standard.tables);

        let gfm = MarkdownOptions::gfm();
        assert!(gfm.gfm);
        assert!(gfm.tables);
        assert!(gfm.strikethrough);

        let full = MarkdownOptions::full();
        assert!(full.smart_punctuation);
        assert!(full.footnotes);
    }
}
