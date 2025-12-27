use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use chrono::{Local, NaiveTime};
use format_num_pattern::NumberFormat;
use rat_event::{Outcome, try_flow};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::text::Span;
use ratatui_core::widgets::Widget;
use ratatui_crossterm::crossterm::event::{Event, KeyEvent};

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        journal: Default::default(),
    };

    run_ui("keybinding", mock_init, event, render, &mut state)
}

struct State {
    pub(crate) journal: Vec<(NaiveTime, KeyEvent)>,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    _ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    if state.journal.len() > 0 {
        let numf = NumberFormat::new("##,###,###")?;

        let off = state.journal.len().saturating_sub(area.height as usize);
        let journal = &state.journal[off..];

        let zero = off.saturating_sub(1);
        let mut prev_time = state.journal[zero].0.clone();

        for (n, (time, event)) in journal.iter().enumerate() {
            let row_area = Rect::new(area.x, area.y + n as u16, area.width, 1);

            let dur = time.signed_duration_since(prev_time);

            let msg = Span::from(
                format!(
                    "{:>20} {:?} {:?} {:?} {:?}",
                    numf.fmt_u(dur.num_microseconds().expect("duration")),
                    event.kind,
                    event.code,
                    event.modifiers,
                    event.state
                )
                .to_string(),
            );
            msg.render(row_area, buf);

            prev_time = time.clone();
        }
    }

    Ok(())
}

fn event(
    event: &Event,
    _ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(match event {
        Event::Key(k) => {
            state.journal.push((Local::now().time(), k.clone()));
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}
