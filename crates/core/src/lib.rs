#![deny(missing_docs)]
//! Streaming Markdown core utilities: parser, MarkdownStream, and HTML rewriter glue.

/// Markdown event to `io::Write` bridge utilities.
pub mod adapter;
/// Core event types that decouple Markflow from pulldown-cmark specifics.
#[allow(missing_docs)]
pub mod event;
pub mod streaming_rewriter;

mod html_renderer;

pub use adapter::MarkdownStream;
pub use streaming_rewriter::{RewriteOptions, StreamingRewriter};

use thiserror::Error;

mod markdown_adapter;

/// Errors that can occur during Markdown processing.
#[derive(Debug, Error)]
pub enum MarkflowError {
    /// IO error during streaming.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// UTF-8 encoding error.
    #[error("Encoding error: {0}")]
    EncodingError(#[from] std::string::FromUtf8Error),
    /// markdown-rs parser error surfaced through the adapter.
    #[error("markdown-rs error: {0}")]
    MarkdownAdapter(String),
}

/// Returns an iterator over Markdown events backed by `markdown-rs`.
pub fn get_event_iterator(input: &str) -> Result<markdown_adapter::MarkdownParser, MarkflowError> {
    markdown_adapter::MarkdownParser::new(input)
        .map_err(|err| MarkflowError::MarkdownAdapter(err.to_string()))
}

/// parses Markdown and rewrites the resulting HTML stream with the default rewrite options.
pub fn parse(input: &str) -> Result<String, MarkflowError> {
    let events = get_event_iterator(input)?;
    let rewriter = StreamingRewriter::new(Vec::new(), RewriteOptions::default());

    let rewriter = events.stream_to_writer(rewriter)?;

    let output = rewriter.into_inner()?;
    let string = String::from_utf8(output)?;
    Ok(string)
}

/// Iterator alias so callers don't need to depend on the adapter module path.
pub type MarkdownEventStream = markdown_adapter::MarkdownParser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = "# Hello, World!";
        let output = parse(input).unwrap();
        assert!(output.contains("<h1 id=\"hello-world\">Hello, World!</h1>"));
    }

    #[test]
    fn test_parse_list() {
        let input = "* Item 1\n* Item 2";
        let output = parse(input).unwrap();
        assert!(output.contains("<ul>"));
        assert!(output.contains("<li>Item 1</li>"));
    }

    #[test]
    fn test_parse_applies_lazy_loading() {
        let input = "![alt](img.png)";
        let output = parse(input).unwrap();
        assert!(output.contains("loading=\"lazy\""));
    }

    #[test]
    fn test_parse_table_alignment_and_math() {
        let input = "| A | B |\n|:-|:-:|\n| $x$ | $$y$$ |";
        let output = parse(input).unwrap();
        assert!(output.contains("<table>"));
        assert!(
            output.contains(
                "<td style=\"text-align:left\"><span class=\"math-inline\">x</span></td>"
            )
        );
        assert!(output.contains("<span class=\"math-inline\">y</span>"));
    }

    #[test]
    fn test_parse_frontmatter_passthrough() {
        let input = "---\ntitle: test\n---\n\ncontent";
        let output = parse(input).unwrap();
        assert!(output.contains("frontmatter"));
        assert!(output.contains("title: test"));
    }
}
