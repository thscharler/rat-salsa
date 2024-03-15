//!
//! Status line and a message dialog.
//!

use crate::check_break;
use crate::layout::layout_dialog;
use crate::widget::button::{Button, ButtonState, ButtonStyle};
use crate::ControlUI;
use crate::{DefaultKeys, HandleCrossterm};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Flex, Margin, Rect};
use ratatui::prelude::{StatefulWidget, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Clear, Paragraph, Widget};
use std::fmt::Debug;

/// Basic status line.
#[derive(Debug, Default)]
pub struct StatusLine {
    pub style: Style,
}

/// State for the status line.
#[derive(Debug, Default)]
pub struct StatusLineState {
    pub area: Rect,
    pub status: String,
}

/// Basic status dialog for longer messages.
#[derive(Debug, Default)]
pub struct StatusDialog {
    pub style: Style,
    pub button_style: ButtonStyle,
}

/// Combined style.
#[derive(Default, Debug)]
pub struct StatusDialogStyle {
    pub style: Style,
    pub button: ButtonStyle,
}

/// State for the status dialog.
#[derive(Debug)]
pub struct StatusDialogState {
    pub active: bool,
    pub area: Rect,
    pub button: ButtonState<bool>,
    pub log: String,
}

impl StatusLine {
    pub fn new() -> Self {
        Self {
            style: Default::default(),
        }
    }

    /// Base style.
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl StatusLineState {
    /// Clear.
    pub fn clear_status(&mut self) {
        self.status.clear();
    }

    /// Set status
    pub fn status(&mut self, msg: &str) {
        self.status.clear();
        self.status.push_str(msg);
    }
}

impl StatefulWidget for StatusLine {
    type State = StatusLineState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        let status = Line::styled(&state.status, self.style);
        status.render(area, buf);
    }
}

impl StatusDialog {
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            button_style: Default::default(),
        }
    }

    /// Combined style
    pub fn style(mut self, styles: StatusDialogStyle) -> Self {
        self.style = styles.style;
        self.button_style = styles.button;
        self
    }

    /// Base style
    pub fn base_style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Button style.
    pub fn button_style(mut self, style: ButtonStyle) -> Self {
        self.button_style = style;
        self
    }
}

impl StatusDialogState {
    /// Clear
    pub fn clear_log(&mut self) {
        self.active = false;
        self.log.clear();
    }

    /// *Append* to the message.
    pub fn log(&mut self, msg: &str) {
        self.active = true;
        if !self.log.is_empty() {
            self.log.push('\n');
        }
        self.log.push_str(msg);
    }
}

impl Default for StatusDialogState {
    fn default() -> Self {
        let s = Self {
            active: false,
            area: Default::default(),
            button: Default::default(),
            log: Default::default(),
        };
        s.button.focus.set();
        s
    }
}

impl StatefulWidget for StatusDialog {
    type State = StatusDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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
            let block = Block::default().style(self.style);

            let mut lines = Vec::new();
            for t in state.log.split('\n') {
                lines.push(Line::from(t));
            }
            let text = Text::from(lines).alignment(Alignment::Center);
            let para = Paragraph::new(text);

            let ok = Button::from("Ok").style(self.button_style).action(true);

            Clear.render(l_dlg.dialog, buf);
            block.render(l_dlg.dialog, buf);
            para.render(l_dlg.area, buf);
            ok.render(l_dlg.buttons[0], buf, &mut state.button);
        }
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>> for StatusDialogState {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        check_break!(if self.active {
            self.button.handle(event, DefaultKeys).and_then(|_a| {
                self.clear_log();
                ControlUI::Changed
            })
        } else {
            ControlUI::Continue
        });

        check_break!(match event {
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.active {
                    self.clear_log();
                    ControlUI::Changed
                } else {
                    ControlUI::Continue
                }
            }
            _ => ControlUI::Continue,
        });

        // eat all events.
        ControlUI::Unchanged
    }
}
