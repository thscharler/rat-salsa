#![allow(unreachable_pub)]
#![allow(dead_code)]

use crate::mini_salsa::theme::THEME;
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
use log::error;
use rat_event::Outcome;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Widget;
use ratatui::{Frame, Terminal};
use std::fs;
use std::io::{stdout, Stdout};
use std::time::{Duration, SystemTime};

pub struct MiniSalsaState {
    pub status: [String; 3],
    pub quit: bool,
}

impl Default for MiniSalsaState {
    fn default() -> Self {
        let mut s = Self {
            status: Default::default(),
            quit: false,
        };
        s.status[0] = "Ctrl-Q to quit.".into();
        s
    }
}

pub fn run_ui<Data, State>(
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
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    stdout().execute(EnableBlinking)?;
    stdout().execute(SetCursorStyle::BlinkingBar)?;
    stdout().execute(EnableBracketedPaste)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut istate = MiniSalsaState::default();

    repaint_ui(&mut terminal, repaint, data, &mut istate, state)?;

    let r = 'l: loop {
        let o = match crossterm::event::poll(Duration::from_millis(10)) {
            Ok(true) => {
                let event = match crossterm::event::read() {
                    Ok(v) => v,
                    Err(e) => break 'l Err(anyhow!(e)),
                };
                match handle_event(handle, event, data, &mut istate, state) {
                    Ok(v) => v,
                    Err(e) => break 'l Err(e),
                }
            }
            Ok(false) => continue,
            Err(e) => break 'l Err(anyhow!(e)),
        };

        if istate.quit {
            break 'l Ok(());
        }

        match o {
            Outcome::Changed => {
                match repaint_ui(&mut terminal, repaint, data, &mut istate, state) {
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
) -> Result<(), anyhow::Error> {
    terminal.hide_cursor()?;

    _ = terminal.draw(|frame| {
        match repaint_tui(frame, repaint, data, istate, state) {
            Ok(_) => {}
            Err(e) => {
                error!("{:?}", e)
            }
        };
    });

    Ok(())
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
    let t0 = SystemTime::now();
    let area = frame.size();

    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    repaint(frame, l1[0], data, istate, state)?;

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    istate.status[1] = format!("Render {:?}", el).to_string();

    let l_status = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(17),
        Constraint::Length(17),
    ])
    .split(l1[1]);

    Line::from(istate.status[0].as_str())
        .style(THEME.black(2))
        .render(l_status[0], frame.buffer_mut());
    Line::from(istate.status[1].as_str())
        .style(THEME.deepblue(0))
        .render(l_status[1], frame.buffer_mut());
    Line::from(istate.status[2].as_str())
        .style(THEME.deepblue(1))
        .render(l_status[2], frame.buffer_mut());

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

        let r = handle(&event, data, istate, state)?;

        r
    };

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    istate.status[2] = format!("Handle {:?}", el).to_string();

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

pub mod theme {
    use rat_ftable::FTableStyle;
    use rat_scrolled::ScrollStyle;
    use rat_widget::button::ButtonStyle;
    use rat_widget::file_dialog::FileDialogStyle;
    use rat_widget::input::TextInputStyle;
    use rat_widget::list::RListStyle;
    use rat_widget::menuline::MenuStyle;
    use rat_widget::msgdialog::MsgDialogStyle;
    use rat_widget::splitter::SplitStyle;
    use rat_widget::textarea::TextAreaStyle;
    use ratatui::style::{Color, Style, Stylize};

    #[derive(Debug, Default, Clone)]
    pub struct Scheme {
        pub white: [Color; 4],
        pub black: [Color; 4],
        pub gray: [Color; 4],

        pub red: [Color; 4],
        pub orange: [Color; 4],
        pub yellow: [Color; 4],
        pub limegreen: [Color; 4],
        pub green: [Color; 4],
        pub bluegreen: [Color; 4],
        pub cyan: [Color; 4],
        pub blue: [Color; 4],
        pub deepblue: [Color; 4],
        pub purple: [Color; 4],
        pub magenta: [Color; 4],
        pub redpink: [Color; 4],

        pub primary: [Color; 4],
        pub secondary: [Color; 4],
    }

    impl Scheme {
        /// Create a style from the given white shade.
        /// n is `0..=3`
        pub fn white(&self, n: usize) -> Style {
            self.style(self.white[n])
        }

        /// Create a style from the given black shade.
        /// n is `0..=3`
        pub fn black(&self, n: usize) -> Style {
            self.style(self.black[n])
        }

        /// Create a style from the given gray shade.
        /// n is `0..=3`
        pub fn gray(&self, n: usize) -> Style {
            self.style(self.gray[n])
        }

        /// Create a style from the given red shade.
        /// n is `0..=3`
        pub fn red(&self, n: usize) -> Style {
            self.style(self.red[n])
        }

        /// Create a style from the given orange shade.
        /// n is `0..=3`
        pub fn orange(&self, n: usize) -> Style {
            self.style(self.orange[n])
        }

        /// Create a style from the given yellow shade.
        /// n is `0..=3`
        pub fn yellow(&self, n: usize) -> Style {
            self.style(self.yellow[n])
        }

        /// Create a style from the given limegreen shade.
        /// n is `0..=3`
        pub fn limegreen(&self, n: usize) -> Style {
            self.style(self.limegreen[n])
        }

        /// Create a style from the given green shade.
        /// n is `0..=3`
        pub fn green(&self, n: usize) -> Style {
            self.style(self.green[n])
        }

        /// Create a style from the given bluegreen shade.
        /// n is `0..=3`
        pub fn bluegreen(&self, n: usize) -> Style {
            self.style(self.bluegreen[n])
        }

        /// Create a style from the given cyan shade.
        /// n is `0..=3`
        pub fn cyan(&self, n: usize) -> Style {
            self.style(self.cyan[n])
        }

        /// Create a style from the given blue shade.
        /// n is `0..=3`
        pub fn blue(&self, n: usize) -> Style {
            self.style(self.blue[n])
        }

        /// Create a style from the given deepblue shade.
        /// n is `0..=3`
        pub fn deepblue(&self, n: usize) -> Style {
            self.style(self.deepblue[n])
        }

        /// Create a style from the given purple shade.
        /// n is `0..=3`
        pub fn purple(&self, n: usize) -> Style {
            self.style(self.purple[n])
        }

        /// Create a style from the given magenta shade.
        /// n is `0..=3`
        pub fn magenta(&self, n: usize) -> Style {
            self.style(self.magenta[n])
        }

        /// Create a style from the given redpink shade.
        /// n is `0..=3`
        pub fn redpink(&self, n: usize) -> Style {
            self.style(self.redpink[n])
        }

        /// Create a style from the given primary shade.
        /// n is `0..=3`
        pub fn primary(&self, n: usize) -> Style {
            self.style(self.primary[n])
        }

        /// Create a style from the given secondary shade.
        /// n is `0..=3`
        pub fn secondary(&self, n: usize) -> Style {
            self.style(self.secondary[n])
        }

        /// Focus style
        pub fn focus(&self) -> Style {
            let bg = self.primary[2];
            Style::default().fg(self.text_color(bg)).bg(bg)
        }

        /// Selection style
        pub fn select(&self) -> Style {
            let bg = self.secondary[1];
            Style::default().fg(self.text_color(bg)).bg(bg)
        }

        /// Text field style.
        pub fn text_input(&self) -> Style {
            self.style(self.gray[2])
        }

        /// Focus style
        pub fn text_input_focus(&self) -> Style {
            let bg = self.primary[2];
            Style::default().fg(self.text_color(bg)).bg(bg).underlined()
        }

        pub fn block(&self) -> Style {
            Style::default().fg(self.gray[1]).bg(self.black[1])
        }

        pub fn table(&self) -> Style {
            Style::default().fg(self.white[1]).bg(self.black[0])
        }

        pub fn table_header(&self) -> Style {
            Style::default().fg(self.white[1]).bg(self.blue[2])
        }

        pub fn table_footer(&self) -> Style {
            Style::default().fg(self.white[1]).bg(self.blue[2])
        }

        /// Focused text field style.
        pub fn text_focus(&self) -> Style {
            let bg = self.primary[0];
            Style::default().fg(self.text_color(bg)).bg(bg)
        }

        /// Text selection style.
        pub fn text_select(&self) -> Style {
            let bg = self.secondary[0];
            Style::default().fg(self.text_color(bg)).bg(bg)
        }

        /// Data display style. Used for lists, tables, ...
        pub fn data(&self) -> Style {
            Style::default().fg(self.white[0]).bg(self.black[1])
        }

        /// Background for dialogs.
        pub fn dialog_style(&self) -> Style {
            Style::default().fg(self.white[2]).bg(self.gray[1])
        }

        /// Style for the status line.
        pub fn status_style(&self) -> Style {
            Style::default().fg(self.white[0]).bg(self.black[2])
        }

        /// Complete TextAreaStyle
        pub fn textarea_style(&self) -> TextAreaStyle {
            TextAreaStyle {
                style: self.data(),
                focus: Some(self.focus()),
                select: Some(self.text_select()),
                ..TextAreaStyle::default()
            }
        }

        /// Complete TextInputStyle
        pub fn input_style(&self) -> TextInputStyle {
            TextInputStyle {
                style: self.text_input(),
                focus: Some(self.text_focus()),
                select: Some(self.text_select()),
                invalid: Some(Style::default().bg(self.red[3])),
                ..TextInputStyle::default()
            }
        }

        /// Complete MenuStyle
        pub fn menu_style(&self) -> MenuStyle {
            let menu = Style::default().fg(self.white[3]).bg(self.black[2]);
            MenuStyle {
                style: menu,
                title: Some(Style::default().fg(self.black[0]).bg(self.yellow[2])),
                select: Some(self.select()),
                focus: Some(self.focus()),
                ..Default::default()
            }
        }

        /// Complete FTableStyle
        pub fn table_style(&self) -> FTableStyle {
            FTableStyle {
                style: self.data(),
                select_row_style: Some(self.select()),
                show_row_focus: true,
                focus_style: Some(self.focus()),
                ..Default::default()
            }
        }

        /// Complete ListStyle
        pub fn list_style(&self) -> Style {
            self.data()
        }

        /// Complete ButtonStyle
        pub fn button_style(&self) -> Style {
            Style::default().fg(self.white[0]).bg(self.primary[0])
        }

        pub fn armed_style(&self) -> Style {
            Style::default().fg(self.black[0]).bg(self.secondary[0])
        }

        pub fn list_styles(&self) -> RListStyle {
            RListStyle {
                style: self.list_style(),
                select_style: Some(self.select()),
                focus_style: Some(self.focus()),
                ..Default::default()
            }
        }

        /// Complete ScrolledStyle
        pub fn scrolled_style(&self) -> ScrollStyle {
            let style = Style::default().fg(self.gray[0]).bg(self.black[1]);
            let arrow_style = Style::default().fg(self.secondary[0]).bg(self.black[1]);
            ScrollStyle {
                thumb_style: Some(style),
                track_style: Some(style),
                no_style: Some(style),
                begin_style: Some(arrow_style),
                end_style: Some(arrow_style),
                ..Default::default()
            }
        }

        /// Complete Split style
        pub fn split_style(&self) -> SplitStyle {
            let style = Style::default().fg(self.gray[0]).bg(self.black[1]);
            let arrow_style = Style::default().fg(self.secondary[0]).bg(self.black[1]);
            SplitStyle {
                style,
                arrow_style: Some(arrow_style),
                drag_style: Some(self.focus()),
                ..Default::default()
            }
        }

        /// Complete StatusLineStyle for a StatusLine with 3 indicator fields.
        /// This is what I need for the
        /// [minimal](https://github.com/thscharler/rat-salsa/blob/master/examples/minimal.rs)
        /// example, which shows timings for Render/Event/Action.
        pub fn statusline_style(&self) -> Vec<Style> {
            vec![
                self.status_style(),
                Style::default()
                    .fg(self.text_color(self.white[0]))
                    .bg(self.blue[3]),
                Style::default()
                    .fg(self.text_color(self.white[0]))
                    .bg(self.blue[2]),
                Style::default()
                    .fg(self.text_color(self.white[0]))
                    .bg(self.blue[1]),
            ]
        }

        pub fn file_dialog_style(&self) -> FileDialogStyle {
            FileDialogStyle {
                style: self.dialog_style(),
                list: Some(self.list_style()),
                path: Some(self.text_input()),
                name: Some(self.text_input()),
                invalid: Some(Style::new().fg(self.red[3]).bg(self.gray[2])),
                select: Some(self.select()),
                focus: Some(self.focus()),
                button: Some(ButtonStyle {
                    style: self.button_style(),
                    focus: Some(self.focus()),
                    armed: Some(self.armed_style()),
                    ..Default::default()
                }),
                ..Default::default()
            }
        }

        /// Complete MsgDialogStyle.
        pub fn msg_dialog_style(&self) -> MsgDialogStyle {
            MsgDialogStyle {
                style: self.status_style(),
                button: ButtonStyle {
                    style: self.button_style(),
                    focus: Some(self.focus()),
                    armed: Some(self.armed_style()),
                    ..Default::default()
                },
                ..Default::default()
            }
        }

        pub fn style(&self, color: Color) -> Style {
            Style::new().bg(color).fg(self.text_color(color))
        }

        /// Linear interpolation between the two colors.
        pub const fn linear4(c0: u32, c1: u32) -> [Color; 4] {
            // 1/3
            const fn i1(a: u8, b: u8) -> u8 {
                if a < b {
                    a + (b - a) / 3
                } else {
                    a - (a - b) / 3
                }
            }
            // 2/3
            const fn i2(a: u8, b: u8) -> u8 {
                if a < b {
                    b - (b - a) / 3
                } else {
                    b + (a - b) / 3
                }
            }

            let r0 = (c0 >> 16) as u8;
            let g0 = (c0 >> 8) as u8;
            let b0 = c0 as u8;

            let r3 = (c1 >> 16) as u8;
            let g3 = (c1 >> 8) as u8;
            let b3 = c1 as u8;

            let r1 = i1(r0, r3);
            let g1 = i1(g0, g3);
            let b1 = i1(b0, b3);

            let r2 = i2(r0, r3);
            let g2 = i2(g0, g3);
            let b2 = i2(b0, b3);

            [
                Color::Rgb(r0, g0, b0),
                Color::Rgb(r1, g1, b1),
                Color::Rgb(r2, g2, b2),
                Color::Rgb(r3, g3, b3),
            ]
        }

        /// This gives back `white[3]` or `black[0]` for text foreground
        /// providing good contrast to the given background.
        ///
        /// This converts RGB to grayscale and takes the grayscale value
        /// of VGA cyan as threshold, which is about 105 out of 255.
        /// This point is a bit arbitrary, just based on what I
        /// perceive as acceptable. But it produces a good reading
        /// contrast in my experience.
        ///
        /// For the named colors it takes the VGA equivalent as a base.
        /// For indexed colors it splits the range in half as an estimate.
        pub fn text_color(&self, color: Color) -> Color {
            match color {
                Color::Reset => self.white[3],
                Color::Black => self.white[3],        //0
                Color::Red => self.white[3],          //1
                Color::Green => self.white[3],        //2
                Color::Yellow => self.white[3],       //3
                Color::Blue => self.white[3],         //4
                Color::Magenta => self.white[3],      //5
                Color::Cyan => self.white[3],         //6
                Color::Gray => self.black[0],         //7
                Color::DarkGray => self.white[3],     //8
                Color::LightRed => self.black[0],     //9
                Color::LightGreen => self.black[0],   //10
                Color::LightYellow => self.black[0],  //11
                Color::LightBlue => self.white[3],    //12
                Color::LightMagenta => self.black[0], //13
                Color::LightCyan => self.black[0],    //14
                Color::White => self.black[0],        //15
                Color::Rgb(r, g, b) => {
                    // The formula used in the GIMP is Y = 0.3R + 0.59G + 0.11B;
                    let grey = r as f32 * 0.3f32 + g as f32 * 0.59f32 + b as f32 * 0.11f32;
                    if grey >= 105f32 {
                        self.black[0]
                    } else {
                        self.white[3]
                    }
                }
                Color::Indexed(n) => match n {
                    0..=6 => self.white[3],
                    7 => self.black[0],
                    8 => self.white[3],
                    9..=11 => self.black[0],
                    12 => self.white[3],
                    13..=15 => self.black[0],
                    v @ 16..=231 => {
                        if (v - 16) % 36 < 18 {
                            self.white[3]
                        } else {
                            self.black[0]
                        }
                    }
                    v @ 232..=255 => {
                        if (v - 232) % 24 < 12 {
                            self.white[3]
                        } else {
                            self.black[0]
                        }
                    }
                },
            }
        }
    }

    /// Imperial scheme.
    ///
    /// Uses purple and gold for primary/secondary.
    /// Other colors are bright, strong and slightly smudged.
    ///
    pub const THEME: Scheme = Scheme {
        primary: Scheme::linear4(0x300057, 0x8c00fd),
        secondary: Scheme::linear4(0x574b00, 0xffde00),

        white: Scheme::linear4(0xdedfe3, 0xf6f6f3),
        black: Scheme::linear4(0x0f1014, 0x2a2b37),
        gray: Scheme::linear4(0x3b3d4e, 0x6e7291),

        red: Scheme::linear4(0x480f0f, 0xd22d2d),
        orange: Scheme::linear4(0x482c0f, 0xd4812b),
        yellow: Scheme::linear4(0x756600, 0xffde00),
        limegreen: Scheme::linear4(0x2c4611, 0x80ce31),
        green: Scheme::linear4(0x186218, 0x32cd32),
        bluegreen: Scheme::linear4(0x206a52, 0x3bc49a),
        cyan: Scheme::linear4(0x0f2c48, 0x2bd4d4),
        blue: Scheme::linear4(0x162b41, 0x2b81d4),
        deepblue: Scheme::linear4(0x202083, 0x3232cd),
        purple: Scheme::linear4(0x4d008b, 0x8c00fd),
        magenta: Scheme::linear4(0x401640, 0xbd42bd),
        redpink: Scheme::linear4(0x47101d, 0xc33c5b),
    };
}
