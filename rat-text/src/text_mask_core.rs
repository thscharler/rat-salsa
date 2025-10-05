use crate::clipboard::{Clipboard, global_clipboard};
use crate::core::{TextCore, TextString};
#[allow(deprecated)]
use crate::glyph::{Glyph, GlyphIter};
use crate::glyph2::{Glyph2, GlyphIter2, TextWrap2};
use crate::grapheme::StrGraphemes;
use crate::text_mask_core::mask::{EditDirection, Mask, MaskToken};
use crate::text_store::SkipLine;
use crate::undo_buffer::{UndoBuffer, UndoEntry, UndoVec};
use crate::{Cursor, Grapheme, TextError, TextPosition, TextRange, upos_type};
use format_num_pattern::core::{clean_num, map_num};
use format_num_pattern::{CurrencySym, NumberFormat, NumberSymbols};
use std::borrow::Cow;
use std::iter::once;
use std::ops::Range;
use std::{fmt, slice};
use unicode_segmentation::UnicodeSegmentation;

/// Text editing core for MaskedInput.
#[derive(Debug, Clone)]
pub struct MaskedCore {
    // text
    masked: TextCore<TextString>,
    // number symbols
    sym: Option<NumberSymbols>,
    // parsed mask
    mask: Vec<MaskToken>,
}

impl Default for MaskedCore {
    fn default() -> Self {
        let mut value = TextCore::new(
            Some(Box::new(UndoVec::new(99))),
            Some(Box::new(global_clipboard())),
        );
        value.set_glyph_line_break(false);

        Self {
            masked: value,
            sym: None,
            mask: Default::default(),
        }
    }
}

impl MaskedCore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the decimal separator and other symbols.
    /// Only used for rendering and to map user input.
    /// The value itself uses "."
    pub fn set_num_symbols(&mut self, sym: NumberSymbols) {
        self.sym = Some(sym);
    }

    fn dec_sep(&self) -> char {
        if let Some(sym) = &self.sym {
            sym.decimal_sep
        } else {
            '.'
        }
    }

    fn grp_sep(&self) -> char {
        if let Some(sym) = &self.sym {
            // fallback for empty grp-char.
            // it would be really ugly, if we couldn't keep
            //   mask-idx == grapheme-idx
            sym.decimal_grp.unwrap_or(' ')
        } else {
            ','
        }
    }

    fn neg_sym(&self) -> char {
        if let Some(sym) = &self.sym {
            sym.negative_sym
        } else {
            '-'
        }
    }

    fn pos_sym(&self) -> char {
        if let Some(sym) = &self.sym {
            sym.positive_sym
        } else {
            ' '
        }
    }

    /// Changes the mask.
    /// Resets the value to a default.
    pub fn set_mask<S: AsRef<str>>(&mut self, s: S) -> Result<(), fmt::Error> {
        self.mask = Self::parse_mask(s.as_ref())?;
        self.clear();
        Ok(())
    }

    #[allow(clippy::needless_range_loop)]
    fn parse_mask(mask_str: &str) -> Result<Vec<MaskToken>, fmt::Error> {
        let mut out = Vec::<MaskToken>::new();

        let mut start_sub = 0;
        let mut start_sec = 0;
        let mut sec_id = 0;
        let mut last_mask = Mask::None;
        let mut dec_dir = EditDirection::Rtol;
        let mut esc = false;
        let mut idx = 0;
        for m in mask_str.graphemes(true).chain(once("")) {
            let mask = if esc {
                esc = false;
                Mask::Separator(Box::from(m))
            } else {
                match m {
                    "0" => Mask::Digit0(dec_dir),
                    "9" => Mask::Digit(dec_dir),
                    "#" => Mask::Numeric(dec_dir),
                    "." => Mask::DecimalSep,
                    "," => Mask::GroupingSep,
                    "-" => Mask::Sign,
                    "+" => Mask::Plus,
                    "h" => Mask::Hex,
                    "H" => Mask::Hex0,
                    "o" => Mask::Oct,
                    "O" => Mask::Oct0,
                    "d" => Mask::Dec,
                    "D" => Mask::Dec0,
                    "l" => Mask::Letter,
                    "a" => Mask::LetterOrDigit,
                    "c" => Mask::LetterDigitSpace,
                    "_" => Mask::AnyChar,
                    "" => Mask::None,
                    " " => Mask::Separator(Box::from(m)),
                    "\\" => {
                        esc = true;
                        continue;
                    }
                    _ => return Err(fmt::Error),
                }
            };

            match mask {
                Mask::Digit0(_)
                | Mask::Digit(_)
                | Mask::Numeric(_)
                | Mask::GroupingSep
                | Mask::Sign
                | Mask::Plus => {
                    // no change
                }
                Mask::DecimalSep => {
                    dec_dir = EditDirection::Ltor;
                }
                Mask::Hex0
                | Mask::Hex
                | Mask::Oct0
                | Mask::Oct
                | Mask::Dec0
                | Mask::Dec
                | Mask::Letter
                | Mask::LetterOrDigit
                | Mask::LetterDigitSpace
                | Mask::AnyChar
                | Mask::Separator(_) => {
                    // reset to default number input direction
                    dec_dir = EditDirection::Rtol
                }
                Mask::None => {
                    // no change, doesn't matter
                }
            }

            if matches!(mask, Mask::Separator(_)) || mask.section() != last_mask.section() {
                for j in start_sec..idx {
                    out[j].sec_id = sec_id;
                    out[j].sec_start = start_sec as upos_type;
                    out[j].sec_end = idx as upos_type;
                }
                sec_id += 1;
                start_sec = idx;
            }
            if matches!(mask, Mask::Separator(_)) || mask.sub_section() != last_mask.sub_section() {
                for j in start_sub..idx {
                    out[j].sub_start = start_sub as upos_type;
                    out[j].sub_end = idx as upos_type;
                }
                start_sub = idx;
            }

            let tok = MaskToken {
                sec_id: 0,
                sec_start: 0,
                sec_end: 0,
                sub_start: 0,
                sub_end: 0,
                peek_left: last_mask,
                right: mask.clone(),
                edit: mask.edit_value().into(),
            };
            out.push(tok);

            idx += 1;
            last_mask = mask;
        }
        for j in start_sec..out.len() {
            out[j].sec_id = sec_id;
            out[j].sec_start = start_sec as upos_type;
            out[j].sec_end = mask_str.graphemes(true).count() as upos_type;
        }
        for j in start_sub..out.len() {
            out[j].sub_start = start_sub as upos_type;
            out[j].sub_end = mask_str.graphemes(true).count() as upos_type;
        }

        Ok(out)
    }

    /// Return the mask.
    pub fn mask(&self) -> String {
        use std::fmt::Write;

        let mut buf = String::new();
        for t in self.mask.iter() {
            _ = write!(buf, "{}", t.right);
        }
        buf
    }
}

impl MaskedCore {
    /// Clipboard
    pub fn set_clipboard(&mut self, clip: Option<Box<dyn Clipboard + 'static>>) {
        self.masked.set_clipboard(clip);
    }

    /// Clipboard
    pub fn clipboard(&self) -> Option<&dyn Clipboard> {
        self.masked.clipboard()
    }
}

impl MaskedCore {
    /// Undo
    #[inline]
    pub fn set_undo_buffer(&mut self, undo: Option<Box<dyn UndoBuffer>>) {
        self.masked.set_undo_buffer(undo);
    }

    /// Set undo count
    #[inline]
    pub fn set_undo_count(&mut self, n: u32) {
        self.masked.set_undo_count(n);
    }

    /// Begin a sequence of changes that should be undone in one go.
    #[inline]
    pub fn begin_undo_seq(&mut self) {
        self.masked.begin_undo_seq();
    }

    /// End a sequence of changes that should be undone in one go.
    #[inline]
    pub fn end_undo_seq(&mut self) {
        self.masked.end_undo_seq();
    }

    /// Undo
    #[inline]
    pub fn undo_buffer(&self) -> Option<&dyn UndoBuffer> {
        self.masked.undo_buffer()
    }

    /// Undo
    #[inline]
    pub fn undo_buffer_mut(&mut self) -> Option<&mut dyn UndoBuffer> {
        self.masked.undo_buffer_mut()
    }

    /// Undo last.
    pub fn undo(&mut self) -> bool {
        self.masked.undo()
    }

    /// Redo last.
    pub fn redo(&mut self) -> bool {
        self.masked.redo()
    }

    /// Get last replay recording.
    pub fn recent_replay_log(&mut self) -> Vec<UndoEntry> {
        self.masked.recent_replay_log()
    }

    /// Replay a recording of changes.
    pub fn replay_log(&mut self, replay: &[UndoEntry]) {
        self.masked.replay_log(replay)
    }
}

impl MaskedCore {
    /// Set all styles.
    ///
    /// The ranges are byte-ranges. The usize value is the index of the
    /// actual style. Those are set with the widget.
    #[inline]
    pub fn set_styles(&mut self, new_styles: Vec<(Range<usize>, usize)>) {
        self.masked.set_styles(new_styles);
    }

    /// Add a style for the given byte-range.
    ///
    /// The usize value is the index of the actual style.
    /// Those are set at the widget.
    #[inline]
    pub fn add_style(&mut self, range: Range<usize>, style: usize) {
        self.masked.add_style(range, style);
    }

