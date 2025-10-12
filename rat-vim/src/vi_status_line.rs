use crate::{Mode, VI};
use rat_text::text_area::TextAreaState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::prelude::{BlockExt, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::marker::PhantomData;
use std::mem;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct VIStatusLine<'a> {
    style: Style,
    name: &'a str,
    name_style_1: Style,
    name_style_2: Style,
    normal_style: Style,
    insert_style: Style,
    visual_style: Style,
    msg: String,
    pos_style_1: Style,
    pos_style_2: Style,
    block: Option<Block<'a>>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> VIStatusLine<'a> {
    pub fn new() -> Self {
        Self {
            style: Default::default(),
            name: "",
            name_style_1: Default::default(),
            name_style_2: Default::default(),
            normal_style: Default::default(),
            insert_style: Default::default(),
            visual_style: Default::default(),
            msg: Default::default(),
            pos_style_1: Default::default(),
            pos_style_2: Default::default(),
            block: Default::default(),
            _phantom: Default::default(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
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

    pub fn name_style(mut self, style1: Style, style2: Style) -> Self {
        self.name_style_1 = style1;
        self.name_style_2 = style2;
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

    pub fn pos_style(mut self, style1: Style, style2: Style) -> Self {
        self.pos_style_1 = style1;
        self.pos_style_2 = style2;
        self
    }
}

impl<'a> StatefulWidget for VIStatusLine<'a> {
    type State = (&'a mut TextAreaState, &'a mut VI);

    // ◤
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = self.block.inner_if_some(area);

        let name_len = self.name.graphemes(true).count();
        let motion_len = state.1.motion_display.borrow().graphemes(true).count();

        let ll = Layout::horizontal([
            Constraint::Length(name_len as u16 + 2),
            Constraint::Length(7),
            Constraint::Length(motion_len as u16 + 1),
            Constraint::Fill(1),
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ])
        .split(inner);

        self.block.render(area, buf);

        // let sx = "\u{e0b8}";
        // let sx = "\u{e0b9}";
        // let sx = "\u{e0ba}";
        // let sx = "\u{e0bb}";
        // let sx = "\u{e0bc}";
        // let sx = "\u{e0bd}";
        // let sx = "\u{e0be}";

        Line::from_iter([
            Span::from(self.name).style(self.name_style_1),
            Span::from("\u{e0b8}").style(
                Style::new()
                    .fg(self.name_style_1.bg.unwrap_or(Color::Cyan))
                    .bg(self.name_style_2.bg.unwrap_or(Color::Cyan)),
            ),
            Span::from("\u{e0b8}").style(
                Style::new()
                    .fg(self.name_style_2.bg.unwrap_or(Color::Cyan))
                    .bg(match state.1.mode {
                        Mode::Normal => self.normal_style.bg.unwrap_or(Color::Cyan),
                        Mode::Insert => self.insert_style.bg.unwrap_or(Color::Cyan),
                        Mode::Visual => self.visual_style.bg.unwrap_or(Color::Cyan),
                    }),
            ),
        ])
        .render(ll[0], buf);
        Line::from_iter(match state.1.mode {
            Mode::Normal => [
                Span::from("Normal").style(self.normal_style),
                Span::from("\u{e0b8}").style(revert_style(self.normal_style)),
            ],
            Mode::Insert => [
                Span::from("Insert").style(self.insert_style),
                Span::from("\u{e0b8}").style(revert_style(self.insert_style)),
            ],
            Mode::Visual => [
                Span::from("Visual").style(self.visual_style),
                Span::from("\u{e0b8}").style(revert_style(self.visual_style)),
            ],
        })
        .render(ll[1], buf);

        let md = state.1.motion_display.borrow();
        Line::from(md.as_str()) //
            .style(self.style)
            .render(ll[2], buf);

        Line::from(self.msg) //
            .style(self.style)
            .render(ll[3], buf);

        let marks = state.1.marks.iter().filter(|v| v.is_some()).count();
        if marks > 0 {
            Line::from(format!("🖈 {}", marks))
                .style(self.style)
                .render(ll[4], buf);
        } else {
            Line::from("").style(self.style).render(ll[4], buf);
        }
        if !state.1.finds.list.is_empty() {
            if let Some(idx) = state.1.finds.idx {
                Line::from(format!("🖛 {}/{}", idx, state.1.finds.list.len() - 1))
                    .style(self.style)
                    .render(ll[5], buf);
            } else {
                Line::from("").style(self.style).render(ll[5], buf);
            }
        } else {
            Line::from("").style(self.style).render(ll[5], buf);
        }
        if !state.1.matches.list.is_empty() {
            if let Some(idx) = state.1.matches.idx {
                Line::from(format!("🕶 {}/{}", idx, state.1.matches.list.len() - 1))
                    .style(self.style)
                    .render(ll[6], buf);
            } else {
                Line::from("").style(self.style).render(ll[6], buf);
            }
        } else {
            Line::from("").style(self.style).render(ll[6], buf);
        }
        Line::from_iter([
            Span::from("\u{e0ba}").style(
                Style::new()
                    .fg(self.pos_style_1.bg.unwrap_or(Color::Black))
                    .bg(self.style.bg.unwrap_or(Color::Black)),
            ),
            Span::from("\u{e0ba}").style(
                Style::new()
                    .fg(self.pos_style_2.bg.unwrap_or(Color::Black))
                    .bg(self.pos_style_1.bg.unwrap_or(Color::Black)),
            ),
            Span::from(format!(" {}:{} ", state.0.cursor().x, state.0.cursor().y))
                .style(self.pos_style_2),
        ])
        .alignment(Alignment::Right)
        .style(self.style)
        .render(ll[7], buf);
    }
}

fn revert_style(mut style: Style) -> Style {
    if style.fg.is_some() || style.bg.is_some() {
        mem::swap(&mut style.fg, &mut style.bg);
        style
    } else {
        style.black().on_white()
    }
}
