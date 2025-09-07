#![allow(clippy::uninlined_format_args)]

use crate::styles::MDStyle;
use log::info;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use rat_text::TextRange;
use rat_text::event::TextOutcome;
use rat_text::text_area::TextAreaState;

pub fn md_dump_styles(state: &mut TextAreaState) -> TextOutcome {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let mut sty = Vec::new();
    state.styles_at(cursor_byte, &mut sty);
    for (_r, s) in sty {
        info!("style {:?}: {:?}", cursor, MDStyle::try_from(s));
    }

    TextOutcome::Unchanged
}

pub fn md_dump(state: &TextAreaState) -> TextOutcome {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let selection = if state.selection().is_empty() {
        let mut sty = Vec::new();
        state.styles_at(cursor_byte, &mut sty);

        let first = sty.iter().find(|(_, s)| {
            matches!(
                MDStyle::try_from(*s).expect("fine"),
                MDStyle::Heading1
                    | MDStyle::Heading2
                    | MDStyle::Heading3
                    | MDStyle::Heading4
                    | MDStyle::Heading5
                    | MDStyle::Heading6
                    | MDStyle::Paragraph
                    | MDStyle::BlockQuote
                    | MDStyle::CodeBlock
                    | MDStyle::MathDisplay
                    | MDStyle::Rule
                    | MDStyle::Html
                    | MDStyle::FootnoteDefinition
                    | MDStyle::List
                    | MDStyle::DefinitionList
                    | MDStyle::Table
            )
        });

        if let Some((r, _)) = first {
            let r = state.byte_range(r.clone());
            TextRange::new((0, r.start.y), r.end)
        } else {
            TextRange::new((0, cursor.y), (0, cursor.y + 1))
        }
    } else {
        TextRange::new(
            (0, state.selection().start.y),
            (0, state.selection().end.y + 1),
        )
    };

    dump_md(state.str_slice(selection).as_ref());

    TextOutcome::Unchanged
}

