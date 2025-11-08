use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use log::warn;
use rat_event::try_flow;
use rat_text::clipboard::{Clipboard, ClipboardError, set_global_clipboard};
use rat_text::color_input::{ColorInput, ColorInputState};
use rat_text::{HasScreenCursor, color_input};
use rat_widget::event::Outcome;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, StatefulWidget, Widget};
use std::cell::RefCell;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;
    set_global_clipboard(CliClipboard::default());

    let mut data = Data {};

    let mut state = State {
        input: ColorInputState::default(),
    };

    run_ui(
        "colorinput1",
        mock_init,
        event,
        render,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    pub(crate) input: ColorInputState,
}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([
        Constraint::Length(4),
        Constraint::Length(18),
        Constraint::Length(20),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .split(area);

    let l1 = Layout::vertical([
        Constraint::Length(7),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .split(l0[1]);

    let l2 = Layout::vertical([
        Constraint::Length(7),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .split(l0[2]);

    ColorInput::new() //
        .styles(istate.theme.color_input_style())
        .block(Block::bordered().border_type(BorderType::Rounded))
        .render(l1[1], frame.buffer_mut(), &mut state.input);

    if let Some((x, y)) = state.input.screen_cursor() {
        frame.set_cursor_position((x, y));
    }

    Span::from(format!(" -> {:?}", state.input.value())) //
        .render(l2[1], frame.buffer_mut());

    Ok(())
}

fn event(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(color_input::handle_events(&mut state.input, true, event));

    Ok(Outcome::Continue)
}

#[derive(Debug, Default, Clone)]
struct CliClipboard {
    clip: RefCell<String>,
}

impl Clipboard for CliClipboard {
    fn get_string(&self) -> Result<String, ClipboardError> {
        match cli_clipboard::get_contents() {
            Ok(v) => Ok(v),
            Err(e) => {
                warn!("{:?}", e);
                Ok(self.clip.borrow().clone())
            }
        }
    }

    fn set_string(&self, s: &str) -> Result<(), ClipboardError> {
        let mut clip = self.clip.borrow_mut();
        *clip = s.to_string();

        match cli_clipboard::set_contents(s.to_string()) {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("{:?}", e);
                Err(ClipboardError)
            }
        }
    }
}
