//!
//! Label/Caption for a linked widget with hotkey focusing.
//!
//! ** unstable **
//!
use crate::_private::NonExhaustive;
use crate::util::revert_style;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, HandleEvent, Outcome};
use rat_focus::{Focus, FocusFlag, HasFocus};
use rat_reloc::{relocate_area, RelocatableState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{StatefulWidget, Widget};
use std::borrow::Cow;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

///
/// A label/caption linked to another widget.
///
/// *** unstable ***
///
#[derive(Debug, Clone)]
pub struct Caption<'a> {
    /// Text
    text: Cow<'a, str>,
    /// Text range to highlight. A byte-range into text.
    highlight: Option<Range<usize>>,
    /// Navigation key char.
    navchar: Option<char>,
    /// Hotkey text.
    hotkey_text: Cow<'a, str>,
    /// Hotkey alignment
    hotkey_align: HotkeyAlignment,
    /// Hotkey policy
    hotkey_policy: HotkeyPolicy,
    /// Hot-key 2
    hotkey: Option<crossterm::event::KeyEvent>,
    /// Text/Hotkey spacing
    spacing: u16,
    /// Label alignment
    align: Alignment,

    /// Linked widget
    linked: Option<FocusFlag>,

    style: Style,
    hover_style: Option<Style>,
    highlight_style: Option<Style>,
    hotkey_style: Option<Style>,
    focus_style: Option<Style>,
}

#[derive(Debug)]
pub struct CaptionState {
    /// Area for the whole widget.
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// Hot-key 1
    /// __readonly__. renewed for each render.
    pub navchar: Option<char>,
    /// Hot-key 2
    /// __limited__. renewed for each render if set with the widget.
    pub hotkey: Option<crossterm::event::KeyEvent>,

    /// Associated widget
    pub linked: FocusFlag,

    /// Flags for mouse handling.
    /// __used for mouse interaction__
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

/// Composite style for the caption.
#[derive(Debug, Clone)]
pub struct CaptionStyle {
    /// Base style
    pub style: Style,
    /// Hover style
    pub hover: Option<Style>,
    /// Highlight style
    pub highlight: Option<Style>,
    /// Hotkey style
    pub hotkey: Option<Style>,
    /// Focus style
    pub focus: Option<Style>,

    /// Label alignment
    pub align: Option<Alignment>,
    /// Hotkey alignment
    pub hotkey_align: Option<HotkeyAlignment>,
    /// Hotkey policy
    pub hotkey_policy: Option<HotkeyPolicy>,
    /// Label/hotkey spacing
    pub spacing: Option<u16>,

    pub non_exhaustive: NonExhaustive,
}

/// Policy for hover.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyPolicy {
    /// No special behaviour. Hotkey text is always shown.
    #[default]
    Always,
    /// Only show the hotkey text on hover.
    OnHover,
    /// Only show the hotkey text when the main widget is focused.
    WhenFocused,
}

/// Alignment of label-text and hotkey-text.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAlignment {
    /// Display as "label hotkey"
    #[default]
    LabelHotkey,
    /// Display as "hotkey label"
    HotkeyLabel,
}

impl<'a> Default for Caption<'a> {
    fn default() -> Self {
        Self {
            text: Default::default(),
            highlight: Default::default(),
            navchar: Default::default(),
            hotkey_text: Default::default(),
            hotkey_align: Default::default(),
            hotkey_policy: Default::default(),
            hotkey: Default::default(),
            spacing: 1,
            align: Default::default(),
            linked: Default::default(),
            style: Default::default(),
            hover_style: Default::default(),
            highlight_style: Default::default(),
            hotkey_style: Default::default(),
            focus_style: Default::default(),
        }
    }
}

impl<'a> Caption<'a> {
    /// New
    pub fn new() -> Self {
        Default::default()
    }

