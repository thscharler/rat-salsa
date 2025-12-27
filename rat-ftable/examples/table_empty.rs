//!
//! Render a table with neither iter() nor data() being called.
//! More a test than an actual example.
//!

use crate::mini_salsa::mock_init;
use crate::mini_salsa::{MiniSalsaState, run_ui, setup_logging};
use rat_ftable::event::Outcome;
use rat_ftable::selection::{NoSelection, noselection};
use rat_ftable::textdata::{Cell, Row};
use rat_ftable::{Table, TableState};
use rat_scrolled::{Scroll, ScrollStyle};
use rat_theme4::StyleName;
use rat_theme4::theme::SalsaTheme;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Flex, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        table: Default::default(),
    };

    run_ui("empty", mock_init, event, render, &mut state)
}

struct State {
    table: TableState<NoSelection>,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([Constraint::Percentage(61)])
        .flex(Flex::Center)
        .split(area);

    Table::default()
        .widths([
            Constraint::Length(6),
            Constraint::Length(20),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(3),
        ])
        .column_spacing(1)
        .header(Row::new([
            Cell::from("Nr"),
            Cell::from("Text"),
            Cell::from("Val1"),
            Cell::from("Val2"),
            Cell::from("State"),
        ]))
        .footer(Row::new(["a", "b", "c", "d", "e"]))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("empty"),
        )
        .vscroll(Scroll::new())
        .flex(Flex::End)
        .styles(table(&ctx.theme))
        .render(l0[0], buf, &mut state.table);

    Ok(())
}

fn table(th: &SalsaTheme) -> rat_ftable::TableStyle {
    rat_ftable::TableStyle {
        style: th.style(Style::CONTAINER_BASE),
        select_row: Some(th.style(Style::SELECT)),
        show_row_focus: true,
        focus_style: Some(th.style(Style::FOCUS)),
        border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        scroll: Some(scroll(th)),
        header: Some(th.style(Style::HEADER)),
        footer: Some(th.style(Style::FOOTER)),
        ..Default::default()
    }
}

fn scroll(th: &SalsaTheme) -> ScrollStyle {
    ScrollStyle {
        thumb_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        track_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        min_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        begin_style: Some(th.style(Style::CONTAINER_ARROW_FG)),
        end_style: Some(th.style(Style::CONTAINER_ARROW_FG)),
        ..Default::default()
    }
}

fn event(
    event: &Event,
    _ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let r = noselection::handle_events(&mut state.table, true, event);
    Ok(r.into())
}
