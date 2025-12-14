#![allow(unused)]

use crate::_private::NonExhaustive;
use crossterm::event::{Event, KeyEvent};
use log::debug;
use rat_widget::button::{Button, ButtonState, ButtonStyle};
use rat_widget::checkbox::{Checkbox, CheckboxState, CheckboxStyle};
use rat_widget::choice::{
    Choice, ChoicePopup, ChoiceSelect, ChoiceState, ChoiceStyle, ChoiceWidget,
};
use rat_widget::event::{
    ButtonOutcome, CheckOutcome, ChoiceOutcome, ConsumedEvent, HandleEvent, Outcome, Popup, Regular,
};
use rat_widget::focus::{Focus, FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_widget::paired::{PairSplit, Paired, PairedState, PairedWidget};
use rat_widget::reloc::RelocatableState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::borrow::Cow;
use std::mem;

#[derive(Debug)]
enum Tool<'a> {
    CollapsedButtons(Cow<'a, str>),
    BasicButton(Cow<'a, str>, Cow<'a, str>, bool),
    BasicCheckbox(Cow<'a, str>, Cow<'a, str>, bool),
    BasicChoice(Cow<'a, str>, Vec<Line<'a>>),
    Text(Line<'a>),
}

#[derive(Debug)]
pub enum ToolState {
    BasicButton(ButtonState, bool),
    BasicCheckbox(CheckboxState),
    BasicChoice(ChoiceState<usize>),
}

#[derive(Debug)]
pub struct Toolbar<'a> {
    tools: Vec<Tool<'a>>,
    style: Style,
    block: Option<Block<'a>>,
    spacing: u16,
    key_style: Option<Style>,
    button_style: ButtonStyle,
    checkbox_style: CheckboxStyle,
    choice_style: ChoiceStyle,
}

#[derive(Debug)]
pub struct ToolbarWidget<'a> {
    tools: Vec<ToolWidget1<'a>>,
    block: Option<Block<'a>>,
}

#[derive(Debug)]
pub struct ToolbarPopup<'a> {
    tools: Vec<ToolWidget2<'a>>,
}

#[derive(Debug, Clone)]
pub struct ToolbarStyles {
    pub style: Style,
    pub block: Option<Block<'static>>,
    pub border_style: Option<Style>,
    pub title_style: Option<Style>,

    pub key_style: Option<Style>,

    pub button: Option<ButtonStyle>,
    pub checkbox: Option<CheckboxStyle>,
    pub choice: Option<ChoiceStyle>,

    pub non_exhaustive: NonExhaustive,
}

#[derive(Debug)]
pub struct ToolbarState {
    pub area: Rect,
    pub inner: Rect,

    pub collapsed: ChoiceState<Option<usize>>,
    pub collapsed_active: bool,
    pub tools: Vec<Option<ToolState>>,
    pub focus_before: Option<FocusFlag>,

