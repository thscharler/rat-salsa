use crate::text_store::SkipLine;
use crate::{upos_type, Grapheme, TextPosition};
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::ControlFlow::{Break, Continue};
use std::ops::{ControlFlow, Range};
use std::rc::Rc;

/// Data for rendering/mapping graphemes to screen coordinates.
#[derive(Debug)]
pub(crate) struct Glyph2<'a> {
    /// Display glyph.
    pub(crate) glyph: Cow<'a, str>,
    /// byte-range of the glyph in the given slice.
    pub(crate) text_bytes: Range<usize>,
    /// screen-position corrected by text_offset.
    /// first visible column is at 0.
    /// Warning: this is not an upos_type, but an u16.
    pub(crate) screen_pos: (upos_type, upos_type),
    /// Display length for the glyph.
    pub(crate) screen_width: upos_type,
    /// Last item in this screen-line.
    pub(crate) line_break: bool,
    /// text-position
    pub(crate) pos: TextPosition,
}

impl<'a> Glyph2<'a> {
    /// Get the glyph.
    pub fn glyph(&'a self) -> &'a str {
        self.glyph.as_ref()
    }

    /// Get the byte-range as absolute range into the complete text.
    pub fn text_bytes(&self) -> Range<usize> {
        self.text_bytes.clone()
    }

    /// Get the position of the glyph
    pub fn pos(&self) -> TextPosition {
        self.pos
    }

    /// Get the screen position of the glyph. Starts at (0,0) in
    /// the top/left of the widget.
    pub fn screen_pos(&self) -> (u16, u16) {
        (self.screen_pos.0 as u16, self.screen_pos.1 as u16)
    }

    /// Display width of the glyph.
    pub fn screen_width(&self) -> u16 {
        self.screen_width as u16
    }

    /// Last item in this screen line
    pub fn line_break(&self) -> bool {
        self.line_break
    }

    /// Does the glyph cover the given screen-position?
    pub fn contains_screen_pos(&self, screen_pos: (u16, u16)) -> bool {
        if self.screen_pos.1 == screen_pos.1 as upos_type {
            if screen_pos.0 as upos_type >= self.screen_pos.0 {
                if (screen_pos.0 as upos_type) < self.screen_pos.0 + self.screen_width as upos_type
                {
                    return true;
                }
                if self.line_break {
                    return true;
                }
            }
        }

        false
    }

    /// Does the glyph cover the given x-position.
    /// Doesn't check for the y-position.
    pub fn contains_screen_x(&self, screen_x: u16) -> bool {
        if screen_x as upos_type >= self.screen_pos.0 {
            if (screen_x as upos_type) < self.screen_pos.0 + self.screen_width as upos_type {
                return true;
            }
            if self.line_break {
                return true;
            }
        }

        false
    }

    /// Validite bounds.
    fn validate(&self) {
        assert!(self.screen_pos.0 <= u16::MAX as upos_type);
        assert!(self.screen_pos.1 <= u16::MAX as upos_type);
        assert!(self.screen_width <= u16::MAX as upos_type);
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum TextWrap2 {
    /// shift glyphs to the left and clip at right margin.
    Shift,
    /// hard break text at right margin.
    Hard,
    /// word break the text.
    Word,
}

impl Default for TextWrap2 {
    fn default() -> Self {
        Self::Shift
    }
}

/// Glyph cache.
#[derive(Debug, Clone, Default)]
pub(crate) struct GlyphCache {
    /// Cache validity: base offset
    pub offset: Cell<(usize, usize)>,
    /// Cache validity: sub_row offset.
    pub sub_row_offset: Cell<upos_type>,
    /// Cache validity: rendered text-width.
    pub text_width: Cell<upos_type>,

    /// Mark the byte-positions of each line-start.
    ///
    /// Used when text-wrap is ShiftText.
    pub line_start: Rc<RefCell<HashMap<upos_type, GlyphCacheLine>>>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct GlyphCacheLine {
    pub pos_x: upos_type,
    pub screen_pos_x: upos_type,
    pub byte_pos: usize,
}

impl GlyphCache {
    /// Invalidate all entries past this byte-position.
    pub(crate) fn invalidate(
        &self,
        offset: (usize, usize),
        sub_row_offset: upos_type,
        text_width: upos_type,
        byte_pos: Option<usize>,
    ) {
        if self.offset.get() != offset {
            self.line_start.borrow_mut().clear();
        } else {
            if let Some(byte_pos) = byte_pos {
                self.line_start
                    .borrow_mut()
                    .retain(|_, cache| cache.byte_pos < byte_pos);
            }
        }

        self.offset.set(offset);
        self.sub_row_offset.set(sub_row_offset);
        self.text_width.set(text_width);
    }
}

pub(crate) struct GlyphIter2<'a, Graphemes> {
    iter: Graphemes,
    done: bool,

    /// Sometimes one grapheme creates two glyphs.
    next_glyph: Option<Glyph2<'static>>,

    /// Next glyph position.
    next_pos: TextPosition,
    next_screen_pos: (upos_type, upos_type),
    /// Text position of the previous glyph.
    last_pos: TextPosition,
    last_byte: usize,

    /// Glyph cache
    cache: GlyphCache,

    /// Tab expansion
    tabs: upos_type,
    /// Show CTRL chars
    show_ctrl: bool,
    /// Line-break enabled?
    line_break: bool,
    /// Text-break enabled?
    text_wrap: TextWrap2,
    /// Left margin
    left_margin: upos_type,
    /// Right margin
    right_margin: upos_type,
    /// Word breaking after this margin.
    word_margin: upos_type,

    _phantom: PhantomData<&'a ()>,
}

impl<'a, Graphemes> Debug for GlyphIter2<'a, Graphemes> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphIter2")
            .field("done", &self.done)
            .field("next_glyph", &self.next_glyph)
            .field("next_pos", &self.next_pos)
            .field("next_screen_pos", &self.next_screen_pos)
            .field("last_pos", &self.last_pos)
            .field("last_byte", &self.last_byte)
            .field("tabs", &self.tabs)
            .field("show_ctrl", &self.show_ctrl)
            .field("line_break", &self.line_break)
            .field("text_wrap", &self.text_wrap)
            .field("left_margin", &self.left_margin)
            .field("right_margin", &self.right_margin)
            .field("word_margin", &self.word_margin)
            .finish()
    }
}

impl<'a, Graphemes> GlyphIter2<'a, Graphemes> {
    /// New iterator.
    pub(crate) fn new(pos: TextPosition, iter: Graphemes, cache: GlyphCache) -> Self {
        Self {
            iter,
            done: Default::default(),
            next_glyph: Default::default(),
            next_pos: pos,
            next_screen_pos: Default::default(),
            last_pos: Default::default(),
            last_byte: Default::default(),
            cache,
            tabs: 8,
            show_ctrl: false,
            line_break: true,
            text_wrap: Default::default(),
            left_margin: Default::default(),
            right_margin: Default::default(),
            word_margin: Default::default(),
            _phantom: Default::default(),
        }
    }

