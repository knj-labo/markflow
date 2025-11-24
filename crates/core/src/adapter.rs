use pulldown_cmark::{Event, html};
use std::io::{self, Write};

/// A bridge that accepts an Iterator of Markdown Events and streams
/// the resulting HTML directly to an io::Write, avoiding intermediate String allocation.
pub struct PipeAdapter<W> {
    writer: W,
}

impl<W: Write> PipeAdapter<W> {
    /// Create a new adapter wrapping an IO writer
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Consumes the event iterator and drives the data into the writer.
    ///
    /// # Arguments
    /// * `events` - The iterator yielding Markdown events.
    pub fn drive<'a, I>(self, events: I) -> io::Result<()>
    where
        I: Iterator<Item = Event<'a>>,
    {
        // Destructure 'self' to move 'writer' out.
        // This satisfies the compiler's "never read" check because moving is a read.
        let mut writer = self.writer;

        html::write_html_io(&mut writer, events)?;

        // Flush the underlying writer to ensure all bytes are sent
        writer.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{Options, Parser};

    #[test]
    fn test_streaming_output() {
        let markdown_input = "# Hello Stream\n\n* Item 1\n* Item 2";
        let mut output_buffer = Vec::new();

        let parser = Parser::new_ext(markdown_input, Options::empty());
        let adapter = PipeAdapter::new(&mut output_buffer);

        adapter.drive(parser).expect("Failed to drive stream");

        let output_str = String::from_utf8(output_buffer).unwrap();

        assert!(output_str.contains("<h1>Hello Stream</h1>"));
        assert!(output_str.contains("<li>Item 1</li>"));
    }
}