    /// Remove a style for the given byte-range.
    ///
    /// Range and style must match to be removed.
    #[inline]
    pub fn remove_style(&mut self, range: Range<usize>, style: usize) {
        self.masked.remove_style(range, style);
    }

    /// Find all values for the given position.
    ///
    /// Creates a cache for the styles in range.
    #[inline]
    pub(crate) fn styles_at_page(&self, pos: usize, range: Range<usize>, buf: &mut Vec<usize>) {
        self.masked.styles_at_page(pos, range, buf);
    }

    /// Find all styles that touch the given range.
    pub fn styles_in(&self, range: Range<usize>, buf: &mut Vec<(Range<usize>, usize)>) {
        self.masked.styles_in(range, buf)
    }

    /// Finds all styles for the given position.
    #[inline]
    pub fn styles_at(&self, byte_pos: usize, buf: &mut Vec<(Range<usize>, usize)>) {
        self.masked.styles_at(byte_pos, buf);
    }

    /// Check if the given style applies at the position and
    /// return the complete range for the style.
    #[inline]
    pub fn style_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
        self.masked.style_match(byte_pos, style)
    }

    /// List of all styles.
    #[inline]
    pub fn styles(&self) -> Option<impl Iterator<Item = (Range<usize>, usize)> + '_> {
        self.masked.styles()
    }
}

impl MaskedCore {
    /// Set the cursor position.
    /// The value is capped to the number of text lines and
    /// the line-width for the given line.
    ///
    /// Returns true, if the cursor actually changed.
    pub fn set_cursor(&mut self, cursor: upos_type, extend_selection: bool) -> bool {
        self.masked
            .set_cursor(TextPosition::new(cursor, 0), extend_selection)
    }

    // find default cursor position for a number range
    fn number_cursor(&self, range: Range<upos_type>) -> upos_type {
        for (i, t) in self.mask[range.start as usize..range.end as usize]
            .iter()
            .enumerate()
            .rev()
        {
            match t.right {
                Mask::Digit(EditDirection::Rtol)
                | Mask::Digit0(EditDirection::Rtol)
                | Mask::Numeric(EditDirection::Rtol) => {
                    return range.start + i as upos_type + 1;
                }
                _ => {}
            }
        }
        range.start
    }

    /// Get the default cursor for the section at the given cursor position,
    /// if it is an editable section.
    pub fn section_cursor(&self, cursor: upos_type) -> Option<upos_type> {
        if cursor as usize >= self.mask.len() {
            return None;
        }

        let mask = &self.mask[cursor as usize];

        if mask.right.is_number() {
            Some(self.number_cursor(mask.sec_start..mask.sec_end))
        } else if mask.right.is_separator() {
            None
        } else if mask.right.is_none() {
            None
        } else {
            Some(mask.sec_start)
        }
    }

    /// Get the default cursor position for the next editable section.
    pub fn next_section_cursor(&self, cursor: upos_type) -> Option<upos_type> {
        if cursor as usize >= self.mask.len() {
            return None;
        }

        let mut mask = &self.mask[cursor as usize];
        let mut next;
        loop {
            if mask.right.is_none() {
                return None;
            }

            next = mask.sec_end;
            mask = &self.mask[next as usize];

            if mask.right.is_number() {
                return Some(self.number_cursor(mask.sec_start..mask.sec_end));
            } else if mask.right.is_separator() {
                continue;
            } else if mask.right.is_none() {
                return None;
            } else {
                return Some(mask.sec_start);
            }
        }
    }

    /// Get the default cursor position for the next editable section.
    pub fn prev_section_cursor(&self, cursor: upos_type) -> Option<upos_type> {
        if cursor as usize >= self.mask.len() {
            return None;
        }

        let mut prev = self.mask[cursor as usize].sec_start;
        let mut mask = &self.mask[prev as usize];

        loop {
            if mask.peek_left.is_none() {
                return None;
            }

            prev = self.mask[mask.sec_start as usize - 1].sec_start;
            mask = &self.mask[prev as usize];

            if mask.right.is_number() {
                return Some(self.number_cursor(mask.sec_start..mask.sec_end));
            } else if mask.right.is_separator() {
                continue;
            } else {
                return Some(mask.sec_start);
            }
        }
    }

    /// Is the position at a word boundary?
    pub fn is_section_boundary(&self, pos: upos_type) -> bool {
        if pos == 0 {
            return false;
        }
        if pos as usize >= self.mask.len() {
            return false;
        }
        let prev = &self.mask[pos as usize - 1];
        let mask = &self.mask[pos as usize];
        prev.sec_id != mask.sec_id
    }

    /// Get the range for the section at the given cursor position,
    /// if it is an editable section.
    pub fn section_range(&self, cursor: upos_type) -> Option<Range<upos_type>> {
        if cursor as usize >= self.mask.len() {
            return None;
        }

        let mask = &self.mask[cursor as usize];
        if mask.right.is_number() {
            Some(mask.sec_start..mask.sec_end)
        } else if mask.right.is_separator() {
            None
        } else if mask.right.is_none() {
            None
        } else {
            Some(mask.sec_start..mask.sec_end)
        }
    }

    /// Get the default cursor position for the next editable section.
    pub fn next_section_range(&self, cursor: upos_type) -> Option<Range<upos_type>> {
        if cursor as usize >= self.mask.len() {
            return None;
        }

        let mut mask = &self.mask[cursor as usize];
        let mut next;
        loop {
            if mask.right.is_none() {
                return None;
            }

            next = mask.sec_end;
            mask = &self.mask[next as usize];

            if mask.right.is_number() {
                return Some(mask.sec_start..mask.sec_end);
            } else if mask.right.is_separator() {
                continue;
            } else if mask.right.is_none() {
                return None;
            } else {
                return Some(mask.sec_start..mask.sec_end);
            }
        }
    }

    /// Get the default cursor position for the next editable section.
    pub fn prev_section_range(&self, cursor: upos_type) -> Option<Range<upos_type>> {
        if cursor as usize >= self.mask.len() {
            return None;
        }

        let mut prev = self.mask[cursor as usize].sec_start;
        let mut mask = &self.mask[prev as usize];
        loop {
            if mask.peek_left.is_none() {
                return None;
            }

            prev = self.mask[mask.sec_start as usize - 1].sec_start;
            mask = &self.mask[prev as usize];

            if mask.right.is_number() {
                return Some(mask.sec_start..mask.sec_end);
            } else if mask.right.is_separator() {
                continue;
            } else {
                return Some(mask.sec_start..mask.sec_end);
            }
        }
    }

    /// Place cursor at decimal separator, if any.
    /// 0 otherwise.
    #[inline]
    pub fn set_default_cursor(&mut self) {
        if let Some(pos) = self.section_cursor(0) {
            self.masked.set_cursor(TextPosition::new(pos, 0), false);
        } else if let Some(pos) = self.next_section_cursor(0) {
            self.masked.set_cursor(TextPosition::new(pos, 0), false);
        } else {
            self.masked.set_cursor(TextPosition::new(0, 0), false);
        }
    }

    /// Cursor position as grapheme-idx.
    #[inline]
    pub fn cursor(&self) -> upos_type {
        self.masked.cursor().x
    }

    /// Selection anchor
    #[inline]
    pub fn anchor(&self) -> upos_type {
        self.masked.anchor().x
    }

    /// Any text selection.
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.masked.has_selection()
    }

    /// Select text.
    #[inline]
    pub fn set_selection(&mut self, anchor: upos_type, cursor: upos_type) -> bool {
        self.masked
            .set_selection(TextPosition::new(anchor, 0), TextPosition::new(cursor, 0))
    }

    /// Select all text.
    #[inline]
    pub fn select_all(&mut self) -> bool {
        self.masked.select_all()
    }

    /// Returns the selection as TextRange.
    #[inline]
    pub fn selection(&self) -> Range<upos_type> {
        let mut v = self.masked.selection();
        if v.start == TextPosition::new(0, 1) {
            v.start = TextPosition::new(self.line_width(), 0);
        }
        if v.end == TextPosition::new(0, 1) {
            v.end = TextPosition::new(self.line_width(), 0);
        }
        v.start.x..v.end.x
    }

    /// Selection.
    #[inline]
    pub fn selected_text(&self) -> &str {
        match self
            .masked
            .str_slice(self.masked.selection())
            .expect("valid_range")
        {
            Cow::Borrowed(v) => v,
            Cow::Owned(_) => {
                unreachable!()
            }
        }
    }
}

impl MaskedCore {
    /// Empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.masked.text().as_str() == self.default_value()
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    #[inline]
    pub fn byte_at(&self, pos: upos_type) -> Result<Range<usize>, TextError> {
        self.masked.byte_at(TextPosition::new(pos, 0))
    }

    /// Grapheme range to byte range.
    #[inline]
    pub fn bytes_at_range(&self, range: Range<upos_type>) -> Result<Range<usize>, TextError> {
        self.masked
            .bytes_at_range(TextRange::new((range.start, 0), (range.end, 0)))
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    #[inline]
    pub fn byte_pos(&self, byte: usize) -> Result<upos_type, TextError> {
        Ok(self.masked.byte_pos(byte)?.x)
    }

    /// Byte range to grapheme range.
    #[inline]
    pub fn byte_range(&self, bytes: Range<usize>) -> Result<Range<upos_type>, TextError> {
        let r = self.masked.byte_range(bytes)?;
        Ok(r.start.x..r.end.x)
    }

    /// Text slice as `Cow<str>`. Uses a byte range.
    #[inline]
    pub fn str_slice_byte(&self, range: Range<usize>) -> Result<Cow<'_, str>, TextError> {
        self.masked.str_slice_byte(range)
    }

