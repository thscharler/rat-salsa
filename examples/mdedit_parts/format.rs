use crate::mdedit_parts::parser::{parse_md_block_quote2, parse_md_item, parse_md_row, MDItem};
use crate::mdedit_parts::str_line_len;
use crate::mdedit_parts::styles::MDStyle;
use log::debug;
use pulldown_cmark::{
    BlockQuoteKind, CodeBlockKind, CowStr, Event, HeadingLevel, OffsetIter, Options, Parser, Tag,
    TagEnd,
};
use rat_widget::event::TextOutcome;
use rat_widget::text::{upos_type, TextPosition, TextRange};
use rat_widget::textarea::TextAreaState;
use std::convert::TryFrom;
use std::ops::Range;
use textwrap::core::Word;
use textwrap::wrap_algorithms::Penalties;
use textwrap::{WordSeparator, WrapAlgorithm};
use unicode_segmentation::UnicodeSegmentation;

pub fn md_format(state: &mut TextAreaState, table_eq_width: bool) -> TextOutcome {
    let cursor = state.cursor();
    let cursor_byte = state.byte_at(cursor).start;

    let selection = if state.selection().is_empty() {
        let mut sty = Vec::new();
        state.styles_at(cursor_byte, &mut sty);

        let first = sty
            .iter()
            .filter(|(_, s)| {
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
            })
            .next();

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
    let selection_byte = state.bytes_at_range(selection);
    debug!("SELECTION {:?}", selection_byte);

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
        table_eq_width,
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

#[derive(Debug)]
struct Reformat<'a> {
    txt: &'a str,
    txt_width: usize,
    cursor: TextPosition,
    cursor_byte: usize,
    newline: &'a str,
    table_eq_width: bool,

    first: Vec<CowStr<'a>>,
    follow: Vec<CowStr<'a>>,

    stack: Vec<Vec<CowStr<'a>>>,
}

impl<'a> Reformat<'a> {
    fn enter_frame(&mut self) {
        self.stack.push(self.follow.clone());
    }

    fn leave_frame(&mut self) {
        let Some(follow) = self.stack.pop() else {
            panic!("no more frame");
        };
        assert!(self.first.is_empty());
        assert_eq!(self.follow, follow);
    }

    // write indent + linebreak
    fn empty_out(&mut self, out: &mut ReformatOut) {
        self.first_out(out);
        out.txt.push_str(self.newline);
    }

    // write first indent
    fn first_out(&mut self, out: &mut ReformatOut) {
        if !self.first.is_empty() {
            for v in self.first.iter() {
                out.txt.push_str(v);
            }
            self.first.clear();
        } else {
            for v in self.follow.iter() {
                out.txt.push_str(v);
            }
        }
    }

    // set line prefix as indent
    fn indent_prefix(&mut self, pos_byte: usize, out: &mut ReformatOut) {
        assert!(self.first.is_empty());
        assert!(self.follow.is_empty());

        let mut gr_it = self.txt[..pos_byte].grapheme_indices(true).rev();
        let mut start_byte = pos_byte;
        loop {
            let Some((idx, gr)) = gr_it.next() else {
                break;
            };
            start_byte = idx;
            if gr == "\n" || gr == "\r\n" {
                start_byte = idx + gr.len();
                break;
            }
        }

        self.first
            .push(CowStr::Borrowed(&self.txt[start_byte..pos_byte]));
        self.follow
            .push(CowStr::Borrowed(&self.txt[start_byte..pos_byte]));
    }

    // add indent
    fn indent(&mut self, first: CowStr<'a>, follow: CowStr<'a>) {
        if self.first.is_empty() {
            self.first.extend_from_slice(&self.follow);
            self.first.push(first);
        } else {
            self.first.push(first);
        }
        self.follow.push(follow);
    }

    // remove indent
    fn dedent(&mut self) {
        assert!(self.first.is_empty());
        self.follow.pop();
    }

    fn first_len(&self) -> usize {
        if self.first.is_empty() {
            self.follow.iter().map(|v| v.len()).sum()
        } else {
            self.first.iter().map(|v| v.len()).sum()
        }
    }

    fn follow_len(&self) -> usize {
        self.follow.iter().map(|v| v.len()).sum()
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

#[derive(Debug)]
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

    let mut arg = Reformat {
        txt,
        txt_width,
        cursor,
        cursor_byte,
        newline,
        table_eq_width,
        first: Vec::new(),
        follow: Vec::new(),
        stack: Vec::new(),
    };
    let mut out = ReformatOut {
        txt: String::new(),
        cursor: 0,
    };

    let mut last_r = 0..0;
    loop {
        let Some((e, r)) = it.next() else {
            break;
        };

        // copy the gaps.
        if last_r.end != r.start {
            out.txt.push_str(&arg.txt[last_r.end..r.start]);
        }

        match e {
            Event::Start(Tag::Paragraph) => {
                arg.indent_prefix(r.start, &mut out);
                reformat_paragraph(&mut arg, &mut it, &mut out);
                arg.dedent();
            }
            Event::Start(Tag::Heading { level, .. }) => {
                arg.indent_prefix(r.start, &mut out);
                reformat_heading(&mut arg, &mut it, level, &mut out);
                arg.dedent();
            }
            Event::Start(Tag::BlockQuote(kind)) => {
                arg.indent_prefix(r.start, &mut out);
                reformat_blockquote(&mut arg, &mut it, r.clone(), kind, &mut out);
                arg.dedent();
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                arg.indent_prefix(r.start, &mut out);
                reformat_codeblock(&mut arg, &mut it, r.clone(), kind, &mut out);
                arg.dedent();
            }
            Event::Start(Tag::List(v)) => {
                let mut out_extended = false;
                arg.indent_prefix(r.start, &mut out);
                reformat_list(&mut arg, &mut it, r.clone(), &mut out, &mut out_extended);
                if !out_extended {
                    arg.empty_out(&mut out);
                }
                arg.dedent();
            }
            Event::Start(Tag::Item) => {
                unreachable!("list {:?} {:?}", e, r.clone());
            }
            Event::Start(Tag::HtmlBlock) => {}

            Event::Start(Tag::FootnoteDefinition(v)) => {
                arg.indent_prefix(r.start, &mut out);
                reformat_footnote(&mut arg, &mut it, v, &mut out);
                arg.dedent();
            }

            Event::Start(Tag::DefinitionList) => {
                arg.indent_prefix(r.start, &mut out);
                reformat_definition(&mut arg, &mut it, &mut out);
                arg.dedent();
            }
            Event::Start(Tag::DefinitionListTitle)
            | Event::Start(Tag::DefinitionListDefinition) => {
                unreachable!("def-list {:?} {:?}", e, r.clone());
            }

            Event::Start(Tag::Table(_)) => {
                arg.indent_prefix(r.start, &mut out);
                reformat_table(&mut arg, &mut it, r.clone(), &mut out);
                arg.dedent();
            }
            Event::Start(Tag::TableHead)
            | Event::Start(Tag::TableRow)
            | Event::Start(Tag::TableCell) => {
                unreachable!("table {:?} {:?}", e, r.clone());
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
                unreachable!("inline {:?} {:?}", e, r.clone());
            }

            Event::Start(Tag::MetadataBlock(_)) => {}
            Event::End(_) => {}
            Event::Html(_) => {}
            Event::Rule => {}
            Event::TaskListMarker(_) => {}
        }

        last_r = r;
    }
    for (link_name, linkdef) in it.reference_definitions().iter() {
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
    // copy the gaps.
    if last_r.end != txt.len() {
        out.txt.push_str(&arg.txt[last_r.end..txt.len()]);
    }
    (out.txt, out.cursor)
}

fn reformat_list<'a>(
    arg: &mut Reformat<'a>,
    it: &mut OffsetIter<'a>,
    range: Range<usize>,
    out: &mut ReformatOut,
    out_extended: &mut bool,
) {
    arg.enter_frame();

    // list prefix
    let first_item = parse_md_item(range.start, &arg.txt[range]);

    let mut nr = first_item.mark_nr;
    loop {
        let Some((e, r)) = it.next() else {
            break;
        };

        match e {
            Event::End(TagEnd::List(_)) => {
                // if !*out_extended {
                //     arg.empty_out(out);
                // }
                break;
            }
            Event::Start(Tag::Item) => {
                let mut item = parse_md_item(r.start, &arg.txt[r]);
                item.mark_nr = nr;

                arg.indent(
                    CowStr::Borrowed(first_item.prefix),
                    CowStr::Borrowed(first_item.prefix),
                );

                reformat_list_item(arg, it, &item, out, out_extended);

                arg.dedent();

                nr = nr.map(|v| v + 1);
            }
            _ => unreachable!("{:?} {:?}", e, r),
        }
    }

    arg.leave_frame();
}

fn reformat_list_item<'a>(
    arg: &mut Reformat<'a>,
    it: &mut OffsetIter<'a>,
    item: &MDItem<'a>,
    out: &mut ReformatOut,
    out_extended: &mut bool,
) {
    arg.enter_frame();

    if let Some(nr) = item.mark_nr {
        let len = (nr.ilog10() + 1) as usize + 2;
        arg.indent(
            CowStr::Boxed(format!("{}{} ", nr, item.mark_suffix).into_boxed_str()),
            CowStr::Boxed(" ".repeat(len).into_boxed_str()),
        );
    } else {
        arg.indent(
            CowStr::Boxed(format!("{} ", item.mark).into_boxed_str()),
            CowStr::Borrowed("  "),
        );
    }

    let mut words = Vec::new();
    let mut skip_txt = false;
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
                wrap_words(arg, &mut words, NewLine::Hard, out);
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
                if !words.is_empty() {
                    wrap_words(arg, &mut words, NewLine::Soft, out);
                }
                reformat_paragraph(arg, it, out);
                arg.empty_out(out);
                *out_extended = true;
            }
            Event::Start(Tag::Heading { level, .. }) => {
                if !words.is_empty() {
                    wrap_words(arg, &mut words, NewLine::Soft, out);
                }
                reformat_heading(arg, it, level, out);
                arg.empty_out(out);
                *out_extended = true;
            }
            Event::Start(Tag::BlockQuote(kind)) => {
                if !words.is_empty() {
                    wrap_words(arg, &mut words, NewLine::Soft, out);
                }
                reformat_blockquote(arg, it, r, kind, out);
                arg.empty_out(out);
                *out_extended = true;
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                if !words.is_empty() {
                    wrap_words(arg, &mut words, NewLine::Soft, out);
                }
                reformat_codeblock(arg, it, r, kind, out);
                arg.empty_out(out);
                *out_extended = true;
            }
            Event::Start(Tag::List(_)) => {
                if !words.is_empty() {
                    wrap_words(arg, &mut words, NewLine::Soft, out);
                }
                reformat_list(arg, it, r, out, out_extended);
            }
            Event::Start(Tag::FootnoteDefinition(v)) => {
                if !words.is_empty() {
                    wrap_words(arg, &mut words, NewLine::Soft, out);
                }
                reformat_footnote(arg, it, v, out);
                arg.empty_out(out);
                *out_extended = true;
            }
            Event::Start(Tag::DefinitionList) => {
                if !words.is_empty() {
                    wrap_words(arg, &mut words, NewLine::Soft, out);
                }
                reformat_definition(arg, it, out);
                arg.empty_out(out);
                *out_extended = true;
            }
            Event::Start(Tag::Table(_)) => {
                if !words.is_empty() {
                    wrap_words(arg, &mut words, NewLine::Soft, out);
                }
                arg.empty_out(out);
                *out_extended = true;
            }

            _ => unreachable!("{:?} {:?}", e, r),
        }
    }

    if !words.is_empty() {
        wrap_words(arg, &mut words, NewLine::Soft, out);
        *out_extended = false;
    }

    arg.dedent();

    arg.leave_frame();
}