    /// Tab width
    pub(crate) fn set_tabs(&mut self, tabs: upos_type) {
        self.tabs = tabs;
    }

    /// Handle line-breaks. If false everything is treated as one line.
    pub(crate) fn set_line_break(&mut self, line_break: bool) {
        self.line_break = line_break;
    }

    /// Show ASCII control codes.
    pub(crate) fn set_show_ctrl(&mut self, show_ctrl: bool) {
        self.show_ctrl = show_ctrl;
    }

    /// Handle text-breaks. Breaks the line and continues on the
    /// next screen line.
    pub(crate) fn set_text_wrap(&mut self, text_wrap: TextWrap2) {
        self.text_wrap = text_wrap;
    }

    pub(crate) fn set_left_margin(&mut self, left_margin: upos_type) {
        self.left_margin = left_margin;
    }

    pub(crate) fn set_right_margin(&mut self, right_margin: upos_type) {
        self.right_margin = right_margin;
    }

    #[inline]
    pub(crate) fn true_right_margin(&self) -> upos_type {
        if self.show_ctrl {
            self.right_margin.saturating_sub(1)
        } else {
            self.right_margin
        }
    }

    pub(crate) fn set_word_margin(&mut self, word_margin: upos_type) {
        self.word_margin = word_margin;
    }
}

impl<'a, Graphemes> Iterator for GlyphIter2<'a, Graphemes>
where
    Graphemes: SkipLine + Iterator<Item = Grapheme<'a>>,
{
    type Item = Glyph2<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        if let Some(glyph) = self.next_glyph.take() {
            self.last_pos = glyph.pos;
            self.last_byte = glyph.text_bytes.end;

            glyph.validate();
            return Some(glyph);
        }

        loop {
            let Some(grapheme) = self.iter.next() else {
                self.done = true;

                // emit a synthetic EOT at the very end.
                // helps if the last line doesn't end in a line-break.
                let glyph = Glyph2 {
                    glyph: if self.show_ctrl {
                        Cow::Borrowed("\u{2403}")
                    } else {
                        Cow::Borrowed("")
                    },
                    text_bytes: self.last_byte..self.last_byte,
                    screen_pos: (
                        self.next_screen_pos.0.saturating_sub(self.left_margin),
                        self.next_screen_pos.1,
                    ),
                    screen_width: if self.show_ctrl { 1 } else { 0 },
                    line_break: true,
                    pos: self.next_pos,
                };

                return Some(glyph);
            };

            let (grapheme, grapheme_bytes) = grapheme.into_parts();

            let mut glyph = Glyph2 {
                glyph: grapheme,
                text_bytes: grapheme_bytes,
                screen_pos: (self.next_screen_pos.0, self.next_screen_pos.1),
                screen_width: 0,
                line_break: false,
                pos: self.next_pos,
            };

            // remap grapheme
            remap_glyph(&mut glyph, self.line_break, self.show_ctrl, self.tabs);

            // next glyph positioning
            let r = match self.text_wrap {
                TextWrap2::Shift => shift_clip_next(self, glyph),
                TextWrap2::Hard => hard_wrap_next(self, glyph),
                TextWrap2::Word => word_wrap_next(self, glyph),
            };
            match r {
                Continue(_) => continue,
                Break(glyph) => return glyph,
            }
        }
    }
}

