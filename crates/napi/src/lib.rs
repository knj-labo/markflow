#![deny(missing_docs)]
//! Node.js bindings that surface Markflow's Rust implementation.

use napi_derive::napi;

/// Parses markdown string to HTML
#[napi]
pub fn parse(input: String) -> napi::Result<String> {
    markflow_core::parse(&input).map_err(|e| napi::Error::from_reason(e.to_string()))
}
