use crate::cache::{Cache, LineBreakCache, LineOffsetCache};
use crate::text_store::SkipLine;
use crate::{Grapheme, TextError, TextPosition, upos_type};
use log::debug;
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::ControlFlow::{Break, Continue};
use std::ops::{ControlFlow, Range};

/// Data for rendering/mapping graphemes to screen coordinates.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Glyph2<'a> {
    /// Display glyph.
    glyph: Cow<'a, str>,
    /// byte-range of the glyph in the given slice.
    text_bytes: Range<usize>,

    /// screen-position corrected by text_offset.
    /// first visible column is at 0.
    /// Warning: this is not an upos_type, but an u16.
    screen_pos: (upos_type, upos_type),
    /// Display length for the glyph.
    screen_width: upos_type,
    /// text-position
    pos: TextPosition,

    /// Last item in this screen-line.
    line_break: bool,
    /// Is the line-break a soft-break used for text-wrapping.
    soft_break: bool,
    /// Is this a Unicode character for a hidden word-break.
    hidden_break: bool,
    /// The replacement glyph in case that a word-break happens.
    hidden_glyph: Cow<'a, str>,
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

    /// Possible soft-break.
    pub fn soft_break(&self) -> bool {
        self.soft_break
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub(crate) struct GlyphIter2<'a, Graphemes> {
    iter: Graphemes,
    /// Sometimes one grapheme creates two glyphs.
    emit: Option<Glyph2<'static>>,
    /// Iteration is done.
    done: bool,

    /// Next glyph position.
    next_pos: TextPosition,
    /// Next glyph screen position.
    next_screen_pos: (upos_type, upos_type),
    /// Next glyph start byte position.
    next_byte: usize,

    /// Last glyph was a line-break.
    was_lf: bool,

    /// Glyph cache
    cache: Cache,

    /// Tab expansion
    tabs: upos_type,
    /// Show CTRL chars
    show_ctrl: bool,
    /// Show TEXT-WRAP glyphs
    wrap_ctrl: bool,
    /// Line-break enabled?
    lf_breaks: bool,
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
            .field("next_glyph", &self.emit)
            .field("next_pos", &self.next_pos)
            .field("next_screen_pos", &self.next_screen_pos)
            .field("next_byte", &self.next_byte)
            .field("was_lf", &self.was_lf)
            .field("tabs", &self.tabs)
            .field("show_ctrl", &self.show_ctrl)
            .field("wrap_ctrl", &self.wrap_ctrl)
            .field("lf_breaks", &self.lf_breaks)
            .field("text_wrap", &self.text_wrap)
            .field("left_margin", &self.left_margin)
            .field("right_margin", &self.right_margin)
            .field("word_margin", &self.word_margin)
            .finish()
    }
}

