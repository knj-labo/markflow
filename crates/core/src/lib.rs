#![deny(missing_docs)]
//! Core Markflow streaming primitives.

/// Returns the semantic version of the core crate.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
