#![allow(unreachable_pub)]
#![allow(dead_code)]

use crate::mini_salsa::palette::Palette;
use crate::mini_salsa::theme::ShellTheme;
use anyhow::anyhow;
use crossterm::ExecutableCommand;
use crossterm::cursor::{DisableBlinking, EnableBlinking, SetCursorStyle};
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, KeyCode,
    KeyEvent, KeyEventKind, KeyModifiers,
};
#[cfg(not(windows))]
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
#[cfg(not(windows))]
use crossterm::terminal::supports_keyboard_enhancement;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use log::error;
use rat_event::Outcome;
use rat_event::util::set_have_keyboard_enhancement;
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Widget;
use ratatui::{Frame, Terminal};
use std::cell::Cell;
use std::cmp::max;
use std::fs;
use std::io::{Stdout, stdout};
use std::time::{Duration, SystemTime};
use unicode_segmentation::UnicodeSegmentation;

pub struct MiniSalsaState {
    pub name: String,
    pub theme: &'static ShellTheme,
    pub frame: usize,
    pub event_cnt: usize,
    pub timing: bool,
    pub status: [String; 3],
    pub focus_outcome: Outcome,
    pub focus_outcome_cell: Cell<Outcome>,
    pub quit: bool,
}

impl MiniSalsaState {
    fn new(name: &str) -> Self {
        let mut s = Self {
            name: name.to_string(),
            theme: &THEME,
            frame: 0,
            event_cnt: 0,
            timing: true,
            status: Default::default(),
            focus_outcome: Default::default(),
            focus_outcome_cell: Default::default(),
            quit: false,
        };
        s.status[0] = "Ctrl-Q to quit.".into();
        s
    }
}

pub fn run_ui<Data, State>(
    name: &str,
    init: fn(&mut Data, &mut MiniSalsaState, &mut State),
    handle: fn(
        &crossterm::event::Event,
        data: &mut Data,
        istate: &mut MiniSalsaState,
        state: &mut State,
    ) -> Result<Outcome, anyhow::Error>,
    repaint: fn(
        &mut Frame<'_>,
        Rect,
        &mut Data,
        &mut MiniSalsaState,
        &mut State,
    ) -> Result<(), anyhow::Error>,
    data: &mut Data,
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

    let mut istate = MiniSalsaState::new(name);

    init(data, &mut istate, state);

    istate.frame = repaint_ui(&mut terminal, repaint, data, &mut istate, state)?;

    let r = 'l: loop {
        istate.focus_outcome = Outcome::Continue;
        istate.focus_outcome_cell.set(Outcome::Continue);

        let o = match crossterm::event::poll(Duration::from_millis(10)) {
            Ok(true) => {
                let event = match crossterm::event::read() {
                    Ok(v) => v,
                    Err(e) => break 'l Err(anyhow!(e)),
                };
                match handle_event(handle, event, data, &mut istate, state) {
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
                match repaint_ui(&mut terminal, repaint, data, &mut istate, state) {
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

pub fn mock_init<Data, State>(_data: &mut Data, _istate: &mut MiniSalsaState, _state: &mut State) {}

fn repaint_ui<Data, State>(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    repaint: fn(
        &mut Frame<'_>,
        Rect,
        &mut Data,
        &mut MiniSalsaState,
        &mut State,
    ) -> Result<(), anyhow::Error>,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<usize, anyhow::Error> {
    terminal.hide_cursor()?;

    let completed = terminal.draw(|frame| {
        match repaint_tui(frame, repaint, data, istate, state) {
            Ok(_) => {}
            Err(e) => {
                error!("{:?}", e)
            }
        };
    })?;

    Ok(completed.count)
}

fn repaint_tui<Data, State>(
    frame: &mut Frame<'_>,
    repaint: fn(
        &mut Frame<'_>,
        Rect,
        &mut Data,
        &mut MiniSalsaState,
        &mut State,
    ) -> Result<(), anyhow::Error>,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let area = frame.area();

    let l1 = Layout::vertical([
        Constraint::Fill(1), //
        Constraint::Length(1),
    ])
    .split(area);

    let t0 = SystemTime::now();
    repaint(frame, l1[0], data, istate, state)?;
    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));

    if istate.timing {
        istate.status[1] = format!("Render #{} | {:.0?}", frame.count(), el).to_string();
    }

    let l_status = Layout::horizontal([
        Constraint::Length(2 + istate.name.graphemes(true).count() as u16),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(18),
        Constraint::Length(18),
    ])
    .split(l1[1]);

    Line::from_iter(["[", istate.name.as_str(), "]"])
        .style(istate.theme.status_base())
        .render(l_status[0], frame.buffer_mut());
    Line::from(" ")
        .style(istate.theme.status_base())
        .render(l_status[1], frame.buffer_mut());
    Line::from(istate.status[0].as_str())
        .style(istate.theme.statusline_style()[0])
        .render(l_status[2], frame.buffer_mut());
    Line::from(istate.status[1].as_str())
        .style(istate.theme.statusline_style()[1])
        .render(l_status[3], frame.buffer_mut());
    Line::from(istate.status[2].as_str())
        .style(istate.theme.statusline_style()[2])
        .render(l_status[4], frame.buffer_mut());

    Ok(())
}

fn handle_event<Data, State>(
    handle: fn(
        &crossterm::event::Event,
        data: &mut Data,
        istate: &mut MiniSalsaState,
        state: &mut State,
    ) -> Result<Outcome, anyhow::Error>,
    event: crossterm::event::Event,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    istate.event_cnt += 1;

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
                istate.quit = true;
                return Ok(Outcome::Changed);
            }
            Event::Resize(_, _) => return Ok(Outcome::Changed),
            _ => {}
        }

        handle(&event, data, istate, state)?
    };
    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));

    if istate.timing {
        istate.status[2] = format!(" Handle {:.0?}", el).to_string();
    }

    Ok(r)
}