impl<'a, Graphemes> GlyphIter2<'a, Graphemes>
where
    Graphemes: SkipLine + Iterator<Item = Grapheme<'a>> + Clone,
{
    /// New iterator.
    pub(crate) fn new(pos: TextPosition, byte: usize, iter: Graphemes, cache: Cache) -> Self {
        Self {
            iter,
            done: Default::default(),
            emit: Default::default(),

            next_pos: pos,
            next_screen_pos: (0, 0),
            next_byte: byte,
            was_lf: true,

            cache,

            tabs: 8,
            show_ctrl: false,
            wrap_ctrl: false,
            lf_breaks: true,
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
    pub(crate) fn set_lf_breaks(&mut self, line_break: bool) {
        self.lf_breaks = line_break;
    }

    /// Show ASCII control codes.
    pub(crate) fn set_show_ctrl(&mut self, show_ctrl: bool) {
        self.show_ctrl = show_ctrl;
    }

    /// Show glyphs for text-breaks.
    pub(crate) fn set_wrap_ctrl(&mut self, wrap_ctrl: bool) {
        self.wrap_ctrl = wrap_ctrl;
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
        if self.show_ctrl || self.wrap_ctrl {
            self.right_margin.saturating_sub(1)
        } else {
            self.right_margin
        }
    }

    pub(crate) fn set_word_margin(&mut self, word_margin: upos_type) {
        self.word_margin = word_margin;
    }

    /// Build cache before running the iterator.
    pub(crate) fn prepare(&mut self) -> Result<(), TextError> {
        // debug!("glyph2 param {:?}", self);
        let r = match self.text_wrap {
            TextWrap2::Shift => prepare_shift_clip(self),
            TextWrap2::Hard => prepare_hard_wrap(self),
            TextWrap2::Word => prepare_word_wrap(self),
        };
        // debug!("glyph2 cache {:#?}", self.cache);
        r
    }
}

impl<'a, Graphemes> Iterator for GlyphIter2<'a, Graphemes>
where
    Graphemes: SkipLine + Iterator<Item = Grapheme<'a>> + Clone,
{
    type Item = Glyph2<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        if let Some(emit) = self.emit.take() {
            // self.next_pos already correct at this point.
            // self.next_screen_pos already correct at this point.
            self.next_byte = emit.text_bytes.end;
            self.was_lf = emit.line_break;
            emit.validate();
            return Some(emit);
        }

        loop {
            let grapheme = self.iter.next();

            let grapheme = if let Some(grapheme) = grapheme {
                grapheme
            } else {
                self.done = true;
                if self.was_lf {
                    return None;
                } else {
                    debug!("next auto-nl {}", self.next_byte);
                    Grapheme::new(Cow::Borrowed("\n"), self.next_byte..self.next_byte)
                }
            };

            let glyph = create_glyph(
                self, //
                grapheme,
                self.next_pos,
                self.next_screen_pos,
            );

            // next glyph positioning
            let r = match self.text_wrap {
                TextWrap2::Shift => shift_clip_next(self, glyph),
                TextWrap2::Hard => wrap_next(self, glyph),
                TextWrap2::Word => wrap_next(self, glyph),
            };

            match r {
                Continue(_) => continue,
                Break(glyph) => return glyph,
            }
        }
    }
}

fn prepare_word_wrap<'a, Graphemes>(glyphs: &mut GlyphIter2<'a, Graphemes>) -> Result<(), TextError>
where
    Graphemes: Iterator<Item = Grapheme<'a>> + SkipLine + Clone,
{
    let mut iter = glyphs.iter.clone();
    let cache = &glyphs.cache;

    // Next glyph position.
    let mut next_pos = glyphs.next_pos;
    let mut next_screen_x = glyphs.next_screen_pos.0;
    let mut next_byte = glyphs.next_byte;
    let mut was_lf = true;

    // Last space seen.
    let mut space_pos: Option<TextPosition> = None;
    let mut space_screen_x = None;
    let mut space_byte = None;

    // Col 0 has been seen.
    let mut see_zero_col = None;

    loop {
        let grapheme = iter.next();
        // debug!("*glyph2 prepare word {:?}", grapheme);
        let grapheme = if let Some(grapheme) = grapheme {
            grapheme
        } else {
            if was_lf {
                break;
            } else {
                debug!("glyph2 auto-nl {}", next_byte);
                Grapheme::new(Cow::Borrowed("\n"), next_byte..next_byte)
            }
        };

        let glyph = create_glyph(
            glyphs, //
            grapheme,
            next_pos,
            (next_screen_x, 0),
        );

        // row-start seen
        if glyph.pos.x == 0 {
            see_zero_col = Some(glyph.pos.y);
        }

        // Is the current row already cached?
        // Guard: Do nothing for empty rows.
        if !glyph.line_break {
            if let Some(lb) = cache.full_line_break.borrow().get(&glyph.pos.y) {
                // debug!("glyph2 reuse break {:?} {:?}", glyph.pos, lb);

                iter.skip_line()?;

                next_screen_x = 0;
                next_pos.x = 0;
                next_pos.y += 1;
                next_byte = lb.byte_pos;
                was_lf = true;
                (space_pos, space_screen_x, space_byte) = (None, None, None);
                see_zero_col = None;

                continue;
            }
        }

        if glyph.line_break {
            next_screen_x = 0;
            next_pos.x = 0;
            next_pos.y += 1;
            next_byte = glyph.text_bytes.end;
            was_lf = true;

            // caching
            cache.line_break.borrow_mut().insert(
                glyph.pos,
                LineBreakCache {
                    start_pos: next_pos,
                    byte_pos: glyph.text_bytes.end,
                },
            );
            if Some(glyph.pos.y) == see_zero_col {
                cache.full_line_break.borrow_mut().insert(
                    glyph.pos.y,
                    LineBreakCache {
                        start_pos: next_pos,
                        byte_pos: glyph.text_bytes.end,
                    },
                );
            } else {
                cache.full_line_break.borrow_mut().remove(&glyph.pos.y);
            }

            (space_pos, space_screen_x, space_byte) = (None, None, None);
        } else if glyph.screen_pos.0 > glyphs.word_margin
            && (glyph.glyph == " " || glyph.glyph == "-" || glyph.hidden_break)
        {
            // break after space

            next_screen_x = 0;
            next_pos.x += 1;
            // next_pos.y doesn't change
            next_byte = glyph.text_bytes.end;
            was_lf = false;

            // caching
            cache.line_break.borrow_mut().insert(
                glyph.pos,
                LineBreakCache {
                    start_pos: next_pos,
                    byte_pos: glyph.text_bytes.end,
                },
            );

            (space_pos, space_screen_x, space_byte) = (None, None, None);
        } else if glyph.screen_pos.0 + glyph.screen_width >= glyphs.true_right_margin() {
            // break at last space before

            if let (Some(space_screen_pos), Some(space_pos), Some(space_byte)) =
                (space_screen_x, space_pos, space_byte)
            {
                next_screen_x = glyph.screen_pos.0 - space_screen_pos;
                next_pos.x += 1;
                // next_pos.y doesn't change
                next_byte = glyph.text_bytes.end;
                was_lf = false;

                // caching
                cache.line_break.borrow_mut().insert(
                    space_pos,
                    LineBreakCache {
                        start_pos: TextPosition::new(space_pos.x + 1, space_pos.y),
                        byte_pos: space_byte,
                    },
                );
            } else {
                // no space on this text-row. hard-break.

                // next glyph positioning
                next_screen_x = 0;
                next_pos.x += 1;
                // next_pos.y doesn't change
                next_byte = glyph.text_bytes.end;
                was_lf = false;

                // caching
                cache.line_break.borrow_mut().insert(
                    glyph.pos,
                    LineBreakCache {
                        start_pos: next_pos,
                        byte_pos: glyph.text_bytes.start,
                    },
                );
            }

            (space_pos, space_screen_x, space_byte) = (None, None, None);
        } else {
            next_screen_x += glyph.screen_width as upos_type;
            // next_screen_pos.1 doesn't change
            next_pos.x += 1;
            // next_pos.1 doesn't change
            next_byte = glyph.text_bytes.end;
            was_lf = false;

            if glyph.glyph == " " || glyph.glyph == "-" || glyph.hidden_break {
                space_pos = Some(glyph.pos);
                space_screen_x = Some(glyph.screen_pos.0);
                space_byte = Some(glyph.text_bytes.end);
            }
        }
    }

    Ok(())
}

