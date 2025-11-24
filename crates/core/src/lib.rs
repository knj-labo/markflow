use markdown::to_html;

/// Parses Markdown into an HTML string using the `markdown` crate.
pub fn parse(input: &str) -> String {
    to_html(input)
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
