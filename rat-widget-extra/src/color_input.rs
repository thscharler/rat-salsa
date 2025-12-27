//! Widget for color input.
//!
//! Currently, supports
//! * RGB
//! * HSV
//! * hexdigits
//!
//! __Keybindings__
//!
//! * Switch between color-mode with Up/Down. (or m/M)
//! * '+' and Alt-'+' increase the value
//! * '-' and Alt-'-' decrease the value
//! * 'r', 'h', 'x' switch mode
//!
//! __Clipboard__
//!
//! Recognizes common formats when pasted from the clipboard.
//! * #000000 and #00000000
//! * 0x000000 and 0x00000000
//! * 000000 and 00000000
//!

use crate::_private::NonExhaustive;
use palette::{FromColor, Hsv, Srgb};
use rat_event::{HandleEvent, MouseOnly, Regular, ct_event, flow};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::RelocatableState;
use rat_text::clipboard::Clipboard;
use rat_text::event::{ReadOnly, TextOutcome};
use rat_text::text_input_mask::{MaskedInput, MaskedInputState};
use rat_text::{
    TextError, TextFocusGained, TextFocusLost, TextStyle, TextTab, derive_text_widget_state,
    upos_type,
};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Style};
use ratatui_core::text::Line;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::{Block, BlockExt};
use std::cmp::min;
use std::ops::Range;

/// Color input widget.
///
/// A text input for colors.
///
#[derive(Debug, Clone)]
pub struct ColorInput<'a> {
    style: Style,
    block: Option<Block<'a>>,

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
    /// ...
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
#[derive(Debug, Clone)]
pub struct ColorInputState {
    /// Area of the widget.
    /// __read only__ renewed with each render.
    pub area: Rect,
    /// Area inside the block.
    pub inner: Rect,
    /// Area for the mode.
    /// __read_only__ renewed with each render.
    pub mode_area: Rect,
    /// Area for the mode label.
    /// __read_only__ renewed with each render.
    pub label_area: Rect,

    /// value as RGB with 0.0..1.0 ranges.
    value: (f32, f32, f32),

    /// Disable keys for mode switching.
    /// __read only__
    disable_modes: bool,
    /// __read only__
    mode: Mode,
    /// __read only__
    pub widget: MaskedInputState,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> Default for ColorInput<'a> {
    fn default() -> Self {
        let mut z = Self {
            style: Default::default(),
            disable_modes: Default::default(),
            mode: Default::default(),
            block: Default::default(),
            widget: MaskedInput::default(),
        };
        z.widget = z.widget.on_focus_lost(TextFocusLost::Position0);
        z
    }
}

// derive_text_widget!(ColorInput<'a>);

