//!
//! Toolbar with Buttons, Checkbox and Choices.
//!
//! ```
//! # use rat_event::Regular;
//! # use rat_theme4::{StyleName, WidgetStyle};
//! # use rat_widget::event::{HandleEvent, Outcome, event_flow};
//! # use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
//! # use rat_widget::toolbar::{Toolbar, ToolbarOutcome, ToolbarState};
//! # use ratatui_core::buffer::Buffer;
//! # use ratatui_core::layout::{Constraint, Layout, Rect};
//! # use ratatui_core::style::Style;
//! # use ratatui_core::widgets::StatefulWidget;
//! # use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
//!
//!     let (w, p) = Toolbar::new()
//!         .text("ABC")
//!         .button("F1", "Function")
//!         .hotkey(KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE))
//!         .button("F2", "Function")
//!         .hotkey(KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE))
//!         .checkbox("A", "nticipate", state.anticipate)
//!         .hotkey(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE))
//!         .button("F3", "Function")
//!         .hotkey(KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE))
//!         .checkbox("B", "eatification", state.beatification)
//!         .hotkey(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE))
//!         .choice("F7", ["Choose 1", "Choose 2", "Choose 3"], state.chosen)
//!         .hotkey(KeyEvent::new(KeyCode::F(7), KeyModifiers::NONE))
//!         .styles(ctx.theme.style(WidgetStyle::TOOLBAR))
//!         .into_widgets(l0[0], &mut state.tools);
//!
//!     w.render(l0[0], buf, &mut state.tools);
//! ```

use crate::_private::NonExhaustive;
use crate::button::{Button, ButtonState, ButtonStyle};
use crate::checkbox::{Checkbox, CheckboxState, CheckboxStyle};
use crate::choice::{Choice, ChoicePopup, ChoiceState, ChoiceStyle, ChoiceWidget};
use crate::event::{ButtonOutcome, CheckOutcome, ChoiceOutcome};
use crate::paired::{PairSplit, Paired, PairedState, PairedWidget};
use crate::util::block_size;
use rat_event::{ConsumedEvent, HandleEvent, Outcome, Popup, Regular, event_flow};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_reloc::RelocatableState;
use rat_scrolled::Scroll;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Flex, Layout, Rect, Spacing};
use ratatui_core::style::Style;
use ratatui_core::text::{Line, Span};
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::{Event, KeyEvent, KeyEventKind};
use ratatui_widgets::block::{Block, BlockExt};
use std::borrow::Cow;
use std::mem;

#[derive(Debug)]
enum Tool<'a> {
    BasicButton(Cow<'a, str>, Cow<'a, str>, Option<KeyEvent>),
    BasicCheckbox(Cow<'a, str>, Cow<'a, str>, bool, Option<KeyEvent>),
    BasicChoice(Cow<'a, str>, Vec<Line<'a>>, usize, Option<KeyEvent>),
    Text(Line<'a>),
}

#[allow(clippy::large_enum_variant)]
/// States for the different toolbar-widgets.
#[derive(Debug)]
pub enum ToolState {
    None,
    BasicButton(ButtonState, Option<KeyEvent>),
    BasicCheckbox(CheckboxState, Option<KeyEvent>),
    BasicChoice(ChoiceState<usize>, Option<KeyEvent>),
    Text,
}

/// A toolbar with Buttons, Checkboxes and Choices.
#[derive(Debug, Default)]
pub struct Toolbar<'a> {
    tools: Vec<Tool<'a>>,
    style: Style,
    block: Option<Block<'a>>,
    spacing: Spacing,
    flex: Flex,
    key_style: Option<Style>,
    button_style: ButtonStyle,
    checkbox_style: CheckboxStyle,
    choice_style: ChoiceStyle,
}

/// Widget for the toolbar.
#[derive(Debug)]
pub struct ToolbarWidget<'a> {
    tools: Vec<ToolWidget1<'a>>,
    style: Style,
    block: Option<Block<'a>>,
}

