use std::convert::TryFrom;

// Extended text-editing for markdown.
use anyhow::{anyhow, Error};
use log::{debug, info};
use pulldown_cmark::{
    BlockQuoteKind, CodeBlockKind, CowStr, Event, HeadingLevel, InlineStr, OffsetIter, Options,
    Parser, Tag, TagEnd,
};
use rat_salsa::event::ct_event;
use rat_widget::event::{flow, HandleEvent, Regular, TextOutcome};
use rat_widget::text::{upos_type, TextPosition, TextRange};
use rat_widget::textarea::TextAreaState;
use std::cmp::min;
use std::ops::Range;
use textwrap::core::Word;
use textwrap::wrap_algorithms::Penalties;
use textwrap::{WordSeparator, WrapAlgorithm};
use unicode_segmentation::UnicodeSegmentation;

// Markdown styles
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
    type Error = Error;

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
            _ => return Err(anyhow!("invalid style {}", value)),
        })
    }
}

pub fn md_format(state: &mut TextAreaState, shift: bool) -> TextOutcome {
    if let Some((table_byte, table_range)) = md_table(state) {
        let cursor = state.cursor();

        let (table, new_cursor) = reformat_md_table(
            state.str_slice_byte(table_byte).as_ref(),
            table_range,
            cursor,
            shift,
            state.newline(),
        );

        state.begin_undo_seq();
        state.delete_range(table_range);
        state
            .value
            .insert_str(table_range.start, &table)
            .expect("fine");
        state.set_cursor(new_cursor, false);
        state.end_undo_seq();
        TextOutcome::TextChanged
    } else if let Some((
        item_byte, //
        item_range,
        para_byte,
        para_range,
    )) = md_item_paragraph(state)
    {
        let cursor = state.cursor();

        let item_str = state.str_slice_byte(item_byte.clone());
        let item = parse_md_item(item_byte.start, item_str.as_ref());
        let item_pos = state.byte_pos(item.mark_bytes.start);
        let item_text_pos = state.byte_pos(item.text_bytes.start);
        let text_indent0 = if item_pos.y == para_range.start.y {
            "".to_string()
        } else {
            " ".repeat((item_text_pos.x - para_range.start.x) as usize)
        };
        let text_indent = " ".repeat(item_text_pos.x as usize);
        let wrap_pos = if cursor.x <= item_text_pos.x {
            65
        } else {
            if shift {
                cursor.x
            } else {
                65
            }
        };

        let para_text = state.str_slice_byte(para_byte);
        let (para_text, _) = textwrap::unfill(para_text.as_ref());
        let wrap = textwrap::fill(
            para_text.as_ref(),
            textwrap::Options::new(wrap_pos as usize)
                .initial_indent(&text_indent0)
                .subsequent_indent(&text_indent)
                .break_words(false),
        );

        state.begin_undo_seq();
        state.delete_range(para_range);
        state
            .value
            .insert_str(para_range.start, &wrap)
            .expect("fine");
        state.set_cursor(para_range.start, false);
        state.end_undo_seq();
        TextOutcome::TextChanged
    } else if let Some((item_byte, item_range)) = md_item(state) {
        let cursor = state.cursor();

        let item_str = state.str_slice_byte(item_byte.clone());
        let item = parse_md_item(item_byte.start, item_str.as_ref());
        let item_text_range = state.byte_range(item.text_bytes.clone());
        let text_indent = " ".repeat(item_text_range.start.x as usize);
        let wrap_pos = if cursor.x <= item_text_range.start.x {
            65
        } else {
            if shift {
                cursor.x
            } else {
                65
            }
        };

        let last_item = md_next_item(state).is_none();

        let item_text = state.str_slice_byte(item.text_bytes);
        let (item_text, _) = textwrap::unfill(item_text.as_ref());
        let item_wrap = textwrap::fill(
            item_text.as_ref(),
            textwrap::Options::new(wrap_pos as usize)
                .subsequent_indent(&text_indent)
                .break_words(false),
        );

        state.begin_undo_seq();
        state.delete_range(item_text_range);
        if last_item {
            // trims newlines after the item. add here.
            let newline = state.newline().to_string();
            state
                .value
                .insert_str(item_text_range.start, &newline)
                .expect("fine");
        }
        state
            .value
            .insert_str(item_text_range.start, &item_wrap)
            .expect("fine");
        state.set_cursor(item_text_range.start, false);
        state.end_undo_seq();
        TextOutcome::TextChanged
    } else if let Some((block_byte, block_range)) = md_block_quote(state) {
        let cursor = state.cursor();

        let txt = state.str_slice_byte(block_byte.clone());
        let block = parse_md_block_quote(block_byte.start, txt.as_ref());

        let text_start = state.byte_pos(block.text_start_byte);
        let mut text_indent0 = " ".repeat(text_start.x as usize - 1);
        text_indent0.insert(0, '>');
        let mut text_indent = " ".repeat(text_start.x as usize - 1);
        text_indent.insert(0, '>');
        let wrap_pos = if cursor.x <= text_start.x {
            65
        } else {
            if shift {
                cursor.x
            } else {
                65
            }
        };

        let mut wrap = textwrap::fill(
            &block.text,
            textwrap::Options::new(wrap_pos as usize)
                .initial_indent(&text_indent0)
                .subsequent_indent(&text_indent)
                .break_words(false),
        );
        wrap.push_str(state.newline());

        state.begin_undo_seq();
        state.delete_range(block_range);
        state
            .value
            .insert_str(block_range.start, &wrap)
            .expect("fine");
        state.set_cursor(block_range.start, false);
        state.end_undo_seq();
        TextOutcome::TextChanged
    } else if let Some((para_byte, para_range)) = md_paragraph(state) {
        let cursor = state.cursor();
        let wrap_pos = if cursor.x == 0 {
            65
        } else {
            if shift {
                cursor.x
            } else {
                65
            }
        };

        let txt = state.str_slice_byte(para_byte);
        // unfill does too much.
        let unfill = txt
            .as_ref()
            .bytes()
            .map(|v| if v == b'\n' || v == b'\r' { b' ' } else { v })
            .chain(state.newline().bytes())
            .collect::<Vec<_>>();
        let unfill = String::from_utf8(unfill).unwrap_or_default();
        let wrap = textwrap::fill(
            unfill.as_ref(),
            textwrap::Options::new(wrap_pos as usize).break_words(false),
        );

        state.begin_undo_seq();
        state.delete_range(para_range);
        state
            .value
            .insert_str(para_range.start, &wrap)
            .expect("fine");
        state.set_cursor(para_range.start, false);
        state.end_undo_seq();
        TextOutcome::TextChanged
    } else {
        TextOutcome::Continue
    }
}

pub fn md_line_break(state: &mut TextAreaState) -> TextOutcome {
    let cursor = state.cursor();
    if is_md_table(state) {
        let line = state.line_at(cursor.y);
        if cursor.x == state.line_width(cursor.y) {
            let (x, row) = empty_md_row(line.as_ref(), state.newline());
            state.insert_str(row);
            state.set_cursor((x, cursor.y + 1), false);
            TextOutcome::TextChanged
        } else {
            let (x, row) = split_md_row(line.as_ref(), cursor.x, state.newline());
            state.begin_undo_seq();
            state.delete_range(TextRange::new((0, cursor.y), (0, cursor.y + 1)));
            state.insert_str(row);
            state.set_cursor((x, cursor.y + 1), false);
            state.end_undo_seq();
            TextOutcome::TextChanged
        }
    } else {
        let cursor = state.cursor();
        if cursor.x == state.line_width(cursor.y) {
            let (maybe_table, maybe_header) = is_md_maybe_table(state);
            if maybe_header {
                let line = state.line_at(cursor.y);
                let (x, row) = empty_md_row(line.as_ref(), state.newline());
                state.insert_str(row);
                state.set_cursor((x, cursor.y + 1), false);
                TextOutcome::TextChanged
            } else if maybe_table {
                let line = state.line_at(cursor.y);
                let (x, row) = create_md_title(line.as_ref(), state.newline());
                state.insert_str(row);
                state.set_cursor((x, cursor.y + 1), false);
                TextOutcome::TextChanged
            } else {
                TextOutcome::Continue
            }
        } else {
            TextOutcome::Continue
        }
    }
}