fn word_wrap_next<'a, Graphemes>(
    iter: &mut GlyphIter2<'a, Graphemes>,
    glyph: Glyph2<'a>,
) -> ControlFlow<Option<Glyph2<'a>>>
where
    Graphemes: Iterator<Item = Grapheme<'a>> + SkipLine,
{
    if glyph.line_break {
        // new-line

        iter.next_screen_pos.0 = 0;
        iter.next_screen_pos.1 += 1;
        iter.next_pos.x = 0;
        iter.next_pos.y += 1;
        iter.last_pos = glyph.pos;
        iter.last_byte = glyph.text_bytes.end;

        glyph.validate();
        Break(Some(glyph))
    } else if glyph.screen_pos.0 > iter.word_margin && glyph.glyph == " " {
        // break after space

        iter.next_screen_pos.0 = 0;
        iter.next_screen_pos.1 += 1;
        iter.next_pos.x += 1;
        // next_pos.y doesn't change
        iter.last_pos = glyph.pos;
        iter.last_byte = glyph.text_bytes.end;

        iter.next_glyph = Some(Glyph2 {
            glyph: if iter.show_ctrl {
                Cow::Borrowed("\u{2424}")
            } else {
                Cow::Borrowed("")
            },
            text_bytes: glyph.text_bytes.end..glyph.text_bytes.end,
            screen_pos: (glyph.screen_pos.0 + 1, glyph.screen_pos.1),
            screen_width: if iter.show_ctrl { 1 } else { 0 },
            line_break: true,
            pos: glyph.pos,
        });

        glyph.validate();
        Break(Some(glyph))
    } else {
        iter.next_screen_pos.0 += glyph.screen_width as upos_type;
        // next_screen_pos.1 doesn't change
        iter.next_pos.x += 1;
        // next_pos.1 doesn't change
        iter.last_pos = glyph.pos;
        iter.last_byte = glyph.text_bytes.end;

        glyph.validate();
        Break(Some(glyph))
    }
}