    /// Uses '_' as special character.
    ///
    /// __Item__
    ///
    /// The first '_' marks the navigation-char.
    /// Pipe '|' separates the item text and the hotkey text.
    pub fn parse(txt: &'a str) -> Self {
        let mut zelf = Caption::default();

        let mut idx_underscore = None;
        let mut idx_navchar_start = None;
        let mut idx_navchar_end = None;
        let mut idx_pipe = None;

        let cit = txt.char_indices();
        for (idx, c) in cit {
            if idx_underscore.is_none() && c == '_' {
                idx_underscore = Some(idx);
            } else if idx_underscore.is_some() && idx_navchar_start.is_none() {
                idx_navchar_start = Some(idx);
            } else if idx_navchar_start.is_some() && idx_navchar_end.is_none() {
                idx_navchar_end = Some(idx);
            }
            if c == '|' {
                idx_pipe = Some(idx);
            }
        }
        if idx_navchar_start.is_some() && idx_navchar_end.is_none() {
            idx_navchar_end = Some(txt.len());
        }

        if let Some(pipe) = idx_pipe {
            if let Some(navchar_end) = idx_navchar_end {
                if navchar_end > pipe {
                    idx_pipe = None;
                }
            }
        }

        let (text, hotkey_text) = if let Some(idx_pipe) = idx_pipe {
            (&txt[..idx_pipe], &txt[idx_pipe + 1..])
        } else {
            (txt, "")
        };

        if let Some(idx_navchar_start) = idx_navchar_start {
            if let Some(idx_navchar_end) = idx_navchar_end {
                zelf.text = Cow::Borrowed(text);
                zelf.highlight = Some(idx_navchar_start..idx_navchar_end);
                zelf.navchar = Some(
                    text[idx_navchar_start..idx_navchar_end]
                        .chars()
                        .next()
                        .expect("char")
                        .to_ascii_lowercase(),
                );
                zelf.hotkey_text = Cow::Borrowed(hotkey_text);
            } else {
                unreachable!();
            }
        } else {
            zelf.text = Cow::Borrowed(text);
            zelf.highlight = None;
            zelf.navchar = None;
            zelf.hotkey_text = Cow::Borrowed(hotkey_text);
        }

        zelf
    }

    /// Set the label text.
    ///
    /// You probably want to use [parsed] instead.
    /// This is only useful if you want manual control over
    /// highlight-range and hotkey text.
    pub fn text(mut self, txt: &'a str) -> Self {
        self.text = Cow::Borrowed(txt);
        self
    }

    /// Spacing between text and hotkey-text.
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Alternate navigation key
    pub fn hotkey(mut self, hotkey: crossterm::event::KeyEvent) -> Self {
        self.hotkey = Some(hotkey);
        self
    }

    /// Hotkey text
    pub fn hotkey_text(mut self, hotkey: &'a str) -> Self {
        self.hotkey_text = Cow::Borrowed(hotkey);
        self
    }

    /// Alignment of the hotkey text.
    pub fn hotkey_align(mut self, align: HotkeyAlignment) -> Self {
        self.hotkey_align = align;
        self
    }

    /// Policy for when to show the hotkey text.
    pub fn hotkey_policy(mut self, policy: HotkeyPolicy) -> Self {
        self.hotkey_policy = policy;
        self
    }

    /// Set the linked widget.
    pub fn link(mut self, widget: &impl HasFocus) -> Self {
        self.linked = Some(widget.focus());
        self
    }

    /// Byte-range into text to be highlighted.
    pub fn highlight(mut self, bytes: Range<usize>) -> Self {
        self.highlight = Some(bytes);
        self
    }

    /// Navigation-char.
    pub fn navchar(mut self, navchar: char) -> Self {
        self.navchar = Some(navchar);
        self
    }

    /// Label alignment.
    pub fn align(mut self, align: Alignment) -> Self {
        self.align = align;
        self
    }

    /// Set all styles.
    pub fn styles(mut self, styles: CaptionStyle) -> Self {
        self.style = styles.style;
        if styles.hover.is_some() {
            self.hover_style = styles.hover;
        }
        if styles.highlight.is_some() {
            self.highlight_style = styles.highlight;
        }
        if styles.hotkey.is_some() {
            self.hotkey_style = styles.hotkey;
        }
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        if let Some(spacing) = styles.spacing {
            self.spacing = spacing;
        }
        if let Some(align) = styles.hotkey_align {
            self.hotkey_align = align;
        }
        if let Some(align) = styles.align {
            self.align = align;
        }
        if let Some(hotkey_policy) = styles.hotkey_policy {
            self.hotkey_policy = hotkey_policy;
        }
        self
    }