pub fn md_tab(state: &mut TextAreaState) -> TextOutcome {
    if is_md_table(state) {
        let cursor = state.cursor();
        let row = state.line_at(cursor.y);
        let x = next_tab_md_row(row.as_ref(), cursor.x);
        state.set_cursor((x, cursor.y), false);
        state.set_move_col(Some(x));
        TextOutcome::TextChanged
    } else if is_md_item(state) {
        let cursor = state.cursor();

        let (item_byte, item_range) = md_item(state).expect("item");
        let indent_x = if item_range.start.y < cursor.y {
            let item_str = state.str_slice_byte(item_byte.clone());
            let item = parse_md_item(item_byte.start, item_str.as_ref());
            state.byte_pos(item.text_bytes.start).x
        } else if let Some((prev_byte, prev_range)) = md_prev_item(state) {
            let prev_str = state.str_slice_byte(prev_byte.clone());
            let prev_item = parse_md_item(prev_byte.start, prev_str.as_ref());
            state.byte_pos(prev_item.text_bytes.start).x
        } else {
            0
        };

        if cursor.x < indent_x {
            state
                .value
                .insert_str(cursor, &(" ".repeat((indent_x - cursor.x) as usize)))
                .expect("fine");
            TextOutcome::TextChanged
        } else {
            TextOutcome::Continue
        }
    } else {
        TextOutcome::Continue
    }
}

pub fn md_backtab(state: &mut TextAreaState) -> TextOutcome {
    if is_md_table(state) {
        let cursor = state.cursor();

        let row_str = state.line_at(cursor.y);
        let x = prev_tab_md_row(row_str.as_ref(), cursor.x);

        state.set_cursor((x, cursor.y), false);
        state.set_move_col(Some(x));
        TextOutcome::TextChanged
    } else {
        TextOutcome::Continue
    }
}

