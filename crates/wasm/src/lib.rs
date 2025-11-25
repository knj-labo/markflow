use wasm_bindgen::prelude::*;

/// Parses markdown string to HTML.
/// Returns a Result explicitly to handle errors in JS as exceptions.
#[wasm_bindgen]
pub fn parse(input: &str) -> Result<String, JsError> {
    markflow_core::parse(input).map_err(|e| JsError::new(&e.to_string()))
}