    /// Base style.
    #[inline]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Hover style.
    #[inline]
    pub fn hover_style(mut self, style: Style) -> Self {
        self.hover_style = Some(style);
        self
    }

    /// Hover style.
    #[inline]
    pub fn hover_opt(mut self, style: Option<Style>) -> Self {
        self.hover_style = style;
        self
    }

    /// Shortcut highlight style.
    #[inline]
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = Some(style);
        self
    }

    /// Shortcut highlight style.
    #[inline]
    pub fn highlight_style_opt(mut self, style: Option<Style>) -> Self {
        self.highlight_style = style;
        self
    }

    /// Style for the hotkey.
    #[inline]
    pub fn hotkey_style(mut self, style: Style) -> Self {
        self.hotkey_style = Some(style);
        self
    }

    /// Style for the hotkey.
    #[inline]
    pub fn hotkey_style_opt(mut self, style: Option<Style>) -> Self {
        self.hotkey_style = style;
        self
    }

    /// Base-style when the main widget is focused.
    #[inline]
    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    /// Base-style when the main widget is focused.
    #[inline]
    pub fn focus_style_opt(mut self, style: Option<Style>) -> Self {
        self.focus_style = style;
        self
    }

    /// Inherent width
    pub fn text_width(&self) -> u16 {
        self.text.graphemes(true).count() as u16
    }

    /// Inherent width
    pub fn hotkey_width(&self) -> u16 {
        self.hotkey_text.graphemes(true).count() as u16
    }

    /// Inherent width
    pub fn width(&self) -> u16 {
        self.text_width() + self.hotkey_width()
    }

    /// Inherent height
    pub fn height(&self) -> u16 {
        1
    }
}

impl<'a> StatefulWidget for &Caption<'a> {
    type State = CaptionState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for Caption<'a> {
    type State = CaptionState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &Caption<'_>, area: Rect, buf: &mut Buffer, state: &mut CaptionState) {
    state.area = area;

    if widget.navchar.is_some() {
        state.navchar = widget.navchar;
    }
    if widget.hotkey.is_some() {
        state.hotkey = widget.hotkey;
    }
    if let Some(linked) = &widget.linked {
        state.linked = linked.clone();
    }

    let mut prepend = String::default();
    let mut append = String::default();

    // styles
    let style = if state.linked.is_focused() {
        if let Some(focus_style) = widget.focus_style {
            focus_style
        } else {
            revert_style(widget.style)
        }
    } else {
        widget.style
    };

    let mut highlight_style = if let Some(highlight_style) = widget.highlight_style {
        highlight_style
    } else {
        Style::new().underlined()
    };
    if let Some(hover_style) = widget.hover_style {
        if state.mouse.hover.get() {
            highlight_style = highlight_style.patch(hover_style);
        }
    }
    highlight_style = style.patch(highlight_style);

    let mut hotkey_style = if let Some(hotkey_style) = widget.hotkey_style {
        hotkey_style
    } else {
        Style::default()
    };
    if let Some(hover_style) = widget.hover_style {
        if state.mouse.hover.get() {
            hotkey_style = hotkey_style.patch(hover_style);
        }
    }
    hotkey_style = style.patch(hotkey_style);

    // layout

    let hotkey_text = if widget.hotkey_policy == HotkeyPolicy::WhenFocused && state.linked.get()
        || widget.hotkey_policy == HotkeyPolicy::OnHover && state.mouse.hover.get()
        || widget.hotkey_policy == HotkeyPolicy::Always
    {
        widget.hotkey_text.as_ref()
    } else {
        ""
    };

    if !hotkey_text.is_empty() && widget.spacing > 0 {
        match widget.hotkey_align {
            HotkeyAlignment::LabelHotkey => {
                append = " ".repeat(widget.spacing as usize);
            }
            HotkeyAlignment::HotkeyLabel => {
                prepend = " ".repeat(widget.spacing as usize);
            }
        }
    }