fn md_dump(state: &mut TextAreaState) -> TextOutcome {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let selection = if state.selection().is_empty() {
        let mut sty = Vec::new();
        state.styles_at(cursor_byte, &mut sty);

        if let Some((r, _)) = sty.iter().find(|(_, s)| {
            matches!(
                MDStyle::try_from(*s).expect("fine"),
                MDStyle::Table | MDStyle::DefinitionList | MDStyle::List
            )
        }) {
            let r = state.byte_range(r.clone());
            TextRange::new((0, r.start.y), r.end)
        } else if let Some((r, _)) = sty.first() {
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
    let selection_byte = state.bytes_at_range(selection);

    dump_md(state.str_slice(selection).as_ref());

    TextOutcome::Unchanged
}

fn md_dump_styles(state: &mut TextAreaState) -> TextOutcome {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let mut sty = Vec::new();
    state.styles_at(cursor_byte, &mut sty);
    for (r, s) in sty {
        debug!("style {:?}: {:?}", cursor, MDStyle::try_from(s));
    }

    TextOutcome::Unchanged
}

fn md_debug(state: &mut TextAreaState) -> TextOutcome {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let selection = if state.selection().is_empty() {
        let mut sty = Vec::new();
        state.styles_at(cursor_byte, &mut sty);

        if let Some((r, _)) = sty.iter().find(|(_, s)| {
            matches!(
                MDStyle::try_from(*s).expect("fine"),
                MDStyle::Table | MDStyle::DefinitionList | MDStyle::List
            )
        }) {
            let r = state.byte_range(r.clone());
            TextRange::new((0, r.start.y), r.end)
        } else if let Some((r, _)) = sty.first() {
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
    let selection_byte = state.bytes_at_range(selection);

    // relative cursor
    let (txt_cursor, txt_cursor_byte) = if selection.contains_pos(cursor) {
        (
            TextPosition::new(cursor.x, cursor.y - selection.start.y),
            cursor_byte - selection_byte.start,
        )
    } else {
        (TextPosition::new(0, 0), 0)
    };

    let (wrapped, new_cursor) = reformat(
        state.str_slice(selection).as_ref(),
        txt_cursor,
        txt_cursor_byte,
        65,
        false,
        state.newline(),
    );
    let new_cursor = selection_byte.start + new_cursor;

    state.begin_undo_seq();
    state.delete_range(selection);
    state
        .value
        .insert_str(selection.start, &wrapped)
        .expect("fine");
    let new_cursor = state.byte_pos(new_cursor);
    state.set_cursor(new_cursor, false);
    state.end_undo_seq();

    TextOutcome::TextChanged
}

// qualifier for markdown-editing.
#[derive(Debug)]
pub struct MarkDown;

impl HandleEvent<crossterm::event::Event, MarkDown, TextOutcome> for TextAreaState {
    fn handle(&mut self, event: &crossterm::event::Event, qualifier: MarkDown) -> TextOutcome {
        flow!(match event {
            ct_event!(key press ALT-'p') => md_debug(self),
            ct_event!(key press ALT-'d') => md_dump(self),
            ct_event!(key press ALT-'s') => md_dump_styles(self),

            ct_event!(key press ALT-'f') => md_format(self, false),
            ct_event!(key press ALT_SHIFT-'F') => md_format(self, true),
            ct_event!(keycode press Enter) => md_line_break(self),
            ct_event!(keycode press Tab) => md_tab(self),
            ct_event!(keycode press SHIFT-BackTab) => md_backtab(self),
            _ => TextOutcome::Continue,
        });

        self.handle(event, Regular)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MDFormat {
    None,
    Heading,
    Paragraph,
    Footnote,
    DefinitionList,
    ReferenceDefs,
    Table,
    BlockQuote,
    CodeBlock,
    List,
    Item,
}

struct Reformat<'a> {
    txt: &'a str,
    txt_width: usize,
    cursor: TextPosition,
    cursor_byte: usize,
    newline: &'a str,
    table_eq_width: bool,

    prefix: Vec<usize>,
    follow: Vec<CowStr<'a>>,
}

impl<'a> Reformat<'a> {
    fn assert_indent_empty(&self) {
        assert!(self.prefix.is_empty());
        assert!(self.follow.is_empty());
    }

    fn indent_val(&self) -> (usize, usize) {
        (self.prefix.len(), self.follow.len())
    }

    fn assert_indent(&self, before: (usize, usize)) {
        assert_eq!(before, self.indent_val());
    }

    // write follow indent
    fn indent_follow(&mut self, out: &mut ReformatOut) {
        for v in self.follow.iter() {
            out.txt.push_str(v);
        }
    }

    // set line prefix as indent
    fn indent_from_prefix(&mut self, pos_byte: usize, out: &mut ReformatOut) {
        let mut gr_it = self.txt[..pos_byte].grapheme_indices(true).rev();
        let mut start_byte = pos_byte;
        let mut prefix_len = 0;
        loop {
            let Some((idx, gr)) = gr_it.next() else {
                break;
            };
            start_byte = idx;
            if gr == "\n" || gr == "\n\r" {
                start_byte = idx + gr.len();
                break;
            }
            prefix_len += 1;
        }

        self.assert_indent_empty();
        self.prefix.push(prefix_len);
        self.follow
            .push(CowStr::Borrowed(&self.txt[start_byte..pos_byte]));

        for v in self.follow.iter() {
            out.txt.push_str(v);
        }
    }

    // set blanks before as indent
    fn indent_extend_from_blank(&mut self, pos_byte: usize, out: &mut ReformatOut) {
        let mut gr_it = self.txt[..pos_byte].grapheme_indices(true).rev();
        let mut start_byte = pos_byte;
        let mut prefix_len = 0;
        loop {
            let Some((idx, gr)) = gr_it.next() else {
                break;
            };
            start_byte = idx;
            if gr != " " {
                start_byte = idx + gr.len();
                break;
            }
            prefix_len += 1;
        }

        out.txt.push_str(&self.txt[start_byte..pos_byte]);

        self.prefix.push(prefix_len);
        self.follow
            .push(CowStr::Borrowed(&self.txt[start_byte..pos_byte]));
    }

    fn indent(&mut self, prefix: usize, follow: CowStr<'a>) {
        self.prefix.push(prefix);
        self.follow.push(follow);
    }

    fn dedent(&mut self) {
        self.prefix.pop();
        self.follow.pop();
    }

    fn total_prefix(&self) -> usize {
        self.prefix.iter().sum()
    }

    // word-relative cursor position, if any.
    #[allow(trivial_casts)]
    fn word_cursor(&self, word: &str) -> Option<usize> {
        let txt_ptr = self.txt as *const str as *const u8 as usize;
        let word_ptr = word as *const str as *const u8 as usize;

        let word_range = if word_ptr >= txt_ptr && word_ptr < txt_ptr + self.txt.len() {
            let word_offset = word_ptr - txt_ptr;
            word_offset..word_offset + word.len()
        } else {
            0..0
        };

        if word_range.contains(&self.cursor_byte) {
            Some(self.cursor_byte - word_range.start)
        } else {
            None
        }
    }
}

struct ReformatOut {
    txt: String,
    cursor: usize,
}

pub fn reformat(
    txt: &str,
    cursor: TextPosition,
    cursor_byte: usize,
    txt_width: usize,
    table_eq_width: bool,
    newline: &str,
) -> (String, usize) {
    let mut p = Parser::new_ext(
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

    let mut arg = Reformat {
        txt,
        txt_width,
        cursor,
        cursor_byte,
        newline,
        table_eq_width,
        follow: Vec::new(),
        prefix: Vec::new(),
    };
    let mut out = ReformatOut {
        txt: String::new(),
        cursor: 0,
    };
    let mut md_last = MDFormat::None;
    loop {
        let Some((e, r)) = p.next() else {
            break;
        };

        match e {
            Event::Start(Tag::Paragraph) => {
                insert_empty(&mut arg, md_last, MDFormat::Paragraph, &mut out);

                arg.assert_indent_empty();
                let ind = arg.indent_val();
                arg.indent_from_prefix(r.start, &mut out);
                reformat_paragraph(&mut arg, &mut p, &mut out);
                arg.dedent();
                arg.assert_indent(ind);

                md_last = MDFormat::Paragraph;
            }
            Event::Start(Tag::Heading { level, .. }) => {
                insert_empty(&mut arg, md_last, MDFormat::Heading, &mut out);

                arg.assert_indent_empty();
                let ind = arg.indent_val();
                reformat_heading(&mut arg, &mut p, level, &mut out);
                arg.assert_indent(ind);

                md_last = MDFormat::Heading;
            }
            Event::Start(Tag::BlockQuote(kind)) => {
                insert_empty(&mut arg, md_last, MDFormat::BlockQuote, &mut out);

                arg.assert_indent_empty();
                let ind = arg.indent_val();
                arg.indent_from_prefix(r.start, &mut out);
                reformat_blockquote(&mut arg, &mut p, r, kind, &mut out);
                arg.dedent();
                arg.assert_indent(ind);

                md_last = MDFormat::BlockQuote;
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                insert_empty(&mut arg, md_last, MDFormat::CodeBlock, &mut out);

                arg.assert_indent_empty();
                let ind = arg.indent_val();
                arg.indent_from_prefix(r.start, &mut out);
                reformat_codeblock(&mut arg, &mut p, r, kind, &mut out);
                arg.dedent();
                arg.assert_indent(ind);

                md_last = MDFormat::CodeBlock;
            }
            Event::Start(Tag::List(v)) => {
                insert_empty(&mut arg, md_last, MDFormat::List, &mut out);

                arg.assert_indent_empty();
                let ind = arg.indent_val();
                arg.indent_from_prefix(r.start, &mut out);
                reformat_list(&mut arg, &mut p, r, &mut out);
                arg.dedent();
                arg.assert_indent(ind);

                insert_empty(&mut arg, MDFormat::List, MDFormat::Paragraph, &mut out);

                md_last = MDFormat::List;
            }
            Event::Start(Tag::Item) => {
                unreachable!("list {:?} {:?}", e, r);
            }
            Event::Start(Tag::HtmlBlock) => {}

            Event::Start(Tag::FootnoteDefinition(v)) => {
                insert_empty(&mut arg, md_last, MDFormat::Footnote, &mut out);

                arg.assert_indent_empty();
                let ind = arg.indent_val();
                arg.indent_from_prefix(r.start, &mut out);
                reformat_footnote(&mut arg, &mut p, v, &mut out);
                arg.dedent();
                arg.assert_indent(ind);

                md_last = MDFormat::Footnote;
            }

            Event::Start(Tag::DefinitionList) => {
                insert_empty(&mut arg, md_last, MDFormat::DefinitionList, &mut out);

                arg.assert_indent_empty();
                let ind = arg.indent_val();
                arg.indent_from_prefix(r.start, &mut out);
                reformat_definition(&mut arg, &mut p, &mut out);
                arg.dedent();
                arg.assert_indent(ind);

                md_last = MDFormat::DefinitionList;
            }
            Event::Start(Tag::DefinitionListTitle)
            | Event::Start(Tag::DefinitionListDefinition) => {
                unreachable!("def-list {:?} {:?}", e, r);
            }

            Event::Start(Tag::Table(_)) => {
                insert_empty(&mut arg, md_last, MDFormat::Table, &mut out);

                arg.assert_indent_empty();
                let ind = arg.indent_val();
                arg.indent_from_prefix(r.start, &mut out);
                reformat_table(&mut arg, &mut p, r, &mut out);
                arg.dedent();
                arg.assert_indent(ind);

                md_last = MDFormat::Table;
            }
            Event::Start(Tag::TableHead)
            | Event::Start(Tag::TableRow)
            | Event::Start(Tag::TableCell) => {
                unreachable!("table {:?} {:?}", e, r);
            }

            Event::Start(Tag::Emphasis)
            | Event::Start(Tag::Strong)
            | Event::Start(Tag::Strikethrough)
            | Event::Start(Tag::Link { .. })
            | Event::Start(Tag::Image { .. })
            | Event::Text(_)
            | Event::Code(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::SoftBreak
            | Event::HardBreak => {
                unreachable!("inline {:?} {:?}", e, r);
            }

            Event::Start(Tag::MetadataBlock(_)) => {}
            Event::End(_) => {}
            Event::Html(_) => {}
            Event::Rule => {}
            Event::TaskListMarker(_) => {}
        }
    }

    for (link_name, linkdef) in p.reference_definitions().iter() {
        insert_empty(&mut arg, md_last, MDFormat::ReferenceDefs, &mut out);

        out.txt.push_str("[");
        out.txt.push_str(link_name);
        out.txt.push_str("]: ");
        out.txt.push_str(linkdef.dest.as_ref());
        if let Some(title) = linkdef.title.as_ref() {
            out.txt.push_str(" ");
            out.txt.push_str(title.as_ref());
        }
        out.txt.push_str(newline);
    }

    (out.txt, out.cursor)
}

fn reformat_list<'a>(
    arg: &mut Reformat<'a>,
    it: &mut OffsetIter<'a>,
    range: Range<usize>,
    out: &mut ReformatOut,
) {
    let ind = arg.indent_val();

    let item = parse_md_item(range.start, &arg.txt[range]);
    out.txt.push_str(item.prefix);
    arg.prefix.push(str_line_len(item.prefix) as usize);
    arg.follow.push(CowStr::Borrowed(item.prefix));
    let mut nr = item.mark_nr;

    let mut md_last = MDFormat::None;
    let mut was_complex = false;
    loop {
        let Some((e, r)) = it.next() else {
            break;
        };

        match e {
            Event::End(TagEnd::List(_)) => {
                break;
            }
            Event::Start(Tag::Item) => {
                // multi paragraph items get an extra separator
                if was_complex {
                    was_complex = false;
                    arg.indent_follow(out);
                    out.txt.push_str(arg.newline);
                }
                if md_last != MDFormat::None {
                    arg.indent_follow(out);
                }

                let mut item = parse_md_item(r.start, &arg.txt[r]);
                item.mark_nr = nr;

                let ind = arg.indent_val();
                reformat_list_item(arg, it, &item, out, &mut was_complex);
                arg.assert_indent(ind);

                nr = nr.map(|v| v + 1);
                md_last = MDFormat::Item;
            }
            _ => unreachable!("{:?} {:?}", e, r),
        }
    }

    if md_last != MDFormat::None {
        arg.dedent();
    }

    arg.assert_indent(ind);
}

fn reformat_list_item<'a>(
    arg: &mut Reformat<'a>,
    it: &mut OffsetIter<'a>,
    item: &MDItem<'a>,
    out: &mut ReformatOut,
    out_was_complex: &mut bool,
) {
    use std::fmt::Write;

    let ind = arg.indent_val();

    if let Some(nr) = item.mark_nr {
        _ = write!(out.txt, "{}{}", nr, item.mark_suffix);
        let len = (nr.ilog10() + 1) as usize + 1;
        arg.prefix.push(len);
        arg.follow
            .push(CowStr::Boxed(" ".repeat(len).into_boxed_str()));
    } else {
        out.txt.push_str(item.mark);
        arg.prefix.push(1);
        arg.follow.push(CowStr::Borrowed(" "));
    }

    let txt_prefix_len = str_line_len(item.text_prefix) as usize;
    out.txt.push_str(item.text_prefix);
    arg.prefix.push(txt_prefix_len);
    arg.follow
        .push(CowStr::Boxed(" ".repeat(txt_prefix_len).into_boxed_str()));

    let mut words = Vec::new();
    let mut skip_txt = false;
    let mut md_last = MDFormat::None;

    loop {
        let Some((e, r)) = it.next() else {
            break;
        };

        match e {
            Event::End(TagEnd::Item) => {
                break;
            }

            Event::Start(Tag::Emphasis) | Event::End(TagEnd::Emphasis) => {
                let word = Word::from(&arg.txt[r.start..r.start + 1]);
                words.push(word);
            }
            Event::Start(Tag::Strong)
            | Event::End(TagEnd::Strong)
            | Event::Start(Tag::Strikethrough)
            | Event::End(TagEnd::Strikethrough) => {
                let word = Word::from(&arg.txt[r.start..r.start + 2]);
                words.push(word);
            }

            Event::Start(Tag::Link { .. }) | Event::Start(Tag::Image { .. }) => {
                skip_txt = true;
                let word = Word::from(&arg.txt[r]);
                words.push(word);
            }
            Event::End(TagEnd::Link) | Event::End(TagEnd::Image) => {
                skip_txt = false;
            }

            Event::Code(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_) => {
                let word = Word::from(&arg.txt[r]);
                words.push(word);
            }
            Event::TaskListMarker(_) => {
                let mut word = Word::from(&arg.txt[r]);
                word.whitespace = " ";
                words.push(word);
            }

            Event::SoftBreak => {
                if skip_txt {
                    continue;
                }
                if let Some(mut word) = words.pop() {
                    word.whitespace = " ";
                    words.push(word);
                }
            }
            Event::HardBreak => {
                if skip_txt {
                    continue;
                }
                if md_last != MDFormat::None {
                    arg.indent_follow(out);
                }
                wrap_words(arg, &mut words, NewLine::Hard, out);

                md_last = MDFormat::Item;
            }

            Event::Text(_) => {
                if skip_txt {
                    continue;
                }
                for w in WordSeparator::UnicodeBreakProperties.find_words(&arg.txt[r]) {
                    words.push(w);
                }
            }

            Event::Start(Tag::Paragraph) => {
                let ind = arg.indent_val();
                if !words.is_empty() {
                    if md_last != MDFormat::None {
                        arg.indent_follow(out);
                    }
                    wrap_words(arg, &mut words, NewLine::Soft, out);
                }
                // recurse
                if insert_empty(arg, md_last, MDFormat::Paragraph, out) {
                    arg.indent_follow(out);
                }
                reformat_paragraph(arg, it, out);
                arg.assert_indent(ind);

                *out_was_complex = true;
                md_last = MDFormat::Paragraph;
            }
            Event::Start(Tag::List(_)) => {
                let ind = arg.indent_val();
                if !words.is_empty() {
                    if md_last != MDFormat::None {
                        arg.indent_follow(out);
                    }
                    wrap_words(arg, &mut words, NewLine::Soft, out);
                    arg.indent_follow(out);
                }
                reformat_list(arg, it, r, out);
                arg.assert_indent(ind);

                // lists per se are not complex :-)
                // *out_was_complex = true;
                md_last = MDFormat::List;
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let ind = arg.indent_val();
                if !words.is_empty() {
                    if md_last != MDFormat::None {
                        arg.indent_follow(out);
                    }
                    wrap_words(arg, &mut words, NewLine::Soft, out);
                }

                if insert_empty(arg, md_last, MDFormat::CodeBlock, out) {
                    arg.indent_follow(out);
                }
                out.txt.push_str("    ");
                arg.prefix.push(4);
                arg.follow
                    .push(CowStr::Boxed(" ".repeat(4).into_boxed_str()));
                reformat_codeblock(arg, it, r, kind, out);
                arg.dedent();
                arg.assert_indent(ind);

                *out_was_complex = true;
                md_last = MDFormat::CodeBlock;
            }

            _ => unreachable!("{:?} {:?}", e, r),
        }
    }

    if !words.is_empty() {
        if md_last != MDFormat::None {
            debug!("last_words {:?}", arg.follow);
            arg.indent_follow(out);
        }
        wrap_words(arg, &mut words, NewLine::Soft, out);
    }

    arg.dedent();
    arg.dedent();

    arg.assert_indent(ind);
}

fn reformat_codeblock<'a>(
    arg: &mut Reformat<'a>,
    it: &mut OffsetIter<'a>,
    range: Range<usize>,
    kind: CodeBlockKind<'a>,
    out: &mut ReformatOut,
) {
    let ind = arg.indent_val();

    let fence = if matches!(kind, CodeBlockKind::Fenced(_)) {
        &arg.txt[range.start..range.start + 3]
    } else {
        ""
    };

    let mut md_last = MDFormat::None;

    match &kind {
        CodeBlockKind::Indented => {}
        CodeBlockKind::Fenced(lang) => {
            out.txt.push_str(fence);
            out.txt.push_str(lang);
            out.txt.push_str(arg.newline);
            md_last = MDFormat::CodeBlock;
        }
    }

    let mut buf = String::new();
    for (e, r) in it {
        match e {
            Event::End(TagEnd::CodeBlock) => {
                break;
            }
            Event::Text(v) => {
                // text segments come in strange order.
                // collect first, line-break later.
                buf.push_str(v.as_ref());

                // do cursor work here, later we lost the slice
                if let Some(cur) = arg.word_cursor(v.as_ref()) {
                    out.cursor = out.txt.len() + cur;
                }
            }
            _ => unreachable!("{:?} {:?}", e, r),
        }
    }

    if buf.len() > 0 {
        for line in buf.lines() {
            if md_last != MDFormat::None {
                arg.indent_follow(out);
            }
            out.txt.push_str(line);
            out.txt.push_str(arg.newline);

            md_last = MDFormat::CodeBlock;
        }

        match &kind {
            CodeBlockKind::Indented => {}
            CodeBlockKind::Fenced(lang) => {
                arg.indent_follow(out);
                out.txt.push_str(fence);
            }
        }
    }

    arg.assert_indent(ind);
}

fn reformat_blockquote<'a>(
    arg: &mut Reformat<'a>,
    it: &mut OffsetIter<'a>,
    range: Range<usize>,
    kind: Option<BlockQuoteKind>,
    out: &mut ReformatOut,
) {
    let ind = arg.indent_val();

    if let Some(kind) = kind {
        out.txt.push_str("> ");
        match kind {
            BlockQuoteKind::Note => _ = out.txt.push_str("[!NOTE] "),
            BlockQuoteKind::Tip => _ = out.txt.push_str("[!TIP] "),
            BlockQuoteKind::Important => _ = out.txt.push_str("[!IMPORTANT] "),
            BlockQuoteKind::Warning => _ = out.txt.push_str("[!WARNING] "),
            BlockQuoteKind::Caution => _ = out.txt.push_str("[!CAUTION] "),
        }
        out.txt.push_str(arg.newline);
    }

    let block_quote = parse_md_block_quote2(range.start, &arg.txt[range.clone()]);

    out.txt.push_str(block_quote.quote);
    out.txt.push_str(block_quote.text_prefix);
    arg.prefix.push(1);
    arg.prefix.push(block_quote.text_prefix.len());
    arg.follow.push(CowStr::Borrowed(">"));
    arg.follow.push(CowStr::Borrowed(block_quote.text_prefix));

    let mut md_last = MDFormat::None;
    loop {
        let Some((e, r)) = it.next() else { break };
        match e {
            Event::End(TagEnd::BlockQuote(_)) => {
                break;
            }
            Event::Start(Tag::Paragraph) => {
                if insert_empty(arg, md_last, MDFormat::Paragraph, out) {
                    arg.indent_follow(out);
                }
                reformat_paragraph(arg, it, out);

                md_last = MDFormat::Paragraph;
            }
            Event::Start(Tag::BlockQuote(kind)) => {
                if insert_empty(arg, md_last, MDFormat::BlockQuote, out) {
                    arg.indent_follow(out);
                }
                reformat_blockquote(arg, it, r, kind, out);

                md_last = MDFormat::BlockQuote;
            }

            _ => unreachable!("{:?} {:?}", e, r),
        }
    }

    arg.dedent();
    arg.dedent();

    arg.assert_indent(ind);
}

fn reformat_table<'a>(
    arg: &mut Reformat<'a>,
    it: &mut OffsetIter<'a>,
    range: Range<usize>,
    out: &mut ReformatOut,
) {
    // eat events. don't use them for the table.
    loop {
        let Some((e, r)) = it.next() else { break };
        match e {
            Event::End(TagEnd::Table) => {
                break;
            }
            _ => {}
        }
    }

    use std::fmt::Write;

    let table_txt = &arg.txt[range];

    let mut table = Vec::new();
    for (n, row) in table_txt.lines().enumerate() {
        if !row.is_empty() {
            if arg.cursor.y == n as upos_type {
                table.push(parse_md_row(row, arg.cursor.x));
            } else {
                table.push(parse_md_row(row, 0));
            }
        }
    }
    let mut width = Vec::new();
    // only use header widths
    if let Some(row) = table.first() {
        for (idx, cell) in row.row.iter().enumerate() {
            width.push(str_line_len(cell.txt));
        }
    }
    if arg.table_eq_width {
        let max_w = width.iter().max().copied().unwrap_or_default();
        let width_end = width.len() - 1;
        for w in &mut width[1..width_end] {
            *w = max_w;
        }
    }

    for (n, row) in table.iter().enumerate() {
        let row_pos = n as upos_type;

        if n > 0 {
            arg.indent_follow(out);
        }

        // cell 0. before the first |
        if let Some(cell) = row.row.get(0) {
            let col_idx = 0;
            let len = width[col_idx];
            if !cell.txt.trim().is_empty() {
                if row_pos == arg.cursor.y && col_idx == row.cursor_cell {
                    out.cursor = out.txt.len() + 1 + row.cursor_byte_offset;
                }
                _ = write!(out.txt, "| {} ", cell.txt.trim(),);
            }
        }

        // main columns
        for col_idx in 1..width.len() - 1 {
            let len = width[col_idx];

            if row_pos == arg.cursor.y && col_idx == row.cursor_cell {
                out.cursor = out.txt.len() + 1 + row.cursor_byte_offset;
            }

            if let Some(cell) = row.row.get(col_idx) {
                if n == 1 {
                    _ = write!(out.txt, "|{}", "-".repeat(len as usize));
                } else {
                    _ = write!(
                        out.txt,
                        "| {:1$} ",
                        cell.txt.trim(),
                        len.saturating_sub(2) as usize
                    );
                }
            } else {
                _ = write!(out.txt, "|{}", " ".repeat(len as usize));
            }
        }

        // cells after the official last
        for col_idx in width.len() - 1..row.row.len() {
            if let Some(cell) = row.row.get(col_idx) {
                if !cell.txt.trim().is_empty() {
                    if row_pos == arg.cursor.y && col_idx == row.cursor_cell {
                        out.cursor = out.txt.len() + 1 + row.cursor_byte_offset;
                    }
                    if cell.txt.trim().is_empty() {
                        _ = write!(out.txt, "| ",);
                    } else {
                        _ = write!(out.txt, "| {} ", cell.txt.trim());
                    }
                }
            }
        }

        out.txt.push('|');

        out.txt.push_str(arg.newline);
    }
}

fn reformat_footnote<'a>(
    arg: &mut Reformat<'a>,
    it: &mut OffsetIter<'a>,
    footnote: CowStr<'a>,
    out: &mut ReformatOut,
) {
    let footnote_len = str_line_len(footnote.as_ref()) as usize;

    out.txt.push_str("[^");
    out.txt.push_str(footnote.as_ref());
    out.txt.push_str("]: ");

    arg.prefix.push(footnote_len + 5);
    arg.follow
        .push(CowStr::Boxed(" ".repeat(footnote_len + 5).into_boxed_str()));

    let mut md_last = MDFormat::None;
    loop {
        let Some((e, r)) = it.next() else { break };
        match e {
            Event::End(TagEnd::FootnoteDefinition) => {
                break;
            }
            Event::Start(Tag::Paragraph) => {
                insert_empty(arg, md_last, MDFormat::Paragraph, out);

                if md_last != MDFormat::None {
                    arg.indent_follow(out);
                }
                reformat_paragraph(arg, it, out);

                md_last = MDFormat::Paragraph;
            }
            _ => unreachable!("{:?} {:?}", e, r),
        }
    }

    arg.dedent();
}

fn reformat_definition<'a>(arg: &mut Reformat<'a>, it: &mut OffsetIter<'a>, out: &mut ReformatOut) {
    let mut words = Vec::new();
    let mut skip_txt = false;
    let mut md_last = MDFormat::None;

    for (e, r) in it {
        match e {
            Event::End(TagEnd::DefinitionList) => {
                break;
            }

            Event::Start(Tag::Emphasis) | Event::End(TagEnd::Emphasis) => {
                let word = Word::from(&arg.txt[r.start..r.start + 1]);
                words.push(word);
            }
            Event::Start(Tag::Strong)
            | Event::End(TagEnd::Strong)
            | Event::Start(Tag::Strikethrough)
            | Event::End(TagEnd::Strikethrough) => {
                let word = Word::from(&arg.txt[r.start..r.start + 2]);
                words.push(word);
            }

            Event::Start(Tag::Link { .. }) | Event::Start(Tag::Image { .. }) => {
                skip_txt = true;
                let word = Word::from(&arg.txt[r]);
                words.push(word);
            }
            Event::End(TagEnd::Link) | Event::End(TagEnd::Image) => {
                skip_txt = false;
            }

            Event::Code(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_) => {
                let word = Word::from(&arg.txt[r]);
                words.push(word);
            }
            Event::TaskListMarker(_) => {
                let mut word = Word::from(&arg.txt[r]);
                word.whitespace = " ";
                words.push(word);
            }

            Event::SoftBreak => {
                if skip_txt {
                    continue;
                }
                if let Some(mut word) = words.pop() {
                    word.whitespace = " ";
                    words.push(word);
                }
            }
            Event::HardBreak => {
                if skip_txt {
                    continue;
                }
                if md_last != MDFormat::None {
                    arg.indent_follow(out);
                }
                wrap_words(arg, &mut words, NewLine::Hard, out);

                md_last = MDFormat::DefinitionList;
            }

            Event::Start(Tag::DefinitionListTitle) => {}
            Event::End(TagEnd::DefinitionListTitle) => {
                insert_empty(arg, md_last, MDFormat::DefinitionList, out);

                if md_last != MDFormat::None {
                    arg.indent_follow(out);
                }
                wrap_words(arg, &mut words, NewLine::Soft, out);

                md_last = MDFormat::DefinitionList;
            }
            Event::Start(Tag::DefinitionListDefinition) => {}
            Event::End(TagEnd::DefinitionListDefinition) => {
                out.txt.push_str(": ");
                arg.prefix.push(2);
                arg.follow.push(CowStr::Borrowed("  "));

                wrap_words(arg, &mut words, NewLine::Soft, out);

                arg.dedent();

                md_last = MDFormat::DefinitionList;
            }

            Event::Text(_) => {
                if skip_txt {
                    continue;
                }
                for w in WordSeparator::UnicodeBreakProperties.find_words(&arg.txt[r]) {
                    words.push(w);
                }
            }

            _ => unreachable!("{:?} {:?}", e, r),
        }
    }

    assert!(words.is_empty());
}

