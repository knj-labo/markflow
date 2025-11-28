//! Adapter that exposes `markdown-rs` AST nodes as Markflow core events.

use std::borrow::Cow;
use std::convert::TryFrom;

use log::warn;
use markdown::{ParseOptions, mdast, message::Message, to_mdast};

use crate::event::{Alignment, CodeBlockKind, Event, HeadingLevel, LinkType, Tag};

pub struct MarkdownRsEventIter {
    events: Vec<Event<'static>>,
    cursor: usize,
}

impl MarkdownRsEventIter {
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
    #[allow(unreachable_patterns)]
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
                let tag = Tag::Heading {
                    level: HeadingLevel::try_from(heading.depth as usize)
                        .unwrap_or(HeadingLevel::H6),
                    id: None,
                    classes: Vec::new(),
                    attrs: Vec::new(),
                };
                self.with_tag(tag, &heading.children)
            }
            mdast::Node::Blockquote(block) => self.with_tag(Tag::BlockQuote, &block.children),
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
                let tag = Tag::CodeBlock(match &code.lang {
                    Some(lang) => CodeBlockKind::Fenced(Cow::Owned(lang.clone())),
                    None => CodeBlockKind::Indented,
                });
                self.events.push(Event::Start(tag.clone()));
                self.events
                    .push(Event::Text(Cow::Owned(code.value.clone())));
                self.events.push(Event::End(tag.to_end()));
            }
            mdast::Node::Text(text) => {
                self.events
                    .push(Event::Text(Cow::Owned(text.value.clone())));
            }
            mdast::Node::Emphasis(emphasis) => self.with_tag(Tag::Emphasis, &emphasis.children),
            mdast::Node::Strong(strong) => self.with_tag(Tag::Strong, &strong.children),
            mdast::Node::Delete(delete) => self.with_tag(Tag::Strikethrough, &delete.children),
            mdast::Node::InlineCode(code) => {
                self.events
                    .push(Event::Code(Cow::Owned(code.value.clone())));
            }
            mdast::Node::InlineMath(math) => {
                self.events
                    .push(Event::InlineMath(Cow::Owned(math.value.clone())));
            }
            mdast::Node::Math(math) => {
                self.events
                    .push(Event::DisplayMath(Cow::Owned(math.value.clone())));
            }
            mdast::Node::Break(_) => self.events.push(Event::HardBreak),
            mdast::Node::Link(link) => self.handle_link(link),
            mdast::Node::Image(image) => self.handle_image(image),
            mdast::Node::Html(html) => {
                self.events
                    .push(Event::Html(Cow::Owned(html.value.clone())));
            }
            mdast::Node::Table(table) => self.handle_table(table),
            mdast::Node::TableRow(row) => self.with_tag(Tag::TableRow, &row.children),
            mdast::Node::TableCell(cell) => self.with_tag(Tag::TableCell, &cell.children),
            mdast::Node::FootnoteDefinition(def) => self.with_tag(
                Tag::FootnoteDefinition(Cow::Owned(def.identifier.clone())),
                &def.children,
            ),
            mdast::Node::FootnoteReference(reference) => {
                self.events.push(Event::FootnoteReference(Cow::Owned(
                    reference.identifier.clone(),
                )));
            }
            mdast::Node::LinkReference(link) => self.handle_link_reference(link),
            mdast::Node::ImageReference(image) => self.handle_image_reference(image),
            mdast::Node::Definition(_) => self.warn_unsupported("definition"),
            mdast::Node::Toml(_) => self.warn_unsupported("toml"),
            mdast::Node::Yaml(_) => self.warn_unsupported("yaml"),
            mdast::Node::MdxjsEsm(_) => self.warn_unsupported("mdxjsEsm"),
            mdast::Node::MdxFlowExpression(_) => self.warn_unsupported("mdxFlowExpression"),
            mdast::Node::MdxTextExpression(_) => self.warn_unsupported("mdxTextExpression"),
            mdast::Node::MdxJsxFlowElement(_) => self.warn_unsupported("mdxJsxFlowElement"),
            mdast::Node::MdxJsxTextElement(_) => self.warn_unsupported("mdxJsxTextElement"),
            other => {
                if let Some(children) = other.children() {
                    self.visit_children(children);
                } else {
                    self.warn_unsupported("unknown");
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
            dest_url: Cow::Owned(link.url.clone()),
            title: link
                .title
                .clone()
                .map_or(Cow::Borrowed(""), |t| Cow::Owned(t)),
            id: Cow::Owned(String::new()),
        };
        self.with_tag(tag, &link.children);
    }

    fn handle_image(&mut self, image: &mdast::Image) {
        let tag = Tag::Image {
            link_type: LinkType::Inline,
            dest_url: Cow::Owned(image.url.clone()),
            title: image
                .title
                .clone()
                .map_or(Cow::Borrowed(""), |t| Cow::Owned(t)),
            id: Cow::Owned(String::new()),
        };
        self.events.push(Event::Start(tag.clone()));
        if !image.alt.is_empty() {
            self.events.push(Event::Text(Cow::Owned(image.alt.clone())));
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
            dest_url: Cow::Borrowed(""),
            title: Cow::Borrowed(""),
            id: Cow::Owned(link.identifier.clone()),
        };
        self.with_tag(tag, &link.children);
    }

    fn handle_image_reference(&mut self, image: &mdast::ImageReference) {
        let tag = Tag::Image {
            link_type: LinkType::Reference,
            dest_url: Cow::Borrowed(""),
            title: Cow::Borrowed(""),
            id: Cow::Owned(image.identifier.clone()),
        };
        self.events.push(Event::Start(tag.clone()));
        if !image.alt.is_empty() {
            self.events.push(Event::Text(Cow::Owned(image.alt.clone())));
        }
        self.events.push(Event::End(tag.to_end()));
    }

    fn warn_unsupported(&self, node_name: &str) {
        warn!("Skipping unsupported markdown node: {node_name}");
    }
}
