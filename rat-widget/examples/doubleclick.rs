use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use chrono::{Local, NaiveTime};
use crossterm::event::{Event, KeyModifiers, MouseEvent, MouseEventKind};
use format_num_pattern::NumberFormat;
use rat_event::util::{Clicks, MouseFlags, set_double_click_timeout};
use rat_event::{Outcome, ct_event, try_flow};
use rat_widget::layout::layout_as_grid;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Span;
use ratatui::widgets::Widget;
use std::time::{Duration, SystemTime};

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {
        journal: Default::default(),
    };

    let mut state = State {
        area: Default::default(),
        mouse: Default::default(),
        flip: false,
        flip2: false,
        drag_pos: None,
    };

    set_double_click_timeout(350);

    run_ui(
        "doubleclick",
        mock_init,
        event,
        render,
        &mut data,
        &mut state,
    )
}

enum Journal {
    Mouse(MouseEvent, Option<SystemTime>, Clicks),
    DoubleClick(),
}

struct Data {
    journal: Vec<(NaiveTime, Journal)>,
}

struct State {
    area: Rect,
    mouse: MouseFlags,
    flip: bool,
    flip2: bool,
    drag_pos: Option<(u16, u16)>,
}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l = layout_as_grid(
        area,
        Layout::horizontal([
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Fill(1),
            Constraint::Length(5),
        ]),
        Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Fill(1),
            Constraint::Length(5),
            Constraint::Length(1),
        ]),
    );

    if state.flip2 {
        if state.flip {
            frame
                .buffer_mut()
                .set_style(l.widget_for((1, 2)), Style::new().on_white());
        } else {
            frame
                .buffer_mut()
                .set_style(l.widget_for((1, 2)), Style::new().on_red());
        }
    } else {
        if state.flip {
            frame
                .buffer_mut()
                .set_style(l.widget_for((1, 2)), Style::new().on_green());
        } else {
            frame
                .buffer_mut()
                .set_style(l.widget_for((1, 2)), Style::new().on_blue());
        }
    }
    state.area = l.widget_for((1, 2));

    if state.mouse.drag.get() {
        if let Some((c, r)) = state.drag_pos {
            let numf = NumberFormat::new("###")?;
            let drag = Span::from(
                format!(
                    " DRAG: {}:{}",
                    numf.fmt_u(c as isize - state.area.x as isize),
                    numf.fmt_u(r as isize - state.area.y as isize)
                )
                .to_string(),
            );
            drag.render(l.widget_for((3, 2)), frame.buffer_mut());
        }
    }

    if data.journal.len() > 0 {
        let numf = NumberFormat::new("##,###,###")?;

        let off = data
            .journal
            .len()
            .saturating_sub(l.widget_for((2, 2)).height as usize);
        let journal = &data.journal[off..];

        let zero = off.saturating_sub(1);
        let mut prev_time = data.journal[zero].0.clone();

        for (n, (time, event)) in journal.iter().enumerate() {
            let journal_area = l.widget_for((2, 2));
            let row_area = Rect::new(
                journal_area.x,
                journal_area.y + n as u16,
                journal_area.width,
                1,
            );

            let dur = time.signed_duration_since(prev_time);

            let msg = match event {
                Journal::Mouse(event, sys, click) => {
                    let delta = match sys {
                        None => 0,
                        Some(v) => v
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or(Duration::default())
                            .as_millis(),
                    };

                    Span::from(
                        format!(
                            "{:>20} {:02}:{:02} {:>15} {:9} {:?}",
                            numf.fmt_u(dur.num_microseconds().expect("duration")),
                            event.column,
                            event.row,
                            format!("{:?}", event.kind),
                            delta,
                            click
                        )
                        .to_string(),
                    )
                }
                Journal::DoubleClick() => Span::from(
                    format!(
                        "{:>20}   :   CLICK CLICK",
                        numf.fmt_u(dur.num_microseconds().expect("duration")),
                    )
                    .to_string(),
                ),
            };
            msg.render(row_area, frame.buffer_mut());

            prev_time = time.clone();
        }
    }

    Ok(())
}

fn event(
    event: &Event,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(match event {
        ct_event!(mouse any for m) if state.mouse.doubleclick(state.area, m) => {
            data.journal
                .push((Local::now().time(), Journal::DoubleClick()));
            state.flip = !state.flip;
            Outcome::Changed
        }
        ct_event!(mouse any for m)
            if state
                .mouse
                .doubleclick2(state.area, m, KeyModifiers::CONTROL) =>
        {
            data.journal
                .push((Local::now().time(), Journal::DoubleClick()));
            state.flip2 = !state.flip2;
            Outcome::Changed
        }
        ct_event!(mouse any for m) if state.mouse.drag(state.area, m) => {
            state.drag_pos = Some(state.mouse.pos_of(m));
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    try_flow!(match event {
        Event::Mouse(
            m @ MouseEvent {
                kind: MouseEventKind::Up(_) | MouseEventKind::Down(_) | MouseEventKind::Drag(_),
                ..
            },
        ) => {
            if state.area.contains((m.column, m.row).into()) {
                data.journal.push((
                    Local::now().time(),
                    Journal::Mouse(m.clone(), state.mouse.time.get(), state.mouse.click.get()),
                ));
                Outcome::Changed
            } else {
                Outcome::Unchanged
            }
        }
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}