fn hard_wrap_next<'a, Graphemes>(
    iter: &mut GlyphIter2<'a, Graphemes>,
    mut glyph: Glyph2<'a>,
) -> ControlFlow<Option<Glyph2<'a>>>
where
    Graphemes: Iterator<Item = Grapheme<'a>> + SkipLine,
{
    if glyph.line_break {
        // new-line

        iter.next_screen_pos.0 = 0;
        iter.next_screen_pos.1 += 1;
        iter.next_pos.x = 0;
        iter.next_pos.y += 1;
        iter.last_pos = glyph.pos;
        iter.last_byte = glyph.text_bytes.end;

        glyph.validate();
        Break(Some(glyph))
    } else if glyph.screen_pos.0 + glyph.screen_width as upos_type > iter.true_right_margin() {
        // break before glyph

        iter.next_screen_pos.0 = glyph.screen_width as upos_type;
        iter.next_screen_pos.1 += 1;
        iter.next_pos.x += 1;
        // next_pos.y doesn't change
        iter.last_pos = glyph.pos;
        iter.last_byte = glyph.text_bytes.end;

        iter.next_glyph = Some(Glyph2 {
            glyph: Cow::Owned(glyph.glyph.to_string()),
            text_bytes: glyph.text_bytes.clone(),
            screen_pos: (0, glyph.screen_pos.1 + 1),
            screen_width: glyph.screen_width,
            line_break: false,
            pos: glyph.pos,
        });

        glyph.glyph = if iter.show_ctrl {
            Cow::Borrowed("\u{2424}")
        } else {
            Cow::Borrowed("")
        };
        glyph.text_bytes = glyph.text_bytes.start..glyph.text_bytes.start;
        // screen_pos is ok
        glyph.screen_width = if iter.show_ctrl { 1 } else { 0 };
        glyph.line_break = true;
        glyph.pos = iter.last_pos;

        glyph.validate();
        Break(Some(glyph))
    } else {
        iter.next_screen_pos.0 += glyph.screen_width as upos_type;
        // next_screen_pos.1 doesn't change
        iter.next_pos.x += 1;
        // next_pos.1 doesn't change
        iter.last_pos = glyph.pos;
        iter.last_byte = glyph.text_bytes.end;

        glyph.validate();
        Break(Some(glyph))
    }
}

