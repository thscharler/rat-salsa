use crate::parser::parse_md_item;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag};
use std::num::ParseIntError;
use std::ops::Range;

/// Markdown styles.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MDStyle {
    Heading1 = 0,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,

    Paragraph,
    BlockQuote,
    CodeBlock,
    MathDisplay,
    Rule = 10,
    Html,

    Link,
    LinkDef,
    Image,
    FootnoteDefinition,
    FootnoteReference,

    List,
    Item,
    TaskListMarker,
    ItemTag = 20,
    DefinitionList,
    DefinitionListTitle,
    DefinitionListDefinition,

    Table,
    TableHead,
    TableRow,
    TableCell,

    Emphasis,
    Strong,
    Strikethrough = 30,
    CodeInline,
    MathInline,

    MetadataBlock,
}

impl From<MDStyle> for usize {
    fn from(value: MDStyle) -> Self {
        value as usize
    }
}

impl TryFrom<usize> for MDStyle {
    type Error = ParseIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        use MDStyle::*;
        Ok(match value {
            0 => Heading1,
            1 => Heading2,
            2 => Heading3,
            3 => Heading4,
            4 => Heading5,
            5 => Heading6,

            6 => Paragraph,
            7 => BlockQuote,
            8 => CodeBlock,
            9 => MathDisplay,
            10 => Rule,
            11 => Html,

            12 => Link,
            13 => LinkDef,
            14 => Image,
            15 => FootnoteDefinition,
            16 => FootnoteReference,

            17 => List,
            18 => Item,
            19 => TaskListMarker,
            20 => ItemTag,
            21 => DefinitionList,
            22 => DefinitionListTitle,
            23 => DefinitionListDefinition,

            24 => Table,
            25 => TableHead,
            26 => TableRow,
            27 => TableCell,

            28 => Emphasis,
            29 => Strong,
            30 => Strikethrough,
            31 => CodeInline,
            32 => MathInline,

            33 => MetadataBlock,
            _ => return Err("256".parse::<u8>().expect_err("should fail")),
        })
    }
}

/// Parse the text and return the byte ranges for each structural element.
///
/// Returns a format suitable for TextArea: byte-range->style-idx.
/// The style-idx is convertible to a MDStyle.
///
pub fn parse_md_styles(txt: &str) -> Vec<(Range<usize>, usize)> {
    let mut styles = Vec::new();

    let p = Parser::new_ext(
        txt,
        Options::ENABLE_MATH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_SMART_PUNCTUATION
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_GFM
            | Options::ENABLE_DEFINITION_LIST,
    )
    .into_offset_iter();

    for (_, linkdef) in p.reference_definitions().iter() {
        styles.push((linkdef.span.clone(), MDStyle::LinkDef as usize));
    }

    for (e, r) in p {
        match e {
            Event::Start(Tag::Heading { level, .. }) => match level {
                HeadingLevel::H1 => styles.push((r, MDStyle::Heading1 as usize)),
                HeadingLevel::H2 => styles.push((r, MDStyle::Heading2 as usize)),
                HeadingLevel::H3 => styles.push((r, MDStyle::Heading3 as usize)),
                HeadingLevel::H4 => styles.push((r, MDStyle::Heading4 as usize)),
                HeadingLevel::H5 => styles.push((r, MDStyle::Heading5 as usize)),
                HeadingLevel::H6 => styles.push((r, MDStyle::Heading6 as usize)),
            },
            Event::Start(Tag::BlockQuote(_)) => {
                styles.push((r, MDStyle::BlockQuote as usize));
            }
            Event::Start(Tag::CodeBlock(_)) => {
                styles.push((r, MDStyle::CodeBlock as usize));
            }
            Event::Start(Tag::FootnoteDefinition(_)) => {
                styles.push((r, MDStyle::FootnoteDefinition as usize));
            }
            Event::Start(Tag::Item) => {
                // only color the marker
                let item_text = &txt[r.clone()];
                let item = parse_md_item(r.start, item_text.as_ref()).expect("md item");
                styles.push((
                    item.mark_bytes.start..item.mark_bytes.end,
                    MDStyle::ItemTag as usize,
                ));
                styles.push((r, MDStyle::Item as usize));
            }
            Event::Start(Tag::Emphasis) => {
                styles.push((r, MDStyle::Emphasis as usize));
            }
            Event::Start(Tag::Strong) => {
                styles.push((r, MDStyle::Strong as usize));
            }
            Event::Start(Tag::Strikethrough) => {
                styles.push((r, MDStyle::Strikethrough as usize));
            }
            Event::Start(Tag::Link { .. }) => {
                styles.push((r, MDStyle::Link as usize));
            }
            Event::Start(Tag::Image { .. }) => {
                styles.push((r, MDStyle::Image as usize));
            }
            Event::Start(Tag::MetadataBlock { .. }) => {
                styles.push((r, MDStyle::MetadataBlock as usize));
            }
            Event::Start(Tag::Paragraph) => {
                styles.push((r, MDStyle::Paragraph as usize));
            }
            Event::Start(Tag::HtmlBlock) => {
                styles.push((r, MDStyle::Html as usize));
            }
            Event::Start(Tag::List(_)) => {
                styles.push((r, MDStyle::List as usize));
            }
            Event::Start(Tag::Table(_)) => {
                styles.push((r, MDStyle::Table as usize));
            }
            Event::Start(Tag::TableHead) => {
                styles.push((r, MDStyle::TableHead as usize));
            }
            Event::Start(Tag::TableRow) => {
                styles.push((r, MDStyle::TableRow as usize));
            }
            Event::Start(Tag::TableCell) => {
                styles.push((r, MDStyle::TableCell as usize));
            }
            Event::Start(Tag::DefinitionList) => {
                styles.push((r, MDStyle::DefinitionList as usize));
            }
            Event::Start(Tag::DefinitionListTitle) => {
                styles.push((r, MDStyle::DefinitionListTitle as usize));
            }
            Event::Start(Tag::DefinitionListDefinition) => {
                styles.push((r, MDStyle::DefinitionListDefinition as usize));
            }

            Event::Code(_) => {
                styles.push((r, MDStyle::CodeInline as usize));
            }
            Event::InlineMath(_) => {
                styles.push((r, MDStyle::MathInline as usize));
            }
            Event::DisplayMath(_) => {
                styles.push((r, MDStyle::MathDisplay as usize));
            }
            Event::FootnoteReference(_) => {
                styles.push((r, MDStyle::FootnoteReference as usize));
            }
            Event::Rule => {
                styles.push((r, MDStyle::Rule as usize));
            }
            Event::TaskListMarker(_) => {
                styles.push((r, MDStyle::TaskListMarker as usize));
            }
            Event::Html(_) | Event::InlineHtml(_) => {
                styles.push((r, MDStyle::Html as usize));
            }

            Event::End(_) => {}
            Event::Text(_) => {}
            Event::SoftBreak => {}
            Event::HardBreak => {}
        }
    }

    styles
}