fn reformat_codeblock<'a>(
    arg: &mut Reformat<'a>,
    it: &mut OffsetIter<'a>,
    range: Range<usize>,
    kind: CodeBlockKind<'a>,
    out: &mut ReformatOut,
) {
    arg.enter_frame();

    let fence = if matches!(kind, CodeBlockKind::Fenced(_)) {
        &arg.txt[range.start..range.start + 3]
    } else {
        ""
    };

    match &kind {
        CodeBlockKind::Indented => {}
        CodeBlockKind::Fenced(lang) => {
            arg.first_out(out);
            out.txt.push_str(fence);
            out.txt.push_str(lang);
            out.txt.push_str(arg.newline);
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
            }
            _ => unreachable!("{:?} {:?}", e, r),
        }
    }

    if buf.len() > 0 {
        for line in buf.lines() {
            arg.first_out(out);
            out.txt.push_str(line);
            out.txt.push_str(arg.newline);
        }

        match &kind {
            CodeBlockKind::Indented => {}
            CodeBlockKind::Fenced(lang) => {
                arg.first_out(out);
                out.txt.push_str(fence);
                // out.txt.push_str(arg.newline);
            }
        }
    }

    arg.leave_frame();
}

fn reformat_blockquote<'a>(
    arg: &mut Reformat<'a>,
    it: &mut OffsetIter<'a>,
    range: Range<usize>,
    kind: Option<BlockQuoteKind>,
    out: &mut ReformatOut,
) {
    arg.enter_frame();

    if let Some(kind) = kind {
        arg.first_out(out);
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

    arg.indent(
        CowStr::Borrowed(block_quote.quote),
        CowStr::Borrowed(block_quote.quote),
    );
    arg.indent(
        CowStr::Borrowed(block_quote.text_prefix),
        CowStr::Borrowed(block_quote.text_prefix),
    );

    loop {
        let Some((e, r)) = it.next() else { break };
        match e {
            Event::End(TagEnd::BlockQuote(_)) => {
                break;
            }
            Event::Start(Tag::Paragraph) => {
                reformat_paragraph(arg, it, out);
            }
            Event::Start(Tag::BlockQuote(kind)) => {
                reformat_blockquote(arg, it, r, kind, out);
            }
            _ => unreachable!("{:?} {:?}", e, r),
        }
    }

    arg.dedent();
    arg.dedent();

    arg.leave_frame();
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

        arg.first_out(out);

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
    arg.enter_frame();

    let footnote_len = str_line_len(footnote.as_ref()) as usize;

    arg.indent(
        CowStr::Boxed(format!("[^{}]: ", footnote.as_ref()).into_boxed_str()),
        CowStr::Boxed(" ".repeat(footnote_len + 5).into_boxed_str()),
    );

    loop {
        let Some((e, r)) = it.next() else {
            break;
        };

        match e {
            Event::End(TagEnd::FootnoteDefinition) => {
                break;
            }
            Event::Start(Tag::Paragraph) => {
                reformat_paragraph(arg, it, out);
                arg.empty_out(out);
            }
            _ => unreachable!("{:?} {:?}", e, r),
        }
    }

    arg.dedent();

    arg.leave_frame();
}

fn reformat_definition<'a>(arg: &mut Reformat<'a>, it: &mut OffsetIter<'a>, out: &mut ReformatOut) {
    let mut words = Vec::new();
    let mut skip_txt = false;

    loop {
        let Some((e, r)) = it.next() else {
            break;
        };

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
                wrap_words(arg, &mut words, NewLine::Hard, out);
            }

            Event::Start(Tag::DefinitionListTitle) => {}
            Event::End(TagEnd::DefinitionListTitle) => {
                if !words.is_empty() {
                    wrap_words(arg, &mut words, NewLine::Hard, out);
                }
            }
            Event::Start(Tag::DefinitionListDefinition) => {
                arg.indent(CowStr::Borrowed(": "), CowStr::Borrowed("  "));
            }
            Event::End(TagEnd::DefinitionListDefinition) => {
                if !words.is_empty() {
                    wrap_words(arg, &mut words, NewLine::Hard, out);
                }

                arg.dedent();
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
    arg.enter_frame();

    for _ in 0..level as usize {
        arg.first.push(CowStr::Borrowed("#"));
    }
    arg.first.push(CowStr::Borrowed(" "));

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

    arg.leave_frame();
}

fn reformat_paragraph<'a>(arg: &mut Reformat<'a>, it: &mut OffsetIter<'a>, out: &mut ReformatOut) {
    let mut words = Vec::new();
    let mut skip_txt = false;

    arg.enter_frame();

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
                wrap_words(arg, &mut words, NewLine::Hard, out);
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

    wrap_words(arg, &mut words, NewLine::Soft, out);

    arg.leave_frame();
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
    let widths = [
        arg.txt_width.saturating_sub(arg.first_len()), //
        arg.txt_width.saturating_sub(arg.follow_len()),
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
        arg.first_out(out);

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