fn reformat_heading<'a>(
    arg: &mut Reformat<'a>,
    it: &mut OffsetIter<'a>,
    level: HeadingLevel,
    out: &mut ReformatOut,
) {
    for _ in 0..level as usize {
        out.txt.push('#');
    }
    out.txt.push(' ');

    let mut words = Vec::new();
    let mut skip_txt = false;
    for (e, r) in it {
        match e {
            Event::End(TagEnd::Heading(_)) => {
                break;
            }

            Event::Start(Tag::Emphasis) | Event::End(TagEnd::Emphasis) => {
                let word = Word::from(&arg.txt[r.start..r.start + 1]);
                words.push(word);
            }
            Event::Start(Tag::Strong)
            | Event::End(TagEnd::Strong)
            | Event::Start(Tag::Strikethrough)
            | Event::End(TagEnd::Strikethrough) => {
                let word = Word::from(&arg.txt[r.start..r.start + 2]);
                words.push(word);
            }

            Event::Start(Tag::Link { .. }) | Event::Start(Tag::Image { .. }) => {
                skip_txt = true;
                let word = Word::from(&arg.txt[r]);
                words.push(word);
            }
            Event::End(TagEnd::Link) | Event::End(TagEnd::Image) => {
                skip_txt = false;
            }

            Event::Code(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_) => {
                let word = Word::from(&arg.txt[r]);
                words.push(word);
            }
            Event::TaskListMarker(_) => {
                let mut word = Word::from(&arg.txt[r]);
                word.whitespace = " ";
                words.push(word);
            }

            Event::Text(_) => {
                if skip_txt {
                    continue;
                }
                for w in WordSeparator::UnicodeBreakProperties.find_words(&arg.txt[r]) {
                    words.push(w);
                }
            }

            _ => unreachable!("{:?} {:?}", e, r),
        }
    }

    append_wrapped(arg, vec![&words], NewLine::Soft, out);
}

