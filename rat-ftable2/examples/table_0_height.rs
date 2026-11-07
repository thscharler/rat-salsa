//!
//! Render rows with row-height 0.
//!

use crate::data::SMALL_DATA;
use crate::mini_salsa::mock_init;
use crate::mini_salsa::{MiniSalsaState, run_ui, setup_logging};
use crate::widget::render_tablestate::render_tablestate_row;
use format_num_pattern::NumberFormat;
use rat_event::{HandleEvent, Regular, event_flow};
use rat_focus::HasFocus;
use rat_ftable2::event::Outcome;
use rat_ftable2::selection::row::RowSelection;
use rat_ftable2::{Table, TableState};
use rat_scrolled::{Scroll, ScrollStyle};
use rat_theme4::StyleName;
use rat_theme4::theme::SalsaTheme;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Flex, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;

mod data;
mod mini_salsa;
mod widget;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        table_data: data::DATA
            .iter()
            .map(|v| Sample {
                text: *v,
                num1: rand::random(),
                num2: rand::random(),
                check: rand::random(),
                row_height: rand::random::<u16>() % 3,
            })
            .take(100_000)
            .collect(),
        table: Default::default(),
    };

    run_ui("0_height", mock_init, event, render, &mut state)
}

struct Sample {
    pub(crate) text: &'static str,
    pub(crate) num1: f32,
    pub(crate) num2: f32,
    pub(crate) check: bool,
    pub(crate) row_height: u16,
}

struct State {
    table_data: Vec<Sample>,
    table: TableState<RowSelection>,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([Constraint::Percentage(61), Constraint::Percentage(39)])
        .flex(Flex::Center)
        .split(area);

    let mut tmp = String::default();
    let fmt1 = NumberFormat::new("000000").expect("fmt");
    let fmt2 = NumberFormat::new("####0.00").expect("fmt");

    Table::default()
        .data(state.table_data.iter())
        .widths([
            Constraint::Length(6),
            Constraint::Length(20),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(3),
        ])
        .varying_heights(true)
        .row_height(|v, _| v.row_height)
        .render_row(|_v, mut area, buf, _ctx| {
            if area.height > 1 {
                area.x = 7;
                area.y = 1;
                for r in 1..area.height {
                    tmp.clear();
                    for w in 0..7 {
                        tmp.push_str(SMALL_DATA[(r * 7 + w) as usize]);
                        tmp.push(' ');
                    }
                    tmp.as_str().render(area, buf);
                    area.y += 1;
                }
            }
            true
        })
        .render_cell(0, |_v, area, buf, ctx| {
            fmt1.fmt_u(ctx.row).render(area, buf)
        })
        .render_cell(1, |v, area, buf, _ctx| v.text.render(area, buf))
        .render_cell(2, |v, area, buf, _ctx| fmt2.fmt_u(v.num1).render(area, buf))
        .render_cell(3, |v, area, buf, _ctx| fmt1.fmt_u(v.num2).render(area, buf))
        .render_cell(4, |v, area, buf, _ctx| {
            let cc = if v.check { "\u{2622}" } else { "\u{2623}" };
            cc.render(area, buf)
        })
        .column_spacing(1)
        .header(vec!["Nr", "Text", "Val1", "Val2", "State"])
        .footer(vec!["a", "b", "c", "d", "e"])
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("0-height rows"),
        )
        .vscroll(Scroll::new())
        .flex(Flex::End)
        .styles(table(&ctx.theme))
        .render(l0[0], buf, &mut state.table);

    render_tablestate_row(&state.table, l0[1], buf);

    Ok(())
}

fn table(th: &SalsaTheme) -> rat_ftable2::TableStyle {
    rat_ftable2::TableStyle {
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
    // override focus
    state.table.focus().set(true);

    event_flow!(state.table.handle(event, Regular));

    Ok(Outcome::Continue)
}