fn wrap_next<'a, Graphemes>(
    glyphs: &mut GlyphIter2<'a, Graphemes>,
    mut glyph: Glyph2<'a>,
) -> ControlFlow<Option<Glyph2<'a>>>
where
    Graphemes: Iterator<Item = Grapheme<'a>> + SkipLine + Clone,
{
    let cache = &glyphs.cache;

    if glyph.line_break {
        // new-line
        assert!(cache.line_break.borrow().contains_key(&glyph.pos));

        glyphs.next_screen_pos.0 = 0;
        glyphs.next_screen_pos.1 += 1;
        glyphs.next_pos.x = 0;
        glyphs.next_pos.y += 1;
        glyphs.next_byte = glyph.text_bytes.end;
        glyphs.was_lf = glyph.line_break;

        glyph.validate();

        Break(Some(glyph))
    } else if cache.line_break.borrow().contains_key(&glyph.pos) {
        // found a line-break

        glyphs.next_screen_pos.0 = 0;
        glyphs.next_screen_pos.1 += 1;
        glyphs.next_pos.x += 1;
        // next_pos.y doesn't change
        glyphs.next_byte = glyph.text_bytes.end;
        glyphs.was_lf = glyph.line_break;

        if glyphs.wrap_ctrl {
            // show soft-break
            glyphs.emit = Some(Glyph2 {
                glyph: Cow::Borrowed("\u{21B5}"),
                text_bytes: glyph.text_bytes.end..glyph.text_bytes.end,
                screen_pos: (glyph.screen_pos.0 + 1, glyph.screen_pos.1),
                screen_width: 1,
                line_break: true,
                soft_break: true,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
                pos: glyph.pos,
            });
        } else {
            glyph.line_break = true;
            glyph.soft_break = true;
        }

        if glyph.hidden_break {
            glyph.screen_width = 1;
            glyph.glyph = glyph.hidden_glyph.clone();
        }

        glyph.validate();

        Break(Some(glyph))
    } else {
        glyphs.next_screen_pos.0 += glyph.screen_width as upos_type;
        // next_screen_pos.1 doesn't change
        glyphs.next_pos.x += 1;
        // next_pos.1 doesn't change
        glyphs.next_byte = glyph.text_bytes.end;
        glyphs.was_lf = glyph.line_break;

        glyph.validate();

        Break(Some(glyph))
    }
}