fn reformat_paragraph<'a>(arg: &mut Reformat<'a>, it: &mut OffsetIter<'a>, out: &mut ReformatOut) {
    let mut words = Vec::new();
    let mut skip_txt = false;
    let mut md_last = MDFormat::None;

    let ind = arg.indent_val();

    for (e, r) in it {
        match e {
            Event::End(TagEnd::Paragraph) => {
                break;
            }

            Event::Start(Tag::Emphasis) | Event::End(TagEnd::Emphasis) => {
                let word = Word::from(&arg.txt[r.start..r.start + 1]);
                words.push(word);
            }
            Event::Start(Tag::Strong)
            | Event::End(TagEnd::Strong)
            | Event::Start(Tag::Strikethrough)
            | Event::End(TagEnd::Strikethrough) => {
                let word = Word::from(&arg.txt[r.start..r.start + 2]);
                words.push(word);
            }

            Event::Start(Tag::Link { .. }) | Event::Start(Tag::Image { .. }) => {
                skip_txt = true;
                let word = Word::from(&arg.txt[r]);
                words.push(word);
            }
            Event::End(TagEnd::Link) | Event::End(TagEnd::Image) => {
                skip_txt = false;
            }

            Event::Code(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_) => {
                let word = Word::from(&arg.txt[r]);
                words.push(word);
            }
            Event::TaskListMarker(_) => {
                let mut word = Word::from(&arg.txt[r]);
                word.whitespace = " ";
                words.push(word);
            }

            Event::SoftBreak => {
                if skip_txt {
                    continue;
                }
                if let Some(mut word) = words.pop() {
                    word.whitespace = " ";
                    words.push(word);
                }
            }
            Event::HardBreak => {
                if skip_txt {
                    continue;
                }
                if md_last != MDFormat::None {
                    arg.indent_follow(out);
                }
                wrap_words(arg, &mut words, NewLine::Hard, out);

                md_last = MDFormat::Paragraph;
            }

            Event::Text(_) => {
                if skip_txt {
                    continue;
                }
                for w in WordSeparator::UnicodeBreakProperties.find_words(&arg.txt[r]) {
                    words.push(w);
                }
            }

            _ => unreachable!("{:?} {:?}", e, r),
        }
    }

    if md_last != MDFormat::None {
        arg.indent_follow(out);
    }
    wrap_words(arg, &mut words, NewLine::Soft, out);

    arg.assert_indent(ind);
}