/// Widget for any popups.
#[derive(Debug)]
pub struct ToolbarPopup<'a> {
    tools: Vec<ToolWidget2<'a>>,
}

/// Comprehensive styles.
#[derive(Debug, Clone)]
pub struct ToolbarStyle {
    pub style: Style,
    pub spacing: Option<Spacing>,
    pub flex: Option<Flex>,
    pub block: Option<Block<'static>>,
    pub border_style: Option<Style>,
    pub title_style: Option<Style>,

    pub key_style: Option<Style>,

    pub button: Option<ButtonStyle>,
    pub checkbox: Option<CheckboxStyle>,
    pub choice: Option<ChoiceStyle>,

    pub non_exhaustive: NonExhaustive,
}

/// State
#[derive(Debug)]
pub struct ToolbarState {
    /// Full area.
    /// __read only__
    pub area: Rect,
    /// Area inside the block.
    /// __read only__
    pub inner: Rect,

    /// Tool state.
    pub tools: Vec<ToolState>,
    /// Choices can get the focus. This is to flip back to the
    /// focus before.
    pub focus_before: Option<FocusFlag>,

    /// Container focus flag.
    pub container: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ToolbarStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            spacing: Default::default(),
            flex: Default::default(),
            block: Default::default(),
            border_style: Default::default(),
            title_style: Default::default(),
            key_style: Default::default(),
            button: Default::default(),
            checkbox: Default::default(),
            choice: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> Toolbar<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Spacing between tools.
    pub fn spacing(mut self, sp: impl Into<Spacing>) -> Self {
        self.spacing = sp.into();
        self
    }

    /// Flex for the tool layout.
    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }

    /// Add a button.
    /// * key - Hotkey text
    /// * text - Button text
    ///
    /// To set the actual hotkey call [hotkey](Toolbar::hotkey) next.
    pub fn button(mut self, key: impl Into<Cow<'a, str>>, text: impl Into<Cow<'a, str>>) -> Self {
        self.tools
            .push(Tool::BasicButton(key.into(), text.into(), None));
        self
    }

    /// Add a checkbox.
    /// * key - hotkey
    /// * text - text
    /// * checked
    ///
    /// To set the actual hotkey call [hotkey](Toolbar::hotkey) next.
    pub fn checkbox(
        mut self,
        key: impl Into<Cow<'a, str>>,
        text: impl Into<Cow<'a, str>>,
        checked: bool,
    ) -> Self {
        self.tools
            .push(Tool::BasicCheckbox(key.into(), text.into(), checked, None));
        self
    }

    /// Add a choice.
    ///
    /// To set the actual hotkey call [hotkey](Toolbar::hotkey) next.
    pub fn choice<V: Into<Line<'a>>>(
        mut self,
        key: impl Into<Cow<'a, str>>,
        items: impl IntoIterator<Item = V>,
        selected: usize,
    ) -> Self {
        self.tools.push(Tool::BasicChoice(
            key.into(),
            items.into_iter().map(|v| v.into()).collect(),
            selected,
            None,
        ));
        self
    }

    /// Define the hotkey for the previously added tool.
    pub fn hotkey(mut self, key: KeyEvent) -> Self {
        match self.tools.last_mut() {
            Some(Tool::BasicButton(_, _, k))
            | Some(Tool::BasicCheckbox(_, _, _, k))
            | Some(Tool::BasicChoice(_, _, _, k)) => {
                *k = Some(key);
            }
            Some(Tool::Text(..)) => {}
            None => {}
        }
        self
    }

    /// Add some text to the toolbar.
    pub fn text(mut self, text: impl Into<Line<'a>>) -> Self {
        self.tools.push(Tool::Text(text.into()));
        self
    }

    /// Set combined styles.
    pub fn styles(mut self, styles: ToolbarStyle) -> Self {
        self.style = styles.style;
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if let Some(border_style) = styles.border_style {
            self.block = self.block.map(|v| v.border_style(border_style));
        }
        if let Some(title_style) = styles.title_style {
            self.block = self.block.map(|v| v.title_style(title_style));
        }
        self.block = self.block.map(|v| v.style(self.style));

        if let Some(spacing) = styles.spacing {
            self.spacing = spacing;
        }
        if let Some(flex) = styles.flex {
            self.flex = flex;
        }

        if styles.key_style.is_some() {
            self.key_style = styles.key_style;
        }
        if let Some(button) = styles.button {
            self.button_style = button;
        }
        if let Some(checkbox) = styles.checkbox {
            self.checkbox_style = checkbox;
        }
        if let Some(choice) = styles.choice {
            self.choice_style = choice;
        }
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Block for the main widget.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Keyboard short-cut style.
    pub fn key_style(mut self, style: Style) -> Self {
        self.key_style = Some(style);
        self
    }

    /// Button style.
    pub fn button_style(mut self, style: ButtonStyle) -> Self {
        self.button_style = style;
        self
    }

    /// Checkbox style.
    pub fn checkbox_style(mut self, style: CheckboxStyle) -> Self {
        self.checkbox_style = style;
        self
    }

    /// Choice style.
    pub fn choice_style(mut self, style: ChoiceStyle) -> Self {
        self.choice_style = style;
        self
    }

    // no inherent width, at least none that can be calculated easily

    pub fn height(&self) -> u16 {
        block_size(&self.block).height + 1
    }

    /// Create the final widgets for rendering.
    pub fn into_widgets(
        self,
        area: Rect,
        state: &mut ToolbarState,
    ) -> (ToolbarWidget<'a>, ToolbarPopup<'a>) {
        let block = self.block.clone();
        let style = self.style.clone();

        let (t1, t2) = layout(self, area, state);
        (
            ToolbarWidget {
                tools: t1,
                style,
                block,
            },
            ToolbarPopup {
                tools: t2 //
            },
        )
    }
}