fn shift_clip_next<'a, Graphemes>(
    iter: &mut GlyphIter2<'a, Graphemes>,
    mut glyph: Glyph2<'a>,
) -> ControlFlow<Option<Glyph2<'a>>>
where
    Graphemes: SkipLine + Iterator<Item = Grapheme<'a>>,
{
    // Clip glyphs and correct left offset
    if glyph.line_break {
        iter.next_screen_pos.0 = 0;
        iter.next_screen_pos.1 += 1;
        iter.next_pos.x = 0;
        iter.next_pos.y += 1;
        iter.last_pos = glyph.pos;
        iter.last_byte = glyph.text_bytes.end;

        if glyph.screen_pos.0 <= iter.true_right_margin() {
            glyph.screen_pos.0 = glyph.screen_pos.0.saturating_sub(iter.left_margin);
            glyph.validate();
            Break(Some(glyph))
        } else {
            // shouldn't happen with skip_line?
            unreachable!("line-break should have been skipped");
        }
    } else if glyph.screen_pos.0 < iter.left_margin
        && glyph.screen_pos.0 + glyph.screen_width > iter.left_margin
    {
        // show replacement for split glyph
        glyph.glyph = Cow::Borrowed("\u{2426}");
        glyph.screen_width = iter.next_screen_pos.0 + glyph.screen_width - iter.left_margin;
        glyph.screen_pos.0 = 0;

        // cache line start position.
        iter.cache.line_start.borrow_mut().insert(
            glyph.pos.y,
            GlyphCacheLine {
                pos_x: glyph.pos.x,
                screen_pos_x: iter.next_screen_pos.0,
                byte_pos: glyph.text_bytes.start,
            },
        );

        iter.next_screen_pos.0 += glyph.screen_width;
        // iter.next_screen_pos.1 doesn't change
        iter.next_pos.x += 1;
        // next_pos.y doesn't change
        iter.last_pos = glyph.pos;
        iter.last_byte = glyph.text_bytes.end;

        glyph.validate();
        Break(Some(glyph))
    } else if glyph.screen_pos.0 < iter.left_margin {
        iter.next_screen_pos.0 += glyph.screen_width;
        // iter.next_screen_pos.1 doesn't change
        iter.next_pos.x += 1;
        // next_pos.y doesn't change
        iter.last_pos = glyph.pos;
        iter.last_byte = glyph.text_bytes.end;

        if let Some(cached) = iter.cache.line_start.borrow().get(&glyph.pos.y) {
            iter.iter.skip_to(cached.byte_pos).expect("valid-pos");
            iter.next_pos.x = cached.pos_x;
            iter.next_screen_pos.0 = cached.screen_pos_x;
            Continue(())
        } else {
            // not yet cached. go the long way.
            Continue(())
        }
    } else if glyph.screen_pos.0 == iter.true_right_margin() {
        glyph.line_break = true;
        glyph.screen_pos.0 = glyph.screen_pos.0.saturating_sub(iter.left_margin);
        glyph.screen_width = if iter.show_ctrl { 1 } else { 0 };
        glyph.glyph = Cow::Borrowed(if iter.show_ctrl { "\u{2424}" } else { "" });
        glyph.text_bytes = iter.last_byte..iter.last_byte;

        iter.next_screen_pos.0 += 1;
        // iter.next_screen_pos.1 doesn't change
        iter.next_pos.x += 1;
        // next_pos.y doesn't change
        iter.last_pos = glyph.pos;
        iter.last_byte = glyph.text_bytes.end;

        glyph.validate();
        Break(Some(glyph))
    } else if iter.next_screen_pos.0 > iter.true_right_margin() {
        // skip to next_line
        iter.iter.skip_line().expect("fine");

        // do the line-break here.
        iter.next_screen_pos.0 = 0;
        iter.next_screen_pos.1 += 1;
        iter.next_pos.x = 0;
        iter.next_pos.y += 1;
        iter.last_pos = glyph.pos;
        iter.last_byte = glyph.text_bytes.end;

        Continue(())
    } else if iter.next_screen_pos.0 < iter.true_right_margin()
        && iter.next_screen_pos.0 + glyph.screen_width as upos_type > iter.true_right_margin()
    {
        // show replacement for split glyph
        glyph.glyph = Cow::Borrowed("\u{2426}");
        glyph.screen_pos.0 = glyph.screen_pos.0.saturating_sub(iter.left_margin);
        glyph.screen_width = iter.next_screen_pos.0 + glyph.screen_width - iter.right_margin;

        iter.next_screen_pos.0 += glyph.screen_width as upos_type;
        // iter.next_screen_pos.1 doesn't change
        iter.next_pos.x += 1;
        // next_pos.y doesn't change
        iter.last_pos = glyph.pos;
        iter.last_byte = glyph.text_bytes.end;

        glyph.validate();
        Break(Some(glyph))
    } else {
        glyph.screen_pos.0 = glyph.screen_pos.0.saturating_sub(iter.left_margin);

        if glyph.screen_pos.0 == 0 {
            iter.cache.line_start.borrow_mut().insert(
                glyph.pos.y,
                GlyphCacheLine {
                    pos_x: glyph.pos.x,
                    screen_pos_x: iter.next_screen_pos.0,
                    byte_pos: glyph.text_bytes.start,
                },
            );
        }

        iter.next_screen_pos.0 += glyph.screen_width as upos_type;
        // iter.next_screen_pos.1 doesn't change
        iter.next_pos.x += 1;
        // next_pos.y doesn't change
        iter.last_pos = glyph.pos;
        iter.last_byte = glyph.text_bytes.end;

        glyph.validate();
        Break(Some(glyph))
    }
}

