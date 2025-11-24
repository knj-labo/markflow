#![deny(missing_docs)]
//! Node.js bindings that surface Markflow's Rust implementation.

use napi_derive::napi;

/// Parses markdown string to HTML
#[napi]
pub fn parse(input: String) -> String {
    markflow_core::parse(&input)
}