#[derive(Debug)]
enum ToolWidget1<'a> {
    BasicButton(Rect, Button<'a>),
    BasicCheckbox(Rect, Checkbox<'a>),
    BasicChoice(
        Rect,
        Paired<'a, PairedWidget<'a, Line<'a>>, ChoiceWidget<'a, usize>>,
    ),
    Text(Rect, Line<'a>),
}

#[derive(Debug)]
enum ToolWidget2<'a> {
    None,
    BasicChoice(ChoicePopup<'a, usize>),
}

fn layout<'a>(
    mut widget: Toolbar<'a>,
    area: Rect,
    state: &mut ToolbarState,
) -> (Vec<ToolWidget1<'a>>, Vec<ToolWidget2<'a>>) {
    let inner = widget.block.inner_if_some(area);
    let key_style = widget.key_style.unwrap_or_default();

    enum ToolLayout<'a> {
        BasicButton(Button<'a>),
        BasicCheckbox(Checkbox<'a>),
        BasicChoice(Line<'a>, Choice<'a>, usize),
        BasicText(Line<'a>),
    }

    let mut tools = Vec::with_capacity(widget.tools.len());
    let mut layout = Vec::with_capacity(widget.tools.len());
    for tool in &mut widget.tools {
        match tool {
            Tool::BasicButton(k, t, _) => {
                let text = Line::from_iter([
                    Span::from(mem::take(k)).style(key_style.clone()),
                    Span::from(mem::take(t)),
                ]);
                let b = Button::new(text).styles(widget.button_style.clone());
                let w = b.width();
                tools.push(ToolLayout::BasicButton(b));
                layout.push(Constraint::Length(w));
            }
            Tool::BasicCheckbox(k, t, c, _) => {
                let text = Line::from_iter([
                    Span::from(mem::take(k)).style(key_style.clone()),
                    Span::from(mem::take(t)),
                ]);
                let c = Checkbox::new()
                    .text(text)
                    .checked(*c)
                    .styles(widget.checkbox_style.clone());
                let w = c.width();
                tools.push(ToolLayout::BasicCheckbox(c));
                layout.push(Constraint::Length(w));
            }
            Tool::BasicChoice(k, i, s, _) => {
                let k = Line::from(mem::take(k)).style(key_style.clone());
                let c = Choice::new()
                    .items(mem::take(i).into_iter().enumerate())
                    .popup_scroll(Scroll::new())
                    .styles(widget.choice_style.clone());
                let w = k.width() as u16 + c.width();
                tools.push(ToolLayout::BasicChoice(k, c, *s));
                layout.push(Constraint::Length(w));
            }
            Tool::Text(t) => {
                let w = t.width() as u16;
                tools.push(ToolLayout::BasicText(mem::take(t)));
                layout.push(Constraint::Length(w));
            }
        }
    }

    let ll = Layout::horizontal(&layout)
        .spacing(widget.spacing)
        .flex(widget.flex)
        .horizontal_margin(0)
        .split(inner);

    let mut widgets1 = Vec::with_capacity(ll.len());
    let mut widgets2 = Vec::with_capacity(ll.len());

    state.tools.resize_with(tools.len(), || ToolState::None);

    for (n, tool) in tools.into_iter().enumerate() {
        let hk = match &widget.tools[n] {
            Tool::BasicButton(_, _, k) => k.clone(),
            Tool::BasicCheckbox(_, _, _, k) => k.clone(),
            Tool::BasicChoice(_, _, _, k) => k.clone(),
            Tool::Text(_) => None,
        };

        match tool {
            ToolLayout::BasicButton(b) => {
                match &mut state.tools[n] {
                    ToolState::BasicButton(_, hotkey) => {
                        *hotkey = hk;
                    }
                    t => *t = ToolState::BasicButton(ButtonState::default(), hk),
                }

                widgets1.push(ToolWidget1::BasicButton(ll[n], b));
                widgets2.push(ToolWidget2::None);
            }
            ToolLayout::BasicCheckbox(c) => {
                match &mut state.tools[n] {
                    ToolState::BasicCheckbox(_, hotkey) => {
                        *hotkey = hk;
                    }
                    t => *t = ToolState::BasicCheckbox(CheckboxState::default(), hk),
                }

                widgets1.push(ToolWidget1::BasicCheckbox(ll[n], c));
                widgets2.push(ToolWidget2::None);
            }
            ToolLayout::BasicChoice(k, c, selected) => {
                match &mut state.tools[n] {
                    ToolState::BasicChoice(_, hotkey) => {
                        *hotkey = hk;
                    }
                    t => *t = ToolState::BasicChoice(ChoiceState::default(), hk),
                }

                let ToolState::BasicChoice(choice_state, _) = &mut state.tools[n] else {
                    unreachable!("invalid_state");
                };
                if !choice_state.is_popup_active() {
                    choice_state.set_value(selected);
                }

                let key_len = k.width() as u16;
                let (w, p) = c.into_widgets();

                widgets1.push(ToolWidget1::BasicChoice(
                    ll[n],
                    Paired::new(PairedWidget::new(k), w)
                        .spacing(0)
                        .split(PairSplit::Fix1(key_len)),
                ));
                widgets2.push(ToolWidget2::BasicChoice(p));
            }
            ToolLayout::BasicText(t) => {
                match &mut state.tools[n] {
                    ToolState::Text => {}
                    t => *t = ToolState::Text,
                }

                widgets1.push(ToolWidget1::Text(ll[n], t));
                widgets2.push(ToolWidget2::None);
            }
        }
    }

    (widgets1, widgets2)
}

impl<'a> StatefulWidget for ToolbarWidget<'a> {
    type State = ToolbarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.inner = self.block.inner_if_some(area);

        if self.block.is_some() {
            self.block.render(area, buf);
        } else {
            buf.set_style(area, self.style);
        }

        for (n, w) in self.tools.into_iter().enumerate() {
            match w {
                ToolWidget1::BasicButton(area, w) => {
                    let ToolState::BasicButton(state, _) = &mut state.tools[n] else {
                        panic!("state doesn't match tool-widget");
                    };
                    w.render(area, buf, state);
                }
                ToolWidget1::BasicCheckbox(area, w) => {
                    let ToolState::BasicCheckbox(state, _) = &mut state.tools[n] else {
                        panic!("state doesn't match tool-widget");
                    };
                    w.render(area, buf, state);
                }
                ToolWidget1::BasicChoice(area, w) => {
                    let ToolState::BasicChoice(state, _) = &mut state.tools[n] else {
                        panic!("state doesn't match tool-widget");
                    };
                    w.render(area, buf, &mut PairedState::new(&mut (), state));
                }
                ToolWidget1::Text(area, w) => {
                    w.render(area, buf);
                }
            }
        }
    }
}

impl<'a> StatefulWidget for ToolbarPopup<'a> {
    type State = ToolbarState;

    fn render(self, _area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        for (n, w) in self.tools.into_iter().enumerate() {
            match w {
                ToolWidget2::BasicChoice(w) => {
                    let ToolState::BasicChoice(state, _) = &mut state.tools[n] else {
                        panic!("state doesn't match tool-widget");
                    };
                    w.render(Rect::default(), buf, state);
                }
                ToolWidget2::None => {}
            }
        }
    }
}

impl Default for ToolbarState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            tools: Default::default(),
            focus_before: Default::default(),
            container: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocus for ToolbarState {
    fn build(&self, builder: &mut FocusBuilder) {
        let f = builder.start(self);
        for w in &self.tools {
            match w {
                ToolState::BasicButton(w, _) => {
                    builder.widget_navigate(w, Navigation::None);
                }
                ToolState::BasicCheckbox(w, _) => {
                    builder.widget_navigate(w, Navigation::None);
                }
                ToolState::BasicChoice(w, _) => {
                    builder.widget_navigate(w, Navigation::Leave);
                }
                ToolState::None => {}
                ToolState::Text => {}
            }
        }
        builder.end(f);
    }

    fn focus(&self) -> FocusFlag {
        self.container.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl RelocatableState for ToolbarState {
    fn relocate(&mut self, _shift: (i16, i16), _clip: Rect) {}

    fn relocate_popup(&mut self, shift: (i16, i16), clip: Rect) {
        self.area.relocate(shift, clip);
        self.inner.relocate(shift, clip);
        for w in &mut self.tools {
            match w {
                ToolState::BasicButton(w, _) => {
                    w.relocate(shift, clip);
                }
                ToolState::BasicCheckbox(w, _) => {
                    w.relocate(shift, clip);
                }
                ToolState::BasicChoice(w, _) => {
                    w.relocate_popup(shift, clip);
                }
                ToolState::None => {}
                ToolState::Text => {}
            }
        }
    }
}

impl ToolbarState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ToolState {
    pub fn hotkey(&self) -> Option<KeyEvent> {
        match self {
            ToolState::BasicButton(_, hk) => hk.clone(),
            ToolState::BasicCheckbox(_, hk) => hk.clone(),
            ToolState::BasicChoice(_, hk) => hk.clone(),
            ToolState::None => None,
            ToolState::Text => None,
        }
    }
}

/// Result type for event-handling.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ToolbarOutcome {
    /// The given event was not handled at all.
    #[default]
    Continue,
    /// The event was handled, no repaint necessary.
    Unchanged,
    /// The event was handled, repaint necessary.
    Changed,
    /// Button N has been activated.
    Pressed(usize),
    /// Checkbox N has been checked/unchecked.
    Checked(usize, bool),
    /// Choice N has changed selection.
    /// (N, Selection)
    Selected(usize, usize),
}

impl ConsumedEvent for ToolbarOutcome {
    fn is_consumed(&self) -> bool {
        !matches!(self, ToolbarOutcome::Continue)
    }
}

impl From<Outcome> for ToolbarOutcome {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::Continue => ToolbarOutcome::Continue,
            Outcome::Unchanged => ToolbarOutcome::Unchanged,
            Outcome::Changed => ToolbarOutcome::Changed,
        }
    }
}

impl From<ToolbarOutcome> for Outcome {
    fn from(value: ToolbarOutcome) -> Self {
        match value {
            ToolbarOutcome::Continue => Outcome::Continue,
            ToolbarOutcome::Unchanged => Outcome::Unchanged,
            ToolbarOutcome::Changed => Outcome::Changed,
            ToolbarOutcome::Pressed(_) => Outcome::Changed,
            ToolbarOutcome::Selected(_, _) => Outcome::Changed,
            ToolbarOutcome::Checked(_, _) => Outcome::Changed,
        }
    }
}

impl<'a> HandleEvent<Event, Regular, ToolbarOutcome> for ToolbarState {
    fn handle(&mut self, event: &Event, _: Regular) -> ToolbarOutcome {
        if let Event::Key(event) = event {
            for (n, tool) in self.tools.iter_mut().enumerate() {
                let Some(hotkey) = tool.hotkey() else {
                    continue;
                };
                if event.code == hotkey.code && event.modifiers == hotkey.modifiers {
                    match tool {
                        ToolState::None => {}
                        ToolState::BasicButton(w, _) => event_flow!(
                            return {
                                match w.pressed(event.kind == KeyEventKind::Press) {
                                    ButtonOutcome::Pressed => ToolbarOutcome::Pressed(n),
                                    r => ToolbarOutcome::from(Outcome::from(r)),
                                }
                            }
                        ),
                        ToolState::BasicCheckbox(w, _) => event_flow!(
                            return {
                                if event.kind == KeyEventKind::Press {
                                    w.flip_checked();
                                    ToolbarOutcome::Checked(n, w.value())
                                } else {
                                    ToolbarOutcome::Unchanged
                                }
                            }
                        ),
                        ToolState::BasicChoice(w, _) => event_flow!(
                            return {
                                if event.kind == KeyEventKind::Press {
                                    if let Some(focus) = self.container.focus() {
                                        focus.flip_focus(w, &mut self.focus_before);
                                    }
                                    ToolbarOutcome::Changed
                                } else {
                                    ToolbarOutcome::Unchanged
                                }
                            }
                        ),
                        ToolState::Text => {}
                    }
                }
            }
        }

        for (n, w) in self.tools.iter_mut().enumerate() {
            match w {
                ToolState::None => {}
                ToolState::BasicButton(w, _) => event_flow!(
                    return match w.handle(event, Regular) {
                        ButtonOutcome::Pressed => ToolbarOutcome::Pressed(n),
                        r => ToolbarOutcome::from(Outcome::from(r)),
                    }
                ),
                ToolState::BasicCheckbox(w, _) => event_flow!(
                    return match w.handle(event, Regular) {
                        CheckOutcome::Value => ToolbarOutcome::Checked(n, w.value()),
                        r => ToolbarOutcome::from(Outcome::from(r)),
                    }
                ),
                ToolState::BasicChoice(w, _) => event_flow!(
                    return match w.handle(event, Popup) {
                        ChoiceOutcome::Value | ChoiceOutcome::Changed => {
                            if !w.is_popup_active() {
                                if let Some(focus) = self.container.focus() {
                                    if let Some(focus_before) = self.focus_before.as_ref() {
                                        focus.focus(focus_before);
                                    } else {
                                        focus.next();
                                    }
                                }
                                ToolbarOutcome::Selected(n, w.value())
                            } else {
                                ToolbarOutcome::Changed
                            }
                        }
                        r => {
                            ToolbarOutcome::from(Outcome::from(r))
                        }
                    }
                ),
                ToolState::Text => {}
            }
        }

        ToolbarOutcome::Continue
    }
}
