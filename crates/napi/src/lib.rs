#![deny(missing_docs)]
//! Node.js bindings that surface Markflow's Rust implementation.

use napi_derive::napi;

/// Returns the version string reported by the core crate.
#[napi]
pub fn version() -> String {
    markflow_core::version().to_string()
}
