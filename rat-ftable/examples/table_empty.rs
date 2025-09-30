//!
//! Render a table with neither iter() nor data() being called.
//! More a test than an actual example.
//!

use crate::mini_salsa::{MiniSalsaState, run_ui, setup_logging};
use crate::mini_salsa::{THEME, mock_init};
use rat_ftable::event::Outcome;
use rat_ftable::selection::{NoSelection, noselection};
use rat_ftable::textdata::{Cell, Row};
use rat_ftable::{Table, TableState};
use rat_scrolled::Scroll;
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::{Block, block};

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        table: Default::default(),
    };

    run_ui("empty", mock_init, event, render, &mut data, &mut state)
}

struct Data {}

struct State {
    pub(crate) table: TableState<NoSelection>,
}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
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
        .header(
            Row::new([
                Cell::from("Nr"),
                Cell::from("Text"),
                Cell::from("Val1"),
                Cell::from("Val2"),
                Cell::from("State"),
            ])
            .style(Some(THEME.table_header())),
        )
        .footer(Row::new(["a", "b", "c", "d", "e"]).style(Some(THEME.table_footer())))
        .block(
            Block::bordered()
                .border_type(block::BorderType::Rounded)
                .border_style(THEME.container_border())
                .title("empty"),
        )
        .vscroll(Scroll::new().style(THEME.container_border()))
        .flex(Flex::End)
        .styles(THEME.table_style())
        .select_row_style(Some(THEME.gray(3)))
        .render(l0[0], frame.buffer_mut(), &mut state.table);

    Ok(())
}

fn event(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let r = noselection::handle_events(&mut state.table, true, event);
    Ok(r.into())
}