fn remap_glyph(glyph: &mut Glyph2<'_>, lf_breaks: bool, show_ctrl: bool, tabs: upos_type) {
    if glyph.glyph == "\n" || glyph.glyph == "\r\n" {
        if lf_breaks {
            glyph.line_break = true;
            glyph.screen_width = if show_ctrl { 1 } else { 0 };
            glyph.glyph = Cow::Borrowed(if show_ctrl { "\u{240A}" } else { "" });
        } else {
            glyph.line_break = false;
            glyph.screen_width = 1;
            glyph.glyph = Cow::Borrowed("\u{240A}");
        }
    } else if glyph.glyph == "\t" {
        glyph.line_break = false;
        glyph.screen_width = tabs - (glyph.screen_pos.0 % tabs);
        glyph.glyph = Cow::Borrowed(if show_ctrl { "\u{2409}" } else { " " });
    } else if ("\x00".."\x20").contains(&glyph.glyph.as_ref()) {
        glyph.line_break = false;
        glyph.screen_width = 1;

        // Control char unicode display replacement.
        static CONTROL_CHARS: [&str; 32] = [
            "\u{2400}", "\u{2401}", "\u{2402}", "\u{2403}", "\u{2404}", "\u{2405}", "\u{2406}",
            "\u{2407}", "\u{2408}", "\u{2409}", "\u{240A}", "\u{240B}", "\u{240C}", "\u{240D}",
            "\u{240E}", "\u{240F}", "\u{2410}", "\u{2411}", "\u{2412}", "\u{2413}", "\u{2414}",
            "\u{2415}", "\u{2416}", "\u{2417}", "\u{2418}", "\u{2419}", "\u{241A}", "\u{241B}",
            "\u{241C}", "\u{241D}", "\u{241E}", "\u{241F}",
        ];

        glyph.glyph = Cow::Borrowed(if show_ctrl {
            CONTROL_CHARS[glyph.glyph.as_bytes()[0] as usize]
        } else {
            "\u{FFFD}"
        });
    } else {
        glyph.line_break = false;
        glyph.screen_width = unicode_display_width::width(&glyph.glyph) as upos_type;
        // glyph.glyph = glyph.glyph;
    }
}

#[cfg(test)]
mod test_glyph {
    use crate::glyph::GlyphIter;
    use crate::grapheme::RopeGraphemes;
    use crate::TextPosition;
    use ropey::Rope;

