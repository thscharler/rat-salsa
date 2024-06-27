use crate::substratum1::{Substratum, SubstratumState};
use anyhow::anyhow;
use crossterm::cursor::{DisableBlinking, EnableBlinking, SetCursorStyle};
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, KeyCode,
    KeyEvent, KeyEventKind, KeyModifiers,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use log::debug;
use rat_event::ConsumedEvent;
use rat_focus::Focus;
use rat_input::event::{FocusKeys, HandleEvent, Outcome};
use rat_input::statusline::{StatusLine, StatusLineState};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::Block;
use ratatui::{Frame, Terminal};
use std::fs;
use std::io::{stdout, Stdout};
use std::time::{Duration, SystemTime};

pub mod adapter;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        sub1: Default::default(),
        sub2: Default::default(),
        sub3: Default::default(),
        sub4: Default::default(),
        status: Default::default(),
    };
    focus_input(&mut state).next();
    state.status.status(0, "Ctrl+Q to quit.");

    run_ui(&mut data, &mut state)
}

fn setup_logging() -> Result<(), anyhow::Error> {
    _ = fs::remove_file("log.log");
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
        .apply()?;
    Ok(())
}

struct Data {}

struct State {
    pub(crate) sub1: SubstratumState,
    pub(crate) sub2: SubstratumState,
    pub(crate) sub3: SubstratumState,
    pub(crate) sub4: SubstratumState,

    pub(crate) status: StatusLineState,
}

fn run_ui(data: &mut Data, state: &mut State) -> Result<(), anyhow::Error> {
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    stdout().execute(EnableBlinking)?;
    stdout().execute(SetCursorStyle::BlinkingBar)?;
    stdout().execute(EnableBracketedPaste)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    repaint_ui(&mut terminal, data, state)?;

    let r = 'l: loop {
        let o = match crossterm::event::poll(Duration::from_millis(10)) {
            Ok(true) => {
                let event = match crossterm::event::read() {
                    Ok(v) => v,
                    Err(e) => break 'l Err(anyhow!(e)),
                };
                match handle_event(event, data, state) {
                    Ok(v) => v,
                    Err(e) => break 'l Err(e),
                }
            }
            Ok(false) => continue,
            Err(e) => break 'l Err(anyhow!(e)),
        };

        match o {
            Outcome::Changed => {
                match repaint_ui(&mut terminal, data, state) {
                    Ok(_) => {}
                    Err(e) => break 'l Err(e),
                };
            }
            _ => {
                // noop
            }
        }
    };

    disable_raw_mode()?;
    stdout().execute(DisableBracketedPaste)?;
    stdout().execute(SetCursorStyle::DefaultUserShape)?;
    stdout().execute(DisableBlinking)?;
    stdout().execute(DisableMouseCapture)?;
    stdout().execute(LeaveAlternateScreen)?;

    r
}

fn repaint_ui(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    data: &mut Data,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    terminal.hide_cursor()?;

    _ = terminal.draw(|frame| {
        repaint_tui(frame, data, state);
    });

    Ok(())
}

fn repaint_tui(frame: &mut Frame<'_>, data: &mut Data, state: &mut State) {
    let t0 = SystemTime::now();
    let area = frame.size();

    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    repaint_input(frame, l1[0], data, state);

    let status1 = StatusLine::new()
        .layout([
            Constraint::Fill(1),
            Constraint::Length(17),
            Constraint::Length(17),
        ])
        .styles([
            Style::default().black().on_dark_gray(),
            Style::default().white().on_blue(),
            Style::default().white().on_light_blue(),
        ]);

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state
        .status
        .status(1, format!("Render {:?}", el).to_string());
    frame.render_stateful_widget(status1, l1[1], &mut state.status);
}

fn handle_event(
    event: crossterm::event::Event,
    data: &mut Data,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let t0 = SystemTime::now();

    use crossterm::event::Event;
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            ..
        }) => {
            return Err(anyhow!("quit"));
        }
        Event::Resize(_, _) => return Ok(Outcome::Changed),
        _ => {}
    }

    let r = handle_input(&event, data, state)?;
    if r.is_consumed() {
        debug!("handle_input {:?}", r);
    }

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state
        .status
        .status(2, format!("Handle {:?}", el).to_string());

    Ok(r)
}

fn repaint_input(frame: &mut Frame<'_>, area: Rect, _data: &mut Data, state: &mut State) {
    let l0 = Layout::horizontal([
        Constraint::Length(25),
        Constraint::Length(25),
        Constraint::Fill(1),
    ])
    .split(area);

    let l00 = Layout::vertical([Constraint::Length(8), Constraint::Length(8)]).split(l0[0]);

    let w1 = Substratum::new().block(Block::bordered().title("First"));
    frame.render_stateful_widget(w1, l00[0], &mut state.sub1);
    let w2 = Substratum::new().block(Block::bordered().title("Second"));
    frame.render_stateful_widget(w2, l00[1], &mut state.sub2);

    let l11 = Layout::vertical([Constraint::Length(8), Constraint::Length(8)]).split(l0[1]);

    let w3 = Substratum::new().block(Block::bordered().title("Third"));
    frame.render_stateful_widget(w3, l11[0], &mut state.sub3);
    let w4 = Substratum::new().block(Block::bordered().title("Forth"));
    frame.render_stateful_widget(w4, l11[1], &mut state.sub4);

    let cursor = state.sub1.screen_cursor().or_else(|| {
        state.sub2.screen_cursor().or_else(|| {
            state
                .sub3
                .screen_cursor()
                .or_else(|| state.sub4.screen_cursor())
        })
    });
    if let Some((x, y)) = cursor {
        frame.set_cursor(x, y);
    }
}

