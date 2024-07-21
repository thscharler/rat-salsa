//!
//! A message dialog.
//!

use crate::_private::NonExhaustive;
use crate::button::{Button, ButtonOutcome, ButtonState, ButtonStyle};
use crate::fill::Fill;
use crate::layout::layout_dialog;
use rat_event::{ct_event, ConsumedEvent, Dialog, HandleEvent, Outcome, Regular};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Flex, Margin, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, StatefulWidgetRef, Widget};
use std::cell::{Cell, RefCell};
use std::fmt::Debug;

/// Basic status dialog for longer messages.
#[derive(Debug, Default, Clone)]
pub struct MsgDialog<'a> {
    block: Option<Block<'a>>,
    style: Style,
    button_style: ButtonStyle,
}

/// Combined style.
#[derive(Debug, Clone)]
pub struct MsgDialogStyle {
    pub style: Style,
    pub button: ButtonStyle,
    pub non_exhaustive: NonExhaustive,
}

/// State for the status dialog.
#[derive(Debug, Clone)]
pub struct MsgDialogState {
    /// Area
    pub area: Rect,

    /// Dialog is active.
    pub active: Cell<bool>,
    /// Dialog text.
    pub message: RefCell<String>,

    /// Ok button
    pub button: ButtonState,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> MsgDialog<'a> {
    /// New widget
    pub fn new() -> Self {
        Self {
            block: None,
            style: Default::default(),
            button_style: Default::default(),
        }
    }

    /// Block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Combined style
    pub fn styles(mut self, styles: MsgDialogStyle) -> Self {
        self.style = styles.style;
        self.button_style = styles.button;
        self
    }

    /// Base style
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Button style.
    pub fn button_style(mut self, style: ButtonStyle) -> Self {
        self.button_style = style;
        self
    }
}

impl Default for MsgDialogStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            button: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl MsgDialogState {
    pub fn set_active(&self, active: bool) {
        self.active.set(active);
    }

    pub fn active(&self) -> bool {
        self.active.get()
    }

    /// Clear message text, set active to false.
    pub fn clear(&self) {
        self.active.set(false);
        *self.message.borrow_mut() = Default::default();
    }

    /// *Append* to the message.
    pub fn append(&self, msg: &str) {
        self.active.set(true);
        let mut message = self.message.borrow_mut();
        if !message.is_empty() {
            message.push('\n');
        }
        message.push_str(msg);
    }
}

impl Default for MsgDialogState {
    fn default() -> Self {
        let s = Self {
            active: Default::default(),
            area: Default::default(),
            message: Default::default(),
            button: Default::default(),
            non_exhaustive: NonExhaustive,
        };
        s.button.focus.set(true);
        s
    }
}

impl<'a> StatefulWidgetRef for MsgDialog<'a> {
    type State = MsgDialogState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for MsgDialog<'a> {
    type State = MsgDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &MsgDialog<'_>, area: Rect, buf: &mut Buffer, state: &mut MsgDialogState) {
    if state.active.get() {
        let l_dlg = layout_dialog(
            area,
            Constraint::Percentage(61),
            Constraint::Percentage(61),
            Margin::new(1, 1),
            [Constraint::Length(10)],
            0,
            Flex::End,
        );

        state.area = l_dlg.area;

        Fill::new()
            .fill_char(" ")
            .style(widget.style)
            .render(state.area, buf);

        widget.block.render(l_dlg.dialog, buf);

        {
            let message = state.message.borrow();
            let mut lines = Vec::new();
            for t in message.split('\n') {
                lines.push(Line::from(t));
            }
            let text = Text::from(lines).alignment(Alignment::Center);
            Paragraph::new(text).render(l_dlg.area, buf);
        }

        Button::from("Ok")
            .styles(widget.button_style.clone())
            .render(l_dlg.buttons[0], buf, &mut state.button);
    }
}

impl HandleEvent<crossterm::event::Event, Dialog, Outcome> for MsgDialogState {
    fn handle(&mut self, event: &crossterm::event::Event, _: Dialog) -> Outcome {
        if self.active.get() {
            match self.button.handle(event, Regular) {
                ButtonOutcome::Pressed => {
                    self.clear();
                    self.active.set(false);
                    Outcome::Changed
                }
                v => v.into(),
            }
            .or_else(|| match event {
                ct_event!(keycode press Esc) => {
                    self.clear();
                    self.active.set(false);
                    Outcome::Changed
                }
                _ => Outcome::NotUsed,
            })
            .or_else(|| {
                // mandatory consume everything else.
                Outcome::Unchanged
            })
        } else {
            Outcome::NotUsed
        }
    }
}

/// Handle events for the MsgDialog.
pub fn handle_dialog_events(
    state: &mut MsgDialogState,
    event: &crossterm::event::Event,
) -> Outcome {
    state.handle(event, Dialog)
}