    #[test]
    fn test_glyph1() {
        let s = Rope::from(
            r#"0123456789
abcdefghij
jklöjklöjk
uiopü+uiop"#,
        );
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let mut glyphs = GlyphIter::new(TextPosition::new(0, 0), r);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "0");
        assert_eq!(n.text_bytes(), 0..1);
        assert_eq!(n.screen_pos(), (0, 0));
        assert_eq!(n.pos(), TextPosition::new(0, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "1");
        assert_eq!(n.text_bytes(), 1..2);
        assert_eq!(n.screen_pos(), (1, 0));
        assert_eq!(n.pos(), TextPosition::new(1, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "2");
        assert_eq!(n.text_bytes(), 2..3);
        assert_eq!(n.screen_pos(), (2, 0));
        assert_eq!(n.pos(), TextPosition::new(2, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.nth(7).unwrap();
        assert_eq!(n.glyph(), "");
        assert_eq!(n.text_bytes(), 10..11);
        assert_eq!(n.screen_pos(), (10, 0));
        assert_eq!(n.pos(), TextPosition::new(10, 0));
        assert_eq!(n.screen_width(), 0);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "a");
        assert_eq!(n.text_bytes(), 11..12);
        assert_eq!(n.screen_pos(), (0, 1));
        assert_eq!(n.pos(), TextPosition::new(0, 1));
        assert_eq!(n.screen_width(), 1);
    }

    #[test]
    fn test_glyph2() {
        // screen offset
        let s = Rope::from(
            r#"0123456789
abcdefghij
jklöjklöjk
uiopü+uiop"#,
        );
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let mut glyphs = GlyphIter::new(TextPosition::new(0, 0), r);
        glyphs.set_screen_offset(2);
        glyphs.set_screen_width(100);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "2");
        assert_eq!(n.text_bytes(), 2..3);
        assert_eq!(n.screen_pos(), (0, 0));
        assert_eq!(n.pos(), TextPosition::new(2, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "3");
        assert_eq!(n.text_bytes(), 3..4);
        assert_eq!(n.screen_pos(), (1, 0));
        assert_eq!(n.pos(), TextPosition::new(3, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.nth(6).unwrap();
        assert_eq!(n.glyph(), "");
        assert_eq!(n.text_bytes(), 10..11);
        assert_eq!(n.screen_pos(), (8, 0));
        assert_eq!(n.pos(), TextPosition::new(10, 0));
        assert_eq!(n.screen_width(), 0);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "c");
        assert_eq!(n.text_bytes(), 13..14);
        assert_eq!(n.screen_pos(), (0, 1));
        assert_eq!(n.pos(), TextPosition::new(2, 1));
        assert_eq!(n.screen_width(), 1);
    }

    #[test]
    fn test_glyph3() {
        // screen offset + width
        let s = Rope::from(
            r#"0123456789
abcdefghij
jklöjklöjk
uiopü+uiop"#,
        );
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let mut glyphs = GlyphIter::new(TextPosition::new(0, 0), r);
        glyphs.set_screen_offset(2);
        glyphs.set_screen_width(6);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "2");
        assert_eq!(n.text_bytes(), 2..3);
        assert_eq!(n.screen_pos(), (0, 0));
        assert_eq!(n.pos(), TextPosition::new(2, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "3");
        assert_eq!(n.text_bytes(), 3..4);
        assert_eq!(n.screen_pos(), (1, 0));
        assert_eq!(n.pos(), TextPosition::new(3, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.nth(2).unwrap();
        assert_eq!(n.glyph(), "6");

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "7");
        assert_eq!(n.text_bytes(), 7..8);
        assert_eq!(n.screen_pos(), (5, 0));
        assert_eq!(n.pos(), TextPosition::new(7, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "c");
        assert_eq!(n.text_bytes(), 13..14);
        assert_eq!(n.screen_pos(), (0, 1));
        assert_eq!(n.pos(), TextPosition::new(2, 1));
        assert_eq!(n.screen_width(), 1);
    }

    #[test]
    fn test_glyph4() {
        // tabs
        let s = Rope::from(
            "012\t3456789
abcdefghij
jklöjklöjk
uiopü+uiop",
        );
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let mut glyphs = GlyphIter::new(TextPosition::new(0, 0), r);
        glyphs.set_screen_offset(2);
        glyphs.set_screen_width(100);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "2");
        assert_eq!(n.text_bytes(), 2..3);
        assert_eq!(n.screen_pos(), (0, 0));
        assert_eq!(n.pos(), TextPosition::new(2, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), " ");
        assert_eq!(n.text_bytes(), 3..4);
        assert_eq!(n.screen_pos(), (1, 0));
        assert_eq!(n.pos(), TextPosition::new(3, 0));
        assert_eq!(n.screen_width(), 5);

        let n = glyphs.nth(7).unwrap();
        assert_eq!(n.glyph(), "");
        assert_eq!(n.text_bytes(), 11..12);
        assert_eq!(n.screen_pos(), (13, 0));
        assert_eq!(n.pos(), TextPosition::new(11, 0));
        assert_eq!(n.screen_width(), 0);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "c");
        assert_eq!(n.text_bytes(), 14..15);
        assert_eq!(n.screen_pos(), (0, 1));
        assert_eq!(n.pos(), TextPosition::new(2, 1));
        assert_eq!(n.screen_width(), 1);
    }

    #[test]
    fn test_glyph5() {
        // clipping wide
        let s = Rope::from(
            "0\t12345678\t9
abcdefghij
jklöjklöjk
uiopü+uiop",
        );
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let mut glyphs = GlyphIter::new(TextPosition::new(0, 0), r);
        glyphs.set_screen_offset(2);
        glyphs.set_screen_width(20);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "∃");
        assert_eq!(n.text_bytes(), 1..2);
        assert_eq!(n.screen_pos(), (0, 0));
        assert_eq!(n.pos(), TextPosition::new(1, 0));
        assert_eq!(n.screen_width(), 6);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "1");
        assert_eq!(n.text_bytes(), 2..3);
        assert_eq!(n.screen_pos(), (6, 0));
        assert_eq!(n.pos(), TextPosition::new(2, 0));
        assert_eq!(n.screen_width(), 1);

        let n = glyphs.nth(6).unwrap();
        assert_eq!(n.glyph(), "8");

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "∃");
        assert_eq!(n.text_bytes(), 10..11);
        assert_eq!(n.screen_pos(), (14, 0));
        assert_eq!(n.pos(), TextPosition::new(10, 0));
        assert_eq!(n.screen_width(), 2);

        let n = glyphs.next().unwrap();
        assert_eq!(n.glyph(), "c");
        assert_eq!(n.text_bytes(), 15..16);
        assert_eq!(n.screen_pos(), (0, 1));
        assert_eq!(n.pos(), TextPosition::new(2, 1));
        assert_eq!(n.screen_width(), 1);
    }
}