fn focus_input(state: &mut State) -> Focus<'_> {
    let mut f = Focus::new(&[]);
    f.add_focus(state.sub1.focus())
        .add_focus(state.sub2.focus())
        .add_focus(state.sub3.focus())
        .add_focus(state.sub4.focus());
    f
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let f = focus_input(state).handle(event, FocusKeys);
    if f.is_consumed() {
        debug!("focus {:?}", f);
    }

    let r = state.sub1.handle(event, FocusKeys);
    if r.is_consumed() {
        return Ok(r | f);
    }
    let r = state.sub2.handle(event, FocusKeys);
    if r.is_consumed() {
        return Ok(r | f);
    }
    let r = state.sub3.handle(event, FocusKeys);
    if r.is_consumed() {
        return Ok(r | f);
    }
    let r = state.sub4.handle(event, FocusKeys);
    if r.is_consumed() {
        return Ok(r | f);
    }

    Ok(r | f)
}

pub mod substratum1 {
    use crate::adapter::textinputf::{TextInputF, TextInputFState};
    use log::debug;
    use rat_event::{ConsumedEvent, FocusKeys, HandleEvent, Outcome};
    use rat_focus::{Focus, FocusFlag, HasFocusFlag};
    use rat_input::layout::{layout_edit, EditConstraint};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::prelude::{BlockExt, Span, StatefulWidget, Style};
    use ratatui::style::Stylize;
    use ratatui::widgets::{Block, Widget};

    #[derive(Debug, Default)]
    pub struct Substratum<'a> {
        block: Option<Block<'a>>,
    }

    impl<'a> Substratum<'a> {
        pub fn new() -> Self {
            Self {
                block: Default::default(),
            }
        }

        pub fn block(mut self, block: Block<'a>) -> Self {
            self.block = Some(block);
            self
        }
    }

    #[derive(Debug, Default)]
    pub struct SubstratumState {
        pub focus: FocusFlag,
        pub input1: TextInputFState,
        pub input2: TextInputFState,
        pub input3: TextInputFState,
        pub input4: TextInputFState,
    }

    impl<'a> StatefulWidget for Substratum<'a> {
        type State = SubstratumState;

        fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            let inner = self.block.inner_if_some(area);

            self.block = if state.focus.get() {
                if let Some(block) = self.block {
                    Some(block.light_blue())
                } else {
                    self.block
                }
            } else {
                self.block
            };

            self.block.render(area, buf);

            let le = layout_edit(
                inner,
                &[
                    EditConstraint::Label("Text 1"),
                    EditConstraint::Widget(15),
                    EditConstraint::Label("Text 2"),
                    EditConstraint::Widget(15),
                    EditConstraint::Label("Text 3"),
                    EditConstraint::Widget(15),
                    EditConstraint::Label("Text 4"),
                    EditConstraint::Widget(15),
                ],
            );
            let mut le = le.iter();

            Span::from("Text 1").render(le.label(), buf);
            let input1 = TextInputF::default()
                .style(Style::default().black().on_green())
                .focus_style(Style::default().black().on_light_blue());
            input1.render(le.widget(), buf, &mut state.input1);

            Span::from("Text 2").render(le.label(), buf);
            let input2 = TextInputF::default()
                .style(Style::default().black().on_green())
                .focus_style(Style::default().black().on_light_blue());
            input2.render(le.widget(), buf, &mut state.input2);

            Span::from("Text 3").render(le.label(), buf);
            let input3 = TextInputF::default()
                .style(Style::default().black().on_green())
                .focus_style(Style::default().black().on_light_blue());
            input3.render(le.widget(), buf, &mut state.input3);

            Span::from("Text 4").render(le.label(), buf);
            let input4 = TextInputF::default()
                .style(Style::default().black().on_green())
                .focus_style(Style::default().black().on_light_blue());
            input4.render(le.widget(), buf, &mut state.input4);
        }
    }

    impl SubstratumState {
        pub fn focus(&self) -> Focus<'_> {
            Focus::new_container(
                self,
                &[&self.input1, &self.input2, &self.input3, &self.input4],
            )
        }

        pub fn screen_cursor(&self) -> Option<(u16, u16)> {
            if self.input1.is_focused() {
                self.input1.screen_cursor()
            } else if self.input2.is_focused() {
                self.input2.screen_cursor()
            } else if self.input3.is_focused() {
                self.input3.screen_cursor()
            } else if self.input4.is_focused() {
                self.input4.screen_cursor()
            } else {
                None
            }
        }
    }

    impl HasFocusFlag for SubstratumState {
        fn focus(&self) -> &FocusFlag {
            &self.focus
        }

        fn area(&self) -> Rect {
            Rect::default() // todo?!!
        }
    }

    impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for SubstratumState {
        fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
            let mut r: Outcome = self.input1.handle(event, FocusKeys).into();
            if r.is_consumed() {
                debug!("rr1 {:?}", r);
                return r;
            }
            r = self.input2.handle(event, FocusKeys).into();
            if r.is_consumed() {
                debug!("rr2 {:?}", r);
                return r;
            }
            r = self.input3.handle(event, FocusKeys).into();
            if r.is_consumed() {
                debug!("rr3 {:?}", r);
                return r;
            }
            r = self.input4.handle(event, FocusKeys).into();
            if r.is_consumed() {
                debug!("rr4 {:?}", r);
                return r;
            }
            Outcome::NotUsed
        }
    }
}
