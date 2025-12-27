#![allow(unreachable_pub)]
#![allow(dead_code)]

use anyhow::anyhow;
#[cfg(not(windows))]
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
#[cfg(not(windows))]
use crossterm::terminal::supports_keyboard_enhancement;
use log::error;
use rat_event::util::set_have_keyboard_enhancement;
use rat_event::{HandleEvent, Outcome, Regular};
use rat_focus::Focus;
use rat_theme4::palette::Colors;
use rat_theme4::theme::SalsaTheme;
use rat_theme4::{RatWidgetColor, StyleName, create_salsa_theme, salsa_themes};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::style::{Color, Style};
use ratatui_core::terminal::Terminal;
use ratatui_core::text::Line;
use ratatui_core::widgets::Widget;
use ratatui_crossterm::crossterm::ExecutableCommand;
use ratatui_crossterm::crossterm::cursor::{DisableBlinking, EnableBlinking, SetCursorStyle};
use ratatui_crossterm::crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event,
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MediaKeyCode,
};
use ratatui_crossterm::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui_crossterm::{CrosstermBackend, crossterm};
use std::cell::Cell;
use std::cmp::max;
use std::fs;
use std::io::{Stdout, stdout};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use unicode_segmentation::UnicodeSegmentation;

pub struct MiniSalsaState {
    pub name: String,
    pub theme: SalsaTheme,
    pub frame: usize,
    pub event_cnt: usize,

    pub hide_timing: bool,
    pub last_render: Duration,
    pub last_event: Duration,

    pub hide_status: bool,
    pub status: [String; 3],

    pub focus: Option<Focus>,
    pub focus_outcome: Outcome,
    pub focus_outcome_cell: Cell<Outcome>,

    pub cursor: Option<(u16, u16)>,

    pub quit: bool,
}

impl MiniSalsaState {
    fn new(name: &str, theme: SalsaTheme) -> Self {
        let mut s = Self {
            name: name.to_string(),
            theme,
            frame: Default::default(),
            event_cnt: Default::default(),
            hide_timing: Default::default(),
            last_render: Default::default(),
            last_event: Default::default(),
            hide_status: Default::default(),
            status: Default::default(),
            focus: Default::default(),
            focus_outcome: Default::default(),
            focus_outcome_cell: Default::default(),
            cursor: Default::default(),
            quit: Default::default(),
        };
        s.status[0] = "Ctrl-Q to quit. F8 Theme ".into();
        s
    }

    pub fn focus(&self) -> &Focus {
        self.focus.as_ref().expect("focus")
    }

    pub fn handle_focus(&mut self, event: &crossterm::event::Event) -> Outcome {
        self.focus_outcome = self.focus.as_mut().expect("focus").handle(event, Regular);
        self.focus_outcome
    }
}