fn dump_md(txt: &str) {
    info!("*** DUMP ***");
    info!("{:?}", txt);

    let mut it = Parser::new_ext(
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

    let mut ind = 0;
    loop {
        let Some((e, r)) = it.next() else {
            break;
        };

        match e {
            Event::Start(v) => {
                match v {
                    Tag::Paragraph => {
                        info!("{}Paragraph {:?}", " ".repeat(ind), r.clone(),);
                    }
                    Tag::Heading {
                        level,
                        id,
                        classes: _todo_classes_,
                        attrs: _todo_attrs_,
                    } => {
                        info!(
                            "{}Heading Level={:?} Id={:?} {:?}",
                            " ".repeat(ind),
                            level,
                            id,
                            r.clone(),
                        );
                    }
                    Tag::BlockQuote(kind) => {
                        info!(
                            "{}BlockQuote Kind={:?} {:?}",
                            " ".repeat(ind),
                            kind,
                            r.clone(),
                        );
                    }
                    Tag::CodeBlock(kind) => {
                        info!(
                            "{}CodeBlock Kind={:?} {:?}",
                            " ".repeat(ind),
                            kind,
                            r.clone(),
                        );
                    }
                    Tag::HtmlBlock => {
                        info!("{}HtmlBlock {:?}", " ".repeat(ind), r.clone(),);
                    }
                    Tag::List(first) => {
                        info!("{}List First={:?} {:?}", " ".repeat(ind), first, r.clone(),);
                    }
                    Tag::Item => {
                        info!("{}Item {:?}", " ".repeat(ind), r.clone(),);
                    }
                    Tag::FootnoteDefinition(label) => {
                        info!(
                            "{}FootnoteDefinition Label={:?} {:?}",
                            " ".repeat(ind),
                            label,
                            r.clone(),
                        );
                    }
                    Tag::DefinitionList => {
                        info!("{}DefinitionList {:?}", " ".repeat(ind), r.clone(),);
                    }
                    Tag::DefinitionListTitle => {
                        info!("{}DefinitionListTitle {:?}", " ".repeat(ind), r.clone(),);
                    }
                    Tag::DefinitionListDefinition => {
                        info!(
                            "{}DefinitionListDefinition {:?}",
                            " ".repeat(ind),
                            r.clone(),
                        );
                    }
                    Tag::Table(align) => {
                        info!(
                            "{}Table Alignment={:?} {:?}",
                            " ".repeat(ind),
                            align,
                            r.clone(),
                        );
                    }
                    Tag::TableHead => {
                        info!("{}TableHead {:?}", " ".repeat(ind), r.clone(),);
                    }
                    Tag::TableRow => {
                        info!("{}TableRow {:?}", " ".repeat(ind), r.clone(),);
                    }
                    Tag::TableCell => {
                        info!("{}TableCell {:?}", " ".repeat(ind), r.clone(),);
                    }
                    Tag::Emphasis => {
                        info!(
                            "{}Emphasis {:?} {:?}",
                            " ".repeat(ind),
                            r.clone(),
                            &txt[r.clone()]
                        );
                    }
                    Tag::Strong => {
                        info!(
                            "{}Strong {:?} {:?}",
                            " ".repeat(ind),
                            r.clone(),
                            &txt[r.clone()]
                        );
                    }
                    Tag::Strikethrough => {
                        info!(
                            "{}Strikethrough {:?} {:?}",
                            " ".repeat(ind),
                            r.clone(),
                            &txt[r.clone()]
                        );
                    }
                    Tag::Link {
                        link_type,
                        dest_url,
                        title,
                        id,
                    } => {
                        info!(
                            "{}Link LinkType={:?} DestUrl={:?} Title={:?} Id={:?} {:?} {:?}",
                            " ".repeat(ind),
                            link_type,
                            dest_url,
                            title,
                            id,
                            r.clone(),
                            &txt[r.clone()]
                        );
                    }
                    Tag::Image {
                        link_type,
                        dest_url,
                        title,
                        id,
                    } => {
                        info!(
                            "{}Image LinkType={:?} DestUrl={:?} Title={:?} Id={:?} {:?} {:?}",
                            " ".repeat(ind),
                            link_type,
                            dest_url,
                            title,
                            id,
                            r.clone(),
                            &txt[r.clone()]
                        );
                    }
                    Tag::MetadataBlock(kind) => {
                        info!(
                            "{}MetadataBlock Kind={:?} {:?} {:?}",
                            " ".repeat(ind),
                            kind,
                            r.clone(),
                            &txt[r.clone()]
                        );
                    }
                };
                ind += 4;
            }
            Event::End(v) => {
                ind -= 4;
                match v {
                    TagEnd::Paragraph => {
                        info!("{}/Paragraph {:?}", " ".repeat(ind), r.clone(),);
                    }
                    TagEnd::Heading(level) => {
                        info!(
                            "{}/Heading Level={:?} {:?} ",
                            " ".repeat(ind),
                            level,
                            r.clone(),
                        );
                    }
                    TagEnd::BlockQuote(kind) => {
                        info!(
                            "{}/BlockQuote Kind={:?} {:?}",
                            " ".repeat(ind),
                            kind,
                            r.clone(),
                        );
                    }
                    TagEnd::CodeBlock => {
                        info!("{}/CodeBlock {:?} ", " ".repeat(ind), r.clone(),);
                    }
                    TagEnd::HtmlBlock => {
                        info!("{}/HtmlBlock {:?} ", " ".repeat(ind), r.clone(),);
                    }
                    TagEnd::List(ordered) => {
                        info!(
                            "{}/List Ordered={:?} {:?}",
                            " ".repeat(ind),
                            ordered,
                            r.clone(),
                        );
                    }
                    TagEnd::Item => {
                        info!("{}/Item {:?} ", " ".repeat(ind), r.clone(),);
                    }
                    TagEnd::FootnoteDefinition => {
                        info!("{}/FootnoteDefinition {:?}", " ".repeat(ind), r.clone(),);
                    }
                    TagEnd::DefinitionList => {
                        info!("{}/DefinitionList {:?}", " ".repeat(ind), r.clone(),);
                    }
                    TagEnd::DefinitionListTitle => {
                        info!("{}/DefinitionListTitle {:?}", " ".repeat(ind), r.clone(),);
                    }
                    TagEnd::DefinitionListDefinition => {
                        info!(
                            "{}/DefinitionListDefinition {:?}",
                            " ".repeat(ind),
                            r.clone(),
                        );
                    }
                    TagEnd::Table => {
                        info!("{}/Table {:?}", " ".repeat(ind), r.clone(),);
                    }
                    TagEnd::TableHead => {
                        info!("{}/TableHead {:?}", " ".repeat(ind), r.clone(),);
                    }
                    TagEnd::TableRow => {
                        info!("{}/TableRow {:?}", " ".repeat(ind), r.clone(),);
                    }
                    TagEnd::TableCell => {
                        info!("{}/TableCell {:?}", " ".repeat(ind), r.clone(),);
                    }
                    TagEnd::Emphasis => {
                        info!(
                            "{}/Emphasis {:?} {:?}",
                            " ".repeat(ind),
                            r.clone(),
                            &txt[r.clone()]
                        );
                    }
                    TagEnd::Strong => {
                        info!(
                            "{}/Strong {:?} {:?}",
                            " ".repeat(ind),
                            r.clone(),
                            &txt[r.clone()]
                        );
                    }
                    TagEnd::Strikethrough => {
                        info!(
                            "{}/Strikethrough {:?} {:?}",
                            " ".repeat(ind),
                            r.clone(),
                            &txt[r.clone()]
                        );
                    }
                    TagEnd::Link => {
                        info!(
                            "{}/Link {:?} {:?}",
                            " ".repeat(ind),
                            r.clone(),
                            &txt[r.clone()]
                        );
                    }
                    TagEnd::Image => {
                        info!(
                            "{}/Image {:?} {:?}",
                            " ".repeat(ind),
                            r.clone(),
                            &txt[r.clone()]
                        );
                    }
                    TagEnd::MetadataBlock(kind) => {
                        info!(
                            "{}/MetadataBlock Kind={:?} {:?} {:?}",
                            " ".repeat(ind),
                            kind,
                            r.clone(),
                            &txt[r.clone()]
                        );
                    }
                }
            }
            Event::Text(_v) => {
                info!(
                    "{}Text {:?} {:?}",
                    " ".repeat(ind),
                    r.clone(),
                    &txt[r.clone()]
                );
            }
            Event::Code(v) => {
                info!(
                    "{}Code V={:?} {:?} {:?}",
                    " ".repeat(ind),
                    v.as_ref(),
                    r.clone(),
                    &txt[r.clone()]
                );
            }
            Event::InlineMath(v) => {
                info!(
                    "{}InlineMath V={:?} {:?} {:?}",
                    " ".repeat(ind),
                    v.as_ref(),
                    r.clone(),
                    &txt[r.clone()]
                );
            }
            Event::DisplayMath(v) => {
                info!(
                    "{}DisplayMath V={:?} {:?} {:?}",
                    " ".repeat(ind),
                    v.as_ref(),
                    r.clone(),
                    &txt[r.clone()]
                );
            }
            Event::Html(v) => {
                info!(
                    "{}Html V={:?} {:?} {:?}",
                    " ".repeat(ind),
                    v.as_ref(),
                    r.clone(),
                    &txt[r.clone()]
                );
            }
            Event::InlineHtml(v) => {
                info!(
                    "{}InlineHtml V={:?} {:?} {:?}",
                    " ".repeat(ind),
                    v.as_ref(),
                    r.clone(),
                    &txt[r.clone()]
                );
            }
            Event::FootnoteReference(v) => {
                info!(
                    "{}FootnoteReference V={:?} {:?} {:?}",
                    " ".repeat(ind),
                    v.as_ref(),
                    r.clone(),
                    &txt[r.clone()]
                );
            }
            Event::SoftBreak => {
                info!(
                    "{}SoftBreak {:?} {:?}",
                    " ".repeat(ind),
                    r.clone(),
                    &txt[r.clone()]
                );
            }
            Event::HardBreak => {
                info!(
                    "{}HardBreak {:?} {:?}",
                    " ".repeat(ind),
                    r.clone(),
                    &txt[r.clone()]
                );
            }
            Event::Rule => {
                info!(
                    "{}Rule {:?} {:?}",
                    " ".repeat(ind),
                    r.clone(),
                    &txt[r.clone()]
                );
            }
            Event::TaskListMarker(checked) => {
                info!(
                    "{}TaskListMarker Checked={:?} {:?} {:?}",
                    " ".repeat(ind),
                    checked,
                    r.clone(),
                    &txt[r.clone()]
                );
            }
        }
    }

    let rdef = it.reference_definitions();
    for (rstr, rdef) in rdef.iter() {
        info!(
            "ReferenceDefinition {:?} {:?} = {:?} {:?}",
            rdef.span,
            rstr,
            rdef.dest.as_ref(),
            rdef.title.as_ref().map(|v| v.as_ref())
        )
    }
}
