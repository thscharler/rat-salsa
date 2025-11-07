/// Widget for color input.
///
/// Currently, supports
/// * RGB
/// * HSV
/// * hexdigits
///
/// __Keybindings__
///
/// * Switch between color-mode with Up/Down. (or m/M)
/// * '+' and Alt-'+' increase the value
/// * '-' and Alt-'-' decrease the value
/// * 'r', 'h', 'x' switch mode
///
/// __Clipboard__
///
/// Recognizes common formats when pasted from the clipboard.
/// * #000000 and #00000000
/// * 0x000000 and 0x00000000
/// * 000000 and 00000000
///
use crate::_private::NonExhaustive;
use crate::clipboard::Clipboard;
use crate::event::{ReadOnly, TextOutcome};
use crate::text_input_mask::{MaskedInput, MaskedInputState};
use crate::undo_buffer::{UndoBuffer, UndoEntry};
use crate::{TextError, TextFocusGained, TextFocusLost, TextStyle, upos_type};
use palette::{FromColor, Hsv, Srgb};
use rat_cursor::HasScreenCursor;
use rat_event::{HandleEvent, MouseOnly, Regular, ct_event, flow};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::RelocatableState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::cmp::min;
use std::ops::Range;

/// Color input widget.
///
/// A text input for colors.
///
#[derive(Debug, Default)]
pub struct ColorInput<'a> {
    style: Style,
    disable_modes: bool,
    mode: Option<Mode>,
    widget: MaskedInput<'a>,
}

/// Combined styles.
#[derive(Debug)]
pub struct ColorInputStyle {
    /// Base style.
    pub text: TextStyle,
    /// Highlighting the field of the input.
    pub field_style: Option<Style>,
    /// Disable mode switching.
    pub disable_modes: Option<bool>,
    /// Define default mode.
    pub mode: Option<Mode>,
    ///
    pub non_exhaustive: NonExhaustive,
}

impl Default for ColorInputStyle {
    fn default() -> Self {
        Self {
            text: Default::default(),
            field_style: Default::default(),
            disable_modes: Default::default(),
            mode: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

/// Color mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    RGB,
    HEX,
    HSV,
}

/// State for the color input.
#[derive(Debug)]
pub struct ColorInputState {
    /// Area of the widget.
    /// __read only__ renewed with each render.
    pub area: Rect,
    /// Area for the mode.
    /// __read_only__ renewed with each render.
    pub mode_area: Rect,

    /// value as RGB with 0.0..1.0 ranges.
    pub value: (f32, f32, f32),

    /// Disable keys for mode switching.
    /// __read only__
    pub disable_modes: bool,
    /// __read only__
    pub mode: Mode,
    /// __read only__
    pub widget: MaskedInputState,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> ColorInput<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the combined style.
    #[inline]
    pub fn styles(mut self, style: ColorInputStyle) -> Self {
        self.style = style.text.style;
        self.widget = self.widget.styles(style.text);
        if let Some(disable_modes) = style.disable_modes {
            self.disable_modes = disable_modes;
        }
        if let Some(mode) = style.mode {
            self.mode = Some(mode);
        }
        if let Some(field_style) = style.field_style {
            self.widget = self.widget.text_style_idx(1, field_style);
        }
        self
    }

    /// Base text style.
    #[inline]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        let style = style.into();
        self.style = style;
        self.widget = self.widget.style(style);
        self
    }

    /// Style for the fields of the input.
    #[inline]
    pub fn field_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.text_style_idx(1, style.into());
        self
    }

    /// Disable switching the mode.
    #[inline]
    pub fn disable_modes(mut self) -> Self {
        self.disable_modes = true;
        self
    }

    /// Color mode.
    #[inline]
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Style when focused.
    #[inline]
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.focus_style(style);
        self
    }

    /// Style for selection
    #[inline]
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.select_style(style);
        self
    }

    /// Style for the invalid indicator.
    #[inline]
    pub fn invalid_style(mut self, style: impl Into<Style>) -> Self {
        self.widget = self.widget.invalid_style(style);
        self
    }

    /// Block
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.widget = self.widget.block(block);
        self
    }

    /// Focus behaviour
    #[inline]
    pub fn on_focus_gained(mut self, of: TextFocusGained) -> Self {
        self.widget = self.widget.on_focus_gained(of);
        self
    }

    /// Focus behaviour
    #[inline]
    pub fn on_focus_lost(mut self, of: TextFocusLost) -> Self {
        self.widget = self.widget.on_focus_lost(of);
        self
    }
}

impl<'a> StatefulWidget for &ColorInput<'a> {
    type State = ColorInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render(self, area, buf, state);
    }
}

impl StatefulWidget for ColorInput<'_> {
    type State = ColorInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render(&self, area, buf, state);
    }
}

fn render(widget: &ColorInput<'_>, area: Rect, buf: &mut Buffer, state: &mut ColorInputState) {
    state.disable_modes = widget.disable_modes;
    if let Some(mode) = widget.mode {
        state.mode = mode;
    }

    let mode_area = Rect::new(area.x, area.y, 4, 1);
    let widget_area = Rect::new(
        area.x + mode_area.width,
        area.y,
        area.width.saturating_sub(mode_area.width),
        1,
    );

    state.mode_area = mode_area;

    let bg = state.value();
    let fg_colors = [Color::Black, Color::White];
    let style = high_contrast_color(bg, &fg_colors);

    let mode_str = match state.mode {
        Mode::RGB => "RGB",
        Mode::HEX => "  #",
        Mode::HSV => "HSV",
    };
    Line::from(mode_str).style(style).render(mode_area, buf);

    (&widget.widget).render(widget_area, buf, &mut state.widget);
}

