use crate::vi::{Mode, VI};
use rat_text::text_area::TextAreaState;
use rat_widget::statusline_stacked::{SLANT_BL_TR, SLANT_TL_BR, StatusLineStacked};
use ratatui::buffer::Buffer;
use ratatui::prelude::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{StatefulWidget, Widget};
use std::mem;

#[derive(Debug)]
pub struct VIStatusLine<'a> {
    style: Style,
    name: &'a str,
    name_style: Style,
    normal_style: Style,
    insert_style: Style,
    visual_style: Style,
    msg: String,
    pos_style: Style,
}

impl<'a> VIStatusLine<'a> {
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            name: "",
            name_style: Default::default(),
            normal_style: Default::default(),
            insert_style: Default::default(),
            visual_style: Default::default(),
            msg: Default::default(),
            pos_style: Default::default(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn msg(mut self, msg: String) -> Self {
        self.msg = msg;
        self
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    pub fn name_style(mut self, style: Style) -> Self {
        self.name_style = style;
        self
    }

    pub fn normal_style(mut self, style: Style) -> Self {
        self.normal_style = style;
        self
    }

    pub fn insert_style(mut self, style: Style) -> Self {
        self.insert_style = style;
        self
    }

    pub fn visual_style(mut self, style: Style) -> Self {
        self.visual_style = style;
        self
    }

    pub fn pos_style(mut self, style: Style) -> Self {
        self.pos_style = style;
        self
    }
}

impl<'a> StatefulWidget for VIStatusLine<'a> {
    type State = (&'a mut TextAreaState, &'a mut VI);

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mode_style = match state.1.mode {
            Mode::Normal => self.normal_style,
            Mode::Insert => self.insert_style,
            Mode::Visual => self.visual_style,
        };

        let mut status = StatusLineStacked::new().start(
            Span::from(self.name).style(self.name_style),
            Span::from(SLANT_TL_BR).style(
                Style::new()
                    .fg(self.name_style.bg.unwrap_or(Color::Reset))
                    .bg(mode_style.bg.unwrap_or(Color::Reset)),
            ),
        );

        match state.1.mode {
            Mode::Normal => {
                status = status.start(
                    Span::from(" Normal ").style(self.normal_style),
                    Span::from(SLANT_TL_BR).style(
                        Style::new()
                            .fg(self.normal_style.bg.unwrap_or(Color::Reset))
                            .bg(self.style.bg.unwrap_or(Color::Reset)),
                    ),
                );
            }
            Mode::Insert => {
                status = status.start(
                    Span::from(" Insert ").style(self.insert_style),
                    Span::from(SLANT_TL_BR).style(
                        Style::new()
                            .fg(self.insert_style.bg.unwrap_or(Color::Reset))
                            .bg(self.style.bg.unwrap_or(Color::Reset)),
                    ),
                );
            }
            Mode::Visual => {
                status = status.start(
                    Span::from(" Visual ").style(self.visual_style),
                    Span::from(SLANT_TL_BR).style(
                        Style::new()
                            .fg(self.visual_style.bg.unwrap_or(Color::Reset))
                            .bg(self.style.bg.unwrap_or(Color::Reset)),
                    ),
                );
            }
        }

        let md = state.1.command_display.borrow();
        status = status.start(Span::from(md.as_str()).style(self.style), "");

        status = status.center(Line::from(self.msg).style(self.style));

        status = status.end(
            Span::from(format!(" {}:{} ", state.0.cursor().x, state.0.cursor().y))
                .style(self.pos_style),
            Span::from(SLANT_BL_TR).style(
                Style::new()
                    .fg(self.pos_style.bg.unwrap_or(Color::Black))
                    .bg(self.style.bg.unwrap_or(Color::Black)),
            ),
        );
        if !state.1.matches.list.is_empty() {
            let idx = state.1.matches.idx.unwrap_or(0) + 1;
            status = status.end(
                Span::from(format!("{}/{}", idx, state.1.matches.list.len())).style(self.style),
                Span::from("|").style(self.style),
            );
        }
        if !state.1.finds.list.is_empty() {
            let idx = state.1.finds.idx.unwrap_or(0) + 1;
            status = status.end(
                Span::from(format!("🖛 {}/{}", idx, state.1.finds.list.len())).style(self.style),
                Span::from("|").style(self.style),
            );
        }

        status.render(area, buf);
    }
}

pub fn revert_style(mut style: Style) -> Style {
    if style.fg.is_some() || style.bg.is_some() {
        mem::swap(&mut style.fg, &mut style.bg);
        style
    } else {
        style.black().on_white()
    }
}
