use crate::text_store::SkipLine;
use crate::{upos_type, Grapheme, TextPosition};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::ops::Range;

/// Data for rendering/mapping graphemes to screen coordinates.
#[derive(Debug)]
pub(crate) struct Glyph2<'a> {
    /// Display glyph.
    glyph: Cow<'a, str>,
    /// byte-range of the glyph in the given slice.
    text_bytes: Range<usize>,
    /// screen-position corrected by text_offset.
    /// first visible column is at 0.
    screen_pos: (u16, u16),
    /// Display length for the glyph.
    screen_width: u16,
    /// Last item in this screen-line.
    line_break: bool,
    /// text-position
    pos: TextPosition,
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
pub enum TextWrap2 {
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
    pub(crate) fn new(pos: TextPosition, iter: impl SkipLine<Item = Grapheme<'a>> + 'a) -> Self {
        Self {
            iter: Box::new(iter),
            done: Default::default(),
            next_pos: pos,
            next_screen_pos: Default::default(),
            last_pos: Default::default(),
            last_byte: Default::default(),
            next_glyph: Default::default(),
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

    pub(crate) fn set_word_margin(&mut self, word_margin: upos_type) {
        self.word_margin = word_margin;
    }
}

fn checked_screen_pos(x: upos_type, y: upos_type) -> (u16, u16) {
    (
        u16::try_from(x).expect("in-range"),
        u16::try_from(y).expect("in-range"),
    )
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

            let (grapheme, grapheme_bytes) = grapheme.into_parts();

            let mut glyph;
            let mut text_bytes = grapheme_bytes;
            let mut screen_pos = (self.next_screen_pos.0, self.next_screen_pos.1);
            let mut screen_width;
            let mut line_break;
            let mut pos = self.next_pos;

            // remap grapheme
            if grapheme == "\n" || grapheme == "\r\n" {
                if self.line_break {
                    line_break = true;
                    screen_width = if self.show_ctrl { 1 } else { 0 };
                    glyph = Cow::Borrowed(if self.show_ctrl { "\u{240A}" } else { "" });
                } else {
                    line_break = false;
                    screen_width = 1;
                    glyph = Cow::Borrowed("\u{240A}");
                }
            } else if grapheme == "\t" {
                line_break = false;
                screen_width = (self.tabs - (self.next_screen_pos.0 % self.tabs)) as u16;
                glyph = Cow::Borrowed(if self.show_ctrl { "\u{2409}" } else { " " });
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

                glyph = Cow::Borrowed(if self.show_ctrl {
                    CONTROL_CHARS[grapheme.as_bytes()[0] as usize]
                } else {
                    "\u{FFFD}"
                });
            } else {
                line_break = false;
                screen_width = unicode_display_width::width(&grapheme) as u16;
                glyph = grapheme;
            }

            // next glyph positioning
            if let TextWrap2::ShiftText = self.text_wrap {
                let right_margin = if self.show_ctrl {
                    self.right_margin.saturating_sub(1)
                } else {
                    self.right_margin
                };

                // self.next_screen_pos later
                self.next_pos.x += 1;
                // next_pos.y doesn't change

                // Clip glyphs and correct left offset
                if line_break {
                    self.next_screen_pos.0 = 0;
                    self.next_screen_pos.1 += 1;
                    self.next_pos.x = 0;
                    self.next_pos.y += 1;

                    if screen_pos.0 as upos_type <= right_margin {
                        screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);

                        self.last_pos = pos;
                        self.last_byte = text_bytes.end;

                        return Some(Glyph2 {
                            glyph,
                            text_bytes,
                            screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                            screen_width,
                            line_break,
                            pos,
                        });
                    } else {
                        continue;
                    }
                } else if self.next_screen_pos.0 < self.left_margin
                    && self.next_screen_pos.0 + screen_width as upos_type > self.left_margin
                {
                    // show replacement for split glyph
                    glyph = Cow::Borrowed("\u{2426}");
                    screen_width = (self.next_screen_pos.0 + screen_width as upos_type
                        - self.left_margin) as u16;
                    screen_pos.0 = 0;

                    self.next_screen_pos.0 += screen_width as upos_type;
                    // self.next_screen_pos.1 doesn't change

                    self.last_pos = pos;
                    self.last_byte = text_bytes.end;

                    return Some(Glyph2 {
                        glyph,
                        text_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width,
                        line_break,
                        pos,
                    });
                } else if self.next_screen_pos.0 < self.left_margin {
                    screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);

                    self.next_screen_pos.0 += screen_width as upos_type;
                    // self.next_screen_pos.1 doesn't change

                    // TODO skip glyph. maybe.
                    continue;
                } else if self.next_screen_pos.0 == right_margin {
                    line_break = true;
                    screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);
                    screen_width = if self.show_ctrl { 1 } else { 0 };
                    glyph = Cow::Borrowed(if self.show_ctrl { "\u{2424}" } else { "" });
                    text_bytes = self.last_byte..self.last_byte;

                    self.next_screen_pos.0 += 1;
                    // self.next_screen_pos.1 doesn't change

                    self.last_pos = pos;
                    self.last_byte = text_bytes.end;

                    return Some(Glyph2 {
                        glyph,
                        text_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width,
                        line_break,
                        pos,
                    });
                } else if self.next_screen_pos.0 > right_margin {
                    screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);