    /// A range of the text as `Cow<str>`
    #[inline]
    pub fn str_slice(&self, range: Range<upos_type>) -> Result<Cow<'_, str>, TextError> {
        self.masked
            .str_slice(TextRange::new((range.start, 0), (range.end, 0)))
    }

    /// Iterator for the glyphs of the lines in range.
    /// Glyphs here a grapheme + display length.
    #[inline]
    #[deprecated(since = "1.1.0", note = "discontinued api")]
    #[allow(deprecated)]
    pub fn glyphs(
        &self,
        rows: Range<upos_type>,
        screen_offset: u16,
        screen_width: u16,
    ) -> Result<impl Iterator<Item = Glyph<'_>>, TextError> {
        let grapheme_iter = self.masked.graphemes(
            TextRange::new((0, rows.start), (0, rows.end)),
            TextPosition::new(0, rows.start),
        )?;

        let mask_iter = self.mask.iter();

        let sym_neg = || self.neg_sym().to_string();
        let sym_dec = || self.dec_sep().to_string();
        let sym_grp = || self.grp_sep().to_string();
        let sym_pos = || self.pos_sym().to_string();

        let iter = grapheme_iter
            .zip(mask_iter)
            .map(move |(g, t)| match (&t.right, g.grapheme()) {
                (Mask::Numeric(_), "-") => Grapheme::new(Cow::Owned(sym_neg()), g.text_bytes()),
                (Mask::DecimalSep, ".") => Grapheme::new(Cow::Owned(sym_dec()), g.text_bytes()),
                (Mask::GroupingSep, ",") => Grapheme::new(Cow::Owned(sym_grp()), g.text_bytes()),
                (Mask::GroupingSep, "-") => Grapheme::new(Cow::Owned(sym_neg()), g.text_bytes()),
                (Mask::Sign, "-") => Grapheme::new(Cow::Owned(sym_neg()), g.text_bytes()),
                (Mask::Sign, _) => Grapheme::new(Cow::Owned(sym_pos()), g.text_bytes()),
                (_, _) => g,
            });

        let mut it = GlyphIter::new(TextPosition::new(0, rows.start), iter);
        it.set_screen_offset(screen_offset);
        it.set_screen_width(screen_width);
        it.set_tabs(self.masked.tab_width());
        it.set_show_ctrl(self.masked.glyph_ctrl());
        it.set_line_break(self.masked.glyph_line_break());
        Ok(it)
    }

    /// Iterator for the glyphs of the lines in range.
    /// Glyphs here a grapheme + display length.
    ///
    /// This omits unnecessary white-space.
    #[inline]
    #[deprecated(since = "1.1.0", note = "discontinued api")]
    #[allow(deprecated)]
    pub fn condensed_glyphs(
        &self,
        rows: Range<upos_type>,
        screen_offset: u16,
        screen_width: u16,
    ) -> Result<impl Iterator<Item = Glyph<'_>>, TextError> {
        let grapheme_iter = self.masked.graphemes(
            TextRange::new((0, rows.start), (0, rows.end)),
            TextPosition::new(0, rows.start),
        )?;

        let mask_iter = self.mask.iter();

        let sym_neg = || self.neg_sym().to_string();
        let sym_dec = || self.dec_sep().to_string();
        let sym_grp = || self.grp_sep().to_string();
        let sym_pos = || self.pos_sym().to_string();

        let iter =
            grapheme_iter
                .zip(mask_iter)
                .filter_map(move |(g, t)| match (&t.right, g.grapheme()) {
                    (Mask::Numeric(_), "-") => {
                        Some(Grapheme::new(Cow::Owned(sym_neg()), g.text_bytes()))
                    }
                    (Mask::DecimalSep, ".") => {
                        Some(Grapheme::new(Cow::Owned(sym_dec()), g.text_bytes()))
                    }
                    (Mask::GroupingSep, ",") => {
                        Some(Grapheme::new(Cow::Owned(sym_grp()), g.text_bytes()))
                    }
                    (Mask::GroupingSep, "-") => {
                        Some(Grapheme::new(Cow::Owned(sym_neg()), g.text_bytes()))
                    }
                    (Mask::Sign, "-") => Some(Grapheme::new(Cow::Owned(sym_neg()), g.text_bytes())),

                    (Mask::Numeric(_), " ") => None,
                    (Mask::Digit(_), " ") => None,
                    (Mask::DecimalSep, " ") => None,
                    (Mask::GroupingSep, " ") => None,
                    (Mask::Sign, _) => {
                        if self.pos_sym() != ' ' {
                            Some(Grapheme::new(Cow::Owned(sym_pos()), g.text_bytes()))
                        } else {
                            None
                        }
                    }
                    (Mask::Hex, " ") => None,
                    (Mask::Oct, " ") => None,
                    (Mask::Dec, " ") => None,

                    (_, _) => Some(g),
                });

        let mut it = GlyphIter::new(TextPosition::new(0, rows.start), iter);
        it.set_screen_offset(screen_offset);
        it.set_screen_width(screen_width);
        it.set_tabs(self.masked.tab_width());
        it.set_show_ctrl(self.masked.glyph_ctrl());
        it.set_line_break(self.masked.glyph_line_break());
        Ok(it)
    }

    /// Iterator for the glyphs of the lines in range.
    /// Glyphs here a grapheme + display length.
    ///
    /// * text_break: only TextBreak2::ShiftText has been tested
    /// * condensed: skip unnecessary white-space
    #[inline]
    pub(crate) fn glyphs2(
        &self,
        left_margin: upos_type,
        right_margin: upos_type,
        compact: bool,
    ) -> Result<Box<dyn Iterator<Item = Glyph2<'_>> + '_>, TextError> {
        let grapheme_iter = self
            .masked
            .graphemes(TextRange::new((0, 0), (0, 1)), TextPosition::new(0, 0))?;
        let mask_iter = self.mask.iter();

        let iter = MaskedGraphemes {
            iter_str: grapheme_iter,
            iter_mask: mask_iter,
            compact,
            sym_neg: self.neg_sym().to_string(),
            sym_dec: self.dec_sep().to_string(),
            sym_grp: self.grp_sep().to_string(),
            sym_pos: self.pos_sym().to_string(),
            byte_pos: 0,
        };

        let mut it = GlyphIter2::new(TextPosition::new(0, 0), 0, iter, Default::default());
        it.set_tabs(self.masked.tab_width() as upos_type);
        it.set_show_ctrl(self.masked.glyph_ctrl());
        it.set_lf_breaks(self.masked.glyph_line_break());
        it.set_text_wrap(TextWrap2::Shift);
        it.set_left_margin(left_margin);
        it.set_right_margin(right_margin);
        it.set_word_margin(right_margin);
        it.prepare()?;
        Ok(Box::new(it))
    }

    /// Get the grapheme at the given position.
    #[inline]
    pub fn grapheme_at(&self, pos: upos_type) -> Result<Option<Grapheme<'_>>, TextError> {
        self.masked.grapheme_at(TextPosition::new(pos, 0))
    }

    /// Get a cursor over all the text with the current position set at pos.
    #[inline]
    pub fn text_graphemes(
        &self,
        pos: upos_type,
    ) -> Result<impl Cursor<Item = Grapheme<'_>>, TextError> {
        self.masked.text_graphemes(TextPosition::new(pos, 0))
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn graphemes(
        &self,
        range: Range<upos_type>,
        pos: upos_type,
    ) -> Result<impl Cursor<Item = Grapheme<'_>>, TextError> {
        self.masked.graphemes(
            TextRange::new((range.start, 0), (range.end, 0)),
            TextPosition::new(pos, 0),
        )
    }

    #[inline]
    pub fn line_width(&self) -> upos_type {
        self.masked.line_width(0).expect("valid_row")
    }
}

#[derive(Debug, Clone)]
struct MaskedGraphemes<'a> {
    iter_str: StrGraphemes<'a>,
    iter_mask: slice::Iter<'a, MaskToken>,

    compact: bool,
    sym_neg: String,
    sym_dec: String,
    sym_grp: String,
    sym_pos: String,

    byte_pos: usize,
}

impl<'a> Iterator for MaskedGraphemes<'a> {
    type Item = Grapheme<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let g = self.iter_str.next();
            let t = self.iter_mask.next();

            let (Some(g), Some(t)) = (g, t) else {
                return None;
            };

            self.byte_pos = g.text_bytes().end;

