use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use chrono::{Local, NaiveTime};
use crossterm::event::{Event, KeyEvent};
use format_num_pattern::NumberFormat;
use rat_event::{try_flow, Outcome};
use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::Widget;
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {
        journal: Default::default(),
    };

    let mut state = State {};

    run_ui(
        "keybinding",
        |_, _, _| {},
        handle_buttons,
        repaint_buttons,
        &mut data,
        &mut state,
    )
}

struct Data {
    pub(crate) journal: Vec<(NaiveTime, KeyEvent)>,
}

struct State {}

fn repaint_buttons(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
    _state: &mut State,
) -> Result<(), anyhow::Error> {
    if data.journal.len() > 0 {
        let numf = NumberFormat::new("##,###,###")?;

        let off = data.journal.len().saturating_sub(area.height as usize);
        let journal = &data.journal[off..];

        let zero = off.saturating_sub(1);
        let mut prev_time = data.journal[zero].0.clone();

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
            msg.render(row_area, frame.buffer_mut());

            prev_time = time.clone();
        }
    }

    Ok(())
}

fn handle_buttons(
    event: &Event,
    data: &mut Data,
    _istate: &mut MiniSalsaState,
    _state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(match event {
        Event::Key(k) => {
            data.journal.push((Local::now().time(), k.clone()));
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}