impl<'a> ColorInput<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the combined style.
    #[inline]
    pub fn styles(mut self, mut style: ColorInputStyle) -> Self {
        self.style = style.text.style;
        if let Some(block) = style.text.block.take() {
            self.block = Some(block);
        }
        if let Some(border_style) = style.text.border_style {
            self.block = self.block.map(|v| v.style(border_style));
        }
        if let Some(title_style) = style.text.title_style {
            self.block = self.block.map(|v| v.style(title_style));
        }
        self.block = self.block.map(|v| v.style(self.style));
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
        self.block = self.block.map(|v| v.style(style));
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
        self.block = Some(block);
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

    /// `Tab` behaviour
    #[inline]
    pub fn on_tab(mut self, of: TextTab) -> Self {
        self.widget = self.widget.on_tab(of);
        self
    }

    /// Preferred width
    pub fn width(&self) -> u16 {
        16
    }

    /// Preferred height
    pub fn height(&self) -> u16 {
        1
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

    let inner = widget.block.inner_if_some(area);

    let mode_area = Rect::new(inner.x, inner.y, 4, inner.height);
    let mode_label = Rect::new(mode_area.x, mode_area.y + mode_area.height / 2, 4, 1);
    let widget_area = Rect::new(
        inner.x + mode_area.width,
        inner.y,
        inner.width.saturating_sub(mode_area.width),
        inner.height,
    );

    state.area = area;
    state.inner = inner;
    state.mode_area = mode_area;
    state.label_area = mode_label;

    let bg = state.value();
    let fg_colors = [Color::Black, Color::White];
    let style = high_contrast_color(bg, &fg_colors);

    widget.block.clone().render(area, buf);

    buf.set_style(mode_area, style);
    let mode_str = match state.mode {
        Mode::RGB => "RGB",
        Mode::HEX => "  #",
        Mode::HSV => "HSV",
    };
    Line::from(mode_str).render(mode_label, buf);

    (&widget.widget).render(widget_area, buf, &mut state.widget);
}

derive_text_widget_state!( BASE ColorInputState );
// derive_text_widget_state!( CLIPBOARD ColorInputState );
derive_text_widget_state!( UNDO ColorInputState );
derive_text_widget_state!( STYLE ColorInputState );
derive_text_widget_state!( OFFSET ColorInputState );
// derive_text_widget_state!( EDIT ColorInputState );
// derive_text_widget_state!( FOCUS ColorInputState );
derive_text_widget_state!( SCREENCURSOR ColorInputState );
// derive_text_widget_state!( RELOCATE ColorInputState );

impl Default for ColorInputState {
    fn default() -> Self {
        let mut z = Self {
            area: Default::default(),
            inner: Default::default(),
            mode_area: Default::default(),
            label_area: Default::default(),
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
        let mut z = Self::default();
        z.widget.focus = z.widget.focus.with_name(name);
        z
    }
}

impl ColorInputState {
    /// Clipboard used.
    /// Default is to use the [global_clipboard](rat_text::clipboard::global_clipboard).
    #[inline]
    pub fn set_clipboard(&mut self, clip: Option<impl Clipboard + 'static>) {
        self.widget.set_clipboard(clip);
    }

    /// Clipboard used.
    /// Default is to use the [global_clipboard](rat_text::clipboard::global_clipboard).
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

        if self.has_selection() {
            _ = clip.set_string(self.selected_text().as_ref());
        } else {
            let r = (self.value.0 * 255f32) as u32;
            let g = (self.value.1 * 255f32) as u32;
            let b = (self.value.2 * 255f32) as u32;
            let value_str = format!("{:06x}", (r << 16) + (g << 8) + b);
            _ = clip.set_string(&value_str);
        }
        true
    }

    /// Cut to clipboard
    #[inline]
    pub fn cut_to_clip(&mut self) -> bool {
        let Some(clip) = self.widget.value.clipboard() else {
            return false;
        };

        if self.has_selection() {
            match clip.set_string(self.selected_text().as_ref()) {
                Ok(_) => self.delete_range(self.selection()),
                Err(_) => false,
            }
        } else {
            let r = (self.value.0 * 255f32) as u32;
            let g = (self.value.1 * 255f32) as u32;
            let b = (self.value.2 * 255f32) as u32;
            let value_str = format!("{:06x}", (r << 16) + (g << 8) + b);

            if clip.set_string(&value_str).is_ok() {
                self.clear()
            }
            true
        }
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
    /// Value as Color.
    pub fn value(&self) -> Color {
        Color::Rgb(
            (self.value.0 * 255f32) as u8,
            (self.value.1 * 255f32) as u8,
            (self.value.2 * 255f32) as u8,
        )
    }

    /// Get the value as f32 triple
    pub fn value_f32(&self) -> (f32, f32, f32) {
        self.value
    }

    /// Get the value as u32
    pub fn value_u32(&self) -> u32 {
        (((self.value.0 * 255f32) as u32) << 16)
            + (((self.value.1 * 255f32) as u32) << 8)
            + ((self.value.2 * 255f32) as u32)
    }

    fn parse_value(&self) -> (f32, f32, f32) {
        match self.mode {
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
        }
    }
}

impl ColorInputState {
    /// Reset to empty.
    #[inline]
    pub fn clear(&mut self) {
        self.widget.clear();
    }

    /// Set the color as f32 triple
    pub fn set_value_f32(&mut self, color: (f32, f32, f32)) {
        self.value = color;
        self.value_to_text();
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

const PAT_RGB: &str = "##0 ##0 ##0";
const SEC_R: u16 = 1;
const SEC_G: u16 = 3;
const SEC_B: u16 = 5;

const PAT_HEX: &str = "HHHHHH";
#[allow(dead_code)]
const SEC_X: u16 = 1;

const PAT_HSV: &str = "##0 ##0 ##0";
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
    pub fn change_section(&mut self, n: i32) -> bool {
        self.change_section_pos(self.cursor(), n)
    }

    pub fn change_section_pos(&mut self, pos: upos_type, n: i32) -> bool {
        let section = self.widget.section_id(pos);
        let r = match self.mode {
            Mode::RGB => match section {
                SEC_R | SEC_G | SEC_B => {
                    let r = self
                        .widget
                        .section_value::<u32>(section)
                        .unwrap_or_default();
                    let r_min = min(r.saturating_add_signed(n), 255);
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
                    let r_min = {
                        let mut r_min = (r as i32 + n) % 360;
                        if r_min < 0 {
                            r_min += 360;
                        }
                        r_min as u32
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
                    let r_min = min(r.saturating_add_signed(n), 100);
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

    /// Color mode.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Set the color mode.
    pub fn set_mode(&mut self, mode: Mode) -> bool {
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
        true
    }

    /// Switch to next mode.
    pub fn next_mode(&mut self) -> bool {
        match self.mode {
            Mode::RGB => self.set_mode(Mode::HEX),
            Mode::HEX => self.set_mode(Mode::HSV),
            Mode::HSV => self.set_mode(Mode::RGB),
        }
    }

    /// Switch to prev mode.
    pub fn prev_mode(&mut self) -> bool {
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

impl RelocatableState for ColorInputState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area.relocate(shift, clip);
        self.inner.relocate(shift, clip);
        self.mode_area.relocate(shift, clip);
        self.label_area.relocate(shift, clip);
        self.widget.relocate(shift, clip);
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

impl HandleEvent<Event, Regular, TextOutcome> for ColorInputState {
    fn handle(&mut self, event: &Event, _keymap: Regular) -> TextOutcome {
        if self.is_focused() {
            flow!(match event {
                ct_event!(key press '+') | ct_event!(keycode press Up) =>
                    self.change_section(1).into(),
                ct_event!(key press '-') | ct_event!(keycode press Down) =>
                    self.change_section(-1).into(),
                ct_event!(key press ALT-'+') | ct_event!(keycode press ALT-Up) =>
                    self.change_section(7).into(),
                ct_event!(key press ALT-'-') | ct_event!(keycode press ALT-Down) =>
                    self.change_section(-7).into(),
                ct_event!(key press CONTROL-'v') => self.paste_from_clip().into(),
                ct_event!(key press CONTROL-'c') => self.copy_to_clip().into(),
                ct_event!(key press CONTROL-'x') => self.cut_to_clip().into(),
                _ => TextOutcome::Continue,
            });
            if !self.disable_modes {
                flow!(match event {
                    ct_event!(key press 'r') => self.set_mode(Mode::RGB).into(),
                    ct_event!(key press 'h') => self.set_mode(Mode::HSV).into(),
                    ct_event!(key press 'x') => self.set_mode(Mode::HEX).into(),
                    ct_event!(key press 'm') | ct_event!(keycode press PageUp) =>
                        self.next_mode().into(),
                    ct_event!(key press SHIFT-'M') | ct_event!(keycode press PageDown) =>
                        self.prev_mode().into(),
                    _ => TextOutcome::Continue,
                });
            }
        }

        flow!(handle_mouse(self, event));

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

impl HandleEvent<Event, ReadOnly, TextOutcome> for ColorInputState {
    fn handle(&mut self, event: &Event, _keymap: ReadOnly) -> TextOutcome {
        self.widget.handle(event, ReadOnly)
    }
}

impl HandleEvent<Event, MouseOnly, TextOutcome> for ColorInputState {
    fn handle(&mut self, event: &Event, _keymap: MouseOnly) -> TextOutcome {
        flow!(handle_mouse(self, event));
        self.widget.handle(event, MouseOnly)
    }
}

fn handle_mouse(state: &mut ColorInputState, event: &Event) -> TextOutcome {
    if state.is_focused() {
        match event {
            ct_event!(scroll ALT down for x,y) if state.mode_area.contains((*x, *y).into()) => {
                state.next_mode().into()
            }
            ct_event!(scroll ALT up for x,y) if state.mode_area.contains((*x, *y).into()) => {
                state.prev_mode().into()
            }
            ct_event!(scroll down for x,y) if state.widget.area.contains((*x, *y).into()) => {
                let rx = state
                    .widget
                    .screen_to_col((*x - state.widget.area.x) as i16);
                state.change_section_pos(rx, -1).into()
            }
            ct_event!(scroll up for x,y) if state.widget.area.contains((*x, *y).into()) => {
                let rx = state
                    .widget
                    .screen_to_col((*x - state.widget.area.x) as i16);
                state.change_section_pos(rx, 1).into()
            }
            ct_event!(scroll ALT down for x,y) if state.widget.area.contains((*x, *y).into()) => {
                let rx = state
                    .widget
                    .screen_to_col((*x - state.widget.area.x) as i16);
                state.change_section_pos(rx, -7).into()
            }
            ct_event!(scroll ALT up for x,y) if state.widget.area.contains((*x, *y).into()) => {
                let rx = state
                    .widget
                    .screen_to_col((*x - state.widget.area.x) as i16);
                state.change_section_pos(rx, 7).into()
            }
            _ => TextOutcome::Continue,
        }
    } else {
        TextOutcome::Continue
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(state: &mut ColorInputState, focus: bool, event: &Event) -> TextOutcome {
    state.widget.focus.set(focus);
    HandleEvent::handle(state, event, Regular)
}

/// Handle only navigation events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_readonly_events(
    state: &mut ColorInputState,
    focus: bool,
    event: &Event,
) -> TextOutcome {
    state.widget.focus.set(focus);
    state.handle(event, ReadOnly)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(state: &mut ColorInputState, event: &Event) -> TextOutcome {
    HandleEvent::handle(state, event, MouseOnly)
}