            let r = match (self.compact, &t.right, g.grapheme()) {
                (true, Mask::Numeric(_), "-") => Some(Grapheme::new(
                    Cow::Owned(self.sym_neg.clone()),
                    g.text_bytes(),
                )),
                (true, Mask::DecimalSep, ".") => Some(Grapheme::new(
                    Cow::Owned(self.sym_dec.clone()),
                    g.text_bytes(),
                )),
                (true, Mask::GroupingSep, ",") => Some(Grapheme::new(
                    Cow::Owned(self.sym_grp.clone()),
                    g.text_bytes(),
                )),
                (true, Mask::GroupingSep, "-") => Some(Grapheme::new(
                    Cow::Owned(self.sym_neg.clone()),
                    g.text_bytes(),
                )),
                (true, Mask::Sign, "-") => Some(Grapheme::new(
                    Cow::Owned(self.sym_neg.clone()),
                    g.text_bytes(),
                )),

                (true, Mask::Numeric(_), " ") => None,
                (true, Mask::Digit(_), " ") => None,
                (true, Mask::DecimalSep, " ") => None,
                (true, Mask::GroupingSep, " ") => None,
                (true, Mask::Sign, _) => {
                    if self.sym_pos != " " {
                        Some(Grapheme::new(
                            Cow::Owned(self.sym_pos.clone()),
                            g.text_bytes(),
                        ))
                    } else {
                        None
                    }
                }
                (true, Mask::Hex, " ") => None,
                (true, Mask::Oct, " ") => None,
                (true, Mask::Dec, " ") => None,

                (false, Mask::Numeric(_), "-") => Some(Grapheme::new(
                    Cow::Owned(self.sym_neg.clone()),
                    g.text_bytes(),
                )),
                (false, Mask::DecimalSep, ".") => Some(Grapheme::new(
                    Cow::Owned(self.sym_dec.clone()),
                    g.text_bytes(),
                )),
                (false, Mask::GroupingSep, ",") => Some(Grapheme::new(
                    Cow::Owned(self.sym_grp.clone()),
                    g.text_bytes(),
                )),
                (false, Mask::GroupingSep, "-") => Some(Grapheme::new(
                    Cow::Owned(self.sym_neg.clone()),
                    g.text_bytes(),
                )),
                (false, Mask::Sign, "-") => Some(Grapheme::new(
                    Cow::Owned(self.sym_neg.clone()),
                    g.text_bytes(),
                )),
                (false, Mask::Sign, _) => Some(Grapheme::new(
                    Cow::Owned(self.sym_pos.clone()),
                    g.text_bytes(),
                )),

                (_, _, _) => Some(g),
            };
            if r.is_some() {
                break r;
            }
        }
    }
}

impl<'a> SkipLine for MaskedGraphemes<'a> {
    fn skip_line(&mut self) -> Result<(), TextError> {
        // all in one line, eat the rest.
        for _ in self.iter_str.by_ref() {}
        for _ in self.iter_mask.by_ref() {}
        Ok(())
    }

    fn skip_to(&mut self, byte_pos: usize) -> Result<(), TextError> {
        if byte_pos > self.byte_pos {
            Err(TextError::ByteIndexOutOfBounds(byte_pos, self.byte_pos))
        } else if byte_pos == self.byte_pos {
            Ok(())
        } else {
            for g in self.iter_str.by_ref() {
                _ = self.iter_mask.next();

                if byte_pos == g.text_bytes().end {
                    return Ok(());
                } else if byte_pos < g.text_bytes().end {
                    return Err(TextError::ByteIndexNotCharBoundary(byte_pos));
                }
            }
            Err(TextError::ByteIndexOutOfBounds(byte_pos, byte_pos))
        }
    }
}

impl MaskedCore {
    /// Create a default value according to the mask.
    #[inline]
    fn default_value(&self) -> String {
        MaskToken::empty_section(&self.mask)
    }
}

impl MaskedCore {
    /// Reset value but not the mask and width.
    /// Resets offset and cursor position too.
    #[inline]
    pub fn clear(&mut self) {
        self.masked
            .set_text(TextString::new_string(self.default_value()));
        self.set_default_cursor();
    }

    /// Copy of the text-value.
    pub fn text(&self) -> &str {
        self.masked.text().as_str()
    }

    /// Sets the value.
    /// No checks if the value conforms to the mask.
    /// If the value is too short it will be filled with space.
    /// if the value is too long it will be truncated.
    #[allow(clippy::comparison_chain)]
    pub fn set_text<S: Into<String>>(&mut self, s: S) {
        let mut text = s.into();
        while text.graphemes(true).count() > self.mask.len().saturating_sub(1) {
            text.pop();
        }
        while text.graphemes(true).count() < self.mask.len().saturating_sub(1) {
            text.push(' ');
        }
        let len = text.graphemes(true).count();

        assert_eq!(len, self.mask.len().saturating_sub(1));

        self.masked.set_text(TextString::new_string(text));
    }

    /// Start at the cursor position and find a valid insert position for the input c.
    /// Put the cursor at that position.
    #[allow(clippy::if_same_then_else)]
    pub fn advance_cursor(&mut self, c: char) -> bool {
        if self.mask.is_empty() {
            return false;
        }

        let mask_c = &self.mask[self.masked.cursor().x as usize];

        let mut new_cursor = self.masked.cursor().x;

        loop {
            let mask = &self.mask[new_cursor as usize];

            if self.can_insert_integer_left(mask, new_cursor, c) {
                // At the gap between an integer field and something else.
                // Integer fields are served first.
                break;
            } else if self.can_insert_integer(mask, new_cursor, c) {
                // Insert position inside an integer field. After any spaces
                // and the sign.
                break;
            } else if self.can_insert_sign(mask, new_cursor, c) {
                // Can insert a sign here.
                break;
            } else if self.can_insert_decimal_sep(mask, c) {
                // Decimal separator matches.
                break;
            } else if mask.right == Mask::GroupingSep {
                // Never stop here.
                new_cursor += 1;
            } else if self.can_insert_separator(mask, c) {
                break;
            } else if self.can_move_left_in_fraction(mask_c, mask, new_cursor, c) {
                // skip left
                new_cursor -= 1;
            } else if self.can_insert_fraction(mask_c, mask, c) {
                break;
            } else if self.can_insert_other(mask, c) {
                break;
            } else if mask.right == Mask::None {
                // No better position found. Reset and break;
                new_cursor = self.masked.cursor().x;
                break;
            } else {
                new_cursor += 1;
            }
        }

        self.masked
            .set_cursor(TextPosition::new(new_cursor, 0), false)
    }

    /// Valid input for this mask.
    fn is_valid_char(&self, mask: &Mask, c: char) -> bool {
        match mask {
            Mask::Digit0(_) => c.is_ascii_digit(),
            Mask::Digit(_) => c.is_ascii_digit() || c == ' ',
            Mask::Numeric(_) => c.is_ascii_digit() || c == self.neg_sym() || c == '-',
            Mask::DecimalSep => c == self.dec_sep(),
            Mask::GroupingSep => false,
            Mask::Sign => c == self.neg_sym() || c == '-',
            Mask::Plus => c == self.neg_sym() || c == '-',
            Mask::Hex0 => c.is_ascii_hexdigit(),
            Mask::Hex => c.is_ascii_hexdigit() || c == ' ',
            Mask::Oct0 => c.is_digit(8),
            Mask::Oct => c.is_digit(8) || c == ' ',
            Mask::Dec0 => c.is_ascii_digit(),
            Mask::Dec => c.is_ascii_digit() || c == ' ',
            Mask::Letter => c.is_alphabetic(),
            Mask::LetterOrDigit => c.is_alphanumeric(),
            Mask::LetterDigitSpace => c.is_alphanumeric() || c == ' ',
            Mask::AnyChar => true,
            Mask::Separator(sep) => {
                // ',' and '.' match any separator.
                if c == '.' || c == ',' {
                    true
                } else if let Some(sepc) = sep.chars().next() {
                    // todo: don't know better
                    sepc == c
                } else {
                    false
                }
            }
            Mask::None => false,
        }
    }

    // Can insert other field types
    #[inline]
    fn can_insert_other(&self, mask: &MaskToken, c: char) -> bool {
        match mask.right {
            Mask::Hex0
            | Mask::Hex
            | Mask::Oct0
            | Mask::Oct
            | Mask::Dec0
            | Mask::Dec
            | Mask::Letter
            | Mask::LetterOrDigit
            | Mask::LetterDigitSpace
            | Mask::AnyChar => self.is_valid_char(&mask.right, c),
            _ => false,
        }
    }

    // Can insert fraction.
    #[inline]
    fn can_insert_fraction(&self, mask_c: &MaskToken, mask: &MaskToken, c: char) -> bool {
        if !mask.right.is_fraction() {
            return false;
        }
        if !self.is_valid_char(&mask.right, c) {
            return false;
        }
        // don't jump from integer to fraction
        if mask_c.is_integer_part() {
            return false;
        }

        true
    }

    // When inserting to the fraction we want to left-align
    // the digits. This checks if a digit could possibly be
    // inserted to the left of the current position.
    #[inline]
    fn can_move_left_in_fraction(
        &self,
        mask_c: &MaskToken,
        mask: &MaskToken,
        new_cursor: upos_type,
        c: char,
    ) -> bool {
        if !mask.peek_left.is_fraction() {
            return false;
        }
        if !self.is_valid_char(&mask.peek_left, c) {
            return false;
        }
        // don't jump from integer to fraction
        if mask_c.is_integer_part() {
            return false;
        }

        let gl = self
            .masked
            .grapheme_at(TextPosition::new(new_cursor - 1, 0))
            .expect("valid_position")
            .expect("grapheme");

        // is there space to the left?
        if gl != " " {
            return false;
        }

        true
    }