pub fn setup_logging() -> Result<(), anyhow::Error> {
    _ = fs::remove_file("log.log");
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
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

pub mod endless_scroll;
pub mod palette;
pub mod text_input_mock;
pub mod theme;

pub static THEME: ShellTheme = ShellTheme::new("Imperial Shell", PALETTE);

/// An adaption of nvchad's tundra theme.
///
/// -- Thanks to original theme for existing <https://github.com/sam4llis/nvim-tundra>
/// -- this is a modified version of it
pub const PALETTE: Palette = IMPERIAL;

/// Imperial palette.
///
/// Uses purple and gold for primary/secondary.
/// Other colors are bright, strong and slightly smudged.
///
pub const IMPERIAL: Palette = Palette {
    name: "Imperial",

    primary: Palette::interpolate(0x300057, 0x8c00fd, 63),
    secondary: Palette::interpolate(0x574b00, 0xffde00, 63),

    text_light: Palette::color32(0xdedfe3),
    text_bright: Palette::color32(0xf6f6f3),
    text_dark: Palette::color32(0x2a2b37),
    text_black: Palette::color32(0x0f1014),

    white: Palette::interpolate(0xdedfe3, 0xf6f6f3, 63),
    black: Palette::interpolate(0x0f1014, 0x2a2b37, 63),
    gray: Palette::interpolate(0x3b3d4e, 0x6e7291, 63),

    red: Palette::interpolate(0x480f0f, 0xd22d2d, 63),
    orange: Palette::interpolate(0x482c0f, 0xd4812b, 63),
    yellow: Palette::interpolate(0x756600, 0xffde00, 63),
    limegreen: Palette::interpolate(0x2c4611, 0x80ce31, 63),
    green: Palette::interpolate(0x186218, 0x32cd32, 63),
    bluegreen: Palette::interpolate(0x206a52, 0x3bc49a, 63),
    cyan: Palette::interpolate(0x0f2c48, 0x2bd4d4, 63),
    blue: Palette::interpolate(0x162b41, 0x2b81d4, 63),
    deepblue: Palette::interpolate(0x202083, 0x3232cd, 63),
    purple: Palette::interpolate(0x4d008b, 0x8c00fd, 63),
    magenta: Palette::interpolate(0x401640, 0xbd42bd, 63),
    redpink: Palette::interpolate(0x47101d, 0xc33c5b, 63),
};

pub const OCEAN: Palette = Palette {
    name: "Ocean",

    primary: Palette::interpolate(0xff8d3c, 0xffbf3c, 63),
    secondary: Palette::interpolate(0x2b4779, 0x6688cc, 63),

    text_light: Palette::color32(0xe5e5dd),
    text_bright: Palette::color32(0xf2f2ee),
    text_dark: Palette::color32(0x0c092c),
    text_black: Palette::color32(0x030305),

    white: Palette::interpolate(0xe5e5dd, 0xf2f2ee, 63),
    black: Palette::interpolate(0x030305, 0x0c092c, 63),
    gray: Palette::interpolate(0x4f6167, 0xbcc7cc, 63),

    red: Palette::interpolate(0xff5e7f, 0xff9276, 63),
    orange: Palette::interpolate(0xff9f5b, 0xffdc94, 63),
    yellow: Palette::interpolate(0xffda5d, 0xfff675, 63),
    limegreen: Palette::interpolate(0x7d8447, 0xe1e5b9, 63),
    green: Palette::interpolate(0x658362, 0x99c794, 63),
    bluegreen: Palette::interpolate(0x3a615c, 0x5b9c90, 63),
    cyan: Palette::interpolate(0x24adbc, 0xb8dade, 63),
    blue: Palette::interpolate(0x4f86ca, 0xbfdcff, 63),
    deepblue: Palette::interpolate(0x2b4779, 0x6688cc, 63),
    purple: Palette::interpolate(0x5068d7, 0xc7c4ff, 63),
    magenta: Palette::interpolate(0x7952d6, 0xc9bde4, 63),
    redpink: Palette::interpolate(0x9752d6, 0xcebde4, 63),
};

pub const TUNDRA: Palette = Palette {
    name: "Tundra",

    primary: Palette::interpolate(0xe6eaf2, 0xffffff, 63),
    secondary: Palette::interpolate(0xa8bbd4, 0x719bd3, 63),

    text_light: Palette::color32(0xe6eaf2),
    text_bright: Palette::color32(0xffffff),
    text_dark: Palette::color32(0x1a2130),
    text_black: Palette::color32(0x0b1221),

    white: Palette::interpolate(0xe6eaf2, 0xffffff, 63),
    black: Palette::interpolate(0x0b1221, 0x1a2130, 63),
    gray: Palette::interpolate(0x3e4554, 0x5f6675, 63),

    red: Palette::interpolate(0xfccaca, 0xfca5a5, 63),
    orange: Palette::interpolate(0xfad9c5, 0xfbc19d, 63),
    yellow: Palette::interpolate(0xe8d7b7, 0xe8d4b0, 63),
    limegreen: Palette::interpolate(0xbce8b7, 0xb5e8b0, 63),
    green: Palette::interpolate(0xbce8b7, 0xb5e8b0, 63),
    bluegreen: Palette::interpolate(0xa8bbd4, 0x719bd3, 63),
    cyan: Palette::interpolate(0xc8eafc, 0xbae6fd, 63),
    blue: Palette::interpolate(0xc7d0fc, 0xa5b4fc, 63),
    deepblue: Palette::interpolate(0xbfcaf2, 0x9baaf2, 63),
    purple: Palette::interpolate(0xb7abd9, 0xb3a6da, 63),
    magenta: Palette::interpolate(0xffc9c9, 0xff8e8e, 63),
    redpink: Palette::interpolate(0xfffcad, 0xfecdd3, 63),
};