pub fn run_ui<State>(
    name: &str,
    init: fn(
        &mut MiniSalsaState, //
        &mut State,
    ) -> Result<(), anyhow::Error>,
    handle: fn(&Event, &mut MiniSalsaState, state: &mut State) -> Result<Outcome, anyhow::Error>,
    repaint: fn(
        &mut Buffer, //
        Rect,
        &mut MiniSalsaState,
        &mut State,
    ) -> Result<(), anyhow::Error>,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    stdout().execute(EnableBlinking)?;
    stdout().execute(SetCursorStyle::BlinkingBar)?;
    stdout().execute(EnableBracketedPaste)?;

    #[cfg(not(windows))]
    {
        stdout().execute(PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                | KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES,
        ))?;

        let enhanced = supports_keyboard_enhancement().unwrap_or_default();
        set_have_keyboard_enhancement(enhanced);
    }
    #[cfg(windows)]
    {
        set_have_keyboard_enhancement(true);
    }

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let pal = rat_theme4::palettes::dark::IMPERIAL;
    let theme = rat_theme4::create_palette_theme(pal).expect("valid_palette");
    let mut istate = MiniSalsaState::new(name, theme);

    init(&mut istate, state)?;

    istate.frame = repaint_ui(&mut terminal, repaint, &mut istate, state)?;

    let r = 'l: loop {
        istate.focus_outcome = Outcome::Continue;
        istate.focus_outcome_cell.set(Outcome::Continue);

        let o = match crossterm::event::poll(Duration::from_millis(10)) {
            Ok(true) => {
                let event = match crossterm::event::read() {
                    Ok(v) => v,
                    Err(e) => break 'l Err(anyhow!(e)),
                };
                match handle_event(handle, event, &mut istate, state) {
                    Ok(v) => max(
                        max(v, istate.focus_outcome),
                        istate.focus_outcome_cell.get(),
                    ),
                    Err(e) => break 'l Err(e),
                }
            }
            Ok(false) => continue,
            Err(e) => {
                istate.status[0] = format!("{}", e);
                Outcome::Changed
            }
        };

        if istate.quit {
            break 'l Ok(());
        }

        match o {
            Outcome::Changed => {
                match repaint_ui(&mut terminal, repaint, &mut istate, state) {
                    Ok(f) => istate.frame = f,
                    Err(e) => break 'l Err(e),
                };
            }
            _ => {
                // noop
            }
        }
    };

    #[cfg(not(windows))]
    stdout().execute(PopKeyboardEnhancementFlags)?;

    stdout().execute(DisableBracketedPaste)?;
    stdout().execute(SetCursorStyle::DefaultUserShape)?;
    stdout().execute(DisableBlinking)?;
    stdout().execute(DisableMouseCapture)?;
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    r
}

pub fn mock_init<State>(
    _ctx: &mut MiniSalsaState,
    _state: &mut State,
) -> Result<(), anyhow::Error> {
    Ok(())
}

fn repaint_ui<State>(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    repaint: fn(
        &mut Buffer, //
        Rect,
        &mut MiniSalsaState,
        &mut State,
    ) -> Result<(), anyhow::Error>,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<usize, anyhow::Error> {
    terminal.hide_cursor()?;

    let completed = terminal.draw(|frame| {
        match repaint_tui(frame.buffer_mut(), repaint, ctx, state) {
            Ok(_) => {}
            Err(e) => {
                error!("{:?}", e)
            }
        };
        if let Some(cursor) = ctx.cursor {
            frame.set_cursor_position(cursor);
            ctx.cursor = None;
        }
    })?;

    Ok(completed.count)
}

fn repaint_tui<State>(
    buf: &mut Buffer,
    repaint: fn(
        &mut Buffer, //
        Rect,
        &mut MiniSalsaState,
        &mut State,
    ) -> Result<(), anyhow::Error>,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let area = *buf.area();

    let l1 = if !ctx.hide_status {
        Layout::vertical([
            Constraint::Fill(1), //
            Constraint::Length(1),
        ])
        .split(area)
    } else {
        Layout::vertical([
            Constraint::Fill(1), //
        ])
        .split(area)
    };

    if !ctx.hide_status {
        buf.set_style(l1[1], ctx.theme.style_style(Style::STATUS_BASE));
    }

    let t0 = SystemTime::now();

    repaint(buf, l1[0], ctx, state)?;

    ctx.last_render = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    if !ctx.hide_timing {
        ctx.status[1] = format!("Render #{} | {:.0?}", ctx.frame, ctx.last_render).to_string();
    }

    if !ctx.hide_status {
        let l_status = Layout::horizontal([
            Constraint::Length(2 + ctx.name.graphemes(true).count() as u16),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(18),
            Constraint::Length(18),
        ])
        .split(l1[1]);

        let blue_text = ctx.theme.p.high_contrast_color(
            ctx.theme.p.color_alias(Color::STATUS_BASE_BG),
            &ctx.theme.p.color[Colors::Blue as usize],
        );

        Line::from_iter(["[", ctx.name.as_str(), "]"]).render(l_status[0], buf);
        Line::from(" ").render(l_status[1], buf);
        Line::from(ctx.status[0].as_str()).render(l_status[2], buf);
        Line::from(ctx.status[1].as_str())
            .style(blue_text)
            .render(l_status[3], buf);
        Line::from(ctx.status[2].as_str())
            .style(blue_text)
            .render(l_status[4], buf);
    }

    Ok(())
}

fn handle_event<State>(
    handle: fn(
        &crossterm::event::Event, //
        ctx: &mut MiniSalsaState,
        state: &mut State,
    ) -> Result<Outcome, anyhow::Error>,
    event: crossterm::event::Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    ctx.event_cnt += 1;

    let t0 = SystemTime::now();

    let r = {
        use crossterm::event::Event;
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            }) => {
                ctx.quit = true;
                return Ok(Outcome::Changed);
            }
            Event::Key(KeyEvent {
                code: KeyCode::F(8),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                next_theme(ctx);

                // hack to have some way to notify the app
                let event = Event::Key(KeyEvent::new(
                    KeyCode::Media(MediaKeyCode::Play),
                    KeyModifiers::NONE,
                ));
                handle(&event, ctx, state)?;
                return Ok(Outcome::Changed);
            }
            Event::Key(KeyEvent {
                code: KeyCode::F(8),
                modifiers: KeyModifiers::SHIFT,
                kind: KeyEventKind::Press,
                ..
            }) => {
                prev_theme(ctx);

                // hack to have some way to notify the app
                let event = Event::Key(KeyEvent::new(
                    KeyCode::Media(MediaKeyCode::Play),
                    KeyModifiers::NONE,
                ));
                handle(&event, ctx, state)?;
                return Ok(Outcome::Changed);
            }
            Event::Resize(_, _) => return Ok(Outcome::Changed),
            _ => {}
        }

        handle(&event, ctx, state)?
    };

    ctx.last_event = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    if !ctx.hide_timing {
        ctx.status[2] = format!(" Handle {:.0?}", ctx.last_event).to_string();
    }

    Ok(r)
}

