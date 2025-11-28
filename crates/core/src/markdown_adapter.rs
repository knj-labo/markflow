#![cfg(feature = "markdown-rs")]
//! Adapter that exposes `markdown-rs` AST nodes as `pulldown_cmark::Event`s.

use markdown::{ParseOptions, mdast, message::Message, to_mdast};
use pulldown_cmark::{Alignment, CowStr, Event, HeadingLevel, LinkType, Tag};

/// Iterator that yields `pulldown_cmark` events backed by `markdown-rs`.
///
/// The adapter currently materializes the AST so we can progressively map the
/// constructs we care about onto the legacy event pipeline. Streaming will be
/// re-introduced once we drop `pulldown_cmark` entirely.
pub struct MarkdownRsEventIter {
    events: Vec<Event<'static>>,
    cursor: usize,
}

impl MarkdownRsEventIter {
    /// Parses the input with `markdown-rs` and prepares an event stream.
    pub fn new(input: &str) -> Result<Self, Message> {
        let tree = to_mdast(input, &ParseOptions::default())?;
        let mut builder = EventBuilder::default();
        builder.visit(&tree);
        Ok(Self {
            events: builder.events,
            cursor: 0,
        })
    }
}

impl Iterator for MarkdownRsEventIter {
    type Item = Event<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.events.len() {
            None
        } else {
            let event = self.events[self.cursor].clone();
            self.cursor += 1;
            Some(event)
        }
    }
}

#[derive(Default)]
struct EventBuilder {
    events: Vec<Event<'static>>,
    tight_list_depth: usize,
}

impl EventBuilder {
    fn visit(&mut self, node: &mdast::Node) {
        match node {
            mdast::Node::Root(root) => self.visit_children(&root.children),
            mdast::Node::Paragraph(paragraph) => {
                if self.tight_list_depth > 0 {
                    self.visit_children(&paragraph.children);
                } else {
                    self.with_tag(Tag::Paragraph, &paragraph.children)
                }
            }
            mdast::Node::Heading(heading) => {
                let level =
                    HeadingLevel::try_from(heading.depth as usize).unwrap_or(HeadingLevel::H6);
                let tag = Tag::Heading {
                    level,
                    id: None,
                    classes: Vec::new(),
                    attrs: Vec::new(),
                };
                self.with_tag(tag, &heading.children)
            }
            mdast::Node::Blockquote(block) => self.with_tag(Tag::BlockQuote(None), &block.children),
            mdast::Node::List(list) => {
                let start = if list.ordered {
                    Some(list.start.unwrap_or(1) as u64)
                } else {
                    None
                };
                self.with_tag(Tag::List(start), &list.children)
            }
            mdast::Node::ListItem(item) => {
                self.events.push(Event::Start(Tag::Item));
                if let Some(checked) = item.checked {
                    self.events.push(Event::TaskListMarker(checked));
                }
                let is_tight = !item.spread;
                if is_tight {
                    self.tight_list_depth += 1;
                }
                self.visit_children(&item.children);
                if is_tight {
                    self.tight_list_depth -= 1;
                }
                self.events.push(Event::End(Tag::Item.to_end()));
            }
            mdast::Node::ThematicBreak(_) => self.events.push(Event::Rule),
            mdast::Node::Code(code) => {
                let tag = Tag::CodeBlock(
                    match &code.lang {
                        Some(lang) => TagCodeBlockKind::fenced(lang),
                        None => TagCodeBlockKind::indented(),
                    }
                    .into_kind(),
                );
                self.events.push(Event::Start(tag.clone()));
                self.events
                    .push(Event::Text(CowStr::from(code.value.clone())));
                self.events.push(Event::End(tag.to_end()));
            }
            mdast::Node::Text(text) => {
                self.events
                    .push(Event::Text(CowStr::from(text.value.clone())));
            }
            mdast::Node::Emphasis(emphasis) => self.with_tag(Tag::Emphasis, &emphasis.children),
            mdast::Node::Strong(strong) => self.with_tag(Tag::Strong, &strong.children),
            mdast::Node::Delete(delete) => self.with_tag(Tag::Strikethrough, &delete.children),
            mdast::Node::InlineCode(code) => {
                self.events
                    .push(Event::Code(CowStr::from(code.value.clone())));
            }
            mdast::Node::InlineMath(math) => {
                self.events
                    .push(Event::InlineMath(CowStr::from(math.value.clone())));
            }
            mdast::Node::Math(math) => {
                self.events
                    .push(Event::DisplayMath(CowStr::from(math.value.clone())));
            }
            mdast::Node::Break(_) => self.events.push(Event::HardBreak),
            mdast::Node::Link(link) => self.handle_link(link),
            mdast::Node::Image(image) => self.handle_image(image),
            mdast::Node::Html(html) => {
                self.events
                    .push(Event::Html(CowStr::from(html.value.clone())));
            }
            mdast::Node::Table(table) => self.handle_table(table),
            mdast::Node::TableRow(row) => self.with_tag(Tag::TableRow, &row.children),
            mdast::Node::TableCell(cell) => self.with_tag(Tag::TableCell, &cell.children),
            mdast::Node::FootnoteDefinition(def) => {
                let label = CowStr::from(def.identifier.clone());
                self.with_tag(Tag::FootnoteDefinition(label), &def.children)
            }
            mdast::Node::FootnoteReference(reference) => {
                self.events.push(Event::FootnoteReference(CowStr::from(
                    reference.identifier.clone(),
                )));
            }
            mdast::Node::LinkReference(link) => self.handle_link_reference(link),
            mdast::Node::ImageReference(image) => self.handle_image_reference(image),
            // Default: keep walking children so nested inline nodes still render.
            other => {
                if let Some(children) = other.children() {
                    self.visit_children(children);
                }
            }
        }
    }

