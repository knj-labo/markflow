pub mod adapter;
pub use adapter::PipeAdapter;
use pulldown_cmark::{Options, Parser, html};

/// Helper to get an Event Iterator from a string slice.
/// This is what feeds into the PipeAdapter.
pub fn get_event_iterator(input: &str) -> Parser<'_> {
    Parser::new_ext(input, Options::empty())
}

/// Parses Markdown into an HTML string using pulldown-cmark.
pub fn parse(input: &str) -> String {
    let mut output = String::new();
    html::push_html(&mut output, get_event_iterator(input));
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = "# Hello, World!";
        let expected = "<h1>Hello, World!</h1>";

        assert_eq!(parse(input).trim(), expected);
    }
    #[test]
    fn test_parse_list() {
        let input = "* Item 1\n* Item 2";
        let output = parse(input);

        assert!(output.contains("<ul>"));
        assert!(output.contains("<li>Item 1</li>"));
    }
}