fn prepare_hard_wrap<'a, Graphemes>(glyphs: &mut GlyphIter2<'a, Graphemes>) -> Result<(), TextError>
where
    Graphemes: Iterator<Item = Grapheme<'a>> + SkipLine + Clone,
{
    let mut iter = glyphs.iter.clone();
    let cache = &glyphs.cache;

    // Next glyph position.
    let mut next_pos = glyphs.next_pos;
    let mut next_screen_x = glyphs.next_screen_pos.0;
    let mut next_byte = glyphs.next_byte;
    let mut was_lf = true;

    // Col 0 has been seen.
    let mut see_zero_col = None;

    loop {
        let grapheme = iter.next();
        let grapheme = if let Some(grapheme) = grapheme {
            grapheme
        } else {
            if was_lf {
                break;
            } else {
                Grapheme::new(Cow::Borrowed("\n"), next_byte..next_byte)
            }
        };

        let glyph = create_glyph(
            glyphs, //
            grapheme,
            next_pos,
            (next_screen_x, 0),
        );

        // row-start seen
        if glyph.pos.x == 0 {
            see_zero_col = Some(glyph.pos.y);
        }

        // Is the current row already cached?
        // Guard: Do nothing for empty rows.
        if !glyph.line_break {
            if let Some(lb) = cache.full_line_break.borrow().get(&glyph.pos.y) {
                iter.skip_line()?;

                next_screen_x = 0;
                next_pos.x = 0;
                next_pos.y += 1;
                next_byte = lb.byte_pos;
                was_lf = true;
                see_zero_col = None;

                continue;
            }
        }

        if glyph.line_break {
            next_screen_x = 0;
            next_pos.x = 0;
            next_pos.y += 1;
            next_byte = glyph.text_bytes.end;
            was_lf = true;

            // caching
            cache.line_break.borrow_mut().insert(
                glyph.pos,
                LineBreakCache {
                    start_pos: next_pos,
                    byte_pos: glyph.text_bytes.end,
                },
            );
            if Some(glyph.pos.y) == see_zero_col {
                cache.full_line_break.borrow_mut().insert(
                    glyph.pos.y,
                    LineBreakCache {
                        start_pos: next_pos,
                        byte_pos: glyph.text_bytes.end,
                    },
                );
            } else {
                cache.full_line_break.borrow_mut().remove(&glyph.pos.y);
            }
        } else if glyph.screen_pos.0 + glyph.screen_width >= glyphs.true_right_margin() {
            // break before glyph

            // next glyph positioning
            next_screen_x = 0;
            next_pos.x += 1;
            // next_pos.y doesn't change
            next_byte = glyph.text_bytes.end;
            was_lf = false;

            // caching
            cache.line_break.borrow_mut().insert(
                glyph.pos,
                LineBreakCache {
                    start_pos: next_pos,
                    byte_pos: glyph.text_bytes.start,
                },
            );
        } else {
            next_screen_x += glyph.screen_width as upos_type;
            // next_screen_pos.1 doesn't change
            next_pos.x += 1;
            // next_pos.1 doesn't change
            next_byte = glyph.text_bytes.end;
            was_lf = false;
        }
    }

    Ok(())
}

fn prepare_shift_clip<'a, Graphemes>(
    glyphs: &mut GlyphIter2<'a, Graphemes>,
) -> Result<(), TextError>
where
    Graphemes: SkipLine + Iterator<Item = Grapheme<'a>> + Clone,
{
    let mut iter = glyphs.iter.clone();
    let cache = &glyphs.cache;

    // Next glyph position.
    let mut next_pos = glyphs.next_pos;
    let mut next_byte = glyphs.next_byte;
    let mut was_lf = true;

    // fill in missing line-break data.
    loop {
        let grapheme = iter.next();
        let grapheme = if let Some(grapheme) = grapheme {
            grapheme
        } else {
            if was_lf {
                break;
            } else {
                Grapheme::new(Cow::Borrowed("\n"), next_byte..next_byte)
            }
        };

        let glyph = create_glyph(
            glyphs, //
            grapheme,
            next_pos,
            (0, 0),
        );

        if !glyph.line_break {
            if let Some(lb) = cache.full_line_break.borrow().get(&next_pos.y) {
                iter.skip_line()?;

                next_pos.x = 0;
                next_pos.y += 1;
                next_byte = lb.byte_pos;
                was_lf = true;

                continue;
            }
        }

        if glyph.line_break {
            next_pos.x = 0;
            next_pos.y += 1;
            next_byte = glyph.text_bytes.end;
            was_lf = true;

            cache.line_break.borrow_mut().insert(
                glyph.pos,
                LineBreakCache {
                    start_pos: next_pos,
                    byte_pos: glyph.text_bytes.end,
                },
            );
            cache.full_line_break.borrow_mut().insert(
                glyph.pos.y,
                LineBreakCache {
                    start_pos: next_pos,
                    byte_pos: glyph.text_bytes.end,
                },
            );
        } else {
            next_pos.x += 1;
            // iter.next_pos.y doesn't change.
            next_byte = glyph.text_bytes.end;
            was_lf = false;
        }
    }

    Ok(())
}