                    self.next_screen_pos.0 += screen_width as upos_type;
                    // self.next_screen_pos.1 doesn't change

                    // skip to next_line
                    self.iter.next_line().expect("fine");

                    // do the line-break here.
                    self.next_screen_pos.0 = 0;
                    self.next_screen_pos.1 += 1;
                    self.next_pos.x = 0;
                    self.next_pos.y += 1;

                    continue;
                } else if self.next_screen_pos.0 < right_margin
                    && self.next_screen_pos.0 + screen_width as upos_type > right_margin
                {
                    // show replacement for split glyph
                    glyph = Cow::Borrowed("\u{2426}");
                    screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);
                    screen_width = (self.next_screen_pos.0 + screen_width as upos_type
                        - self.right_margin as upos_type) as u16;

                    self.next_screen_pos.0 += screen_width as upos_type;
                    // self.next_screen_pos.1 doesn't change

                    self.last_pos = pos;
                    self.last_byte = text_bytes.end;

                    return Some(Glyph2 {
                        glyph,
                        text_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width,
                        line_break,
                        pos,
                    });
                } else {
                    screen_pos.0 = screen_pos.0.saturating_sub(self.left_margin);

                    self.next_screen_pos.0 += screen_width as upos_type;
                    // self.next_screen_pos.1 doesn't change

                    self.last_pos = pos;
                    self.last_byte = text_bytes.end;

                    return Some(Glyph2 {
                        glyph,
                        text_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width,
                        line_break,
                        pos,
                    });
                }
            } else if let TextWrap2::BreakText = self.text_wrap {
                let right_margin = if self.show_ctrl {
                    self.right_margin.saturating_sub(1)
                } else {
                    self.right_margin
                };

                // self.next_screen_pos later
                if line_break {
                    self.next_screen_pos.0 = 0;
                    self.next_screen_pos.1 += 1;
                    self.next_pos.x = 0;
                    self.next_pos.y += 1;

                    self.last_pos = pos;
                    self.last_byte = text_bytes.end;

                    return Some(Glyph2 {
                        glyph,
                        text_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width,
                        line_break,
                        pos,
                    });
                } else if screen_pos.0 + screen_width as upos_type > right_margin {
                    // break before glyph

                    // after current grapheme
                    self.next_screen_pos.0 = screen_width as upos_type;
                    self.next_screen_pos.1 += 1;
                    self.next_pos.x += 1;
                    // next_pos.y doesn't change

                    self.next_glyph = Some(Glyph2 {
                        glyph: Cow::Owned(glyph.to_string()),
                        text_bytes: text_bytes.clone(),
                        screen_pos: checked_screen_pos(0, screen_pos.1 + 1),
                        screen_width,
                        line_break: false,
                        pos,
                    });

                    glyph = if self.show_ctrl {
                        Cow::Borrowed("\u{2424}")
                    } else {
                        Cow::Borrowed("")
                    };
                    text_bytes = text_bytes.start..text_bytes.start;
                    // screen_pos is ok
                    screen_width = if self.show_ctrl { 1 } else { 0 };
                    line_break = true;
                    pos = self.last_pos;

                    self.last_pos = pos;
                    self.last_byte = text_bytes.end;

                    return Some(Glyph2 {
                        glyph,
                        text_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width,
                        line_break,
                        pos,
                    });
                } else if screen_pos.0 > self.word_margin && glyph == " " {
                    // break after space

                    self.next_screen_pos.0 = 0;
                    self.next_screen_pos.1 += 1;
                    self.next_pos.x += 1;
                    // next_pos.y doesn't change

                    self.next_glyph = Some(Glyph2 {
                        glyph: if self.show_ctrl {
                            Cow::Borrowed("\u{2424}")
                        } else {
                            Cow::Borrowed("")
                        },
                        text_bytes: text_bytes.end..text_bytes.end,
                        screen_pos: checked_screen_pos(screen_pos.0 + 1, screen_pos.1),
                        screen_width: if self.show_ctrl { 1 } else { 0 },
                        line_break: true,
                        pos,
                    });

                    self.last_pos = pos;
                    self.last_byte = text_bytes.end;

                    return Some(Glyph2 {
                        glyph,
                        text_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width,
                        line_break,
                        pos,
                    });
                } else {
                    self.next_screen_pos.0 += screen_width as upos_type;
                    self.next_pos.x += 1;

                    self.last_pos = pos;
                    self.last_byte = text_bytes.end;

                    return Some(Glyph2 {
                        glyph,
                        text_bytes,
                        screen_pos: checked_screen_pos(screen_pos.0, screen_pos.1),
                        screen_width,
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

/// Data for rendering/mapping graphemes to screen coordinates.
#[derive(Debug)]
#[deprecated]
pub struct Glyph<'a> {
    /// Display glyph.
    glyph: Cow<'a, str>,
    /// byte-range of the glyph in the given slice.
    text_bytes: Range<usize>,
    /// screen-position corrected by text_offset.
    /// first visible column is at 0.
    screen_pos: (u16, u16),
    /// Display length for the glyph.
    screen_width: u16,
    /// text-position
    pos: TextPosition,
}

#[allow(deprecated)]
impl<'a> Glyph<'a> {
    pub fn new(
        glyph: Cow<'a, str>,
        text_bytes: Range<usize>,
        screen_pos: (u16, u16),
        screen_width: u16,
        pos: TextPosition,
    ) -> Self {
        Self {
            glyph,
            text_bytes,
            screen_pos,
            screen_width,
            pos,
        }
    }

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

    /// Get the screen position of the glyph.
    pub fn screen_pos(&self) -> (u16, u16) {
        self.screen_pos
    }

    /// Display width of the glyph.
    pub fn screen_width(&self) -> u16 {
        self.screen_width
    }
}

/// Iterates over the glyphs of a row-range.
///
/// Keeps track of the text-position and the display-position on screen.
/// Does a conversion from graphemes to glyph-text and glyph-width.
///
/// This is used for rendering text, and for mapping text-positions
/// to screen-positions and vice versa.
#[derive(Debug)]
pub(crate) struct GlyphIter<Iter> {
    iter: Iter,

    pos: TextPosition,

    screen_offset: u16,
    screen_width: u16,
    screen_pos: (u16, u16),

    tabs: u16,
    show_ctrl: bool,
    line_break: bool,
}

impl<'a, Iter> GlyphIter<Iter>
where
    Iter: Iterator<Item = Grapheme<'a>>,
{
    /// New iterator.
    pub(crate) fn new(pos: TextPosition, iter: Iter) -> Self {
        Self {
            iter,
            pos,
            screen_offset: 0,
            screen_width: u16::MAX,
            screen_pos: Default::default(),
            tabs: 8,
            show_ctrl: false,
            line_break: true,
        }
    }

    /// Screen offset.
    pub(crate) fn set_screen_offset(&mut self, offset: u16) {
        self.screen_offset = offset;
    }

    /// Screen width.
    pub(crate) fn set_screen_width(&mut self, width: u16) {
        self.screen_width = width;
    }

    /// Tab width
    pub(crate) fn set_tabs(&mut self, tabs: u16) {
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
}

#[allow(deprecated)]
impl<'a, Iter> Iterator for GlyphIter<Iter>
where
    Iter: Iterator<Item = Grapheme<'a>>,
{
    type Item = Glyph<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        for grapheme in self.iter.by_ref() {
            let (grapheme, grapheme_bytes) = grapheme.into_parts();

            let glyph;
            let len: u16;
            let mut lbrk = false;

            // todo: maybe add some ligature support.

            match grapheme.as_ref() {
                "\n" | "\r\n" if self.line_break => {
                    lbrk = true;
                    len = if self.show_ctrl { 1 } else { 0 };
                    glyph = Cow::Borrowed(if self.show_ctrl { "\u{2424}" } else { "" });
                }
                "\n" | "\r\n" if !self.line_break => {
                    lbrk = false;
                    len = 1;
                    glyph = Cow::Borrowed("\u{2424}");
                }
                "\t" => {
                    len = self.tabs - (self.screen_pos.0 % self.tabs);
                    glyph = Cow::Borrowed(if self.show_ctrl { "\u{2409}" } else { " " });
                }
                c if ("\x00".."\x20").contains(&c) => {
                    static CCHAR: [&str; 32] = [
                        "\u{2400}", "\u{2401}", "\u{2402}", "\u{2403}", "\u{2404}", "\u{2405}",
                        "\u{2406}", "\u{2407}", "\u{2408}", "\u{2409}", "\u{240A}", "\u{240B}",
                        "\u{240C}", "\u{240D}", "\u{240E}", "\u{240F}", "\u{2410}", "\u{2411}",
                        "\u{2412}", "\u{2413}", "\u{2414}", "\u{2415}", "\u{2416}", "\u{2417}",
                        "\u{2418}", "\u{2419}", "\u{241A}", "\u{241B}", "\u{241C}", "\u{241D}",
                        "\u{241E}", "\u{241F}",
                    ];
                    let c0 = c.bytes().next().expect("byte");
                    len = 1;
                    glyph = Cow::Borrowed(if self.show_ctrl {
                        CCHAR[c0 as usize]
                    } else {
                        "\u{FFFD}"
                    });
                }
                c => {
                    len = unicode_display_width::width(c) as u16;
                    glyph = grapheme;
                }
            }

            let pos = self.pos;
            let screen_pos = self.screen_pos;

            if lbrk {
                self.screen_pos.0 = 0;
                self.screen_pos.1 += 1;
                self.pos.x = 0;
                self.pos.y += 1;
            } else {
                self.screen_pos.0 += len;
                self.pos.x += 1;
            }

            // clip left
            if screen_pos.0 < self.screen_offset {
                if screen_pos.0 + len > self.screen_offset {
                    // don't show partial glyphs, but show the space they need.
                    // avoids flickering when scrolling left/right.
                    return Some(Glyph {
                        glyph: Cow::Borrowed("\u{2203}"),
                        text_bytes: grapheme_bytes,
                        screen_width: screen_pos.0 + len - self.screen_offset,
                        pos,
                        screen_pos: (0, screen_pos.1),
                    });
                } else {
                    // out left
                }
            } else if screen_pos.0 + len > self.screen_offset + self.screen_width {
                if screen_pos.0 < self.screen_offset + self.screen_width {
                    // don't show partial glyphs, but show the space they need.
                    // avoids flickering when scrolling left/right.
                    return Some(Glyph {
                        glyph: Cow::Borrowed("\u{2203}"),
                        text_bytes: grapheme_bytes,
                        screen_width: screen_pos.0 + len - (self.screen_offset + self.screen_width),
                        pos,
                        screen_pos: (screen_pos.0 - self.screen_offset, screen_pos.1),
                    });
                } else {
                    // out right
                    if !self.line_break {
                        break;
                    }
                }
            } else {
                return Some(Glyph {
                    glyph,
                    text_bytes: grapheme_bytes,
                    screen_width: len,
                    pos,
                    screen_pos: (screen_pos.0 - self.screen_offset, screen_pos.1),
                });
            }
        }

        None
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