    let text_line = match widget.hotkey_align {
        HotkeyAlignment::LabelHotkey => {
            if let Some(highlight) = widget.highlight.clone() {
                Line::from_iter([
                    Span::from(&widget.text[..highlight.start - 1]), // account for _
                    Span::from(&widget.text[highlight.start..highlight.end]).style(highlight_style),
                    Span::from(&widget.text[highlight.end..]),
                    Span::from(append),
                    Span::from(hotkey_text).style(hotkey_style),
                ])
            } else {
                Line::from_iter([
                    Span::from(widget.text.as_ref()), //
                    Span::from(append),
                    Span::from(hotkey_text).style(hotkey_style),
                ])
            }
        }
        HotkeyAlignment::HotkeyLabel => {
            if let Some(highlight) = widget.highlight.clone() {
                Line::from_iter([
                    Span::from(hotkey_text).style(hotkey_style),
                    Span::from(prepend),
                    Span::from(&widget.text[..highlight.start - 1]), // account for _
                    Span::from(&widget.text[highlight.start..highlight.end]).style(highlight_style),
                    Span::from(&widget.text[highlight.end..]),
                ])
            } else {
                Line::from_iter([
                    Span::from(hotkey_text).style(hotkey_style),
                    Span::from(prepend), //
                    Span::from(widget.text.as_ref()),
                ])
            }
        }
    };
    text_line
        .alignment(widget.align) //
        .style(style)
        .render(state.area, buf);
}

impl Default for CaptionStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            hover: Default::default(),
            highlight: Default::default(),
            hotkey: Default::default(),
            focus: Default::default(),
            align: Default::default(),
            hotkey_align: Default::default(),
            hotkey_policy: Default::default(),
            spacing: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Clone for CaptionState {
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            navchar: self.navchar,
            hotkey: self.hotkey,
            linked: self.linked.clone(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for CaptionState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            navchar: Default::default(),
            hotkey: Default::default(),
            linked: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl CaptionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn navchar(&self) -> Option<char> {
        self.navchar
    }

    pub fn set_navchar(&mut self, navchar: Option<char>) {
        self.navchar = navchar;
    }

    pub fn hotkey(&self) -> Option<crossterm::event::KeyEvent> {
        self.hotkey
    }

    pub fn set_hotkey(&mut self, hotkey: Option<crossterm::event::KeyEvent>) {
        self.hotkey = hotkey;
    }

    pub fn linked(&self) -> FocusFlag {
        self.linked.clone()
    }

    pub fn set_linked(&mut self, linked: FocusFlag) {
        self.linked = linked;
    }
}

impl RelocatableState for CaptionState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
    }
}

impl<'a> HandleEvent<crossterm::event::Event, &'a Focus, Outcome> for CaptionState {
    fn handle(&mut self, event: &crossterm::event::Event, focus: &'a Focus) -> Outcome {
        if let Some(navchar) = self.navchar {
            if let crossterm::event::Event::Key(crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Char(test),
                modifiers: crossterm::event::KeyModifiers::ALT,
                kind: crossterm::event::KeyEventKind::Release,
                ..
            }) = event
            {
                if navchar == *test {
                    focus.focus(&self.linked);
                    return Outcome::Changed;
                }
            }
        }
        if let Some(hotkey) = self.hotkey {
            if let crossterm::event::Event::Key(crossterm::event::KeyEvent {
                code,
                modifiers,
                kind,
                ..
            }) = event
            {
                if hotkey.code == *code && hotkey.modifiers == *modifiers && hotkey.kind == *kind {
                    focus.focus(&self.linked);
                    return Outcome::Changed;
                }
            }
        }

        // no separate mouse-handler, isok
        match event {
            ct_event!(mouse any for m) if self.mouse.hover(self.area, m) => {
                return Outcome::Changed
            }
            ct_event!(mouse down Left for x,y) if self.area.contains((*x, *y).into()) => {
                focus.focus(&self.linked);
                return Outcome::Changed;
            }
            _ => {}
        }

        Outcome::Continue
    }
}

/// Handle all events. for a Caption.
///
/// This additionally requires a valid Focus instance to handle
/// the hot-keys.
pub fn handle_events(
    state: &mut CaptionState,
    focus: &Focus,
    event: &crossterm::event::Event,
) -> Outcome {
    HandleEvent::handle(state, event, focus)
}