fn shift_clip_next<'a, Graphemes>(
    glyphs: &mut GlyphIter2<'a, Graphemes>,
    mut glyph: Glyph2<'a>,
) -> ControlFlow<Option<Glyph2<'a>>>
where
    Graphemes: SkipLine + Iterator<Item = Grapheme<'a>> + Clone,
{
    let cache = &glyphs.cache;

    // Clip glyphs and correct left offset

    if glyph.line_break {
        glyphs.next_screen_pos.0 = 0;
        glyphs.next_screen_pos.1 += 1;
        glyphs.next_pos.x = 0;
        glyphs.next_pos.y += 1;
        glyphs.next_byte = glyph.text_bytes.end;
        glyphs.was_lf = glyph.line_break;

        // a line-break just beyond the right margin will end up here.
        // every other line-break will be skipped instead.
        if glyph.screen_pos.0 <= glyphs.true_right_margin() + 1 {
            glyph.screen_pos.0 = glyph.screen_pos.0.saturating_sub(glyphs.left_margin);
            glyph.validate();

            Break(Some(glyph))
        } else {
            // shouldn't happen with skip_line?
            unreachable!("line-break should have been skipped");
        }
    } else if glyph.screen_pos.0 < glyphs.left_margin
        && glyph.screen_pos.0 + glyph.screen_width > glyphs.left_margin
    {
        // glyph crossing the left margin

        // show replacement for split glyph
        glyph.glyph = Cow::Borrowed("\u{2426}");
        glyph.screen_width = glyphs.next_screen_pos.0 + glyph.screen_width - glyphs.left_margin;
        glyph.screen_pos.0 = 0;

        // cache line start position.
        cache.line_start.borrow_mut().insert(
            glyph.pos.y,
            LineOffsetCache {
                pos_x: glyph.pos.x,
                screen_pos_x: glyphs.next_screen_pos.0,
                byte_pos: glyph.text_bytes.start,
            },
        );

        glyphs.next_screen_pos.0 += glyph.screen_width;
        // iter.next_screen_pos.1 doesn't change
        glyphs.next_pos.x += 1;
        // next_pos.y doesn't change
        glyphs.next_byte = glyph.text_bytes.end;
        glyphs.was_lf = glyph.line_break;

        glyph.validate();

        Break(Some(glyph))
    } else if glyph.screen_pos.0 < glyphs.left_margin {
        // glyph before the left margin

        glyphs.next_screen_pos.0 += glyph.screen_width;
        // iter.next_screen_pos.1 doesn't change
        glyphs.next_pos.x += 1;
        // next_pos.y doesn't change
        glyphs.next_byte = glyph.text_bytes.end;
        glyphs.was_lf = glyph.line_break;

        if let Some(cached) = cache.line_start.borrow().get(&glyph.pos.y) {
            glyphs.iter.skip_to(cached.byte_pos).expect("valid-pos");
            glyphs.next_pos.x = cached.pos_x;
            glyphs.next_screen_pos.0 = cached.screen_pos_x;
            Continue(())
        } else {
            // not yet cached. go the long way.
            Continue(())
        }
    } else if glyphs.next_screen_pos.0 < glyphs.true_right_margin()
        && glyphs.next_screen_pos.0 + glyph.screen_width as upos_type > glyphs.true_right_margin()
    {
        // glyph crosses the right margin

        // show replacement for split glyph
        glyph.glyph = Cow::Borrowed("\u{2426}");
        glyph.screen_pos.0 = glyph.screen_pos.0.saturating_sub(glyphs.left_margin);
        glyph.screen_width = glyphs.next_screen_pos.0 + glyph.screen_width - glyphs.right_margin;

        glyphs.next_screen_pos.0 += glyph.screen_width as upos_type;
        // iter.next_screen_pos.1 doesn't change
        glyphs.next_pos.x += 1;
        // next_pos.y doesn't change
        glyphs.next_byte = glyph.text_bytes.end;
        glyphs.was_lf = glyph.line_break;

        glyph.validate();

        Break(Some(glyph))
    } else if glyph.screen_pos.0 == glyphs.true_right_margin() {
        // glyph exactly at right margin

        // drop glyph and emit a line-break instead.
        glyph.line_break = true;
        glyph.screen_pos.0 = glyph.screen_pos.0.saturating_sub(glyphs.left_margin);
        glyph.screen_width = if glyphs.wrap_ctrl { 1 } else { 0 };
        glyph.glyph = Cow::Borrowed(if glyphs.wrap_ctrl { "\u{2424}" } else { "" });
        glyph.text_bytes = glyphs.next_byte..glyphs.next_byte;

        // still advance x, next iteration will skip the rest of the line.
        glyphs.next_screen_pos.0 += 1;
        // iter.next_screen_pos.1 doesn't change
        glyphs.next_pos.x += 1;
        // next_pos.y doesn't change
        glyphs.next_byte = glyph.text_bytes.end;
        glyphs.was_lf = glyph.line_break;

        glyph.validate();

        Break(Some(glyph))
    } else if glyph.screen_pos.0 > glyphs.true_right_margin() {
        // glyph after the right margin.

        // skip to next_line
        glyphs.iter.skip_line().expect("fine");

        // do the line-break here.
        glyphs.next_screen_pos.0 = 0;
        glyphs.next_screen_pos.1 += 1;
        glyphs.next_pos.x = 0;
        glyphs.next_pos.y += 1;
        glyphs.next_byte = glyph.text_bytes.end;
        glyphs.was_lf = glyph.line_break;

        Continue(())
    } else {
        glyph.screen_pos.0 = glyph.screen_pos.0.saturating_sub(glyphs.left_margin);

        if glyph.screen_pos.0 == 0 {
            cache.line_start.borrow_mut().insert(
                glyph.pos.y,
                LineOffsetCache {
                    pos_x: glyph.pos.x,
                    screen_pos_x: glyphs.next_screen_pos.0,
                    byte_pos: glyph.text_bytes.start,
                },
            );
        }

        glyphs.next_screen_pos.0 += glyph.screen_width as upos_type;
        // iter.next_screen_pos.1 doesn't change
        glyphs.next_pos.x += 1;
        // next_pos.y doesn't change
        glyphs.next_byte = glyph.text_bytes.end;
        glyphs.was_lf = glyph.line_break;

        glyph.validate();

        Break(Some(glyph))
    }
}

