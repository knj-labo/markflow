#![deny(missing_docs)]
//! Node.js bindings that surface Markflow's Rust implementation.

use markflow_core::{MarkdownStream, MarkflowError, RewriteOptions, StreamingRewriter};
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Configuration options for the HTML rewriter
#[napi(object)]
#[derive(Debug, Clone)]
pub struct RewriteConfig {
    /// Enable lazy loading for images (default: true)
    pub enforce_img_loading_lazy: bool,
}

impl Default for RewriteConfig {
    fn default() -> Self {
        Self {
            enforce_img_loading_lazy: true,
        }
    }
}

impl From<RewriteConfig> for RewriteOptions {
    fn from(config: RewriteConfig) -> Self {
        RewriteOptions {
            enforce_img_loading_lazy: config.enforce_img_loading_lazy,
        }
    }
}

/// Parse result with HTML output and processing statistics
#[napi(object)]
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// The parsed HTML output
    pub html: String,
    /// Processing time in milliseconds
    pub processing_time_ms: f64,
}

/// Parses markdown string to HTML with default options
#[napi]
pub fn parse(input: String) -> napi::Result<String> {
    markflow_core::parse(&input).map_err(convert_error)
}

/// Parses markdown string to HTML with custom rewrite options
#[napi]
pub fn parse_with_options(input: String, config: RewriteConfig) -> napi::Result<String> {
    let events = markflow_core::get_event_iterator(&input).map_err(convert_error)?;
    let options: RewriteOptions = config.into();
    let rewriter = StreamingRewriter::new(Vec::new(), options);

    let rewriter = events.stream_to_writer(rewriter).map_err(convert_error)?;
    let output = rewriter.into_inner().map_err(convert_error)?;
    String::from_utf8(output).map_err(convert_error)
}

/// Parses markdown and returns both HTML output and processing statistics
#[napi]
pub fn parse_with_stats(input: String) -> napi::Result<ParseResult> {
    use std::time::Instant;

    let start = Instant::now();
    let html = parse(input)?;
    let elapsed = start.elapsed();

    Ok(ParseResult {
        html,
        processing_time_ms: elapsed.as_secs_f64() * 1000.0,
    })
}

/// Improved error converter that matches on enum variants
fn convert_error<E: Into<MarkflowError>>(err: E) -> Error {
    let err = err.into();
    match err {
        // Map specific errors to specific NAPI statuses
        MarkflowError::EncodingError(e) => {
            Error::new(Status::InvalidArg, format!("Encoding error: {}", e))
        }
        // IO errors and Adapter errors usually imply a runtime failure
        MarkflowError::IoError(e) => Error::from_reason(format!("IO error: {}", e)),
        MarkflowError::MarkdownAdapter(msg) => {
            Error::from_reason(format!("Markdown parser error: {}", msg))
        }
    }
}