fn insert_empty<'a>(
    arg: &mut Reformat<'a>,
    last: MDFormat,
    current: MDFormat,
    out: &mut ReformatOut,
) -> bool {
    let b = match last {
        MDFormat::None => false,

        MDFormat::BlockQuote
        | MDFormat::Heading
        | MDFormat::Paragraph
        | MDFormat::DefinitionList
        | MDFormat::Table
        | MDFormat::CodeBlock => match current {
            _ => true,
        },
        MDFormat::Item => match current {
            MDFormat::List => false,
            MDFormat::Item => false,
            _ => true,
        },
        MDFormat::List => match current {
            MDFormat::List => false,
            _ => true,
        },
        MDFormat::Footnote => match current {
            MDFormat::Footnote => false,
            _ => true,
        },
        MDFormat::ReferenceDefs => match current {
            MDFormat::ReferenceDefs => false,
            _ => true,
        },
    };

    if b {
        arg.indent_follow(out);
        out.txt.push_str(arg.newline);
        true
    } else {
        false
    }
}

// wrap words into out.
// use prefix+follow.
// cleanup afterwards.
fn wrap_words<'a>(
    arg: &mut Reformat<'a>,
    words: &mut Vec<Word<'a>>,
    newline: NewLine,
    out: &mut ReformatOut,
) {
    let follow_len = arg.follow.iter().map(|v| v.len()).sum::<usize>();
    let widths = [
        arg.txt_width.saturating_sub(arg.total_prefix()), //
        arg.txt_width.saturating_sub(follow_len),
    ];

    let wrapped = WrapAlgorithm::OptimalFit(Penalties::default()).wrap(&words, widths.as_slice());
    // let wrapped = WrapAlgorithm::FirstFit.wrap(&words, widths.as_slice());

    append_wrapped(arg, wrapped, newline, out);

    // words are consumed
    words.clear();
}

#[derive(Debug, PartialEq)]
enum NewLine {
    None,
    Soft,
    Hard,
}

// append words into out.
// use prefix+follow.
// cleanup afterwards.
fn append_wrapped<'a>(
    arg: &mut Reformat<'a>,
    wrapped: Vec<&[Word<'a>]>,
    newline: NewLine,
    out: &mut ReformatOut,
) {
    for n in 0..wrapped.len() {
        if n > 0 {
            for v in arg.follow.iter() {
                out.txt.push_str(v);
            }
        }

        let line = wrapped[n];

        for i in 0..line.len().saturating_sub(1) {
            if let Some(cur) = arg.word_cursor(line[i].word) {
                out.cursor = out.txt.len() + cur;
            }
            out.txt.push_str(line[i].word);
            if let Some(cur) = arg.word_cursor(line[i].whitespace) {
                out.cursor = out.txt.len() + cur;
            }
            out.txt.push_str(line[i].whitespace);
        }
        if line.len() > 0 {
            let last_idx = line.len() - 1;
            if let Some(cur) = arg.word_cursor(line[last_idx].word) {
                out.cursor = out.txt.len() + cur;
            }
            out.txt.push_str(line[last_idx].word);
            if let Some(cur) = arg.word_cursor(line[last_idx].whitespace) {
                out.cursor = out.txt.len();
            }
            out.txt.push_str(line[line.len() - 1].penalty);
        }

        if newline == NewLine::Hard && n == wrapped.len() - 1 {
            out.txt.push_str("  ");
        }
        if newline != NewLine::None {
            out.txt.push_str(arg.newline);
        }
    }
}

