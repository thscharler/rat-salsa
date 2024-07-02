//!
//! A message dialog.
//!

use crate::_private::NonExhaustive;
use crate::button::{Button, ButtonOutcome, ButtonState, ButtonStyle};
use crate::layout::layout_dialog;
use rat_event::{ct_event, ConsumedEvent, Dialog, FocusKeys, HandleEvent, Outcome};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Flex, Margin, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, StatefulWidgetRef, Widget};
use std::fmt::Debug;

/// Basic status dialog for longer messages.
#[derive(Debug, Default, Clone)]
pub struct MsgDialog {
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
    /// Dialog is active.
    pub active: bool,
    /// Area
    pub area: Rect,
    /// Dialog text.
    pub message: String,
    /// Ok button
    pub button: ButtonState,

    pub non_exhaustive: NonExhaustive,
}

impl MsgDialog {
    /// New widget
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            button_style: Default::default(),
        }
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
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn active(&self) -> bool {
        self.active
    }

    /// Clear message text, set active to false.
    pub fn clear(&mut self) {
        self.active = false;
        self.message = Default::default();
    }

    /// *Append* to the message.
    pub fn append(&mut self, msg: &str) {
        self.active = true;
        if !self.message.is_empty() {
            self.message.push('\n');
        }
        self.message.push_str(msg);
    }
}

impl Default for MsgDialogState {
    fn default() -> Self {
        let s = Self {
            active: false,
            area: Default::default(),
            message: Default::default(),
            button: Default::default(),
            non_exhaustive: NonExhaustive,
        };
        s.button.focus.set(true);
        s
    }
}

impl StatefulWidgetRef for MsgDialog {
    type State = MsgDialogState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl StatefulWidget for MsgDialog {
    type State = MsgDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &MsgDialog, area: Rect, buf: &mut Buffer, state: &mut MsgDialogState) {
    if state.active {
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

        //
        let block = Block::default().style(widget.style);

        let mut lines = Vec::new();
        for t in state.message.split('\n') {
            lines.push(Line::from(t));
        }
        let text = Text::from(lines).alignment(Alignment::Center);
        let para = Paragraph::new(text);

        let ok = Button::from("Ok").styles(widget.button_style.clone());

        for y in l_dlg.dialog.y..l_dlg.dialog.bottom() {
            let idx = buf.index_of(l_dlg.dialog.x, y);
            for x in 0..l_dlg.dialog.width as usize {
                buf.content[idx + x].reset();
                buf.content[idx + x].set_style(widget.style);
            }
        }

        block.render(l_dlg.dialog, buf);
        para.render(l_dlg.area, buf);
        ok.render(l_dlg.buttons[0], buf, &mut state.button);
    }
}

impl HandleEvent<crossterm::event::Event, Dialog, Outcome> for MsgDialogState {
    fn handle(&mut self, event: &crossterm::event::Event, _: Dialog) -> Outcome {
        if self.active {
            match self.button.handle(event, FocusKeys) {
                ButtonOutcome::Pressed => {
                    self.clear();
                    self.active = false;
                    Outcome::Changed
                }
                v => v.into(),
            }
            .or_else(|| match event {
                ct_event!(keycode press Esc) => {
                    self.clear();
                    self.active = false;
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