impl Default for ColorInputState {
    fn default() -> Self {
        let mut z = Self {
            area: Default::default(),
            mode_area: Default::default(),
            value: Default::default(),
            disable_modes: Default::default(),
            mode: Default::default(),
            widget: Default::default(),
            non_exhaustive: NonExhaustive,
        };
        z.set_mode(Mode::RGB);
        z
    }
}

impl HasFocus for ColorInputState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget_with_flags(
            self.widget.focus(),
            self.area,
            self.widget.area_z(),
            self.widget.navigable(),
        );
    }

    fn focus(&self) -> FocusFlag {
        self.widget.focus()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl ColorInputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        Self {
            widget: MaskedInputState::named(name),
            ..Default::default()
        }
    }

    /// The next edit operation will overwrite the current content
    /// instead of adding text. Any move operations will cancel
    /// this overwrite.
    #[inline]
    pub fn set_overwrite(&mut self, overwrite: bool) {
        self.widget.set_overwrite(overwrite);
    }

    /// Will the next edit operation overwrite the content?
    #[inline]
    pub fn overwrite(&self) -> bool {
        self.widget.overwrite()
    }
}

impl ColorInputState {
    /// Clipboard used.
    /// Default is to use the [global_clipboard](crate::clipboard::global_clipboard).
    #[inline]
    pub fn set_clipboard(&mut self, clip: Option<impl Clipboard + 'static>) {
        self.widget.set_clipboard(clip);
    }

    /// Clipboard used.
    /// Default is to use the [global_clipboard](crate::clipboard::global_clipboard).
    #[inline]
    pub fn clipboard(&self) -> Option<&dyn Clipboard> {
        self.widget.clipboard()
    }

    /// Copy to clipboard
    #[inline]
    pub fn copy_to_clip(&mut self) -> bool {
        let Some(clip) = self.widget.value.clipboard() else {
            return false;
        };

        let r = (self.value.0 * 255f32) as u32;
        let g = (self.value.1 * 255f32) as u32;
        let b = (self.value.2 * 255f32) as u32;
        let value_str = format!("{:06x}", (r << 16) + (g << 8) + b);

        _ = clip.set_string(&value_str);

        true
    }

    /// Cut to clipboard
    #[inline]
    pub fn cut_to_clip(&mut self) -> bool {
        let Some(clip) = self.widget.value.clipboard() else {
            return false;
        };

        let r = (self.value.0 * 255f32) as u32;
        let g = (self.value.1 * 255f32) as u32;
        let b = (self.value.2 * 255f32) as u32;
        let value_str = format!("{:06x}", (r << 16) + (g << 8) + b);

        match clip.set_string(&value_str) {
            Ok(_) => self.clear(),
            Err(_) => {}
        }
        true
    }

    /// Paste from clipboard.
    #[inline]
    pub fn paste_from_clip(&mut self) -> bool {
        let Some(clip) = self.widget.value.clipboard() else {
            return false;
        };

        if let Ok(text) = clip.get_string() {
            if text.starts_with("#") && text.len() == 7 {
                // #aabbcc
                if let Ok(v) = u32::from_str_radix(&text[1..7], 16) {
                    self.set_value_u32(v);
                }
            } else if text.starts_with("#") && text.len() == 9 {
                // #aabbccdd
                if let Ok(v) = u32::from_str_radix(&text[1..7], 16) {
                    self.set_value_u32(v);
                }
            } else if text.starts_with("0x") && text.len() == 8 {
                // 0xaabbcc
                if let Ok(v) = u32::from_str_radix(&text[2..8], 16) {
                    self.set_value_u32(v);
                }
            } else if text.starts_with("0x") && text.len() == 10 {
                // 0xaabbccdd
                if let Ok(v) = u32::from_str_radix(&text[2..8], 16) {
                    self.set_value_u32(v);
                }
            } else if text.len() == 6 {
                // aabbcc
                if let Ok(v) = u32::from_str_radix(&text[0..6], 16) {
                    self.set_value_u32(v);
                }
            } else if text.len() == 8 {
                // aabbccdd
                if let Ok(v) = u32::from_str_radix(&text[0..6], 16) {
                    self.set_value_u32(v);
                }
            } else {
                for c in text.chars() {
                    self.widget.insert_char(c);
                }
            }
            true
        } else {
            false
        }
    }
}

impl ColorInputState {
    /// Set undo buffer.
    #[inline]
    pub fn set_undo_buffer(&mut self, undo: Option<impl UndoBuffer + 'static>) {
        self.widget.set_undo_buffer(undo);
    }

    /// Undo
    #[inline]
    pub fn undo_buffer(&self) -> Option<&dyn UndoBuffer> {
        self.widget.undo_buffer()
    }

    /// Undo
    #[inline]
    pub fn undo_buffer_mut(&mut self) -> Option<&mut dyn UndoBuffer> {
        self.widget.undo_buffer_mut()
    }

    /// Get all recent replay recordings.
    #[inline]
    pub fn recent_replay_log(&mut self) -> Vec<UndoEntry> {
        self.widget.recent_replay_log()
    }

    /// Apply the replay recording.
    #[inline]
    pub fn replay_log(&mut self, replay: &[UndoEntry]) {
        self.widget.replay_log(replay)
    }

    /// Undo operation
    #[inline]
    pub fn undo(&mut self) -> bool {
        self.widget.undo()
    }

    /// Redo operation
    #[inline]
    pub fn redo(&mut self) -> bool {
        self.widget.redo()
    }
}