pub fn dump_md(txt: &str) {
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

    info!("*** DUMP ***");

    let rdef = p.reference_definitions();
    for (rstr, rdef) in rdef.iter() {
        info!(
            "ReferenceDefinition {:?} {:?} = {:?} {:?}",
            rdef.span,
            rstr,
            rdef.dest.as_ref(),
            rdef.title.as_ref().map(|v| v.as_ref())
        )
    }

    let mut ind = 0;
    for (e, r) in p {
        match e {
            Event::Start(v) => {
                match v {
                    Tag::Paragraph => {
                        info!("{}Paragraph {:?}", " ".repeat(ind), r.clone(),);
                    }
                    Tag::Heading {
                        level,
                        id,
                        classes,
                        attrs,
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
            Event::Text(v) => {
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
}

pub fn parse_md_styles(state: &TextAreaState) -> Vec<(Range<usize>, usize)> {
    let mut styles = Vec::new();

    let txt = state.text();

    let p = Parser::new_ext(
        txt.as_str(),
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
            Event::Start(Tag::BlockQuote(v)) => {
                styles.push((r, MDStyle::BlockQuote as usize));
            }
            Event::Start(Tag::CodeBlock(v)) => {
                styles.push((r, MDStyle::CodeBlock as usize));
            }
            Event::Start(Tag::FootnoteDefinition(v)) => {
                styles.push((r, MDStyle::FootnoteDefinition as usize));
            }
            Event::Start(Tag::Item) => {
                // only color the marker
                let text = state.str_slice_byte(r.clone());
                let item = parse_md_item(r.start, text.as_ref());
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

            Event::Code(v) => {
                styles.push((r, MDStyle::CodeInline as usize));
            }
            Event::InlineMath(v) => {
                styles.push((r, MDStyle::MathInline as usize));
            }
            Event::DisplayMath(v) => {
                styles.push((r, MDStyle::MathDisplay as usize));
            }
            Event::FootnoteReference(v) => {
                styles.push((r, MDStyle::FootnoteReference as usize));
            }
            Event::Rule => {
                styles.push((r, MDStyle::Rule as usize));
            }
            Event::TaskListMarker(v) => {
                styles.push((r, MDStyle::TaskListMarker as usize));
            }
            Event::Html(v) | Event::InlineHtml(v) => {
                styles.push((r, MDStyle::Html as usize));
            }

            Event::End(v) => {}
            Event::Text(v) => {}
            Event::SoftBreak => {}
            Event::HardBreak => {}
        }
    }

    styles
}

/// Length as grapheme count, excluding line breaks.
fn str_line_len(s: &str) -> upos_type {
    let it = s.graphemes(true);
    it.filter(|c| *c != "\n" && *c != "\r\n").count() as upos_type
}

fn is_md_maybe_table(state: &TextAreaState) -> (bool, bool) {
    let mut gr = state.line_graphemes(state.cursor().y);
    let (maybe_table, maybe_header) = if let Some(first) = gr.next() {
        if first == "|" {
            if let Some(second) = gr.next() {
                if second == "-" {
                    (true, true)
                } else {
                    (true, false)
                }
            } else {
                (true, false)
            }
        } else {
            (false, false)
        }
    } else {
        (false, false)
    };
    (maybe_table, maybe_header)
}

fn is_md_table(state: &TextAreaState) -> bool {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;
    state
        .style_match(cursor_byte, MDStyle::Table as usize)
        .is_some()
}

fn md_table(state: &TextAreaState) -> Option<(Range<usize>, TextRange)> {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let row_byte = state.style_match(cursor_byte, MDStyle::Table as usize);

    if let Some(row_byte) = row_byte {
        Some((row_byte.clone(), state.byte_range(row_byte)))
    } else {
        None
    }
}

fn is_md_paragraph(state: &TextAreaState) -> bool {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;
    state
        .style_match(cursor_byte, MDStyle::Paragraph as usize)
        .is_some()
}

fn md_paragraph(state: &TextAreaState) -> Option<(Range<usize>, TextRange)> {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let row_byte = state.style_match(cursor_byte, MDStyle::Paragraph as usize);

    if let Some(row_byte) = row_byte {
        Some((row_byte.clone(), state.byte_range(row_byte)))
    } else {
        None
    }
}

fn is_md_block_quote(state: &TextAreaState) -> bool {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;
    state
        .style_match(cursor_byte, MDStyle::BlockQuote as usize)
        .is_some()
}

fn md_block_quote(state: &TextAreaState) -> Option<(Range<usize>, TextRange)> {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let row_byte = state.style_match(cursor_byte, MDStyle::BlockQuote as usize);

    if let Some(row_byte) = row_byte {
        Some((row_byte.clone(), state.byte_range(row_byte)))
    } else {
        None
    }
}

fn md_item_paragraph(
    state: &TextAreaState,
) -> Option<(Range<usize>, TextRange, Range<usize>, TextRange)> {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let mut sty = Vec::new();
    state.styles_at(cursor_byte, &mut sty);

    let mut r_list = None;
    let mut r_para = None;
    for (r, s) in sty {
        if s == MDStyle::List as usize {
            r_list = Some(r.clone());
        }
        if s == MDStyle::Paragraph as usize {
            r_para = Some(r.clone());
        }
    }

    if let Some(r_list) = r_list {
        if let Some(r_para) = r_para {
            Some((
                r_list.clone(),
                state.byte_range(r_list),
                r_para.clone(),
                state.byte_range(r_para),
            ))
        } else {
            None
        }
    } else {
        None
    }
}

fn is_md_item(state: &TextAreaState) -> bool {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;
    state
        .style_match(cursor_byte, MDStyle::Item as usize)
        .is_some()
}

fn md_prev_item(state: &TextAreaState) -> Option<(Range<usize>, TextRange)> {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let item_byte = state.style_match(cursor_byte, MDStyle::Item as usize);
    let list_byte = state.style_match(cursor_byte, MDStyle::List as usize);

    if let Some(list_byte) = list_byte {
        if let Some(item_byte) = item_byte {
            let mut sty = Vec::new();
            state.styles_in(list_byte.start..item_byte.start, &mut sty);

            let prev = sty.iter().filter(|v| v.1 == MDStyle::Item as usize).last();

            if let Some((prev_bytes, _)) = prev {
                let prev = state.byte_range(prev_bytes.clone());
                Some((prev_bytes.clone(), prev))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn md_next_item(state: &TextAreaState) -> Option<(Range<usize>, TextRange)> {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let item_byte = state.style_match(cursor_byte, MDStyle::Item as usize);
    let list_byte = state.style_match(cursor_byte, MDStyle::List as usize);

    if let Some(list_byte) = list_byte {
        if let Some(item_byte) = item_byte {
            let mut sty = Vec::new();
            state.styles_in(item_byte.end..list_byte.end, &mut sty);

            let next = sty.iter().filter(|v| v.1 == MDStyle::Item as usize).next();

            if let Some((next_bytes, _)) = next {
                let next = state.byte_range(next_bytes.clone());
                Some((next_bytes.clone(), next))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn md_item(state: &TextAreaState) -> Option<(Range<usize>, TextRange)> {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let item_byte = state.style_match(cursor_byte, MDStyle::Item as usize);

    if let Some(item_byte) = item_byte {
        Some((item_byte.clone(), state.byte_range(item_byte)))
    } else {
        None
    }
}

fn prev_tab_md_row(txt: &str, pos: upos_type) -> upos_type {
    let row = parse_md_row(txt, pos);
    if row.cursor_cell > 0 {
        row.row[row.cursor_cell - 1].txt_graphemes.start
    } else {
        pos
    }
}

fn next_tab_md_row(txt: &str, pos: upos_type) -> upos_type {
    let row = parse_md_row(txt, pos);
    if row.cursor_cell + 1 < row.row.len() {
        row.row[row.cursor_cell + 1].txt_graphemes.start
    } else {
        pos
    }
}

// reformat
fn reformat_md_table(
    txt: &str,
    range: TextRange,
    cursor: TextPosition,
    eq_width: bool,
    new_line: &str,
) -> (String, TextPosition) {
    use std::fmt::Write;

    let table_indent = range.start.x;

    let mut table = Vec::new();
    for (row_idx, row) in txt.lines().enumerate() {
        if !row.is_empty() {
            if range.start.y + row_idx as upos_type == cursor.y {
                table.push(parse_md_row(row, cursor.x));
            } else {
                table.push(parse_md_row(row, 0));
            }
        }
    }
    let mut width = Vec::new();
    // only use header widths
    if let Some(row) = table.first() {
        for (idx, cell) in row.row.iter().enumerate() {
            width.push(str_line_len(cell.txt));
        }
    }
    if eq_width {
        let max_w = width.iter().max().copied().unwrap_or_default();
        let width_end = width.len() - 1;
        for w in &mut width[1..width_end] {
            *w = max_w;
        }
    }

    let mut buf = String::new();
    let mut buf_col = 0;
    for (row_idx, row) in table.iter().enumerate() {
        let row_pos = range.start.y + row_idx as upos_type;
        let mut col_pos = 0;

        if row_idx > 0 && table_indent > 0 {
            col_pos += table_indent;
            _ = write!(buf, "{:1$}", " ", table_indent as usize);
        }

        // cell 0. before the first |
        if let Some(cell) = row.row.get(0) {
            let col_idx = 0;
            let len = width[col_idx];
            if !cell.txt.trim().is_empty() {
                if row_pos == cursor.y && col_idx == row.cursor_cell {
                    buf_col = 1 + row.cursor_offset;
                }
                col_pos += str_line_len(cell.txt.trim()) + 3;
                _ = write!(buf, "| {} ", cell.txt.trim(),);
            }
        }

        // main columns
        for col_idx in 1..width.len() - 1 {
            let len = width[col_idx];

            if row_pos == cursor.y && col_idx == row.cursor_cell {
                buf_col = col_pos + 1 + row.cursor_offset;
            }

            if let Some(cell) = row.row.get(col_idx) {
                if row_idx == 1 {
                    col_pos += len + 1;
                    _ = write!(buf, "|{}", "-".repeat(len as usize));
                } else {
                    col_pos += min(len + 1, str_line_len(cell.txt.trim()) + 3);
                    _ = write!(
                        buf,
                        "| {:1$} ",
                        cell.txt.trim(),
                        len.saturating_sub(2) as usize
                    );
                }
            } else {
                col_pos += len + 1;
                _ = write!(buf, "|{}", " ".repeat(len as usize));
            }
        }

        // cells after the official last
        for col_idx in width.len() - 1..row.row.len() - 1 {
            if let Some(cell) = row.row.get(col_idx) {
                if !cell.txt.trim().is_empty() {
                    if row_pos == cursor.y && col_idx == row.cursor_cell {
                        buf_col = col_pos + row.cursor_offset;
                    }
                    col_pos += str_line_len(cell.txt.trim()) + 3;
                    _ = write!(buf, "| {} ", cell.txt.trim(),);
                }
            }
        }

        col_pos += 1;
        buf.push('|');

        // cell after the last
        #[allow(unused_assignments)]
        if let Some(cell) = row.row.get(width.len() - 1) {
            let col_idx = width.len() - 1;
            let len = width[col_idx];
            if !cell.txt.trim().is_empty() {
                if row_pos == cursor.y && col_idx == row.cursor_cell {
                    buf_col = col_pos + row.cursor_offset;
                }
                col_pos += str_line_len(cell.txt.trim()) + 3;
                _ = write!(buf, " {} ", cell.txt.trim(),);
            }
        }

        buf.push_str(new_line);
    }

    (buf, TextPosition::new(buf_col, cursor.y))
}

// create underlines under the header
fn create_md_title(txt: &str, newline: &str) -> (upos_type, String) {
    let row = parse_md_row(txt, 0);

    let mut new_row = String::new();
    new_row.push_str(newline);
    new_row.push_str(row.row[0].txt);
    new_row.push('|');
    for idx in 1..row.row.len() - 1 {
        for g in row.row[idx].txt.graphemes(true) {
            new_row.push('-');
        }
        new_row.push('|');
    }

    let len = str_line_len(&new_row);

    (len, new_row)
}

// add a line break
fn split_md_row(txt: &str, cursor: upos_type, newline: &str) -> (upos_type, String) {
    let row = parse_md_row(txt, 0);

    let mut tmp0 = String::new();
    let mut tmp1 = String::new();
    let mut tmp_pos = 0;
    tmp0.push('|');
    tmp1.push('|');
    for row in &row.row[1..row.row.len() - 1] {
        if row.txt_graphemes.contains(&cursor) {
            tmp_pos = row.txt_graphemes.start + 1;

            let mut pos = row.txt_graphemes.start;
            if cursor > row.txt_graphemes.start {
                tmp1.push(' ');
            }
            for g in row.txt.graphemes(true) {
                if pos < cursor {
                    tmp0.push_str(g);
                } else {
                    tmp1.push_str(g);
                }
                pos += 1;
            }
            pos = row.txt_graphemes.start;
            for g in row.txt.graphemes(true) {
                if pos < cursor {
                    // omit one blank
                    if pos != row.txt_graphemes.start || cursor == row.txt_graphemes.start {
                        tmp1.push(' ');
                    }
                } else {
                    tmp0.push(' ');
                }
                pos += 1;
            }
        } else if row.txt_graphemes.start < cursor {
            tmp0.push_str(row.txt);
            tmp1.push_str(" ".repeat(row.txt_graphemes.len()).as_str());
        } else if row.txt_graphemes.start >= cursor {
            tmp0.push_str(" ".repeat(row.txt_graphemes.len()).as_str());
            tmp1.push_str(row.txt);
        }

        tmp0.push('|');
        tmp1.push('|');
    }
    tmp0.push_str(newline);
    tmp0.push_str(tmp1.as_str());
    tmp0.push_str(newline);

    (tmp_pos, tmp0)
}

// duplicate as empty row
fn empty_md_row(txt: &str, newline: &str) -> (upos_type, String) {
    let row = parse_md_row(txt, 0);
    let mut new_row = String::new();
    new_row.push_str(newline);
    new_row.push('|');
    for idx in 1..row.row.len() - 1 {
        for g in row.row[idx].txt.graphemes(true) {
            new_row.push(' ');
        }
        new_row.push('|');
    }

    let x = if row.row.len() > 1 && row.row[1].txt.len() > 0 {
        str_line_len(row.row[0].txt) + 1 + 1
    } else {
        str_line_len(row.row[0].txt) + 1
    };

    (x, new_row)
}

// parse quoted text
#[derive(Debug)]
struct MDBlockQuote2<'a> {
    quote: &'a str,
    text_prefix: &'a str,
    text_bytes: Range<usize>,
    text: &'a str,
}

fn parse_md_block_quote2(start: usize, txt: &str) -> MDBlockQuote2<'_> {
    let mut quote_byte = 0;
    let mut text_prefix_byte = 0;
    let mut text_byte = 0;

    #[derive(Debug, PartialEq)]
    enum It {
        Leading,
        TextLeading,
        Text,
        NewLine,
    }

    let mut state = It::Leading;
    for (idx, c) in txt.bytes().enumerate() {
        if state == It::Leading {
            if c == b'>' {
                quote_byte = idx;
                state = It::TextLeading;
            } else if c == b' ' || c == b'\t' {
                // ok
            } else {
                text_prefix_byte = idx;
                text_byte = idx;
                state = It::Text;
            }
        } else if state == It::TextLeading {
            if c == b' ' || c == b'\t' {
                // ok
            } else {
                text_byte = idx;
                state = It::Text;
            }
        } else if state == It::Text {
            break;
        }
    }

    MDBlockQuote2 {
        quote: &txt[quote_byte..quote_byte + 1],
        text_prefix: &txt[text_prefix_byte..text_byte],
        text_bytes: start + text_byte..start + txt.len(),
        text: &txt[text_byte..txt.len()],
    }
}

// parse quoted text
#[derive(Debug)]
struct MDBlockQuote {
    text_start_byte: usize,
    text: String,
}

fn parse_md_block_quote(start: usize, txt: &str) -> MDBlockQuote {
    let mut text_start_byte = 0;
    let mut text_line_byte = 0;
    let mut text = Vec::new();

    #[derive(Debug, PartialEq)]
    enum It {
        Leading,
        TextLeading,
        Text,
        NewLine,
    }

    let mut state = It::Leading;
    for (idx, c) in txt.bytes().enumerate() {
        if state == It::Leading {
            if c == b'>' {
                state = It::TextLeading;
            } else if c == b' ' || c == b'\t' {
                // ok
            } else {
                text_line_byte = idx;
                text.push(c);
                state = It::Text;
            }
        } else if state == It::TextLeading {
            if c == b' ' || c == b'\t' {
                // ok
            } else {
                text_line_byte = idx;
                text.push(c);
                state = It::Text;
            }
        } else if state == It::Text {
            if c == b'\n' || c == b'\r' {
                if text_start_byte == 0 {
                    text_start_byte = text_line_byte;
                }
                text.push(b' ');
                state = It::NewLine;
            } else {
                text.push(c);
            }
        } else if state == It::NewLine {
            if c == b'\n' || c == b'\r' {
                // ok
            } else if c == b' ' || c == b'\t' {
                state = It::Leading;
            } else if c == b'>' {
                state = It::TextLeading;
            } else {
                text_line_byte = idx;
                text.push(c);
                state = It::Text;
            }
        }
    }

    MDBlockQuote {
        text_start_byte: start + text_start_byte,
        text: String::from_utf8_lossy(&text).into_owned(),
    }
}

// parse a single list item into marker and text.
#[derive(Debug)]
struct MDItem<'a> {
    prefix: &'a str,
    mark_bytes: Range<usize>,
    mark: &'a str,
    mark_suffix: &'a str,
    mark_nr: Option<usize>,
    text_prefix: &'a str,
    text_bytes: Range<usize>,
    text: &'a str,
}

fn parse_md_item(start: usize, txt: &str) -> MDItem<'_> {
    let mut mark_byte = 0;
    let mut mark_suffix_byte = 0;
    let mut text_prefix_byte = 0;
    let mut text_byte = 0;

    let mut mark_nr = None;

    #[derive(Debug, PartialEq)]
    enum It {
        Leading,
        OrderedMark,
        TextLeading,
    }

    let mut state = It::Leading;
    for (idx, c) in txt.bytes().enumerate() {
        if state == It::Leading {
            if c == b'+' || c == b'-' || c == b'*' {
                mark_byte = idx;
                mark_suffix_byte = idx + 1;
                text_prefix_byte = idx + 1;
                text_byte = idx + 1;
                state = It::TextLeading;
            } else if c.is_ascii_digit() {
                mark_byte = idx;
                state = It::OrderedMark;
            } else if c == b' ' || c == b'\t' {
                // ok
            } else {
                // broken??
                text_prefix_byte = idx;
                text_byte = idx;
                state = It::TextLeading;
            }
        } else if state == It::OrderedMark {
            if c.is_ascii_digit() {
                // ok
            } else if c == b'.' || c == b')' {
                mark_suffix_byte = idx;
                text_prefix_byte = idx + 1;
                text_byte = idx + 1;
                mark_nr = Some(
                    txt[mark_byte..mark_suffix_byte]
                        .parse::<usize>()
                        .expect("nr"),
                );
                state = It::TextLeading;
            } else {
                // broken??
                text_prefix_byte = idx;
                text_byte = idx;
                state = It::TextLeading;
            }
        } else if state == It::TextLeading {
            if c == b' ' || c == b'\t' {
                // ok
            } else {
                text_byte = idx;
                break;
            }
        }
    }

    MDItem {
        prefix: &txt[0..mark_byte],
        mark_bytes: start + mark_byte..start + text_prefix_byte,
        mark: &txt[mark_byte..mark_suffix_byte],
        mark_suffix: &txt[mark_suffix_byte..text_prefix_byte],
        mark_nr,
        text_prefix: &txt[text_prefix_byte..text_byte],
        text_bytes: start + text_byte..start + txt.len(),
        text: &txt[text_byte..],
    }
}

#[derive(Debug)]
struct MDCell<'a> {
    txt: &'a str,
    txt_graphemes: Range<upos_type>,
    txt_bytes: Range<usize>,
}

#[derive(Debug)]
struct MDRow<'a> {
    row: Vec<MDCell<'a>>,
    // cursor cell-nr
    cursor_cell: usize,
    // cursor grapheme offset into the cell
    cursor_offset: upos_type,
    // cursor byte offset into the cell
    cursor_byte_offset: usize,
}

// split single row. translate x-position to cell+cell_offset.
// __info__: returns the string before the first | and the string after the last | too!!
fn parse_md_row(txt: &str, x: upos_type) -> MDRow<'_> {
    let mut tmp = MDRow {
        row: Default::default(),
        cursor_cell: 0,
        cursor_offset: 0,
        cursor_byte_offset: 0,
    };

    let mut grapheme_start = 0;
    let mut grapheme_last = 0;
    let mut esc = false;
    let mut cell_offset = 0;
    let mut cell_byte_start = 0;
    for (idx, (byte_idx, c)) in txt.grapheme_indices(true).enumerate() {
        if idx == x as usize {
            tmp.cursor_cell = tmp.row.len();
            tmp.cursor_offset = cell_offset;
            tmp.cursor_byte_offset = byte_idx - cell_byte_start;
        }

        if c == "\\" {
            cell_offset += 1;
            esc = true;
        } else if c == "|" && !esc {
            cell_offset = 0;
            tmp.row.push(MDCell {
                txt: &txt[cell_byte_start..byte_idx],
                txt_graphemes: grapheme_start..idx as upos_type,
                txt_bytes: cell_byte_start..byte_idx,
            });
            cell_byte_start = byte_idx + 1;
            grapheme_start = idx as upos_type + 1;
        } else {
            cell_offset += 1;
            esc = false;
        }

        grapheme_last = idx as upos_type;
    }

    tmp.row.push(MDCell {
        txt: &txt[cell_byte_start..txt.len()],
        txt_graphemes: grapheme_start..grapheme_last,
        txt_bytes: cell_byte_start..txt.len(),
    });

    tmp
}
