use pulldown_cmark::{Event, html};
use std::io::{self, Write};

/// Extension trait to pipe Markdown events directly to a Writer.
///
/// This replaces the struct-based `PipeAdapter` with a zero-cost abstraction,
/// allowing for a more fluent method chain style.
pub trait MarkdownStream: Sized {
    /// Drives the iterator events into the writer, converting Markdown to HTML on the fly.
    ///
    /// # Arguments
    /// * `writer` - The destination writer (e.g. `StreamingRewriter`).
    ///
    /// Returns the writer back to the caller upon success.
    fn stream_to_writer<W: Write>(self, writer: W) -> io::Result<W>;
}

impl<'a, I> MarkdownStream for I
where
    I: Iterator<Item = Event<'a>>,
{
    fn stream_to_writer<W: Write>(self, mut writer: W) -> io::Result<W> {
        html::write_html_io(&mut writer, self)?;
        writer.flush()?;

        Ok(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{Options, Parser};

    #[test]
    fn test_streaming_output() {
        let markdown_input = "# Hello Stream\n\n* Item 1\n* Item 2";
        let mut output_buffer = Vec::new(); // Implements Write

        let parser = Parser::new_ext(markdown_input, Options::empty());

        parser
            .stream_to_writer(&mut output_buffer)
            .expect("Failed to drive stream");

        let output_str = String::from_utf8(output_buffer).unwrap();

        assert!(output_str.contains("<h1>Hello Stream</h1>"));
        assert!(output_str.contains("<li>Item 1</li>"));
    }
}
