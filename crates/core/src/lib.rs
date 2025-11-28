#![deny(missing_docs)]
//! Streaming Markdown core utilities: parser, MarkdownStream, and HTML rewriter glue.

/// Markdown event to `io::Write` bridge utilities.
pub mod adapter;
pub mod streaming_rewriter;

pub use adapter::MarkdownStream;
pub use streaming_rewriter::{RewriteOptions, StreamingRewriter};

use pulldown_cmark::Event;
#[cfg(not(feature = "markdown-rs"))]
use pulldown_cmark::{Options, Parser};
use thiserror::Error;

#[cfg(feature = "markdown-rs")]
use std::marker::PhantomData;

#[cfg(feature = "markdown-rs")]
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

/// to get an Event Iterator from a string slice.
pub fn get_event_iterator(input: &str) -> Result<MarkdownEventStream<'_>, MarkflowError> {
    MarkdownEventStream::new(input)
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

/// Unified event stream returned by [`get_event_iterator`].
pub enum MarkdownEventStream<'input> {
    /// Event iterator powered by `pulldown-cmark` (default feature set).
    #[cfg(not(feature = "markdown-rs"))]
    Pulldown(PulldownEventIterator<'input>),
    /// Event iterator backed by `markdown-rs` when the feature flag is enabled.
    #[cfg(feature = "markdown-rs")]
    MarkdownRs {
        /// markdown-rs backed iterator.
        iter: markdown_adapter::MarkdownRsEventIter,
        /// Marker tying the enum to the caller's lifetime for API parity.
        _marker: PhantomData<&'input ()>,
    },
}

impl<'input> MarkdownEventStream<'input> {
    fn new(input: &'input str) -> Result<Self, MarkflowError> {
        #[cfg(feature = "markdown-rs")]
        {
            let iterator = markdown_adapter::MarkdownRsEventIter::new(input)
                .map_err(|err| MarkflowError::MarkdownAdapter(err.to_string()))?;
            return Ok(Self::MarkdownRs {
                iter: iterator,
                _marker: PhantomData,
            });
        }

        #[cfg(not(feature = "markdown-rs"))]
        {
            Ok(Self::Pulldown(PulldownEventIterator::new(input)))
        }
    }
}

impl<'input> Iterator for MarkdownEventStream<'input> {
    type Item = Event<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            #[cfg(not(feature = "markdown-rs"))]
            MarkdownEventStream::Pulldown(iter) => iter.next(),
            #[cfg(feature = "markdown-rs")]
            MarkdownEventStream::MarkdownRs { iter, .. } => iter.next(),
        }
    }
}

/// Wrapper over the legacy `pulldown-cmark` parser, exposed so callers can
/// inspect the iterator type behind the default feature set.
#[cfg(not(feature = "markdown-rs"))]
pub struct PulldownEventIterator<'input> {
    inner: Parser<'input>,
}

#[cfg(not(feature = "markdown-rs"))]
impl<'input> PulldownEventIterator<'input> {
    fn new(input: &'input str) -> Self {
        Self {
            inner: Parser::new_ext(input, Options::empty()),
        }
    }
}

#[cfg(not(feature = "markdown-rs"))]
impl<'input> Iterator for PulldownEventIterator<'input> {
    type Item = Event<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Event::into_static)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = "# Hello, World!";
        let expected = "<h1>Hello, World!</h1>";
        assert_eq!(parse(input).unwrap().trim(), expected);
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
}
