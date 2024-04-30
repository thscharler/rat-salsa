//!
//! Status line and a message dialog.
//!

use crate::layout::layout_dialog;
use crate::widget::button::{Button, ButtonState, ButtonStyle};
use crate::widget::paragraph::{ParagraphExt, ParagraphExtState};
use crate::widget::scrolled::{Scrolled, ScrolledState};
use crate::ControlUI;
use crate::_private::NonExhaustive;
use crate::{check_break, ct_event};
use crate::{DefaultKeys, HandleCrossterm};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Margin, Rect};
use ratatui::prelude::{StatefulWidget, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, Widget};
use std::fmt::Debug;
use std::io::IsTerminal;

/// Basic status line.
#[derive(Debug, Default)]
pub struct StatusLine {
    style: Vec<Style>,
    widths: Vec<Constraint>,
}

/// State for the status line.
#[derive(Debug)]
pub struct StatusLineState {
    pub area: Rect,
    pub status: Vec<String>,
    pub non_exhaustive: NonExhaustive,
}

impl StatusLine {
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            widths: Default::default(),
        }
    }

    /// Layout
    pub fn layout<It, Item>(mut self, widths: It) -> Self
    where
        It: IntoIterator<Item = Item>,
        Item: Into<Constraint>,
    {
        self.widths = widths.into_iter().map(|v| v.into()).collect();
        self
    }

    /// Base style.
    pub fn styles(mut self, style: impl IntoIterator<Item = impl Into<Style>>) -> Self {
        self.style = style.into_iter().map(|v| v.into()).collect();
        self
    }
}

impl Default for StatusLineState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            status: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl StatusLineState {
    /// Clear.
    pub fn clear_status(&mut self) {
        self.status.clear();
    }

    /// Set status
    pub fn status<S: Into<String>>(&mut self, idx: usize, msg: S) {
        while self.status.len() <= idx {
            self.status.push("".to_string());
        }
        self.status[idx] = msg.into();
    }
}

impl StatefulWidget for StatusLine {
    type State = StatusLineState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;

        let layout = Layout::horizontal(self.widths).split(state.area);

        for (i, rect) in layout.iter().enumerate() {
            let style = self.style.get(i).cloned().unwrap_or_default();
            let txt = state.status.get(i).map(|v| v.as_str()).unwrap_or("");

            buf.set_style(*rect, style);
            Span::from(txt).render(*rect, buf);
        }
    }
}

/// Basic status dialog for longer messages.
#[derive(Debug, Default)]
pub struct StatusDialog {
    style: Style,
    button_style: ButtonStyle,
}

/// Combined style.
#[derive(Debug)]
pub struct StatusDialogStyle {
    pub style: Style,
    pub button: ButtonStyle,
    pub non_exhaustive: NonExhaustive,
}

/// State for the status dialog.
#[derive(Debug)]
pub struct StatusDialogState {
    pub active: bool,
    pub area: Rect,
    pub message: ScrolledState<ParagraphExtState>,
    pub button: ButtonState<bool>,
    pub log: String,
    pub non_exhaustive: NonExhaustive,
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

impl Default for StatusDialogStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            button: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl StatusDialogState {
    /// Clear
    pub fn clear_log(&mut self) {
        self.active = false;
        self.message = Default::default();
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
            message: Default::default(),
            button: Default::default(),
            log: Default::default(),
            non_exhaustive: NonExhaustive,
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
            let para = ParagraphExt::new(text);
            let scrolled_para = Scrolled::new(para);

            let ok = Button::from("Ok").style(self.button_style).action(true);

            Clear.render(l_dlg.dialog, buf);
            block.render(l_dlg.dialog, buf);
            scrolled_para.render(l_dlg.area, buf, &mut state.message);
            ok.render(l_dlg.buttons[0], buf, &mut state.button);
        }
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>> for StatusDialogState {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        check_break!(self.message.handle(event, DefaultKeys));

        check_break!(if self.active {
            self.button.handle(event, DefaultKeys).on_action(|_a| {
                self.clear_log();
                ControlUI::Change
            })
        } else {
            ControlUI::Continue
        });

        check_break!(match event {
            ct_event!(keycode press Esc) => {
                if self.active {
                    self.clear_log();
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            _ => ControlUI::Continue,
        });

        // eat all events.
        ControlUI::NoChange
    }
}