    fn visit_children(&mut self, children: &[mdast::Node]) {
        for child in children {
            self.visit(child);
        }
    }

    fn with_tag(&mut self, tag: Tag<'static>, children: &[mdast::Node]) {
        let end = tag.to_end();
        self.events.push(Event::Start(tag));
        self.visit_children(children);
        self.events.push(Event::End(end));
    }

    fn handle_link(&mut self, link: &mdast::Link) {
        let tag = Tag::Link {
            link_type: LinkType::Inline,
            dest_url: CowStr::from(link.url.clone()),
            title: link
                .title
                .clone()
                .map_or(CowStr::Borrowed(""), CowStr::from),
            id: CowStr::from(String::new()),
        };
        self.with_tag(tag, &link.children);
    }

    fn handle_image(&mut self, image: &mdast::Image) {
        let tag = Tag::Image {
            link_type: LinkType::Inline,
            dest_url: CowStr::from(image.url.clone()),
            title: image
                .title
                .clone()
                .map_or(CowStr::Borrowed(""), CowStr::from),
            id: CowStr::from(String::new()),
        };
        self.events.push(Event::Start(tag.clone()));
        if !image.alt.is_empty() {
            self.events
                .push(Event::Text(CowStr::from(image.alt.clone())));
        }
        self.events.push(Event::End(tag.to_end()));
    }

    fn handle_table(&mut self, table: &mdast::Table) {
        let alignments: Vec<Alignment> = table
            .align
            .iter()
            .map(|align| match align {
                mdast::AlignKind::Left => Alignment::Left,
                mdast::AlignKind::Right => Alignment::Right,
                mdast::AlignKind::Center => Alignment::Center,
                mdast::AlignKind::None => Alignment::None,
            })
            .collect();
        self.with_tag(Tag::Table(alignments), &table.children);
    }

    fn handle_link_reference(&mut self, link: &mdast::LinkReference) {
        let tag = Tag::Link {
            link_type: LinkType::Reference,
            dest_url: CowStr::from(String::new()),
            title: CowStr::Borrowed(""),
            id: CowStr::from(link.identifier.clone()),
        };
        self.with_tag(tag, &link.children);
    }

    fn handle_image_reference(&mut self, image: &mdast::ImageReference) {
        let tag = Tag::Image {
            link_type: LinkType::Reference,
            dest_url: CowStr::from(String::new()),
            title: CowStr::Borrowed(""),
            id: CowStr::from(image.identifier.clone()),
        };
        self.events.push(Event::Start(tag.clone()));
        if !image.alt.is_empty() {
            self.events
                .push(Event::Text(CowStr::from(image.alt.clone())));
        }
        self.events.push(Event::End(tag.to_end()));
    }
}

/// Helper for constructing `CodeBlockKind` without leaking pulldown internals
/// into the recursive builder logic.
struct TagCodeBlockKind<'a>(pulldown_cmark::CodeBlockKind<'a>);

impl<'a> TagCodeBlockKind<'a> {
    fn indented() -> Self {
        Self(pulldown_cmark::CodeBlockKind::Indented)
    }

    fn fenced(lang: &str) -> Self {
        Self(pulldown_cmark::CodeBlockKind::Fenced(CowStr::from(
            lang.to_owned(),
        )))
    }

    fn into_kind(self) -> pulldown_cmark::CodeBlockKind<'static> {
        self.0.into_static()
    }
}