pub fn setup_logging() -> Result<(), anyhow::Error> {
    let log = PathBuf::from("test.log");
    if log.exists() {
        fs::remove_file(&log)?;
    }
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(&log)?)
        .apply()?;
    Ok(())
}

pub fn layout_grid<const X: usize, const Y: usize>(
    area: Rect,
    horizontal: Layout,
    vertical: Layout,
) -> [[Rect; Y]; X] {
    let hori = horizontal.split(Rect::new(area.x, 0, area.width, 0));
    let vert = vertical.split(Rect::new(0, area.y, 0, area.height));

    let mut res = [[Rect::default(); Y]; X];
    for x in 0..X {
        let coldata = &mut res[x];
        for y in 0..Y {
            coldata[y].x = hori[x].x;
            coldata[y].width = hori[x].width;
            coldata[y].y = vert[y].y;
            coldata[y].height = vert[y].height;
        }
    }

    res
}

/// Fill the given area of the buffer.
pub fn fill_buf_area(buf: &mut Buffer, area: Rect, symbol: &str, style: impl Into<Style>) {
    let style = style.into();

    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.reset();
                cell.set_symbol(symbol);
                cell.set_style(style);
            }
        }
    }
}

pub fn prev_theme(ctx: &mut MiniSalsaState) {
    let themes = salsa_themes();

    let name = ctx.theme.name();
    let mut pos = themes.iter().position(|n| *n == name).unwrap_or(0);
    if pos == 0 {
        pos = themes.len().saturating_sub(1);
    } else {
        pos = pos - 1;
    }

    ctx.status[0] = format!("Ctrl-Q to quit. F8 Theme [{}]", themes[pos]);
    ctx.theme = create_salsa_theme(themes[pos]);
}

pub fn next_theme(ctx: &mut MiniSalsaState) {
    let themes = salsa_themes();

    let name = ctx.theme.name();
    let mut pos = themes.iter().position(|n| *n == name).unwrap_or(0);
    if pos + 1 == themes.len() {
        pos = 0;
    } else {
        pos = pos + 1;
    }

    ctx.status[0] = format!("Ctrl-Q to quit. F8 Theme [{}]", themes[pos]);
    ctx.theme = create_salsa_theme(themes[pos]);
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}

pub mod endless_scroll;
pub mod text_input_mock;
