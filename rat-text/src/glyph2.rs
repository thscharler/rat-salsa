use crate::text_store::SkipLine;
use crate::{upos_type, Grapheme, TextPosition};
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Range;
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
    pub(crate) screen_pos: (u16, u16),
    /// Display length for the glyph.
    pub(crate) screen_width: u16,
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
        self.screen_pos
    }

    /// Display width of the glyph.
    pub fn screen_width(&self) -> u16 {
        self.screen_width
    }

    /// Last item in this screen line
    pub fn line_break(&self) -> bool {
        self.line_break
    }

    /// Does the glyph cover the given screen-position?
    pub fn contains_screen_pos(&self, screen_pos: (u16, u16)) -> bool {
        if self.screen_pos.1 == screen_pos.1 {
            if screen_pos.0 >= self.screen_pos.0 {
                if screen_pos.0 < self.screen_pos.0 + self.screen_width {
                    return true;
                }
                if self.line_break {
                    return true;
                }
            }
        }

        false
    }

    /// Does teh glyph cover the given x-position.
    /// Doesn't check for the y-position.
    pub fn contains_screen_x(&self, screen_x: u16) -> bool {
        if screen_x >= self.screen_pos.0 {
            if screen_x < self.screen_pos.0 + self.screen_width {
                return true;
            }
            if self.line_break {
                return true;
            }
        }

        false
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum TextWrap2 {
    /// shift glyphs to the left and clip at right margin.
    ShiftText,
    /// break text at right margin.
    /// if a word-margin is set, use it.
    BreakText,
}

impl Default for TextWrap2 {
    fn default() -> Self {
        Self::ShiftText
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

pub(crate) struct GlyphIter2<'a> {
    iter: Box<dyn SkipLine<Item = Grapheme<'a>> + 'a>,
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
}

impl Debug for GlyphIter2<'_> {
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

impl<'a> GlyphIter2<'a> {
    /// New iterator.
    pub(crate) fn new(
        pos: TextPosition,
        iter: impl SkipLine<Item = Grapheme<'a>> + 'a,
        cache: GlyphCache,
    ) -> Self {
        Self {
            iter: Box::new(iter),
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

#[inline]
fn checked_screen_pos(x: upos_type, y: upos_type) -> (u16, u16) {
    (
        u16::try_from(x).expect("in-range"),
        u16::try_from(y).expect("in-range"),
    )
}

#[inline]
fn checked_screen_width(w: upos_type) -> u16 {
    u16::try_from(w).expect("in-range")
}

impl<'a> Iterator for GlyphIter2<'a> {
    type Item = Glyph2<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        if let Some(glyph) = self.next_glyph.take() {
            self.last_pos = glyph.pos;
            self.last_byte = glyph.text_bytes.end;

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
                    screen_pos: checked_screen_pos(
                        self.next_screen_pos.0.saturating_sub(self.left_margin),
                        self.next_screen_pos.1,
                    ),
                    screen_width: if self.show_ctrl { 1 } else { 0 },
                    line_break: true,
                    pos: self.next_pos,
                };

                return Some(glyph);
            };

            let (mut grapheme, mut grapheme_bytes) = grapheme.into_parts();
            let mut screen_pos = (self.next_screen_pos.0, self.next_screen_pos.1);
            let mut screen_width;
            let mut line_break;
            let mut pos = self.next_pos;

            // remap grapheme
            if grapheme == "\n" || grapheme == "\r\n" {
                if self.line_break {
                    line_break = true;
                    screen_width = if self.show_ctrl { 1 } else { 0 };
                    grapheme = Cow::Borrowed(if self.show_ctrl { "\u{240A}" } else { "" });
                } else {
                    line_break = false;
                    screen_width = 1;
                    grapheme = Cow::Borrowed("\u{240A}");
                }
            } else if grapheme == "\t" {
                line_break = false;
                screen_width = self.tabs - (self.next_screen_pos.0 % self.tabs);
                grapheme = Cow::Borrowed(if self.show_ctrl { "\u{2409}" } else { " " });
            } else if ("\x00".."\x20").contains(&grapheme.as_ref()) {
                line_break = false;
                screen_width = 1;

                // Control char unicode display replacement.
                static CONTROL_CHARS: [&str; 32] = [
                    "\u{2400}", "\u{2401}", "\u{2402}", "\u{2403}", "\u{2404}", "\u{2405}",
                    "\u{2406}", "\u{2407}", "\u{2408}", "\u{2409}", "\u{240A}", "\u{240B}",
                    "\u{240C}", "\u{240D}", "\u{240E}", "\u{240F}", "\u{2410}", "\u{2411}",
                    "\u{2412}", "\u{2413}", "\u{2414}", "\u{2415}", "\u{2416}", "\u{2417}",
                    "\u{2418}", "\u{2419}", "\u{241A}", "\u{241B}", "\u{241C}", "\u{241D}",
                    "\u{241E}", "\u{241F}",
                ];

                grapheme = Cow::Borrowed(if self.show_ctrl {
                    CONTROL_CHARS[grapheme.as_bytes()[0] as usize]
                } else {
                    "\u{FFFD}"
                });
            } else {
                line_break = false;
                screen_width = unicode_display_width::width(&grapheme) as upos_type;
                grapheme = grapheme;
            }

            // next glyph positioning
            if let TextWrap2::ShiftText = self.text_wrap {
                // Clip glyphs and correct left offset
                if line_break {
                    self.next_screen_pos.0 = 0;
                    self.next_screen_pos.1 += 1;
                    self.next_pos.x = 0;
                    self.next_pos.y += 1;
                    self.last_pos = pos;
                    self.last_byte = grapheme_bytes.end;

                    if screen_pos.0 <= self.true_right_margin() {
                        screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);

                        return Some(Glyph2 {
                            glyph: grapheme,
                            text_bytes: grapheme_bytes,
                            screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                            screen_width: checked_screen_width(screen_width),
                            line_break,
                            pos,
                        });
                    } else {
                        // shouldn't happen with skip_line?
                        unreachable!("line-break should have been skipped");
                    }
                } else if screen_pos.0 < self.left_margin
                    && screen_pos.0 + screen_width > self.left_margin
                {
                    // show replacement for split glyph
                    grapheme = Cow::Borrowed("\u{2426}");
                    screen_width = self.next_screen_pos.0 + screen_width - self.left_margin;
                    screen_pos.0 = 0;

                    // cache line start position.
                    self.cache.line_start.borrow_mut().insert(
                        pos.y,
                        GlyphCacheLine {
                            pos_x: pos.x,
                            screen_pos_x: self.next_screen_pos.0,
                            byte_pos: grapheme_bytes.start,
                        },
                    );

                    self.next_screen_pos.0 += screen_width as upos_type;
                    // self.next_screen_pos.1 doesn't change
                    self.next_pos.x += 1;
                    // next_pos.y doesn't change
                    self.last_pos = pos;
                    self.last_byte = grapheme_bytes.end;

                    return Some(Glyph2 {
                        glyph: grapheme,
                        text_bytes: grapheme_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width: checked_screen_width(screen_width),
                        line_break,
                        pos,
                    });
                } else if screen_pos.0 < self.left_margin {
                    self.next_screen_pos.0 += screen_width as upos_type;
                    // self.next_screen_pos.1 doesn't change
                    self.next_pos.x += 1;
                    // next_pos.y doesn't change
                    self.last_pos = pos;
                    self.last_byte = grapheme_bytes.end;

                    if let Some(cached) = self.cache.line_start.borrow().get(&pos.y) {
                        self.iter.skip_to(cached.byte_pos).expect("valid-pos");
                        self.next_pos.x = cached.pos_x;
                        self.next_screen_pos.0 = cached.screen_pos_x;
                        continue;
                    } else {
                        // not yet cached. go the long way.
                        continue;
                    }
                } else if screen_pos.0 == self.true_right_margin() {
                    line_break = true;
                    screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);
                    screen_width = if self.show_ctrl { 1 } else { 0 };
                    grapheme = Cow::Borrowed(if self.show_ctrl { "\u{2424}" } else { "" });
                    grapheme_bytes = self.last_byte..self.last_byte;

                    self.next_screen_pos.0 += 1;
                    // self.next_screen_pos.1 doesn't change
                    self.next_pos.x += 1;
                    // next_pos.y doesn't change
                    self.last_pos = pos;
                    self.last_byte = grapheme_bytes.end;

                    return Some(Glyph2 {
                        glyph: grapheme,
                        text_bytes: grapheme_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width: checked_screen_width(screen_width),
                        line_break,
                        pos,
                    });
                } else if self.next_screen_pos.0 > self.true_right_margin() {
                    // skip to next_line
                    self.iter.skip_line().expect("fine");

                    // do the line-break here.
                    self.next_screen_pos.0 = 0;
                    self.next_screen_pos.1 += 1;
                    self.next_pos.x = 0;
                    self.next_pos.y += 1;
                    self.last_pos = pos;
                    self.last_byte = grapheme_bytes.end;

                    continue;
                } else if self.next_screen_pos.0 < self.true_right_margin()
                    && self.next_screen_pos.0 + screen_width as upos_type > self.true_right_margin()
                {
                    // show replacement for split glyph
                    grapheme = Cow::Borrowed("\u{2426}");
                    screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);
                    screen_width = self.next_screen_pos.0 + screen_width - self.right_margin;

                    self.next_screen_pos.0 += screen_width as upos_type;
                    // self.next_screen_pos.1 doesn't change
                    self.next_pos.x += 1;
                    // next_pos.y doesn't change
                    self.last_pos = pos;
                    self.last_byte = grapheme_bytes.end;

                    return Some(Glyph2 {
                        glyph: grapheme,
                        text_bytes: grapheme_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width: checked_screen_width(screen_width),
                        line_break,
                        pos,
                    });
                } else {
                    screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);

                    if screen_pos.0 == 0 {
                        self.cache.line_start.borrow_mut().insert(
                            pos.y,
                            GlyphCacheLine {
                                pos_x: pos.x,
                                screen_pos_x: self.next_screen_pos.0,
                                byte_pos: grapheme_bytes.start,
                            },
                        );
                    }

                    self.next_screen_pos.0 += screen_width as upos_type;
                    // self.next_screen_pos.1 doesn't change
                    self.next_pos.x += 1;
                    // next_pos.y doesn't change
                    self.last_pos = pos;
                    self.last_byte = grapheme_bytes.end;

                    return Some(Glyph2 {
                        glyph: grapheme,
                        text_bytes: grapheme_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width: checked_screen_width(screen_width),
                        line_break,
                        pos,
                    });
                }
            } else if let TextWrap2::BreakText = self.text_wrap {
                if line_break {
                    // new-line

                    self.next_screen_pos.0 = 0;
                    self.next_screen_pos.1 += 1;
                    self.next_pos.x = 0;
                    self.next_pos.y += 1;
                    self.last_pos = pos;
                    self.last_byte = grapheme_bytes.end;

                    return Some(Glyph2 {
                        glyph: grapheme,
                        text_bytes: grapheme_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width: checked_screen_width(screen_width),
                        line_break,
                        pos,
                    });
                } else if screen_pos.0 + screen_width as upos_type > self.true_right_margin() {
                    // break before glyph

                    self.next_screen_pos.0 = screen_width as upos_type;
                    self.next_screen_pos.1 += 1;
                    self.next_pos.x += 1;
                    // next_pos.y doesn't change
                    self.last_pos = pos;
                    self.last_byte = grapheme_bytes.end;

                    self.next_glyph = Some(Glyph2 {
                        glyph: Cow::Owned(grapheme.to_string()),
                        text_bytes: grapheme_bytes.clone(),
                        screen_pos: checked_screen_pos(0, screen_pos.1 + 1),
                        screen_width: checked_screen_width(screen_width),
                        line_break: false,
                        pos,
                    });

                    grapheme = if self.show_ctrl {
                        Cow::Borrowed("\u{2424}")
                    } else {
                        Cow::Borrowed("")
                    };
                    grapheme_bytes = grapheme_bytes.start..grapheme_bytes.start;
                    // screen_pos is ok
                    screen_width = if self.show_ctrl { 1 } else { 0 };
                    line_break = true;
                    pos = self.last_pos;

                    return Some(Glyph2 {
                        glyph: grapheme,
                        text_bytes: grapheme_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width: checked_screen_width(screen_width),
                        line_break,
                        pos,
                    });
                } else if screen_pos.0 > self.word_margin && grapheme == " " {
                    // break after space

                    self.next_screen_pos.0 = 0;
                    self.next_screen_pos.1 += 1;
                    self.next_pos.x += 1;
                    // next_pos.y doesn't change
                    self.last_pos = pos;
                    self.last_byte = grapheme_bytes.end;

                    self.next_glyph = Some(Glyph2 {
                        glyph: if self.show_ctrl {
                            Cow::Borrowed("\u{2424}")
                        } else {
                            Cow::Borrowed("")
                        },
                        text_bytes: grapheme_bytes.end..grapheme_bytes.end,
                        screen_pos: checked_screen_pos(screen_pos.0 + 1, screen_pos.1),
                        screen_width: if self.show_ctrl { 1 } else { 0 },
                        line_break: true,
                        pos,
                    });

                    return Some(Glyph2 {
                        glyph: grapheme,
                        text_bytes: grapheme_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width: checked_screen_width(screen_width),
                        line_break,
                        pos,
                    });
                } else {
                    self.next_screen_pos.0 += screen_width as upos_type;
                    // next_screen_pos.1 doesn't change
                    self.next_pos.x += 1;
                    // next_pos.1 doesn't change
                    self.last_pos = pos;
                    self.last_byte = grapheme_bytes.end;

                    return Some(Glyph2 {
                        glyph: grapheme,
                        text_bytes: grapheme_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width: checked_screen_width(screen_width),
                        line_break,
                        pos,
                    });
                }
            } else {
                unreachable!()
            }
        }
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