impl ColorInputState {
    /// Set and replace all styles.
    #[inline]
    pub fn set_styles(&mut self, styles: Vec<(Range<usize>, usize)>) {
        self.widget.set_styles(styles);
    }

    /// Add a style for a byte-range.
    #[inline]
    pub fn add_style(&mut self, range: Range<usize>, style: usize) {
        self.widget.add_style(range, style);
    }

    /// Add a style for a `Range<upos_type>` .
    /// The style-nr refers to one of the styles set with the widget.
    #[inline]
    pub fn add_range_style(
        &mut self,
        range: Range<upos_type>,
        style: usize,
    ) -> Result<(), TextError> {
        self.widget.add_range_style(range, style)
    }

    /// Remove the exact TextRange and style.
    #[inline]
    pub fn remove_style(&mut self, range: Range<usize>, style: usize) {
        self.widget.remove_style(range, style);
    }

    /// Remove the exact `Range<upos_type>` and style.
    #[inline]
    pub fn remove_range_style(
        &mut self,
        range: Range<upos_type>,
        style: usize,
    ) -> Result<(), TextError> {
        self.widget.remove_range_style(range, style)
    }

    /// Find all styles that touch the given range.
    pub fn styles_in(&self, range: Range<usize>, buf: &mut Vec<(Range<usize>, usize)>) {
        self.widget.styles_in(range, buf)
    }

    /// All styles active at the given position.
    #[inline]
    pub fn styles_at(&self, byte_pos: usize, buf: &mut Vec<(Range<usize>, usize)>) {
        self.widget.styles_at(byte_pos, buf)
    }

    /// Check if the given style applies at the position and
    /// return the complete range for the style.
    #[inline]
    pub fn style_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
        self.widget.styles_at_match(byte_pos, style)
    }

    /// List of all styles.
    #[inline]
    pub fn styles(&self) -> Option<impl Iterator<Item = (Range<usize>, usize)> + '_> {
        self.widget.styles()
    }
}

impl ColorInputState {
    /// Offset shown.
    #[inline]
    pub fn offset(&self) -> upos_type {
        self.widget.offset()
    }

    /// Offset shown. This is corrected if the cursor wouldn't be visible.
    #[inline]
    pub fn set_offset(&mut self, offset: upos_type) {
        self.widget.set_offset(offset)
    }

    /// Cursor position
    #[inline]
    pub fn cursor(&self) -> upos_type {
        self.widget.cursor()
    }

    /// Set the cursor position, reset selection.
    #[inline]
    pub fn set_cursor(&mut self, cursor: upos_type, extend_selection: bool) -> bool {
        self.widget.set_cursor(cursor, extend_selection)
    }

    /// Place cursor at some sensible position according to the mask.
    #[inline]
    pub fn set_default_cursor(&mut self) {
        self.widget.set_default_cursor()
    }

    /// Selection anchor.
    #[inline]
    pub fn anchor(&self) -> upos_type {
        self.widget.anchor()
    }

    /// Selection
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.widget.has_selection()
    }

    /// Selection
    #[inline]
    pub fn selection(&self) -> Range<upos_type> {
        self.widget.selection()
    }

    /// Selection
    #[inline]
    pub fn set_selection(&mut self, anchor: upos_type, cursor: upos_type) -> bool {
        self.widget.set_selection(anchor, cursor)
    }

    /// Select all text.
    #[inline]
    pub fn select_all(&mut self) {
        self.widget.select_all();
    }

    /// Selection
    #[inline]
    pub fn selected_text(&self) -> &str {
        self.widget.selected_text()
    }
}

impl ColorInputState {
    /// Empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.widget.is_empty()
    }

    /// Value as Color.
    pub fn value(&self) -> Color {
        Color::Rgb(
            (self.value.0 * 255f32) as u8,
            (self.value.1 * 255f32) as u8,
            (self.value.2 * 255f32) as u8,
        )
    }

    fn parse_value(&self) -> (f32, f32, f32) {
        let r = match self.mode {
            Mode::RGB => {
                let r = self.widget.section_value::<f32>(SEC_R).unwrap_or_default();
                let g = self.widget.section_value::<f32>(SEC_G).unwrap_or_default();
                let b = self.widget.section_value::<f32>(SEC_B).unwrap_or_default();
                (r / 255f32, g / 255f32, b / 255f32)
            }
            Mode::HEX => {
                let v = u32::from_str_radix(self.widget.section_text(1), 16).expect("hex");
                let r = ((v >> 16) & 255) as f32;
                let g = ((v >> 8) & 255) as f32;
                let b = (v & 255) as f32;
                (r / 255f32, g / 255f32, b / 255f32)
            }
            Mode::HSV => {
                let h = self.widget.section_value::<f32>(SEC_H).unwrap_or_default();
                let s = self.widget.section_value::<f32>(SEC_S).unwrap_or_default();
                let v = self.widget.section_value::<f32>(SEC_V).unwrap_or_default();

                let h = palette::RgbHue::from_degrees(h);
                let s = s / 100f32;
                let v = v / 100f32;

                let hsv = Hsv::from_components((h, s, v));
                let rgb = Srgb::from_color(hsv);

                rgb.into_components()
            }
        };
        r
    }

    /// Length in grapheme count.
    #[inline]
    pub fn len(&self) -> upos_type {
        self.widget.len()
    }

    /// Length as grapheme count.
    #[inline]
    pub fn line_width(&self) -> upos_type {
        self.widget.line_width()
    }
}