    pub container: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ToolbarStyles {
    fn default() -> Self {
        Self {
            style: Default::default(),
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

impl<'a> Default for Toolbar<'a> {
    fn default() -> Self {
        Self {
            tools: Default::default(),
            style: Default::default(),
            block: Default::default(),
            spacing: 1,
            key_style: Default::default(),
            button_style: Default::default(),
            checkbox_style: Default::default(),
            choice_style: Default::default(),
        }
    }
}

impl<'a> Toolbar<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spacing(mut self, sp: u16) -> Self {
        self.spacing = sp;
        self
    }

    pub fn collapsed_buttons(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.tools.push(Tool::CollapsedButtons(text.into()));
        self
    }

    pub fn button(
        mut self,
        key: impl Into<Cow<'a, str>>,
        text: impl Into<Cow<'a, str>>,
        collapsible: bool,
    ) -> Self {
        self.tools
            .push(Tool::BasicButton(key.into(), text.into(), collapsible));
        self
    }

    pub fn checkbox(
        mut self,
        key: impl Into<Cow<'a, str>>,
        text: impl Into<Cow<'a, str>>,
        checked: bool,
    ) -> Self {
        self.tools
            .push(Tool::BasicCheckbox(key.into(), text.into(), checked));
        self
    }

    pub fn choice<V: Into<Line<'a>>>(
        mut self,
        key: impl Into<Cow<'a, str>>,
        items: impl IntoIterator<Item = V>,
    ) -> Self {
        self.tools.push(Tool::BasicChoice(
            key.into(),
            items.into_iter().map(|v| v.into()).collect(),
        ));
        self
    }

    pub fn text(mut self, text: impl Into<Line<'a>>) -> Self {
        self.tools.push(Tool::Text(text.into()));
        self
    }

    pub fn styles(mut self, styles: ToolbarStyles) -> Self {
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

    pub fn button_style(mut self, style: ButtonStyle) -> Self {
        self.button_style = style;
        self
    }

    pub fn checkbox_style(mut self, style: CheckboxStyle) -> Self {
        self.checkbox_style = style;
        self
    }

    pub fn choice_style(mut self, style: ChoiceStyle) -> Self {
        self.choice_style = style;
        self
    }

    // todo: width, height

    pub fn into_widgets(
        self,
        area: Rect,
        state: &mut ToolbarState,
    ) -> (ToolbarWidget<'a>, ToolbarPopup<'a>) {
        let block = self.block.clone();

        let (t1, t2) = layout(self, area, state);
        (
            ToolbarWidget { tools: t1, block },
            ToolbarPopup { tools: t2 },
        )
    }
}

enum ToolLayout<'a> {
    CollapsedButton(Line<'a>),
    BasicButton(u16, Button<'a>, Line<'a>, bool),
    BasicCheckbox(Checkbox<'a>),
    BasicChoice(Line<'a>, Choice<'a, usize>),
    Text(Line<'a>),
}

enum ToolLayout2<'a> {
    CollapsedPlaceHolder(),
    CollapsedButton(Choice<'a, Option<usize>>),
    BasicButton(usize, Button<'a>),
    BasicCheckbox(usize, Checkbox<'a>),
    BasicChoice(usize, Line<'a>, Choice<'a, usize>),
    Text(Line<'a>),
}

#[derive(Debug)]
enum ToolWidget1<'a> {
    CollapsedButton(Rect, ChoiceWidget<'a, Option<usize>>),
    BasicButton(usize, Rect, Button<'a>),
    BasicCheckbox(usize, Rect, Checkbox<'a>),
    BasicChoice(
        usize,
        Rect,
        Paired<'a, PairedWidget<'a, Line<'a>>, ChoiceWidget<'a, usize>>,
    ),
    Text(Rect, Line<'a>),
}

#[derive(Debug)]
enum ToolWidget2<'a> {
    CollapsedButton(ChoicePopup<'a, Option<usize>>),
    BasicChoice(usize, ChoicePopup<'a, usize>),
}

fn layout<'a>(
    widget: Toolbar<'a>,
    area: Rect,
    state: &mut ToolbarState,
) -> (Vec<ToolWidget1<'a>>, Vec<ToolWidget2<'a>>) {
    let inner = widget.block.inner_if_some(area);

    let key_style = widget.key_style.unwrap_or_default();

    // set up uncollapsed widgets
    let mut layout1 = Vec::with_capacity(widget.tools.len());
    let mut total_width = 0;
    let mut have_collapsed = false;
    for tool in widget.tools {
        match tool {
            Tool::CollapsedButtons(text) => {
                let text = Line::from(text);
                have_collapsed = true;
                layout1.push(ToolLayout::CollapsedButton(text));
            }
            Tool::BasicButton(key, text, collapse) => {
                let text =
                    Line::from_iter([Span::from(key).style(key_style.clone()), Span::from(text)]);
                let w = Button::new(text.clone()).styles(widget.button_style.clone());
                let w_width = w.width() + widget.spacing;
                total_width += w_width;
                layout1.push(ToolLayout::BasicButton(w_width, w, text, collapse));
            }
            Tool::BasicCheckbox(key, text, checked) => {
                let text =
                    Line::from_iter([Span::from(key).style(key_style.clone()), Span::from(text)]);
                let c = Checkbox::new()
                    .text(text)
                    .checked(checked)
                    .styles(widget.checkbox_style.clone());
                let w_width = c.width() + widget.spacing;
                total_width += w_width;
                layout1.push(ToolLayout::BasicCheckbox(c));
            }
            Tool::BasicChoice(key, items) => {
                let key = Line::from(key).style(key_style.clone());
                let c = Choice::new()
                    .items(items.into_iter().enumerate())
                    .styles(widget.choice_style.clone());
                let w_width = key.width() as u16 + c.width() + widget.spacing;
                total_width += w_width;
                layout1.push(ToolLayout::BasicChoice(key, c));
            }
            Tool::Text(txt) => {
                let w_width = txt.width() as u16 + widget.spacing;
                total_width += w_width;
                layout1.push(ToolLayout::Text(txt.clone()));
            }
        }
    }

    // reset collapsed active flag
    state.collapsed_active = false;
    for w in state.tools.iter_mut().flatten() {
        if let ToolState::BasicButton(_, active) = w {
            *active = false;
        }
    }

    // collapse buttons if necessary
    let mut layout2 = Vec::with_capacity(layout1.len());
    if total_width > inner.width && have_collapsed {
        let mut collapsed_width = 0;
        let mut collapsed = Choice::<Option<usize>>::new()
            .styles(widget.choice_style.clone())
            .behave_select(ChoiceSelect::MouseMove);

        let mut n = 0;
        for w in layout1.into_iter() {
            match w {
                ToolLayout::CollapsedButton(text) => {
                    collapsed = collapsed.unknown_item(text);
                    layout2.push(ToolLayout2::CollapsedPlaceHolder());
                }
                ToolLayout::BasicButton(w, button, text, collapse) => {
                    if total_width > inner.width && collapse {
                        total_width -= w;
                        total_width -= collapsed_width;

                        collapsed = collapsed.item(Some(n), text);

                        collapsed_width = collapsed.width() + widget.spacing;
                        total_width += collapsed_width;
                    } else {
                        layout2.push(ToolLayout2::BasicButton(n, button));
                    }
                    n += 1;
                }
                ToolLayout::BasicCheckbox(c) => {
                    layout2.push(ToolLayout2::BasicCheckbox(n, c));
                    n += 1;
                }
                ToolLayout::BasicChoice(t, c) => {
                    layout2.push(ToolLayout2::BasicChoice(n, t, c));
                    n += 1;
                }
                ToolLayout::Text(t) => {
                    layout2.push(ToolLayout2::Text(t));
                }
            }
        }

        for i in 0..layout2.len() {
            if matches!(layout2[i], ToolLayout2::CollapsedPlaceHolder()) {
                layout2[i] = ToolLayout2::CollapsedButton(collapsed);
                break;
            }
        }
    } else {
        let mut n = 0;
        for w in layout1.into_iter() {
            match w {
                ToolLayout::CollapsedButton(_) => {
                    layout2.push(ToolLayout2::CollapsedPlaceHolder());
                }
                ToolLayout::BasicButton(_, b, _, _) => {
                    layout2.push(ToolLayout2::BasicButton(n, b));
                    n += 1;
                }
                ToolLayout::BasicCheckbox(c) => {
                    layout2.push(ToolLayout2::BasicCheckbox(n, c));
                    n += 1;
                }
                ToolLayout::BasicChoice(t, c) => {
                    layout2.push(ToolLayout2::BasicChoice(n, t, c));
                    n += 1;
                }
                ToolLayout::Text(t) => {
                    layout2.push(ToolLayout2::Text(t));
                }
            }
        }
    }

    // create effective widgets
    let mut widgets1 = Vec::with_capacity(layout2.len());
    let mut widgets2 = Vec::with_capacity(layout2.len());
    let mut widget_area = inner;
    for w in layout2 {
        match w {
            ToolLayout2::CollapsedPlaceHolder() => {
                widget_area.width = 0;
            }
            ToolLayout2::CollapsedButton(w) => {
                state.collapsed_active = true;

                widget_area.width = w.width();
                let (w, p) = w.into_widgets();
                widgets1.push(ToolWidget1::CollapsedButton(widget_area, w));
                widgets2.push(ToolWidget2::CollapsedButton(p));
            }
            ToolLayout2::BasicButton(n, w) => {
                while state.tools.len() <= n {
                    state.tools.push(None);
                }
                if state.tools[n].is_none() {
                    state.tools[n] = Some(ToolState::BasicButton(ButtonState::default(), true));
                } else {
                    if let Some(ToolState::BasicButton(_, active)) = &mut state.tools[n] {
                        *active = true;
                    }
                }

                widget_area.width = w.width();
                widgets1.push(ToolWidget1::BasicButton(n, widget_area, w));
            }
            ToolLayout2::BasicCheckbox(n, w) => {
                while state.tools.len() <= n {
                    state.tools.push(None);
                }
                if state.tools[n].is_none() {
                    state.tools[n] = Some(ToolState::BasicCheckbox(CheckboxState::default()));
                }
                widget_area.width = w.width();
                widgets1.push(ToolWidget1::BasicCheckbox(n, widget_area, w));
            }
            ToolLayout2::BasicChoice(n, key, w) => {
                while state.tools.len() <= n {
                    state.tools.push(None);
                }
                if state.tools[n].is_none() {
                    state.tools[n] = Some(ToolState::BasicChoice(ChoiceState::default()));
                }

                let key_len = key.width() as u16;
                widget_area.width = key_len + w.width();
                let (w, p) = w.into_widgets();
                widgets1.push(ToolWidget1::BasicChoice(
                    n,
                    widget_area,
                    Paired::new(PairedWidget::new(key), w)
                        .spacing(0)
                        .split(PairSplit::Fix1(key_len)),
                ));
                widgets2.push(ToolWidget2::BasicChoice(n, p));
            }
            ToolLayout2::Text(w) => {
                widget_area.width = w.width() as u16;
                widgets1.push(ToolWidget1::Text(widget_area, w));
            }
        }

        if widget_area.width > 0 {
            widget_area.x += widget_area.width;
            widget_area.x += widget.spacing;
        }
    }

    // hide all.
    state.collapsed.relocate_hidden();
    state.collapsed.relocate_popup_hidden();
    for w in state.tools.iter_mut().flatten() {
        match w {
            ToolState::BasicButton(w, _) => {
                w.relocate_hidden();
            }
            ToolState::BasicCheckbox(w) => {
                w.relocate_hidden();
            }
            ToolState::BasicChoice(w) => {
                w.relocate_hidden();
                w.relocate_popup_hidden();
            }
        }
    }

    (widgets1, widgets2)
}

impl<'a> StatefulWidget for ToolbarWidget<'a> {
    type State = ToolbarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

fn render_ref(widget: ToolbarWidget, area: Rect, buf: &mut Buffer, state: &mut ToolbarState) {
    state.area = area;
    state.inner = widget.block.inner_if_some(area);

    for w in widget.tools {
        match w {
            ToolWidget1::CollapsedButton(widget_area, w) => {
                w.render(widget_area, buf, &mut state.collapsed);
            }
            ToolWidget1::BasicButton(n, widget_area, w) => {
                let ToolState::BasicButton(state, _) = state.tools[n].as_mut().expect("state")
                else {
                    unreachable!("invalid_state");
                };
                w.render(widget_area, buf, state);
            }
            ToolWidget1::BasicCheckbox(n, widget_area, w) => {
                let ToolState::BasicCheckbox(state) = state.tools[n].as_mut().expect("state")
                else {
                    unreachable!("invalid_state");
                };
                w.render(widget_area, buf, state);
            }
            ToolWidget1::BasicChoice(n, widget_area, w) => {
                let ToolState::BasicChoice(state) = state.tools[n].as_mut().expect("state") else {
                    unreachable!("invalid_state");
                };
                w.render(widget_area, buf, &mut PairedState::new(&mut (), state));
            }
            ToolWidget1::Text(widget_area, w) => {
                w.render(widget_area, buf);
            }
        }
    }
}

impl<'a> StatefulWidget for ToolbarPopup<'a> {
    type State = ToolbarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_popup(self, area, buf, state);
    }
}

fn render_popup(widget: ToolbarPopup, _area: Rect, buf: &mut Buffer, state: &mut ToolbarState) {
    for w in widget.tools {
        match w {
            ToolWidget2::CollapsedButton(w) => {
                w.render(Rect::default(), buf, &mut state.collapsed);
            }
            ToolWidget2::BasicChoice(n, w) => {
                let ToolState::BasicChoice(state) = state.tools[n].as_mut().expect("state") else {
                    unreachable!("invalid_state");
                };
                w.render(Rect::default(), buf, state);
            }
        }
    }
}

impl Default for ToolbarState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            collapsed: Default::default(),
            collapsed_active: Default::default(),
            tools: Default::default(),
            focus_before: Default::default(),
            container: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocus for ToolbarState {
    fn build(&self, builder: &mut FocusBuilder) {
        for w in self.tools.iter().flatten() {
            match w {
                ToolState::BasicButton(_, _) => {}
                ToolState::BasicCheckbox(_) => {}
                ToolState::BasicChoice(w) => {
                    builder.widget_navigate(w, Navigation::Leave);
                }
            }
        }
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
        self.collapsed.relocate(shift, clip);
        self.collapsed.relocate_popup(shift, clip);
        for w in self.tools.iter_mut().flatten() {
            match w {
                ToolState::BasicButton(w, active) => {
                    if *active {
                        w.relocate(shift, clip);
                    }
                }
                ToolState::BasicCheckbox(w) => {
                    w.relocate(shift, clip);
                }
                ToolState::BasicChoice(w) => {
                    w.relocate_popup(shift, clip);
                }
            }
        }
    }
}

impl ToolbarState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub enum ToolbarOutcome {
    /// The given event was not handled at all.
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

pub struct ToolbarKeys<'a, const N: usize> {
    pub focus: &'a Focus,
    pub keys: [Option<KeyEvent>; N],
}

impl<const N: usize> HandleEvent<Event, ToolbarKeys<'_, N>, ToolbarOutcome> for ToolbarState {
    fn handle(&mut self, event: &Event, qualifier: ToolbarKeys<N>) -> ToolbarOutcome {
        if let Event::Key(event) = event {
            for (n, key) in qualifier.keys.iter().enumerate() {
                if let Some(key) = key.as_ref()
                    && event == key
                {
                    debug!("n {} {:?}", n, self.tools[n]);
                    match &mut self.tools[n] {
                        Some(ToolState::BasicButton(_, _)) => {
                            return ToolbarOutcome::Pressed(n);
                        }
                        Some(ToolState::BasicCheckbox(w)) => {
                            w.flip_checked();
                            return ToolbarOutcome::Checked(n, w.value());
                        }
                        Some(ToolState::BasicChoice(w)) => {
                            if w.is_focused() {
                                if let Some(focus_before) = self.focus_before.as_ref() {
                                    qualifier.focus.focus(focus_before);
                                } else {
                                    qualifier.focus.next();
                                }
                            } else {
                                qualifier.focus.focus(w);
                                self.focus_before = qualifier.focus.lost_focus();
                            }
                            return ToolbarOutcome::Changed;
                        }
                        None => {}
                    }
                }
            }
        }

        if self.collapsed_active {
            match self.collapsed.handle(event, Popup) {
                ChoiceOutcome::Value | ChoiceOutcome::Changed => {
                    if !self.collapsed.is_popup_active() {
                        if let Some(focus_before) = self.focus_before.as_ref() {
                            qualifier.focus.focus(focus_before);
                        } else {
                            qualifier.focus.next();
                        }
                        if let Some(value) = self.collapsed.value() {
                            self.collapsed.set_value(None);
                            return ToolbarOutcome::Pressed(value);
                        } else {
                            return ToolbarOutcome::Changed;
                        }
                    } else {
                        return ToolbarOutcome::Changed;
                    }
                }
                ChoiceOutcome::Continue => {
                    if self.collapsed.lost_focus() {
                        self.collapsed.set_value(None);
                    }
                }
                r => return ToolbarOutcome::from(Outcome::from(r)),
            }
        }

        for (n, w) in self.tools.iter_mut().enumerate() {
            match w {
                Some(ToolState::BasicButton(w, active)) => {
                    if !*active {
                        continue;
                    }
                    match w.handle(event, Regular) {
                        ButtonOutcome::Pressed => return ToolbarOutcome::Pressed(n),
                        ButtonOutcome::Continue => {}
                        r => return ToolbarOutcome::from(Outcome::from(r)),
                    }
                }
                Some(ToolState::BasicCheckbox(w)) => match w.handle(event, Regular) {
                    CheckOutcome::Value => return ToolbarOutcome::Checked(n, w.value()),
                    CheckOutcome::Continue => {}
                    r => return ToolbarOutcome::from(Outcome::from(r)),
                },
                Some(ToolState::BasicChoice(w)) => match w.handle(event, Popup) {
                    ChoiceOutcome::Value | ChoiceOutcome::Changed => {
                        if !w.is_popup_active() {
                            if let Some(focus_before) = self.focus_before.as_ref() {
                                qualifier.focus.focus(focus_before);
                            } else {
                                qualifier.focus.next();
                            }
                            return ToolbarOutcome::Selected(n, w.value());
                        } else {
                            return ToolbarOutcome::Changed;
                        }
                    }
                    ChoiceOutcome::Continue => {}
                    r => return ToolbarOutcome::from(Outcome::from(r)),
                },
                None => {}
            }
        }

        ToolbarOutcome::Continue
    }
}