#[inline(always)]
fn create_glyph<'a, Graphemes>(
    glyphs: &GlyphIter2<'a, Graphemes>,
    grapheme: Grapheme<'a>,
    pos: TextPosition,
    screen_pos: (u32, u32),
) -> Glyph2<'a> {
    let (grapheme, grapheme_bytes) = grapheme.into_parts();

    let mut glyph = Glyph2 {
        glyph: grapheme,
        text_bytes: grapheme_bytes,
        screen_pos,
        screen_width: 0,
        line_break: false,
        soft_break: false,
        hidden_break: false,
        hidden_glyph: Cow::Borrowed(""),
        pos,
    };

    remap_glyph(
        &mut glyph,
        glyphs.lf_breaks,
        glyphs.show_ctrl,
        glyphs.wrap_ctrl,
        glyphs.tabs,
    );

    glyph
}

#[inline(always)]
fn remap_glyph(
    glyph: &mut Glyph2<'_>,
    lf_breaks: bool,
    show_ctrl: bool,
    wrap_ctrl: bool,
    tabs: upos_type,
) {
    // todo: unicode line breaks
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
    } else if glyph.glyph == "\u{00AD}" {
        glyph.line_break = false;
        glyph.screen_width = if show_ctrl || wrap_ctrl { 1 } else { 0 };
        glyph.glyph = Cow::Borrowed(if show_ctrl || wrap_ctrl {
            "\u{2E1A}"
        } else {
            ""
        });
        glyph.hidden_break = true;
        glyph.hidden_glyph = Cow::Borrowed("-");
    } else if glyph.glyph == "\u{200B}" {
        glyph.line_break = false;
        glyph.screen_width = if show_ctrl || wrap_ctrl { 1 } else { 0 };
        glyph.glyph = Cow::Borrowed(if show_ctrl || wrap_ctrl {
            "\u{00A8}"
        } else {
            ""
        });
        glyph.hidden_break = true;
        glyph.hidden_glyph = Cow::Borrowed(" ");
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
    use crate::TextPosition;
    use crate::cache::{Cache, LineBreakCache};
    use crate::glyph2::{Glyph2, GlyphIter2, TextWrap2};
    use crate::grapheme::RopeGraphemes;
    use ropey::Rope;
    use std::borrow::Cow;

    #[test]
    fn test_word_0() {
        let s = Rope::from("");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Word);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        assert!(
            cc.line_break
                .borrow()
                .contains_key(&TextPosition::new(0, 0))
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 0..0,
                screen_pos: (0, 0),
                screen_width: 0,
                pos: TextPosition::new(0, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_word_1() {
        let s = Rope::from("X");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Word);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 0)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 1),
                byte_pos: 1
            })
            .as_ref()
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("X"),
                text_bytes: 0..1,
                screen_pos: (0, 0),
                screen_width: 1,
                pos: TextPosition::new(0, 0),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 1..1,
                screen_pos: (1, 0),
                screen_width: 0,
                pos: TextPosition::new(1, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_word_2() {
        let s = Rope::from("X\n");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Word);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 0)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 1),
                byte_pos: 2
            })
            .as_ref()
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("X"),
                text_bytes: 0..1,
                screen_pos: (0, 0),
                screen_width: 1,
                pos: TextPosition::new(0, 0),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 1..2,
                screen_pos: (1, 0),
                screen_width: 0,
                pos: TextPosition::new(1, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_word_3() {
        let s = Rope::from("X\nY");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Word);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 0)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 1),
                byte_pos: 2
            })
            .as_ref()
        );
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 1)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 2),
                byte_pos: 3
            })
            .as_ref()
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("X"),
                text_bytes: 0..1,
                screen_pos: (0, 0),
                screen_width: 1,
                pos: TextPosition::new(0, 0),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 1..2,
                screen_pos: (1, 0),
                screen_width: 0,
                pos: TextPosition::new(1, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("Y"),
                text_bytes: 2..3,
                screen_pos: (0, 1),
                screen_width: 1,
                pos: TextPosition::new(0, 1),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 3..3,
                screen_pos: (1, 1),
                screen_width: 0,
                pos: TextPosition::new(1, 1),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_word_4() {
        let s = Rope::from("X\nY\n");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Word);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 0)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 1),
                byte_pos: 2
            })
            .as_ref()
        );
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 1)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 2),
                byte_pos: 4
            })
            .as_ref()
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("X"),
                text_bytes: 0..1,
                screen_pos: (0, 0),
                screen_width: 1,
                pos: TextPosition::new(0, 0),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 1..2,
                screen_pos: (1, 0),
                screen_width: 0,
                pos: TextPosition::new(1, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("Y"),
                text_bytes: 2..3,
                screen_pos: (0, 1),
                screen_width: 1,
                pos: TextPosition::new(0, 1),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 3..4,
                screen_pos: (1, 1),
                screen_width: 0,
                pos: TextPosition::new(1, 1),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_hard_0() {
        let s = Rope::from("");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Hard);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        assert!(
            cc.line_break
                .borrow()
                .contains_key(&TextPosition::new(0, 0))
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 0..0,
                screen_pos: (0, 0),
                screen_width: 0,
                pos: TextPosition::new(0, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_hard_1() {
        let s = Rope::from("X");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Word);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 0)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 1),
                byte_pos: 1
            })
            .as_ref()
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("X"),
                text_bytes: 0..1,
                screen_pos: (0, 0),
                screen_width: 1,
                pos: TextPosition::new(0, 0),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 1..1,
                screen_pos: (1, 0),
                screen_width: 0,
                pos: TextPosition::new(1, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_hard_2() {
        let s = Rope::from("X\n");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Word);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 0)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 1),
                byte_pos: 2
            })
            .as_ref()
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("X"),
                text_bytes: 0..1,
                screen_pos: (0, 0),
                screen_width: 1,
                pos: TextPosition::new(0, 0),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 1..2,
                screen_pos: (1, 0),
                screen_width: 0,
                pos: TextPosition::new(1, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_hard_3() {
        let s = Rope::from("X\nY");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Hard);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 0)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 1),
                byte_pos: 2
            })
            .as_ref()
        );
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 1)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 2),
                byte_pos: 3
            })
            .as_ref()
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("X"),
                text_bytes: 0..1,
                screen_pos: (0, 0),
                screen_width: 1,
                pos: TextPosition::new(0, 0),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 1..2,
                screen_pos: (1, 0),
                screen_width: 0,
                pos: TextPosition::new(1, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("Y"),
                text_bytes: 2..3,
                screen_pos: (0, 1),
                screen_width: 1,
                pos: TextPosition::new(0, 1),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 3..3,
                screen_pos: (1, 1),
                screen_width: 0,
                pos: TextPosition::new(1, 1),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_hard_4() {
        let s = Rope::from("X\nY\n");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Hard);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 0)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 1),
                byte_pos: 2
            })
            .as_ref()
        );
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 1)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 2),
                byte_pos: 4
            })
            .as_ref()
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("X"),
                text_bytes: 0..1,
                screen_pos: (0, 0),
                screen_width: 1,
                pos: TextPosition::new(0, 0),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 1..2,
                screen_pos: (1, 0),
                screen_width: 0,
                pos: TextPosition::new(1, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("Y"),
                text_bytes: 2..3,
                screen_pos: (0, 1),
                screen_width: 1,
                pos: TextPosition::new(0, 1),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 3..4,
                screen_pos: (1, 1),
                screen_width: 0,
                pos: TextPosition::new(1, 1),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_shift_0() {
        let s = Rope::from("");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Shift);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        dbg!(&cc);
        assert!(
            cc.line_break
                .borrow()
                .contains_key(&TextPosition::new(0, 0))
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 0..0,
                screen_pos: (0, 0),
                screen_width: 0,
                pos: TextPosition::new(0, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_shift_1() {
        let s = Rope::from("X");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Shift);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        dbg!(&cc);
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 0)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 1),
                byte_pos: 1
            })
            .as_ref()
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("X"),
                text_bytes: 0..1,
                screen_pos: (0, 0),
                screen_width: 1,
                pos: TextPosition::new(0, 0),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 1..1,
                screen_pos: (1, 0),
                screen_width: 0,
                pos: TextPosition::new(1, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_shift_2() {
        let s = Rope::from("X\n");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Shift);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        dbg!(&cc);
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 0)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 1),
                byte_pos: 2
            })
            .as_ref()
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("X"),
                text_bytes: 0..1,
                screen_pos: (0, 0),
                screen_width: 1,
                pos: TextPosition::new(0, 0),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 1..2,
                screen_pos: (1, 0),
                screen_width: 0,
                pos: TextPosition::new(1, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_shift_3() {
        let s = Rope::from("X\nY");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Shift);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        dbg!(&cc);
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 0)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 1),
                byte_pos: 2
            })
            .as_ref()
        );
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 1)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 2),
                byte_pos: 3
            })
            .as_ref()
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("X"),
                text_bytes: 0..1,
                screen_pos: (0, 0),
                screen_width: 1,
                pos: TextPosition::new(0, 0),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 1..2,
                screen_pos: (1, 0),
                screen_width: 0,
                pos: TextPosition::new(1, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("Y"),
                text_bytes: 2..3,
                screen_pos: (0, 1),
                screen_width: 1,
                pos: TextPosition::new(0, 1),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 3..3,
                screen_pos: (1, 1),
                screen_width: 0,
                pos: TextPosition::new(1, 1),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }

    #[test]
    fn test_shift_4() {
        let s = Rope::from("X\nY\n");
        let r = RopeGraphemes::new(0, s.byte_slice(..));
        let cc = Cache::default();

        let mut glyphs = GlyphIter2::new(TextPosition::new(0, 0), 0, r, cc.clone());
        glyphs.set_lf_breaks(true);
        glyphs.set_text_wrap(TextWrap2::Shift);
        glyphs.set_left_margin(0);
        glyphs.set_right_margin(70);
        glyphs.set_word_margin(68);
        glyphs.prepare().unwrap();
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 0)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 1),
                byte_pos: 2
            })
            .as_ref()
        );
        assert_eq!(
            cc.line_break.borrow().get(&TextPosition::new(1, 1)),
            Some(LineBreakCache {
                start_pos: TextPosition::new(0, 2),
                byte_pos: 4
            })
            .as_ref()
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("X"),
                text_bytes: 0..1,
                screen_pos: (0, 0),
                screen_width: 1,
                pos: TextPosition::new(0, 0),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 1..2,
                screen_pos: (1, 0),
                screen_width: 0,
                pos: TextPosition::new(1, 0),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed("Y"),
                text_bytes: 2..3,
                screen_pos: (0, 1),
                screen_width: 1,
                pos: TextPosition::new(0, 1),
                line_break: false,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(
            glyphs.next(),
            Some(Glyph2 {
                glyph: Cow::Borrowed(""),
                text_bytes: 3..4,
                screen_pos: (1, 1),
                screen_width: 0,
                pos: TextPosition::new(1, 1),
                line_break: true,
                soft_break: false,
                hidden_break: false,
                hidden_glyph: Cow::Borrowed(""),
            })
        );
        assert_eq!(glyphs.next(), None);
    }
}