impl ColorInputState {
    /// Reset to empty.
    #[inline]
    pub fn clear(&mut self) {
        self.widget.clear();
    }

    /// Set the color as u32
    pub fn set_value_u32(&mut self, color: u32) {
        let r = ((color >> 16) & 255) as f32;
        let g = ((color >> 8) & 255) as f32;
        let b = (color & 255) as f32;
        self.value = (r / 255f32, g / 255f32, b / 255f32);
        self.value_to_text();
    }

    /// Set the color as Color.
    pub fn set_value(&mut self, color: Color) {
        let (r, g, b) = color2rgb(color);
        self.value = (r as f32 / 255f32, g as f32 / 255f32, b as f32 / 255f32);
        self.value_to_text();
    }

    fn value_to_text(&mut self) {
        match self.mode {
            Mode::RGB => {
                let r = (self.value.0 * 255f32) as u8;
                let g = (self.value.1 * 255f32) as u8;
                let b = (self.value.2 * 255f32) as u8;
                let value_str = format!("{:3} {:3} {:3}", r, g, b);
                self.widget.set_text(value_str);
                self.set_mode_styles();
            }
            Mode::HEX => {
                let r = (self.value.0 * 255f32) as u32;
                let g = (self.value.1 * 255f32) as u32;
                let b = (self.value.2 * 255f32) as u32;
                let value_str = format!("{:06x}", (r << 16) + (g << 8) + b);
                self.widget.set_text(value_str);
                self.set_mode_styles();
            }
            Mode::HSV => {
                let r = self.value.0;
                let g = self.value.1;
                let b = self.value.2;
                let srgb = Srgb::new(r, g, b);
                let hsv = Hsv::from_color(srgb);
                let (h, s, v) = hsv.into_components();
                let h = h.into_positive_degrees() as u32;
                let s = (s * 100f32) as u32;
                let v = (v * 100f32) as u32;
                let value_str = format!("{:3} {:3} {:3}", h, s, v);
                self.widget.set_text(value_str);
                self.set_mode_styles();
            }
        }
    }

    /// Insert a char at the current position.
    #[inline]
    pub fn insert_char(&mut self, c: char) -> bool {
        let r = self.widget.insert_char(c);
        self.normalize();
        r
    }

    /// Remove the selected range. The text will be replaced with the default value
    /// as defined by the mask.
    #[inline]
    pub fn delete_range(&mut self, range: Range<upos_type>) -> bool {
        let r = self.widget.delete_range(range);
        self.normalize();
        r
    }

    /// Remove the selected range. The text will be replaced with the default value
    /// as defined by the mask.
    #[inline]
    pub fn try_delete_range(&mut self, range: Range<upos_type>) -> Result<bool, TextError> {
        let r = self.widget.try_delete_range(range);
        self.normalize();
        r
    }
}

const PAT_RGB: &'static str = "##0 ##0 ##0";
const SEC_R: u16 = 1;
const SEC_G: u16 = 3;
const SEC_B: u16 = 5;

const PAT_HEX: &'static str = "HHHHHH";
#[allow(dead_code)]
const SEC_X: u16 = 1;

const PAT_HSV: &'static str = "##0 ##0 ##0";
const SEC_H: u16 = 1;
const SEC_S: u16 = 3;
const SEC_V: u16 = 5;

impl ColorInputState {
    fn clamp_section(&mut self, section: u16, clamp: u32) -> bool {
        let r = self
            .widget
            .section_value::<u32>(section)
            .unwrap_or_default();
        let r_min = min(r, clamp);
        if r_min != r {
            self.widget.set_section_value(section, r_min);
            true
        } else {
            false
        }
    }

    /// Correct the numeric values for each component.
    fn normalize(&mut self) -> bool {
        let r = match self.mode {
            Mode::RGB => {
                self.clamp_section(SEC_R, 255)
                    || self.clamp_section(SEC_G, 255)
                    || self.clamp_section(SEC_B, 255)
            }
            Mode::HEX => {
                // noop
                false
            }
            Mode::HSV => {
                self.clamp_section(SEC_H, 360)
                    || self.clamp_section(SEC_S, 100)
                    || self.clamp_section(SEC_V, 100)
            }
        };
        self.set_mode_styles();
        r
    }