    // Can input a sign here?
    #[inline]
    fn can_insert_sign<'a>(
        &'a self,
        mut mask: &'a MaskToken,
        new_cursor: upos_type,
        c: char,
    ) -> bool {
        if !self.is_valid_char(&Mask::Sign, c) {
            return false;
        }
        // boundary right/left. prefer right, change mask.
        if mask.peek_left.is_number() && (mask.right.is_ltor() || mask.right.is_none()) {
            mask = &self.mask[new_cursor as usize - 1];
        }
        if !mask.right.is_number() {
            return false;
        }

        // check possible positions for the sign.
        for i in mask.sec_start..mask.sec_end {
            let t = &self.mask[i as usize];
            match t.right {
                Mask::Plus => return true,
                Mask::Sign => return true,
                Mask::Numeric(EditDirection::Rtol) => {
                    // Numeric fields can hold a sign.
                    // If they are not otherwise occupied.
                    let gi = self
                        .masked
                        .grapheme_at(TextPosition::new(i, 0))
                        .expect("valid_position")
                        .expect("grapheme");

                    return t.right.can_drop(gi.grapheme()) || gi == "-";
                }
                _ => {}
            }
        }

        false
    }

    // Is this the correct input position for a rtol field
    #[inline]
    fn can_insert_integer(&self, mask: &MaskToken, new_cursor: upos_type, c: char) -> bool {
        if !mask.right.is_rtol() {
            return false;
        }

        if !self.is_valid_char(&mask.right, c) {
            return false;
        }

        let g = self
            .masked
            .grapheme_at(TextPosition::new(new_cursor, 0))
            .expect("valid_position")
            .expect("grapheme");
        if mask.right.can_drop(g.grapheme()) {
            return false;
        }
        if g == "-" {
            return false;
        }

        true
    }

    // Separator char matches
    #[inline]
    fn can_insert_separator(&self, mask: &MaskToken, c: char) -> bool {
        if !matches!(mask.right, Mask::Separator(_)) {
            return false;
        }
        if !self.is_valid_char(&mask.right, c) {
            return false;
        }
        true
    }

    // Can insert a decimal separator.
    #[inline]
    fn can_insert_decimal_sep(&self, mask: &MaskToken, c: char) -> bool {
        if mask.right != Mask::DecimalSep {
            return false;
        }
        if !self.is_valid_char(&mask.right, c) {
            return false;
        }
        true
    }

    // Can edit the field left of the cursor.
    #[inline]
    fn can_insert_integer_left(&self, mask: &MaskToken, new_cursor: upos_type, c: char) -> bool {
        if !mask.peek_left.is_rtol() {
            return false;
        }
        if !mask.right.is_ltor() && !mask.right.is_none() {
            return false;
        }

        let left = &self.mask[new_cursor as usize - 1];
        if !self.is_valid_char(&left.right, c) {
            return false;
        }

        let mask0 = &self.mask[left.sub_start as usize];
        let g0 = self
            .masked
            .grapheme_at(TextPosition::new(left.sub_start, 0))
            .expect("valid_position")
            .expect("grapheme");
        if !mask0.right.can_drop(g0.grapheme()) {
            return false;
        }

        true
    }

    /// Insert the char if it matches the cursor mask and the current section is not full.
    ///
    /// `advance_cursor()` must be called before for correct functionality.
    ///
    /// Otherwise: your mileage might vary.
    pub fn insert_char(&mut self, c: char) -> bool {
        if self.mask.is_empty() {
            return false;
        }

        let cursor = self.masked.cursor();

        // note: because of borrow checker. calls &mut methods.
        {
            let mask = &self.mask[cursor.x as usize];
            if mask.right.is_number() && self.can_insert_sign(mask, cursor.x, c) {
                if self.insert_sign(c) {
                    return true;
                }
            }
        }
        {
            let mask = &self.mask[cursor.x as usize];
            if mask.peek_left.is_number() && (mask.right.is_ltor() || mask.right.is_none()) {
                let left = &self.mask[cursor.x as usize - 1];
                if self.can_insert_sign(left, cursor.x, c) {
                    if self.insert_sign(c) {
                        return true;
                    }
                }
            }
        }
        {
            let mask = &self.mask[cursor.x as usize];
            if mask.right.is_rtol() {
                if self.insert_rtol(c) {
                    return true;
                }
            }
        }
        {
            let mask = &self.mask[cursor.x as usize];
            if mask.peek_left.is_rtol() && (mask.right.is_ltor() || mask.right.is_none()) {
                if self.insert_rtol(c) {
                    return true;
                }
            }
        }
        {
            let mask = &self.mask[cursor.x as usize];
            if mask.right.is_ltor() {
                if self.insert_ltor(c) {
                    #[allow(clippy::needless_return)]
                    return true;
                }
            }
        }

        false
    }

    /// Insert c into a ltor section.
    fn insert_ltor(&mut self, c: char) -> bool {
        let cursor = self.masked.cursor();

        let mask = &self.mask[cursor.x as usize];
        let mask9 = &self.mask[mask.sub_end as usize - 1];

        // overwrite digit in fraction?
        let g = self
            .masked
            .grapheme_at(cursor)
            .expect("valid_cursor")
            .expect("mask");
        if mask.right.is_fraction()
            && mask.right.can_overwrite_fraction(g.grapheme())
            && self.is_valid_char(&mask.right, c)
        {
            // to the right only defaults
            let frac_mask = &self.mask[cursor.x as usize + 1..mask.sub_end as usize];
            let frac_str = self
                .masked
                .str_slice(TextRange::new((cursor.x + 1, 0), (mask.sub_end, 0)))
                .expect("valid_range");
            if frac_str == MaskToken::empty_section(frac_mask) {
                self.masked.begin_undo_seq();
                self.masked
                    .remove_char_range(TextRange::new(cursor, (cursor.x + 1, 0)))
                    .expect("valid_cursor");
                self.masked.insert_char(cursor, c).expect("valid_cursor");
                self.masked.end_undo_seq();
                return true;
            }
        }

        let g = self
            .masked
            .grapheme_at(cursor)
            .expect("valid_cursor")
            .expect("mask");
        if mask.right.can_overwrite(g.grapheme()) && self.is_valid_char(&mask.right, c) {
            if mask.right.is_separator() {
                self.masked.begin_undo_seq();
                let r = if let Some(next) = self.next_section_cursor(cursor.x) {
                    self.masked.set_cursor(TextPosition::new(next, 0), false)
                } else {
                    self.masked
                        .set_cursor(TextPosition::new(self.line_width(), 0), false)
                };
                self.masked.end_undo_seq();
                return r;
            } else if mask.right == Mask::DecimalSep {
                self.masked.begin_undo_seq();
                self.masked
                    .set_cursor(TextPosition::new(cursor.x + 1, 0), false);
                self.masked.end_undo_seq();
                return true;
            } else {
                self.masked.begin_undo_seq();
                self.masked
                    .remove_char_range(TextRange::new(cursor, (cursor.x + 1, 0)))
                    .expect("valid_cursor");
                self.masked.insert_char(cursor, c).expect("valid_cursor");
                self.masked.end_undo_seq();
                return true;
            }
        }

        // can shift right
        let g9 = self
            .masked
            .grapheme_at(TextPosition::new(mask.sub_end - 1, 0))
            .expect("valid_pos")
            .expect("mask");
        if mask9.right.can_drop(g9.grapheme()) && self.is_valid_char(&mask.right, c) {
            self.masked.begin_undo_seq();
            self.masked
                .remove_char_range(TextRange::new((mask.sub_end - 1, 0), (mask.sub_end, 0)))
                .expect("valid_range");
            self.masked.insert_char(cursor, c).expect("valid_cursor");
            self.masked.end_undo_seq();
            return true;
        }
        false
    }

    /// Insert c into a rtol section
    fn insert_rtol(&mut self, c: char) -> bool {
        let cursor = self.masked.cursor();

        let mut mask = &self.mask[cursor.x as usize];

        // boundary right/left. prefer right, change mask.
        if mask.peek_left.is_rtol() && (mask.right.is_ltor() || mask.right.is_none()) {
            mask = &self.mask[cursor.x as usize - 1];
        }

        let mask0 = &self.mask[mask.sub_start as usize];

        let g0 = self
            .masked
            .grapheme_at(TextPosition::new(mask.sub_start, 0))
            .expect("valid_pos")
            .expect("grapheme");
        if mask0.right.can_drop(g0.grapheme()) && self.is_valid_char(&mask.right, c) {
            self.masked.begin_undo_seq();
            self.masked
                .remove_char_range(TextRange::new((mask.sub_start, 0), (mask.sub_start + 1, 0)))
                .expect("valid_position");
            self.masked
                .insert_char(TextPosition::new(cursor.x - 1, 0), c)
                .expect("valid_position");
            Self::reformat(&mut self.masked, &self.mask, mask.sub_start..mask.sub_end);
            self.masked.end_undo_seq();
            return true;
        }

        false
    }

    /// Insert a sign c into the current number section
    #[allow(clippy::single_match)]
    fn insert_sign(&mut self, c: char) -> bool {
        let cursor = self.masked.cursor();

        let mut mask = &self.mask[cursor.x as usize];
        // boundary right/left. prefer right, change mask.
        if mask.peek_left.is_number() && (mask.right.is_ltor() || mask.right.is_none()) {
            mask = &self.mask[cursor.x as usize - 1];
        }

        // explicit sign?
        let idx = self.mask[mask.sec_start as usize..mask.sec_end as usize]
            .iter()
            .enumerate()
            .find(|(_, t)| matches!(t.right, Mask::Sign | Mask::Plus))
            .map(|(i, _)| mask.sec_start as usize + i);

        // existing sign somewhere?
        let idx = if idx.is_none() {
            self.masked
                .graphemes(
                    TextRange::new((mask.sec_start, 0), (mask.sec_end, 0)),
                    TextPosition::new(mask.sec_start, 0),
                )
                .expect("valid_range")
                .enumerate()
                .find(|(_, g)| *g == "-" || *g == "+")
                .map(|(i, _)| mask.sec_start as usize + i)
        } else {
            idx
        };

        let idx = if idx.is_none() {
            // moving sign
            let mut idx = mask.sec_end - 1;
            'f: {
                while idx >= mask.sec_start {
                    if self.mask[idx as usize].right == Mask::Numeric(EditDirection::Rtol) {
                        let g = self
                            .grapheme_at(idx)
                            .expect("valid_position")
                            .expect("grapheme");

                        if self.mask[idx as usize].right.can_drop(g.grapheme()) {
                            break 'f Some(idx as usize);
                        }
                    }
                    idx -= 1;
                }
                None
            }
        } else {
            idx
        };

        if let Some(idx) = idx {
            let mask_sign = &self.mask[idx];

            if c == self.neg_sym() || c == '-' {
                // negate current
                let g = self
                    .masked
                    .str_slice(TextRange::new(
                        (idx as upos_type, 0),
                        (idx as upos_type + 1, 0),
                    ))
                    .expect("valid_pos")
                    .to_string();

                self.masked.begin_undo_seq();
                self.masked
                    .remove_char_range(TextRange::new(
                        (idx as upos_type, 0),
                        (idx as upos_type + 1, 0),
                    ))
                    .expect("valid_range");

                let cc = match &mask_sign.right {
                    Mask::Numeric(_) | Mask::Sign => {
                        if g == "-" {
                            ' '
                        } else {
                            '-'
                        }
                    }
                    Mask::Plus => {
                        if g == "-" {
                            '+'
                        } else {
                            '-'
                        }
                    }
                    _ => unreachable!(),
                };

                self.masked
                    .insert_char(TextPosition::new(idx as upos_type, 0), cc)
                    .expect("valid_range");
                self.set_cursor(cursor.x, false);
                self.masked.end_undo_seq();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Remove the previous char.
    pub fn remove_prev(&mut self) {
        let cursor = self.masked.cursor();

        if cursor.x == 0 {
            return;
        }

        let left = &self.mask[cursor.x as usize - 1];

        if left.right.is_rtol() {
            // Check if the section is empty
            let sec_empty = if left.right.is_rtol() {
                let sec_str = self
                    .masked
                    .str_slice(TextRange::new((left.sub_start, 0), (left.sub_end, 0)))
                    .expect("valid_range");
                let sec_mask = &self.mask[left.sub_start as usize..left.sub_end as usize];
                sec_str == MaskToken::empty_section(sec_mask)
            } else {
                false
            };

            let l0 = &self.mask[left.sub_start as usize];

            self.masked.begin_undo_seq();
            self.masked
                .remove_char_range(TextRange::new((cursor.x - 1, 0), cursor))
                .expect("valid_range");
            self.masked
                .insert_str(TextPosition::new(left.sub_start, 0), &l0.edit)
                .expect("valid_position");
            Self::reformat(&mut self.masked, &self.mask, left.sub_start..left.sub_end);

            // in a rtol field keep the cursor at the same position until the
            // whole section is empty. Only then put it at the beginning of the section
            // to continue left of the section.
            if sec_empty {
                self.masked
                    .set_cursor(TextPosition::new(left.sub_start, 0), false);
            } else {
                // cursor stays
            }

            self.masked.end_undo_seq();
        } else if left.right.is_ltor() {
            let l9 = &self.mask[left.sub_end as usize - 1];

            self.masked.begin_undo_seq();
            self.masked
                .remove_char_range(TextRange::new((cursor.x - 1, 0), cursor))
                .expect("valid_range");
            self.masked
                .insert_str(TextPosition::new(left.sub_end - 1, 0), &l9.edit)
                .expect("valid_position");

            Self::reformat(&mut self.masked, &self.mask, left.sub_start..left.sub_end);

            self.masked
                .set_cursor(TextPosition::new(cursor.x - 1, 0), false);

            self.masked.end_undo_seq();
        }
    }

    /// Remove the previous char.
    pub fn remove_next(&mut self) {
        let cursor = self.masked.cursor();

        if cursor.x as usize == self.mask.len() - 1 {
            return;
        }

        let right = &self.mask[cursor.x as usize];

        // remove and fill with empty
        if right.right.is_rtol() {
            let l0 = &self.mask[right.sub_start as usize];

            self.masked.begin_undo_seq();
            self.masked
                .remove_char_range(TextRange::new(cursor, (cursor.x + 1, 0)))
                .expect("valid_range");
            self.masked
                .insert_str(TextPosition::new(right.sub_start, 0), &l0.edit)
                .expect("valid_position");
            Self::reformat(&mut self.masked, &self.mask, right.sub_start..right.sub_end);

            self.masked
                .set_cursor(TextPosition::new(cursor.x + 1, 0), false);

            self.masked.end_undo_seq();
        } else if right.right.is_ltor() {
            // Check if the section is empty
            let sec_str = self
                .masked
                .str_slice(TextRange::new((right.sub_start, 0), (right.sub_end, 0)))
                .expect("valid_range");
            let sec_mask = &self.mask[right.sub_start as usize..right.sub_end as usize];
            let sec_empty = sec_str == MaskToken::empty_section(sec_mask);

            let l9 = &self.mask[right.sub_end as usize - 1];

            self.masked.begin_undo_seq();
            self.masked
                .remove_char_range(TextRange::new(cursor, (cursor.x + 1, 0)))
                .expect("valid_range");
            self.masked
                .insert_str(TextPosition::new(right.sub_end - 1, 0), &l9.edit)
                .expect("valid_position");

            Self::reformat(&mut self.masked, &self.mask, right.sub_start..right.sub_end);

            // in a ltor field keep the cursor at the same position until the
            // whole section is empty. Only then put it at the end of the section
            // to continue right of the section.
            if sec_empty {
                self.masked
                    .set_cursor(TextPosition::new(right.sub_end, 0), false);
            } else {
                // cursor stays
            }

            self.masked.end_undo_seq();
        }
    }

    /// Remove the selection
    pub fn remove_range(&mut self, range: Range<upos_type>) -> Result<bool, TextError> {
        // check valid range
        self.masked
            .bytes_at_range(TextRange::new((range.start, 0), (range.end, 0)))?;

        if range.is_empty() {
            return Ok(false);
        }

        let mask = &self.mask[range.start as usize];
        if range.start >= mask.sub_start && range.end <= mask.sub_end {
            if mask.right.is_rtol() {
                self.masked.begin_undo_seq();
                self.masked
                    .remove_str_range(TextRange::new((range.start, 0), (range.end, 0)))
                    .expect("valid_range");
                let fill_before =
                    &self.mask[mask.sub_start as usize..mask.sub_start as usize + range.len()];
                self.masked
                    .insert_str(
                        TextPosition::new(mask.sub_start, 0),
                        &MaskToken::empty_section(fill_before),
                    )
                    .expect("valid_range");
                Self::reformat(&mut self.masked, &self.mask, mask.sub_start..mask.sub_end);
                self.masked.end_undo_seq();
            } else if mask.right.is_ltor() {
                self.masked.begin_undo_seq();
                self.masked
                    .remove_str_range(TextRange::new((range.start, 0), (range.end, 0)))
                    .expect("valid_range");
                let fill_after =
                    &self.mask[mask.sub_end as usize - range.len()..mask.sub_end as usize];
                self.masked
                    .insert_str(
                        TextPosition::new(mask.sub_end - range.len() as upos_type, 0),
                        &MaskToken::empty_section(fill_after),
                    )
                    .expect("valid_range");
                Self::reformat(&mut self.masked, &self.mask, mask.sub_start..mask.sub_end);
                self.masked.end_undo_seq();
            }

            return Ok(true);
        }

        let mut pos = range.start;
        self.masked.begin_undo_seq();
        loop {
            let mask = &self.mask[pos as usize];

            if mask.sub_start < range.start {
                // partial start
                if mask.right.is_rtol() {
                    self.masked
                        .remove_str_range(TextRange::new((range.start, 0), (mask.sub_end, 0)))
                        .expect("valid_range");

                    let len = mask.sub_end - range.start;
                    let fill_before =
                        &self.mask[mask.sub_start as usize..(mask.sub_start + len) as usize];
                    self.masked
                        .insert_str(
                            TextPosition::new(mask.sub_start, 0),
                            &MaskToken::empty_section(fill_before),
                        )
                        .expect("valid_range");

                    Self::reformat(&mut self.masked, &self.mask, mask.sub_start..mask.sub_end);

                    pos = mask.sub_end;
                } else if mask.right.is_ltor() {
                    self.masked
                        .remove_str_range(TextRange::new((range.start, 0), (mask.sub_end, 0)))
                        .expect("valid_range");

                    let fill_after = &self.mask[range.start as usize..mask.sub_end as usize];
                    self.masked
                        .insert_str(
                            TextPosition::new(range.start, 0),
                            &MaskToken::empty_section(fill_after),
                        )
                        .expect("valid_range");

                    Self::reformat(&mut self.masked, &self.mask, mask.sub_start..mask.sub_end);

                    pos = mask.sub_end;
                }
            } else if mask.sub_end > range.end {
                // partial end
                if mask.right.is_rtol() {
                    self.masked
                        .remove_str_range(TextRange::new((mask.sub_start, 0), (range.end, 0)))
                        .expect("valid_range");

                    let fill_before = &self.mask[mask.sub_start as usize..range.end as usize];
                    self.masked
                        .insert_str(
                            TextPosition::new(mask.sub_start, 0),
                            &MaskToken::empty_section(fill_before),
                        )
                        .expect("valid_range");

                    Self::reformat(&mut self.masked, &self.mask, mask.sub_start..mask.sub_end);
                    pos = mask.sub_end;
                } else if mask.right.is_ltor() {
                    self.masked
                        .remove_str_range(TextRange::new((mask.sub_start, 0), (range.end, 0)))
                        .expect("valid_range");

                    let len = range.end - mask.sub_start;
                    let fill_after =
                        &self.mask[(mask.sub_end - len) as usize..mask.sub_end as usize];
                    self.masked
                        .insert_str(
                            TextPosition::new(mask.sub_end - len, 0),
                            &MaskToken::empty_section(fill_after),
                        )
                        .expect("valid_range");

                    pos = mask.sub_end;
                }
            } else {
                // full section
                self.masked
                    .remove_str_range(TextRange::new((mask.sub_start, 0), (mask.sub_end, 0)))
                    .expect("valid_range");

                let sec_range = &self.mask[mask.sub_start as usize..mask.sub_end as usize];
                self.masked
                    .insert_str(
                        TextPosition::new(mask.sub_start, 0),
                        &MaskToken::empty_section(sec_range),
                    )
                    .expect("valid_range");

                // todo: needed?: Self::reformat(&mut self.masked, &self.mask, mask.sec_start..mask.sec_end);
                pos = mask.sub_end;
            }

            if pos >= range.end {
                break;
            }
        }
        self.masked.end_undo_seq();

        Ok(true)
    }

    /// Rebuild a section according to number-formatting.
    /// The main purpose is to rebuild the grouping separators.
    fn reformat(core: &mut TextCore<TextString>, mask: &[MaskToken], section: Range<upos_type>) {
        if mask[section.start as usize].right.is_rtol() {
            let cursor = core.cursor();
            let anchor = core.anchor();

            let sec_str = core
                .str_slice(TextRange::new((section.start, 0), (section.end, 0)))
                .expect("valid_range");

            // to be safe, always use our internal symbol set.
            let sym = NumberSymbols {
                decimal_sep: '.',
                decimal_grp: Some(','),
                negative_sym: '-',
                positive_sym: ' ',
                exponent_upper_sym: 'E',
                exponent_lower_sym: 'e',
                currency_sym: CurrencySym::new("$"),
            };

            // remove all non numbers and leading 0.
            let mut clean = String::new();
            _ = clean_num(sec_str.as_ref(), &sym, &mut clean);

            // create number format
            let mut tok = String::new();
            let submask = &mask[section.start as usize..section.end as usize];
            // default fmt.sym is nice
            for t in submask {
                match &t.right {
                    Mask::Digit0(_) => tok.push('0'),
                    Mask::Digit(_) => tok.push('9'),
                    Mask::Numeric(_) => tok.push('#'),
                    Mask::DecimalSep => tok.push('.'),
                    Mask::GroupingSep => tok.push(','),
                    Mask::Sign => tok.push('-'),
                    Mask::Plus => tok.push('+'),
                    Mask::Separator(s) => {
                        for c in s.chars() {
                            tok.push('\\');
                            tok.push(c);
                        }
                    }
                    Mask::None => {}
                    _ => unreachable!("invalid mask"),
                }
            }

            let fmt = match NumberFormat::news(tok, sym) {
                Ok(v) => v,
                Err(_) => unreachable!("invalid mask"),
            };
            let mut out = String::new();
            match map_num::<_, false>(clean.as_str(), &fmt, fmt.sym(), &mut out) {
                Ok(_) => {}
                Err(_) => unreachable!("invalid mask"),
            }

            core.remove_char_range(TextRange::new((section.start, 0), (section.end, 0)))
                .expect("valid_range");
            core.insert_str(TextPosition::new(section.start, 0), &out)
                .expect("valid_position");

            // keep cursor intact
            core.set_cursor(anchor, false);
            core.set_cursor(cursor, true);
        } else if mask[section.start as usize].right.is_ltor() {
            let cursor = core.cursor();
            let anchor = core.anchor();

            let sec_str = core
                .str_slice(TextRange::new((section.start, 0), (section.end, 0)))
                .expect("valid_range");
            let sec_mask = &mask[section.start as usize..section.end as usize];
            let mut str_new = String::new();
            for (g, t) in sec_str.graphemes(true).zip(sec_mask.iter()) {
                match t.right {
                    Mask::Digit0(_) | Mask::Hex0 | Mask::Oct0 | Mask::Dec0 => {
                        if g == " " {
                            str_new.push('0');
                        } else {
                            str_new.push_str(g);
                        }
                    }
                    _ => {
                        str_new.push_str(g);
                    }
                }
            }

            if sec_str != str_new {
                core.remove_char_range(TextRange::new((section.start, 0), (section.end, 0)))
                    .expect("valid_range");
                core.insert_str(TextPosition::new(section.start, 0), &str_new)
                    .expect("valid_position");

                // keep cursor intact
                core.set_cursor(anchor, false);
                core.set_cursor(cursor, true);
            }
        }
    }
}

mod mask {
    use crate::upos_type;
    use std::fmt;
    use std::fmt::{Debug, Display, Formatter};

    /// Edit direction for part of a mask.
    /// Numeric values can switch between right-to-left (integer part) and left-to-right (fraction).
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(super) enum EditDirection {
        Ltor,
        Rtol,
    }

    /// One char of the input mask.
    #[allow(variant_size_differences)]
    #[derive(Clone, PartialEq, Eq, Default)]
    #[non_exhaustive]
    pub(super) enum Mask {
        // 0-9, display 0
        Digit0(EditDirection),
        // 0-9, display space
        Digit(EditDirection),
        // 0-9;sign, display space
        Numeric(EditDirection),
        DecimalSep,
        GroupingSep,
        Sign,
        Plus,
        Hex0,
        Hex,
        Oct0,
        Oct,
        Dec0,
        Dec,
        Letter,
        LetterOrDigit,
        LetterDigitSpace,
        AnyChar,
        Separator(Box<str>),
        #[default]
        None,
    }

    /// One token of the input mask.
    ///
    /// Which field of the mask does this belong to:
    /// * Number with integer part, decimal separator, fraction and sign.
    /// * Consecutive mask parts of similar type.
    ///
    /// At this cursor position, what is the main mask (right) and what is possible left of
    /// the cursor position (peek_left).
    ///
    /// Default-values for editing and display.
    #[derive(Clone, PartialEq, Eq)]
    pub(super) struct MaskToken {
        pub sec_id: u16,
        // section/number
        pub sec_start: upos_type,
        // section/number
        pub sec_end: upos_type,
        // part of a number/section
        pub sub_start: upos_type,
        // part of a number/section
        pub sub_end: upos_type,

        // token left of the cursor
        pub peek_left: Mask,
        // token right of the cursor
        pub right: Mask,

        // edit-value of the token
        pub edit: Box<str>,
    }

    impl Debug for EditDirection {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    EditDirection::Ltor => ">",
                    EditDirection::Rtol => "<",
                }
            )
        }
    }

    impl Display for Mask {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            let s = match self {
                Mask::Digit0(_) => "0",
                Mask::Digit(_) => "9",
                Mask::Numeric(_) => "#",
                Mask::DecimalSep => ".",
                Mask::GroupingSep => ",",
                Mask::Sign => "-",
                Mask::Plus => "+",
                Mask::Hex0 => "H",
                Mask::Hex => "h",
                Mask::Oct0 => "O",
                Mask::Oct => "o",
                Mask::Dec0 => "D",
                Mask::Dec => "d",
                Mask::Letter => "l",
                Mask::LetterOrDigit => "a",
                Mask::LetterDigitSpace => "c",
                Mask::AnyChar => "_",
                Mask::Separator(s) => {
                    write!(f, "\\")?;
                    s
                }
                Mask::None => "",
            };
            write!(f, "{}", s)
        }
    }

    impl Debug for Mask {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Mask::Digit0(d) => {
                    write!(f, "{:?}0", d)
                }
                Mask::Digit(d) => {
                    write!(f, "{:?}9", d)
                }
                Mask::Numeric(d) => {
                    write!(f, "{:?}#", d)
                }
                Mask::DecimalSep => write!(f, "."),
                Mask::GroupingSep => write!(f, ","),
                Mask::Sign => write!(f, "-"),
                Mask::Plus => write!(f, "+"),
                Mask::Hex0 => write!(f, "H"),
                Mask::Hex => write!(f, "h"),
                Mask::Oct0 => write!(f, "O"),
                Mask::Oct => write!(f, "o"),
                Mask::Dec0 => write!(f, "D"),
                Mask::Dec => write!(f, "d"),
                Mask::Letter => write!(f, "l"),
                Mask::LetterOrDigit => write!(f, "a"),
                Mask::LetterDigitSpace => write!(f, "c"),
                Mask::AnyChar => write!(f, "_"),
                Mask::Separator(s) => {
                    write!(f, "\\")?;
                    write!(f, "{}", s)
                }
                Mask::None => write!(f, ""),
            }
        }
    }

    impl EditDirection {
        pub(super) fn is_ltor(&self) -> bool {
            *self == EditDirection::Ltor
        }

        pub(super) fn is_rtol(&self) -> bool {
            *self == EditDirection::Rtol
        }
    }

    impl Mask {
        /// is not editable. the last field of the mask at position txt.len() can not be edited,
        /// but it's a valid cursor position.
        pub(super) fn is_none(&self) -> bool {
            *self == Mask::None
        }

        /// left to right editing
        #[inline]
        pub(super) fn is_ltor(&self) -> bool {
            match self {
                Mask::Digit0(d) => d.is_ltor(),
                Mask::Digit(d) => d.is_ltor(),
                Mask::Numeric(d) => d.is_ltor(),
                Mask::GroupingSep => false,
                Mask::Sign => false,
                Mask::Plus => false,
                Mask::DecimalSep => true,
                Mask::Hex0 => true,
                Mask::Hex => true,
                Mask::Oct0 => true,
                Mask::Oct => true,
                Mask::Dec0 => true,
                Mask::Dec => true,
                Mask::Letter => true,
                Mask::LetterOrDigit => true,
                Mask::LetterDigitSpace => true,
                Mask::AnyChar => true,
                Mask::Separator(_) => true,
                Mask::None => false,
            }
        }

        /// right to left editing
        #[inline]
        pub(super) fn is_rtol(&self) -> bool {
            match self {
                Mask::Digit0(d) => d.is_rtol(),
                Mask::Digit(d) => d.is_rtol(),
                Mask::Numeric(d) => d.is_rtol(),
                Mask::GroupingSep => true,
                Mask::Sign => true,
                Mask::Plus => true,
                Mask::DecimalSep => false,
                Mask::Hex0 => false,
                Mask::Hex => false,
                Mask::Oct0 => false,
                Mask::Oct => false,
                Mask::Dec0 => false,
                Mask::Dec => false,
                Mask::Letter => false,
                Mask::LetterOrDigit => false,
                Mask::LetterDigitSpace => false,
                Mask::AnyChar => false,
                Mask::Separator(_) => false,
                Mask::None => false,
            }
        }

        /// is a number mask
        #[inline]
        pub(super) fn is_number(&self) -> bool {
            match self {
                Mask::Digit0(_)
                | Mask::Digit(_)
                | Mask::Numeric(_)
                | Mask::DecimalSep
                | Mask::GroupingSep
                | Mask::Sign
                | Mask::Plus => true,
                Mask::None => false,
                _ => false,
            }
        }

        /// is a separator
        #[inline]
        pub(super) fn is_separator(&self) -> bool {
            match self {
                Mask::Separator(_) => true,
                Mask::None => false,
                _ => false,
            }
        }

        #[inline]
        pub(super) fn is_fraction(&self) -> bool {
            match self {
                Mask::Digit0(d) | Mask::Digit(d) | Mask::Numeric(d) => d.is_ltor(),
                Mask::None => false,
                _ => false,
            }
        }

        /// which mask-types are put together.
        #[inline]
        pub(super) fn sub_section(&self) -> u8 {
            match self {
                Mask::Digit0(_) => 0,
                Mask::Digit(_) => 0,
                Mask::Numeric(_) => 0,
                Mask::GroupingSep => 0,

                Mask::Sign => 1,

                Mask::Plus => 2,

                Mask::DecimalSep => 3,

                Mask::Hex0 => 4,
                Mask::Hex => 4,

                Mask::Oct0 => 5,
                Mask::Oct => 5,

                Mask::Dec0 => 6,
                Mask::Dec => 6,

                Mask::Letter => 7,
                Mask::LetterOrDigit => 8,
                Mask::LetterDigitSpace => 9,
                Mask::AnyChar => 10,

                Mask::Separator(_) => 11,

                Mask::None => 12,
            }
        }

        /// which mask-types constitute a number/section
        #[inline]
        pub(super) fn section(&self) -> u8 {
            match self {
                Mask::Digit0(_) => 0,
                Mask::Digit(_) => 0,
                Mask::Numeric(_) => 0,
                Mask::GroupingSep => 0,
                Mask::Sign => 0,
                Mask::Plus => 0,
                Mask::DecimalSep => 0,

                Mask::Hex0 => 1,
                Mask::Hex => 1,
                Mask::Oct0 => 1,
                Mask::Oct => 1,
                Mask::Dec0 => 1,
                Mask::Dec => 1,
                Mask::Letter => 1,
                Mask::LetterOrDigit => 1,
                Mask::LetterDigitSpace => 1,
                Mask::AnyChar => 1,

                Mask::Separator(_) => 2,
                Mask::None => 3,
            }
        }

        /// mask should overwrite instead of insert
        #[inline]
        pub(super) fn can_overwrite_fraction(&self, c: &str) -> bool {
            match self {
                Mask::Digit0(_) => c == "0",
                Mask::Digit(_) | Mask::Numeric(_) => c == " ",
                Mask::DecimalSep => false,
                Mask::GroupingSep => false,
                Mask::Sign => false,
                Mask::Plus => false,
                Mask::Hex0 => false,
                Mask::Hex => false,
                Mask::Oct0 => false,
                Mask::Oct => false,
                Mask::Dec0 => false,
                Mask::Dec => false,
                Mask::Letter => false,
                Mask::LetterOrDigit => false,
                Mask::LetterDigitSpace => false,
                Mask::AnyChar => false,
                Mask::Separator(_) => false,
                Mask::None => false,
            }
        }

        /// mask should overwrite instead of insert
        #[inline]
        pub(super) fn can_overwrite(&self, c: &str) -> bool {
            match self {
                Mask::Digit0(_) | Mask::Digit(_) | Mask::Numeric(_) => false,
                Mask::DecimalSep => "." == c,
                Mask::GroupingSep => false,
                Mask::Sign => "-" == c || " " == c,
                Mask::Plus => "-" == c || "+" == c || " " == c,
                Mask::Hex0 => c == "0",
                Mask::Hex => false,
                Mask::Oct0 => c == "0",
                Mask::Oct => false,
                Mask::Dec0 => c == "0",
                Mask::Dec => false,
                Mask::Letter => false,
                Mask::LetterOrDigit => false,
                Mask::LetterDigitSpace => false,
                Mask::AnyChar => false,
                Mask::Separator(sep) => sep.as_ref() == c,
                Mask::None => false,
            }
        }

        /// char can be dropped from the text and it's ok.
        #[inline]
        pub(super) fn can_drop(&self, c: &str) -> bool {
            match self {
                Mask::Digit0(_) => c == "0",
                Mask::Digit(_) => c == " ",
                Mask::Numeric(_) => c == " ",
                Mask::DecimalSep => false,
                Mask::Sign => false,
                Mask::Plus => false,
                Mask::GroupingSep => true,
                Mask::Hex0 => c == "0",
                Mask::Hex => c == " ",
                Mask::Oct0 => c == "0",
                Mask::Oct => c == " ",
                Mask::Dec0 => c == "0",
                Mask::Dec => c == " ",
                Mask::Letter => c == " ",
                Mask::LetterOrDigit => c == " ",
                Mask::LetterDigitSpace => c == " ",
                Mask::AnyChar => c == " ",
                Mask::Separator(_sep) => false,
                Mask::None => false,
            }
        }

        /// default char for this mask.
        #[inline]
        pub(super) fn edit_value(&self) -> &str {
            match self {
                Mask::Digit0(_) => "0",
                Mask::Digit(_) => " ",
                Mask::Numeric(_) => " ",
                Mask::DecimalSep => ".",
                Mask::GroupingSep => " ", // don't show. reformat fills it in if necessary.
                Mask::Sign => " ",
                Mask::Plus => "+",
                Mask::Hex0 => "0",
                Mask::Hex => " ",
                Mask::Oct0 => "0",
                Mask::Oct => " ",
                Mask::Dec0 => "0",
                Mask::Dec => " ",
                Mask::Letter => " ",
                Mask::LetterOrDigit => " ",
                Mask::LetterDigitSpace => " ",
                Mask::AnyChar => " ",
                Mask::Separator(g) => g.as_ref(),
                Mask::None => "",
            }
        }
    }

    impl Debug for MaskToken {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "Mask #{}:{}-{} {:?} | {:?}",
                self.sec_id, self.sub_start, self.sub_end, self.peek_left, self.right
            )
        }
    }

    impl MaskToken {
        /// is somewhere in the integer part of a number.
        #[inline]
        pub(super) fn is_integer_part(&self) -> bool {
            self.peek_left.is_rtol() || self.peek_left.is_none() && self.right.is_rtol()
        }

        /// Create a string with the default edit mask.
        pub(super) fn empty_section(mask: &[MaskToken]) -> String {
            let mut buf = String::new();
            for m in mask {
                buf.push_str(&m.edit);
            }
            buf
        }
    }
}