    /// Increment the value at the cursor position.
    pub fn increment(&mut self, n: u32) -> bool {
        let section = self.widget.section_id(self.widget.cursor());
        let r = match self.mode {
            Mode::RGB => match section {
                SEC_R | SEC_G | SEC_B => {
                    let r = self
                        .widget
                        .section_value::<u32>(section)
                        .unwrap_or_default();
                    let r_min = min(r + n, 255);
                    if r_min != r {
                        self.widget.set_section_value(section, r_min);
                        self.set_mode_styles();
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            Mode::HEX => {
                // noop
                false
            }
            Mode::HSV => match section {
                SEC_H => {
                    let r = self
                        .widget
                        .section_value::<u32>(section)
                        .unwrap_or_default();
                    let r_min = (r + n) % 360;
                    if r_min != r {
                        self.widget.set_section_value(section, r_min);
                        self.set_mode_styles();
                        true
                    } else {
                        false
                    }
                }
                SEC_S | SEC_V => {
                    let r = self
                        .widget
                        .section_value::<u32>(section)
                        .unwrap_or_default();
                    let r_min = min(r + n, 100);
                    if r_min != r {
                        self.widget.set_section_value(section, r_min);
                        self.set_mode_styles();
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
        };

        if r {
            self.value = self.parse_value();
        }
        r
    }

    /// Decrement the value at the cursor position.
    pub fn decrement(&mut self, n: u32) -> bool {
        let section = self.widget.section_id(self.widget.cursor());
        let r = match self.mode {
            Mode::RGB => match section {
                SEC_R | SEC_G | SEC_B => {
                    let r = self
                        .widget
                        .section_value::<u32>(section)
                        .unwrap_or_default();
                    let r_min = r.saturating_sub(n);
                    if r_min != r {
                        self.widget.set_section_value(section, r_min);
                        self.set_mode_styles();
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            Mode::HEX => {
                // noop
                false
            }
            Mode::HSV => match section {
                SEC_H => {
                    let r = self
                        .widget
                        .section_value::<u32>(section)
                        .unwrap_or_default();
                    let r_min = if r > n {
                        r - n
                    } else {
                        360u32.saturating_sub(n - r)
                    };
                    if r_min != r {
                        self.widget.set_section_value(section, r_min);
                        self.set_mode_styles();
                        true
                    } else {
                        false
                    }
                }
                SEC_S | SEC_V => {
                    let r = self
                        .widget
                        .section_value::<u32>(section)
                        .unwrap_or_default();
                    let r_min = r.saturating_sub(n);
                    if r_min != r {
                        self.widget.set_section_value(section, r_min);
                        self.set_mode_styles();
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
        };

        if r {
            self.value = self.parse_value();
        }
        r
    }

    fn set_mode_styles(&mut self) {
        match self.mode {
            Mode::RGB => {
                // "##0 ##0 ##0"
                self.widget.clear_styles();
                self.widget.add_range_style(0..3, 1).expect("fine");
                self.widget.add_range_style(4..7, 1).expect("fine");
                self.widget.add_range_style(8..11, 1).expect("fine");
            }
            Mode::HEX => {
                // "hhhhhH"
                self.widget.clear_styles();
                self.widget.add_range_style(0..6, 1).expect("fine");
            }
            Mode::HSV => {
                // "##0 ##0 ##0"
                self.widget.clear_styles();
                self.widget.add_range_style(0..3, 1).expect("fine");
                self.widget.add_range_style(4..7, 1).expect("fine");
                self.widget.add_range_style(8..11, 1).expect("fine");
            }
        }
    }

    /// Set the color mode.
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        match self.mode {
            Mode::RGB => {
                // "##0 ##0 ##0"
                self.widget.set_mask(PAT_RGB).expect("valid-mask");
            }
            Mode::HEX => {
                // "hhhhhH"
                self.widget.set_mask(PAT_HEX).expect("valid-mask");
            }
            Mode::HSV => {
                // "##0 ##0 ##0"
                self.widget.set_mask(PAT_HSV).expect("valid-mask");
            }
        }

        self.set_mode_styles();
        self.value_to_text();
        self.widget.set_default_cursor();
    }

    /// Switch to next mode.
    pub fn next_mode(&mut self) {
        match self.mode {
            Mode::RGB => self.set_mode(Mode::HEX),
            Mode::HEX => self.set_mode(Mode::HSV),
            Mode::HSV => self.set_mode(Mode::RGB),
        }
    }

    /// Switch to prev mode.
    pub fn prev_mode(&mut self) {
        match self.mode {
            Mode::RGB => self.set_mode(Mode::HSV),
            Mode::HEX => self.set_mode(Mode::RGB),
            Mode::HSV => self.set_mode(Mode::HEX),
        }
    }

    /// Move to the next char.
    #[inline]
    pub fn move_right(&mut self, extend_selection: bool) -> bool {
        self.widget.move_right(extend_selection)
    }

    /// Move to the previous char.
    #[inline]
    pub fn move_left(&mut self, extend_selection: bool) -> bool {
        self.widget.move_left(extend_selection)
    }

    /// Start of line
    #[inline]
    pub fn move_to_line_start(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_line_start(extend_selection)
    }

    /// End of line
    #[inline]
    pub fn move_to_line_end(&mut self, extend_selection: bool) -> bool {
        self.widget.move_to_line_end(extend_selection)
    }
}

impl HasScreenCursor for ColorInputState {
    /// The current text cursor as an absolute screen position.
    #[inline]
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.widget.screen_cursor()
    }
}

impl RelocatableState for ColorInputState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.widget.relocate(shift, clip);
    }
}

impl ColorInputState {
    /// Converts a grapheme based position to a screen position
    /// relative to the widget area.
    #[inline]
    pub fn col_to_screen(&self, pos: upos_type) -> Option<u16> {
        self.widget.col_to_screen(pos)
    }

    /// Converts from a widget relative screen coordinate to a grapheme index.
    /// x is the relative screen position.
    #[inline]
    pub fn screen_to_col(&self, scx: i16) -> upos_type {
        self.widget.screen_to_col(scx)
    }

    /// Set the cursor position from a screen position relative to the origin
    /// of the widget. This value can be negative, which selects a currently
    /// not visible position and scrolls to it.
    #[inline]
    pub fn set_screen_cursor(&mut self, cursor: i16, extend_selection: bool) -> bool {
        self.widget.set_screen_cursor(cursor, extend_selection)
    }
}

/// Gives the luminance according to BT.709.
fn luminance_bt_srgb(color: Color) -> f32 {
    let (r, g, b) = color2rgb(color);
    0.2126f32 * ((r as f32) / 255f32).powf(2.2f32)
        + 0.7152f32 * ((g as f32) / 255f32).powf(2.2f32)
        + 0.0722f32 * ((b as f32) / 255f32).powf(2.2f32)
}

/// Contrast between two colors.
fn contrast_bt_srgb(color: Color, color2: Color) -> f32 {
    let lum1 = luminance_bt_srgb(color);
    let lum2 = luminance_bt_srgb(color2);
    (lum1 - lum2).abs()
    // Don't use this prescribed method.
    // The abs diff comes out better.
    // (lum1 + 0.05f32) / (lum2 + 0.05f32)
}

pub fn high_contrast_color(bg: Color, text: &[Color]) -> Style {
    let mut color0 = text[0];
    let mut color1 = text[0];
    let mut contrast1 = contrast_bt_srgb(color1, bg);

    for text_color in text {
        let test = contrast_bt_srgb(*text_color, bg);
        if test > contrast1 {
            color0 = color1;
            color1 = *text_color;
            contrast1 = test;
        }
    }
    // don't use the second brightest.
    _ = color0;

    Style::new().bg(bg).fg(color1)
}

/// Gives back the rgb for any ratatui Color.
/// Has the indexed and the named colors too.
const fn color2rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Black => (0x00, 0x00, 0x00),
        Color::Red => (0xaa, 0x00, 0x00),
        Color::Green => (0x00, 0xaa, 0x00),
        Color::Yellow => (0xaa, 0x55, 0x00),
        Color::Blue => (0x00, 0x00, 0xaa),
        Color::Magenta => (0xaa, 0x00, 0xaa),
        Color::Cyan => (0x00, 0xaa, 0xaa),
        Color::Gray => (0xaa, 0xaa, 0xaa),
        Color::DarkGray => (0x55, 0x55, 0x55),
        Color::LightRed => (0xff, 0x55, 0x55),
        Color::LightGreen => (0x55, 0xff, 0x55),
        Color::LightYellow => (0xff, 0xff, 0x55),
        Color::LightBlue => (0x55, 0x55, 0xff),
        Color::LightMagenta => (0xff, 0x55, 0xff),
        Color::LightCyan => (0x55, 0xff, 0xff),
        Color::White => (0xff, 0xff, 0xff),
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Indexed(i) => {
            const VGA256: [(u8, u8, u8); 256] = [
                (0x00, 0x00, 0x00),
                (0x80, 0x00, 0x00),
                (0x00, 0x80, 0x00),
                (0x80, 0x80, 0x00),
                (0x00, 0x00, 0x80),
                (0x80, 0x00, 0x80),
                (0x00, 0x80, 0x80),
                (0xc0, 0xc0, 0xc0),
                (0x80, 0x80, 0x80),
                (0xff, 0x00, 0x00),
                (0x00, 0xff, 0x00),
                (0xff, 0xff, 0x00),
                (0x00, 0x00, 0xff),
                (0xff, 0x00, 0xff),
                (0x00, 0xff, 0xff),
                (0xff, 0xff, 0xff),
                (0x00, 0x00, 0x00),
                (0x00, 0x00, 0x5f),
                (0x00, 0x00, 0x87),
                (0x00, 0x00, 0xaf),
                (0x00, 0x00, 0xd7),
                (0x00, 0x00, 0xff),
                (0x00, 0x5f, 0x00),
                (0x00, 0x5f, 0x5f),
                (0x00, 0x5f, 0x87),
                (0x00, 0x5f, 0xaf),
                (0x00, 0x5f, 0xd7),
                (0x00, 0x5f, 0xff),
                (0x00, 0x87, 0x00),
                (0x00, 0x87, 0x5f),
                (0x00, 0x87, 0x87),
                (0x00, 0x87, 0xaf),
                (0x00, 0x87, 0xd7),
                (0x00, 0x87, 0xff),
                (0x00, 0xaf, 0x00),
                (0x00, 0xaf, 0x5f),
                (0x00, 0xaf, 0x87),
                (0x00, 0xaf, 0xaf),
                (0x00, 0xaf, 0xd7),
                (0x00, 0xaf, 0xff),
                (0x00, 0xd7, 0x00),
                (0x00, 0xd7, 0x5f),
                (0x00, 0xd7, 0x87),
                (0x00, 0xd7, 0xaf),
                (0x00, 0xd7, 0xd7),
                (0x00, 0xd7, 0xff),
                (0x00, 0xff, 0x00),
                (0x00, 0xff, 0x5f),
                (0x00, 0xff, 0x87),
                (0x00, 0xff, 0xaf),
                (0x00, 0xff, 0xd7),
                (0x00, 0xff, 0xff),
                (0x5f, 0x00, 0x00),
                (0x5f, 0x00, 0x5f),
                (0x5f, 0x00, 0x87),
                (0x5f, 0x00, 0xaf),
                (0x5f, 0x00, 0xd7),
                (0x5f, 0x00, 0xff),
                (0x5f, 0x5f, 0x00),
                (0x5f, 0x5f, 0x5f),
                (0x5f, 0x5f, 0x87),
                (0x5f, 0x5f, 0xaf),
                (0x5f, 0x5f, 0xd7),
                (0x5f, 0x5f, 0xff),
                (0x5f, 0x87, 0x00),
                (0x5f, 0x87, 0x5f),
                (0x5f, 0x87, 0x87),
                (0x5f, 0x87, 0xaf),
                (0x5f, 0x87, 0xd7),
                (0x5f, 0x87, 0xff),
                (0x5f, 0xaf, 0x00),
                (0x5f, 0xaf, 0x5f),
                (0x5f, 0xaf, 0x87),
                (0x5f, 0xaf, 0xaf),
                (0x5f, 0xaf, 0xd7),
                (0x5f, 0xaf, 0xff),
                (0x5f, 0xd7, 0x00),
                (0x5f, 0xd7, 0x5f),
                (0x5f, 0xd7, 0x87),
                (0x5f, 0xd7, 0xaf),
                (0x5f, 0xd7, 0xd7),
                (0x5f, 0xd7, 0xff),
                (0x5f, 0xff, 0x00),
                (0x5f, 0xff, 0x5f),
                (0x5f, 0xff, 0x87),
                (0x5f, 0xff, 0xaf),
                (0x5f, 0xff, 0xd7),
                (0x5f, 0xff, 0xff),
                (0x87, 0x00, 0x00),
                (0x87, 0x00, 0x5f),
                (0x87, 0x00, 0x87),
                (0x87, 0x00, 0xaf),
                (0x87, 0x00, 0xd7),
                (0x87, 0x00, 0xff),
                (0x87, 0x5f, 0x00),
                (0x87, 0x5f, 0x5f),
                (0x87, 0x5f, 0x87),
                (0x87, 0x5f, 0xaf),
                (0x87, 0x5f, 0xd7),
                (0x87, 0x5f, 0xff),
                (0x87, 0x87, 0x00),
                (0x87, 0x87, 0x5f),
                (0x87, 0x87, 0x87),
                (0x87, 0x87, 0xaf),
                (0x87, 0x87, 0xd7),
                (0x87, 0x87, 0xff),
                (0x87, 0xaf, 0x00),
                (0x87, 0xaf, 0x5f),
                (0x87, 0xaf, 0x87),
                (0x87, 0xaf, 0xaf),
                (0x87, 0xaf, 0xd7),
                (0x87, 0xaf, 0xff),
                (0x87, 0xd7, 0x00),
                (0x87, 0xd7, 0x5f),
                (0x87, 0xd7, 0x87),
                (0x87, 0xd7, 0xaf),
                (0x87, 0xd7, 0xd7),
                (0x87, 0xd7, 0xff),
                (0x87, 0xff, 0x00),
                (0x87, 0xff, 0x5f),
                (0x87, 0xff, 0x87),
                (0x87, 0xff, 0xaf),
                (0x87, 0xff, 0xd7),
                (0x87, 0xff, 0xff),
                (0xaf, 0x00, 0x00),
                (0xaf, 0x00, 0x5f),
                (0xaf, 0x00, 0x87),
                (0xaf, 0x00, 0xaf),
                (0xaf, 0x00, 0xd7),
                (0xaf, 0x00, 0xff),
                (0xaf, 0x5f, 0x00),
                (0xaf, 0x5f, 0x5f),
                (0xaf, 0x5f, 0x87),
                (0xaf, 0x5f, 0xaf),
                (0xaf, 0x5f, 0xd7),
                (0xaf, 0x5f, 0xff),
                (0xaf, 0x87, 0x00),
                (0xaf, 0x87, 0x5f),
                (0xaf, 0x87, 0x87),
                (0xaf, 0x87, 0xaf),
                (0xaf, 0x87, 0xd7),
                (0xaf, 0x87, 0xff),
                (0xaf, 0xaf, 0x00),
                (0xaf, 0xaf, 0x5f),
                (0xaf, 0xaf, 0x87),
                (0xaf, 0xaf, 0xaf),
                (0xaf, 0xaf, 0xd7),
                (0xaf, 0xaf, 0xff),
                (0xaf, 0xd7, 0x00),
                (0xaf, 0xd7, 0x5f),
                (0xaf, 0xd7, 0x87),
                (0xaf, 0xd7, 0xaf),
                (0xaf, 0xd7, 0xd7),
                (0xaf, 0xd7, 0xff),
                (0xaf, 0xff, 0x00),
                (0xaf, 0xff, 0x5f),
                (0xaf, 0xff, 0x87),
                (0xaf, 0xff, 0xaf),
                (0xaf, 0xff, 0xd7),
                (0xaf, 0xff, 0xff),
                (0xd7, 0x00, 0x00),
                (0xd7, 0x00, 0x5f),
                (0xd7, 0x00, 0x87),
                (0xd7, 0x00, 0xaf),
                (0xd7, 0x00, 0xd7),
                (0xd7, 0x00, 0xff),
                (0xd7, 0x5f, 0x00),
                (0xd7, 0x5f, 0x5f),
                (0xd7, 0x5f, 0x87),
                (0xd7, 0x5f, 0xaf),
                (0xd7, 0x5f, 0xd7),
                (0xd7, 0x5f, 0xff),
                (0xd7, 0x87, 0x00),
                (0xd7, 0x87, 0x5f),
                (0xd7, 0x87, 0x87),
                (0xd7, 0x87, 0xaf),
                (0xd7, 0x87, 0xd7),
                (0xd7, 0x87, 0xff),
                (0xd7, 0xaf, 0x00),
                (0xd7, 0xaf, 0x5f),
                (0xd7, 0xaf, 0x87),
                (0xd7, 0xaf, 0xaf),
                (0xd7, 0xaf, 0xd7),
                (0xd7, 0xaf, 0xff),
                (0xd7, 0xd7, 0x00),
                (0xd7, 0xd7, 0x5f),
                (0xd7, 0xd7, 0x87),
                (0xd7, 0xd7, 0xaf),
                (0xd7, 0xd7, 0xd7),
                (0xd7, 0xd7, 0xff),
                (0xd7, 0xff, 0x00),
                (0xd7, 0xff, 0x5f),
                (0xd7, 0xff, 0x87),
                (0xd7, 0xff, 0xaf),
                (0xd7, 0xff, 0xd7),
                (0xd7, 0xff, 0xff),
                (0xff, 0x00, 0x00),
                (0xff, 0x00, 0x5f),
                (0xff, 0x00, 0x87),
                (0xff, 0x00, 0xaf),
                (0xff, 0x00, 0xd7),
                (0xff, 0x00, 0xff),
                (0xff, 0x5f, 0x00),
                (0xff, 0x5f, 0x5f),
                (0xff, 0x5f, 0x87),
                (0xff, 0x5f, 0xaf),
                (0xff, 0x5f, 0xd7),
                (0xff, 0x5f, 0xff),
                (0xff, 0x87, 0x00),
                (0xff, 0x87, 0x5f),
                (0xff, 0x87, 0x87),
                (0xff, 0x87, 0xaf),
                (0xff, 0x87, 0xd7),
                (0xff, 0x87, 0xff),
                (0xff, 0xaf, 0x00),
                (0xff, 0xaf, 0x5f),
                (0xff, 0xaf, 0x87),
                (0xff, 0xaf, 0xaf),
                (0xff, 0xaf, 0xd7),
                (0xff, 0xaf, 0xff),
                (0xff, 0xd7, 0x00),
                (0xff, 0xd7, 0x5f),
                (0xff, 0xd7, 0x87),
                (0xff, 0xd7, 0xaf),
                (0xff, 0xd7, 0xd7),
                (0xff, 0xd7, 0xff),
                (0xff, 0xff, 0x00),
                (0xff, 0xff, 0x5f),
                (0xff, 0xff, 0x87),
                (0xff, 0xff, 0xaf),
                (0xff, 0xff, 0xd7),
                (0xff, 0xff, 0xff),
                (0x08, 0x08, 0x08),
                (0x12, 0x12, 0x12),
                (0x1c, 0x1c, 0x1c),
                (0x26, 0x26, 0x26),
                (0x30, 0x30, 0x30),
                (0x3a, 0x3a, 0x3a),
                (0x44, 0x44, 0x44),
                (0x4e, 0x4e, 0x4e),
                (0x58, 0x58, 0x58),
                (0x62, 0x62, 0x62),
                (0x6c, 0x6c, 0x6c),
                (0x76, 0x76, 0x76),
                (0x80, 0x80, 0x80),
                (0x8a, 0x8a, 0x8a),
                (0x94, 0x94, 0x94),
                (0x9e, 0x9e, 0x9e),
                (0xa8, 0xa8, 0xa8),
                (0xb2, 0xb2, 0xb2),
                (0xbc, 0xbc, 0xbc),
                (0xc6, 0xc6, 0xc6),
                (0xd0, 0xd0, 0xd0),
                (0xda, 0xda, 0xda),
                (0xe4, 0xe4, 0xe4),
                (0xee, 0xee, 0xee),
            ];
            VGA256[i as usize]
        }
        Color::Reset => (0, 0, 0),
    }
}

// + #

impl HandleEvent<crossterm::event::Event, Regular, TextOutcome> for ColorInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> TextOutcome {
        flow!(match event {
            ct_event!(key press '+') => {
                self.increment(1);
                TextOutcome::Changed
            }
            ct_event!(key press '-') => {
                self.decrement(1);
                TextOutcome::Changed
            }
            ct_event!(key press ALT-'+') => {
                self.increment(15);
                TextOutcome::Changed
            }
            ct_event!(key press ALT-'-') => {
                self.decrement(15);
                TextOutcome::Changed
            }
            ct_event!(key press CONTROL-'v') => {
                if self.paste_from_clip() {
                    TextOutcome::Changed
                } else {
                    TextOutcome::Continue
                }
            }
            ct_event!(key press CONTROL-'c') => {
                if self.copy_to_clip() {
                    TextOutcome::Changed
                } else {
                    TextOutcome::Continue
                }
            }
            ct_event!(key press CONTROL-'x') => {
                if self.cut_to_clip() {
                    TextOutcome::Changed
                } else {
                    TextOutcome::Continue
                }
            }
            _ => TextOutcome::Continue,
        });
        if !self.disable_modes {
            flow!(match event {
                ct_event!(key press 'r') => {
                    self.set_mode(Mode::RGB);
                    TextOutcome::Changed
                }
                ct_event!(key press 'h') => {
                    self.set_mode(Mode::HSV);
                    TextOutcome::Changed
                }
                ct_event!(key press 'x') => {
                    self.set_mode(Mode::HEX);
                    TextOutcome::Changed
                }
                ct_event!(key press 'm') | ct_event!(keycode press Up) => {
                    self.next_mode();
                    TextOutcome::Changed
                }
                ct_event!(key press SHIFT-'M') | ct_event!(keycode press Down) => {
                    self.prev_mode();
                    TextOutcome::Changed
                }
                _ => TextOutcome::Continue,
            });
        }
        match self.widget.handle(event, Regular) {
            TextOutcome::TextChanged => {
                self.normalize();
                self.value = self.parse_value();
                TextOutcome::TextChanged
            }
            r => r,
        }
    }
}

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for ColorInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ReadOnly) -> TextOutcome {
        self.widget.handle(event, ReadOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for ColorInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        self.widget.handle(event, MouseOnly)
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut ColorInputState,
    focus: bool,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.widget.focus.set(focus);
    HandleEvent::handle(state, event, Regular)
}

/// Handle only navigation events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_readonly_events(
    state: &mut ColorInputState,
    focus: bool,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.widget.focus.set(focus);
    state.handle(event, ReadOnly)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut ColorInputState,
    event: &crossterm::event::Event,
) -> TextOutcome {
    HandleEvent::handle(state, event, MouseOnly)
}
