//!
//! A nice little game of life.
//!
//! Aside from the obvious this is to demonstrate additional
//! event-sources. `PollTick` implements such an event-source
//! that produces tick-events, and distributes them with its
//! own trait.
//!

use crate::app::{Scenery, SceneryState};
use crate::config::LifeConfig;
use crate::global::{GlobalState, PollTick};
use crate::message::LifeMsg;
use anyhow::Error;
use rat_salsa::{run_tui, RunConfig};
use rat_theme::dark_theme::DarkTheme;
use rat_theme::scheme::IMPERIAL;
use std::env::args;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, LifeMsg, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = LifeConfig::default();
    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = GlobalState::new(config, theme, Duration::from_millis(500));

    let app = Scenery;

    let game = if let Some(f) = args().nth(1) {
        game::load_life(&PathBuf::from(f), &global.theme)?
    } else {
        global::rat_state()
    };

    let mut state = SceneryState::default().game(game);

    let poll_tick = PollTick::new(Duration::from_secs(2), global.tick.clone());

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig::default()?.poll(poll_tick).threads(1),
    )?;

    Ok(())
}

/// Globally accessible data/state.
pub mod global {
    use crate::config::LifeConfig;
    use crate::game::LifeGameState;
    use rat_salsa::poll::PollEvents;
    use rat_salsa::{AppContext, AppState, Control};
    use rat_theme::dark_theme::DarkTheme;
    use rat_widget::msgdialog::MsgDialogState;
    use rat_widget::statusline::StatusLineState;
    use std::cell::RefCell;
    use std::fmt::Debug;
    use std::rc::Rc;
    use std::time::{Duration, SystemTime};

    #[rustfmt::skip]
    pub fn rat_state() -> LifeGameState {
        LifeGameState::new(
            "rat",
            "1357/1357",
            (17, 10),
            vec![
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,1,1,1,1,1,0,1,1,0,0,0,
                0,0,0,1,1,1,0,0,0,0,0,1,1,1,1,1,0,
                0,0,1,0,0,0,1,1,1,1,1,0,1,1,0,0,0,
                0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
            ],
        )
    }

    #[derive(Debug)]
    pub struct GlobalState {
        pub cfg: LifeConfig,
        pub theme: DarkTheme,

        pub running: bool,
        pub tick: Rc<RefCell<Duration>>,

        pub status: StatusLineState,
        pub error_dlg: MsgDialogState,
    }

    #[derive(Debug)]
    pub struct PollTick {
        tick: Rc<RefCell<Duration>>,
        next: SystemTime,
    }

    impl PollTick {
        pub fn new(start: Duration, tick: Rc<RefCell<Duration>>) -> Self {
            Self {
                tick,
                next: SystemTime::now() + start,
            }
        }
    }

    pub trait Tick<Global, Message, Error>
    where
        Message: 'static + Send + Debug,
        Error: 'static + Send + Debug,
    {
        fn tick(
            &mut self,
            ctx: &mut AppContext<'_, Global, Message, Error>,
        ) -> Result<Control<Message>, Error>;
    }

    impl<Global, State, Message, Error> PollEvents<Global, State, Message, Error> for PollTick
    where
        State: Tick<Global, Message, Error>,
        State: AppState<Global, Message, Error>,
        Message: 'static + Send + Debug,
        Error: 'static + Send + Debug,
    {
        fn poll(
            &mut self,
            _ctx: &mut AppContext<'_, Global, Message, Error>,
        ) -> Result<bool, Error> {
            Ok(self.next <= SystemTime::now())
        }

        fn read_exec(
            &mut self,
            state: &mut State,
            ctx: &mut AppContext<'_, Global, Message, Error>,
        ) -> Result<Control<Message>, Error> {
            if self.next <= SystemTime::now() {
                let tick = *self.tick.borrow();
                self.next += tick;
                state.tick(ctx)
            } else {
                Ok(Control::Continue)
            }
        }
    }

    impl GlobalState {
        pub fn new(cfg: LifeConfig, theme: DarkTheme, tick: Duration) -> Self {
            Self {
                cfg,
                theme,
                running: true,
                tick: Rc::new(RefCell::new(tick)),
                status: Default::default(),
                error_dlg: Default::default(),
            }
        }
    }
}

/// Configuration.
pub mod config {
    #[derive(Debug, Default)]
    pub struct LifeConfig {}
}

/// Application wide messages.
pub mod message {
    #[derive(Debug)]
    pub enum LifeMsg {
        Message(String),
    }
}

pub mod app {
    use crate::game::LifeGameState;
    use crate::global::{GlobalState, Tick};
    use crate::life::{Life, LifeState};
    use crate::message::LifeMsg;
    use crate::{AppContext, RenderContext};
    use anyhow::Error;
    use crossterm::event::Event;
    use rat_salsa::event::ct_event;
    use rat_salsa::timer::TimeOut;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::{ConsumedEvent, Dialog, HandleEvent};
    use rat_widget::focus::HasFocus;
    use rat_widget::msgdialog::MsgDialog;
    use rat_widget::statusline::StatusLine;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::style::Style;
    use ratatui::widgets::StatefulWidget;
    use std::time::{Duration, SystemTime};

    #[derive(Debug)]
    pub struct Scenery;

    #[derive(Debug, Default)]
    pub struct SceneryState {
        pub life: LifeState,
    }

    impl SceneryState {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn game(mut self, game: LifeGameState) -> Self {
            self.life.game = game;
            self
        }
    }

    impl AppWidget<GlobalState, LifeMsg, Error> for Scenery {
        type State = SceneryState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_>,
        ) -> Result<(), Error> {
            let t0 = SystemTime::now();

            let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

            Life::new()
                .style(ctx.g.theme.limegreen(2))
                .render(area, buf, &mut state.life, ctx)?;

            if ctx.g.error_dlg.active() {
                let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
                err.render(layout[0], buf, &mut ctx.g.error_dlg);
            }

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(2, format!("R {:.0?}", el).to_string());

            let status_layout =
                Layout::horizontal([Constraint::Fill(61), Constraint::Fill(39)]).split(layout[1]);
            let scheme = ctx.g.theme.scheme();
            let status = StatusLine::new()
                .layout([
                    Constraint::Fill(1),
                    Constraint::Length(7),
                    Constraint::Length(8),
                    Constraint::Length(8),
                ])
                .styles(vec![
                    ctx.g.theme.status_style(),
                    Style::default()
                        .fg(scheme.text_color(scheme.white[0]))
                        .bg(scheme.orange[2]),
                    Style::default()
                        .fg(scheme.text_color(scheme.white[0]))
                        .bg(scheme.blue[3]),
                    Style::default()
                        .fg(scheme.text_color(scheme.white[0]))
                        .bg(scheme.blue[2]),
                    Style::default()
                        .fg(scheme.text_color(scheme.white[0]))
                        .bg(scheme.blue[1]),
                ]);
            status.render(status_layout[1], buf, &mut ctx.g.status);

            Ok(())
        }
    }

    impl AppState<GlobalState, LifeMsg, Error> for SceneryState {
        fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            ctx.focus = Some(self.life.focus());
            self.life.init(ctx)?;
            Ok(())
        }

        fn timer(
            &mut self,
            event: &TimeOut,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<LifeMsg>, Error> {
            let t0 = SystemTime::now();

            ctx.focus = Some(self.life.focus());
            let r = self.life.timer(event, ctx)?;

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(3, format!("T {:.0?}", el).to_string());

            Ok(r)
        }

        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<LifeMsg>, Error> {
            let t0 = SystemTime::now();

            let mut r = match &event {
                ct_event!(resized) => {
                    ctx.queue(Control::Changed);
                    Control::Continue
                }
                ct_event!(key press CONTROL-'q') => Control::Quit,
                _ => Control::Continue,
            };

            r = r.or_else(|| {
                if ctx.g.error_dlg.active() {
                    ctx.g.error_dlg.handle(&event, Dialog).into()
                } else {
                    Control::Continue
                }
            });

            r = r.or_else_try(|| {
                ctx.focus = Some(self.life.focus());
                self.life.crossterm(&event, ctx)
            })?;

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(3, format!("H {:.0?}", el).to_string());

            Ok(r)
        }

        fn message(
            &mut self,
            event: &mut LifeMsg,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<LifeMsg>, Error> {
            let t0 = SystemTime::now();

            #[allow(unreachable_patterns)]
            let r = match event {
                LifeMsg::Message(s) => {
                    ctx.g.status.status(0, &*s);
                    Control::Changed
                }
                _ => {
                    ctx.focus = Some(self.life.focus());
                    self.life.message(event, ctx)?
                }
            };

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(3, format!("A {:.0?}", el).to_string());

            Ok(r)
        }

        fn error(&self, event: Error, ctx: &mut AppContext<'_>) -> Result<Control<LifeMsg>, Error> {
            ctx.g.error_dlg.append(format!("{:?}", &*event).as_str());
            Ok(Control::Changed)
        }
    }

    impl Tick<GlobalState, LifeMsg, Error> for SceneryState {
        fn tick(
            &mut self,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, LifeMsg, Error>,
        ) -> Result<Control<LifeMsg>, Error> {
            let t0 = SystemTime::now();

            let r = self.life.tick(ctx)?;

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(3, format!("@ {:.0?}", el).to_string());

            Ok(r)
        }
    }
}

pub mod life {
    use crate::game::{LifeGame, LifeGameState};
    use crate::global::Tick;
    use crate::{AppContext, GlobalState, LifeMsg, RenderContext};
    use anyhow::Error;
    use crossterm::event::Event;
    use log::debug;
    use rat_salsa::{AppState, AppWidget, Control};
    use rat_widget::event::{try_flow, HandleEvent, Regular};
    use rat_widget::focus::{Focus, HasFocus};
    use rat_widget::menuline::{MenuLine, MenuLineState, MenuOutcome};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::style::Style;
    use ratatui::widgets::StatefulWidget;
    use std::fmt::Debug;
    use std::time::Duration;

    #[derive(Debug, Default)]
    pub struct Life {
        pub style: Style,
    }

    impl Life {
        pub fn new() -> Life {
            Self::default()
        }

        pub fn style(mut self, style: Style) -> Self {
            self.style = style;
            self
        }
    }

    #[derive(Debug)]
    pub struct LifeState {
        pub game: LifeGameState,
        pub menu: MenuLineState,
    }

    impl Default for LifeState {
        fn default() -> Self {
            let mut s = Self {
                game: LifeGameState::default(),
                menu: Default::default(),
            };
            s.menu.select(Some(0));
            s
        }
    }

    impl AppWidget<GlobalState, LifeMsg, Error> for Life {
        type State = LifeState;

        fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_>,
        ) -> Result<(), Error> {
            let r = Layout::new(
                Direction::Vertical,
                [
                    Constraint::Fill(1), //
                    Constraint::Length(1),
                ],
            )
            .split(area);

            LifeGame.render(r[0], buf, &mut state.game);

            let menu = MenuLine::new()
                .styles(ctx.g.theme.menu_style())
                .title(format!("--({})>", state.game.name))
                .add_str(if ctx.g.running { "Pau_se" } else { "_Start" })
                .add_str("_Next")
                .add_str("_Faster")
                .add_str("Slowe_r")
                .add_str("Rest_art")
                .add_str("Ran_dom")
                .add_str("_Quit");
            menu.render(r[1], buf, &mut state.menu);

            Ok(())
        }
    }

    impl HasFocus for LifeState {
        fn focus(&self) -> Focus {
            let mut f = Focus::new();
            f.add(&self.menu);
            f
        }
    }

    impl AppState<GlobalState, LifeMsg, Error> for LifeState {
        fn init(
            &mut self,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, LifeMsg, Error>,
        ) -> Result<(), Error> {
            ctx.focus().first();
            debug!("{:?}", self.game);
            Ok(())
        }

        #[allow(unused_variables)]
        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<LifeMsg>, Error> {
            let f = ctx.focus_mut().handle(event, Regular);
            ctx.queue(f);

            try_flow!(match self.menu.handle(event, Regular) {
                MenuOutcome::Activated(0) => {
                    ctx.g.running = !ctx.g.running;
                    Control::Changed
                }
                MenuOutcome::Activated(1) => {
                    self.game.turn();
                    ctx.g.status.status(1, self.game.round.to_string());
                    Control::Changed
                }
                MenuOutcome::Activated(2) => {
                    let mut tick = *ctx.g.tick.borrow();
                    if tick.as_millis() == 0 {
                        // noop
                    } else if tick.as_millis() <= 10 {
                        tick -= Duration::from_millis(1);
                    } else if tick.as_millis() <= 100 {
                        tick -= Duration::from_millis(10);
                    } else {
                        tick -= Duration::from_millis(100);
                    }
                    *ctx.g.tick.borrow_mut() = tick;
                    ctx.g.status.status(0, format!("Tick {:#?}", tick));
                    Control::Changed
                }
                MenuOutcome::Activated(3) => {
                    let mut tick = *ctx.g.tick.borrow();
                    if tick.as_millis() < 10 {
                        tick += Duration::from_millis(1);
                    } else if tick.as_millis() < 100 {
                        tick += Duration::from_millis(10);
                    } else {
                        tick += Duration::from_millis(100);
                    }
                    *ctx.g.tick.borrow_mut() = tick;
                    ctx.g.status.status(0, format!("Tick {:#?}", tick));
                    Control::Changed
                }
                MenuOutcome::Activated(4) => {
                    self.game.restart();
                    Control::Changed
                }
                MenuOutcome::Activated(5) => {
                    self.game.random();
                    Control::Changed
                }
                MenuOutcome::Activated(6) => {
                    Control::Quit
                }
                v => v.into(),
            });

            Ok(Control::Continue)
        }
    }

    impl Tick<GlobalState, LifeMsg, Error> for LifeState {
        fn tick(
            &mut self,
            ctx: &mut rat_salsa::AppContext<'_, GlobalState, LifeMsg, Error>,
        ) -> Result<Control<LifeMsg>, Error> {
            if ctx.g.running {
                self.game.turn();
                ctx.g.status.status(1, self.game.round.to_string());
                Ok(Control::Changed)
            } else {
                Ok(Control::Continue)
            }
        }
    }
}

pub mod game {
    use anyhow::{anyhow, Error};
    use configparser::ini::Ini;
    use log::debug;
    use rand::random;
    use rat_theme::dark_theme::DarkTheme;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::prelude::StatefulWidget;
    use ratatui::style::{Color, Style, Stylize};
    use std::cmp::max;
    use std::fmt::{Debug, Formatter};
    use std::fs::File;
    use std::io::Read;
    use std::mem;
    use std::path::Path;

    #[derive(Debug, Default)]
    pub struct LifeGame;

    #[derive(Default)]
    pub struct LifeGameState {
        pub style1: Style,
        pub style0: Style,
        pub name: String,

        pub area_0: Rect,
        pub world_0: Vec<u8>,

        pub area: Rect,
        pub world: Vec<u8>,
        pub new_world: Vec<u8>,

        pub live: u16,
        pub birth: u16,
        pub round: u32,
    }

    fn rule_str(mut rule: u16) -> String {
        let mut r = String::new();
        for i in 0..=9 {
            if rule % 2 == 1 {
                r.push_str(&i.to_string());
            }
            rule = rule / 2;
        }
        r
    }

    impl Debug for LifeGameState {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            _ = writeln!(f, "LifeGame {} round {}", self.name, self.round);
            _ = writeln!(
                f,
                "    rule={}/{}",
                rule_str(self.live),
                rule_str(self.birth)
            );
            _ = writeln!(f, "    s_one = {:?}", self.style1.bg);
            _ = writeln!(f, "    s_zero = {:?}", self.style0.bg);
            if self.round == 0 {
                _ = writeln!(f, "    init = {}x{}", self.area_0.width, self.area_0.height);
                for y in 0..self.area_0.height {
                    _ = writeln!(f, "    init = ");
                    for x in 0..self.area_0.width {
                        _ = write!(
                            f,
                            "        {:1}",
                            self.world_0[(y * self.area_0.width + x) as usize]
                        );
                    }
                    _ = writeln!(f);
                }
            } else {
                _ = writeln!(f, "    curr = {}x{}", self.area.width, self.area.height);
                for y in 0..self.area.height {
                    _ = writeln!(f, "    curr = ");
                    for x in 0..self.area.width {
                        _ = write!(
                            f,
                            "        {:1}",
                            self.world[(y * self.area.width + x) as usize]
                        );
                    }
                    _ = writeln!(f);
                }
            }

            Ok(())
        }
    }

    impl StatefulWidget for LifeGame {
        type State = LifeGameState;

        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            state.adjust_area(area);

            for y in 0..area.height {
                for x in 0..area.width {
                    let pos = (y * area.width + x) as usize;
                    if let Some(cell) = buf.cell_mut((x + area.x, y + area.y)) {
                        if state.world[pos] == 1 {
                            cell.set_style(state.style1);
                        } else {
                            cell.set_style(state.style0);
                        }
                    }
                }
            }
        }
    }

    impl LifeGameState {
        pub fn new(name: &str, rule: &str, size: (u16, u16), mut data: Vec<u8>) -> Self {
            data.resize_with((size.0 * size.1) as usize, Default::default);
            let mut s = Self {
                style1: Style::default().on_green(),
                style0: Style::default(),
                name: Default::default(),
                live: 0,
                birth: 0,
                round: 0,
                area_0: Rect::new(0, 0, size.0, size.1),
                world_0: data.clone(),
                area: Rect::new(0, 0, size.0, size.1),
                world: data,
                new_world: vec![0; (size.0 * size.1) as usize],
            };
            s.set_name(name);
            s.set_rule(rule);
            s
        }

        pub fn set_name(&mut self, name: &str) {
            self.name = name.to_string();
        }

        /// Set the rules.
        pub fn set_rule(&mut self, r: &str) {
            (self.live, self.birth) = rule(r);
        }

        /// Style for living cells.
        pub fn set_style1(&mut self, style: Style) {
            self.style1 = style;
        }

        /// Style for living cells.
        pub fn set_style0(&mut self, style: Style) {
            self.style0 = style;
        }

        /// Change the area size.
        /// Centers the current area if it's smaller.
        /// Clips otherwise.
        pub fn adjust_area(&mut self, area: Rect) {
            if self.area != area {
                let shift_x = area.width.saturating_sub(self.area.width) / 2;
                let shift_y = area.height.saturating_sub(self.area.height) / 2;

                let mut world = vec![0; area.width as usize * area.height as usize];
                let new_world = vec![0; area.width as usize * area.height as usize];

                for y_old in 0..self.area.height {
                    if y_old >= area.height {
                        break;
                    }
                    for x_old in 0..self.area.width {
                        if x_old >= area.width {
                            break;
                        }
                        world[((shift_y + y_old) * area.width + (shift_x + x_old)) as usize] =
                            self.world[(y_old * self.area.width + x_old) as usize];
                    }
                }

                self.world = world;
                self.new_world = new_world;
                self.area = area;
            }
        }

        #[inline]
        fn wrapping_add(pos: u16, range: u16, r: i16) -> u16 {
            if r < 0 {
                if pos >= r.unsigned_abs() {
                    pos - r.unsigned_abs()
                } else {
                    range + pos - r.unsigned_abs()
                }
            } else {
                if pos + r.unsigned_abs() < range {
                    pos + r.unsigned_abs()
                } else {
                    pos + r.unsigned_abs() - range
                }
            }
        }

        #[inline]
        fn toroid(&self, pos: (u16, u16), r: (i16, i16)) -> u8 {
            let rpos_x = Self::wrapping_add(pos.0, self.area.width, r.0);
            let rpos_y = Self::wrapping_add(pos.1, self.area.height, r.1);
            self.world[(rpos_y * self.area.width + rpos_x) as usize]
        }

        /// Restart
        pub fn restart(&mut self) {
            self.area = self.area_0.clone();
            self.world = self.world_0.clone();
            self.new_world = vec![0; self.area_0.width as usize * self.area_0.height as usize];
            self.round = 0;
        }

        /// Random
        pub fn random(&mut self) {
            let n = max(self.live.count_ones(), self.birth.count_ones());
            let r = n as f64 / 9f64;

            self.area_0 = self.area.clone();
            self.world_0 = vec![0; self.area_0.width as usize * self.area_0.height as usize];
            for y in 0..self.area_0.height {
                for x in 0..self.area_0.width {
                    let pos = (y * self.area_0.width + x) as usize;
                    self.world_0[pos] = if random::<f64>() < r { 1 } else { 0 };
                }
            }

            self.world = self.world_0.clone();
            self.round = 0;
        }

        /// Run a turn
        pub fn turn(&mut self) {
            self.round += 1;

            for y in 0..self.area.height {
                for x in 0..self.area.width {
                    let pos = (y * self.area.width + x) as usize;

                    let n = self.toroid((x, y), (-1, -1))
                        + self.toroid((x, y), (0, -1))
                        + self.toroid((x, y), (1, -1))
                        + self.toroid((x, y), (-1, 0))
                        + self.toroid((x, y), (1, 0))
                        + self.toroid((x, y), (-1, 1))
                        + self.toroid((x, y), (0, 1))
                        + self.toroid((x, y), (1, 1));

                    let nb = 1u16 << n;

                    if (self.live & nb) != 0 && self.world[pos] == 1 {
                        self.new_world[pos] = 1;
                    } else if (self.birth & nb) != 0 && self.world[pos] == 0 {
                        self.new_world[pos] = 1;
                    } else {
                        self.new_world[pos] = 0;
                    }
                }
            }

            mem::swap(&mut self.world, &mut self.new_world);
        }

        pub fn round(&self) -> u32 {
            self.round
        }
    }

    fn color(s: &str, theme: &DarkTheme) -> Result<Style, Error> {
        let s = s.trim().to_lowercase();
        let s = s.as_str();
        let r = match s {
            "black(0)" => theme.black(0),
            "black(1)" => theme.black(1),
            "black(2)" => theme.black(2),
            "black(3)" => theme.black(3),
            "white(0)" => theme.white(0),
            "white(1)" => theme.white(1),
            "white(2)" => theme.white(2),
            "white(3)" => theme.white(3),
            "gray(0)" => theme.gray(0),
            "gray(1)" => theme.gray(1),
            "gray(2)" => theme.gray(2),
            "gray(3)" => theme.gray(3),
            "red(0)" => theme.red(0),
            "red(1)" => theme.red(1),
            "red(2)" => theme.red(2),
            "red(3)" => theme.red(3),
            "orange(0)" => theme.orange(0),
            "orange(1)" => theme.orange(1),
            "orange(2)" => theme.orange(2),
            "orange(3)" => theme.orange(3),
            "yellow(0)" => theme.yellow(0),
            "yellow(1)" => theme.yellow(1),
            "yellow(2)" => theme.yellow(2),
            "yellow(3)" => theme.yellow(3),
            "limegreen(0)" => theme.limegreen(0),
            "limegreen(1)" => theme.limegreen(1),
            "limegreen(2)" => theme.limegreen(2),
            "limegreen(3)" => theme.limegreen(3),
            "green(0)" => theme.green(0),
            "green(1)" => theme.green(1),
            "green(2)" => theme.green(2),
            "green(3)" => theme.green(3),
            "bluegreen(0)" => theme.bluegreen(0),
            "bluegreen(1)" => theme.bluegreen(1),
            "bluegreen(2)" => theme.bluegreen(2),
            "bluegreen(3)" => theme.bluegreen(3),
            "cyan(0)" => theme.cyan(0),
            "cyan(1)" => theme.cyan(1),
            "cyan(2)" => theme.cyan(2),
            "cyan(3)" => theme.cyan(3),
            "blue(0)" => theme.blue(0),
            "blue(1)" => theme.blue(1),
            "blue(2)" => theme.blue(2),
            "blue(3)" => theme.blue(3),
            "deepblue(0)" => theme.deepblue(0),
            "deepblue(1)" => theme.deepblue(1),
            "deepblue(2)" => theme.deepblue(2),
            "deepblue(3)" => theme.deepblue(3),
            "purple(0)" => theme.purple(0),
            "purple(1)" => theme.purple(1),
            "purple(2)" => theme.purple(2),
            "purple(3)" => theme.purple(3),
            "magenta(0)" => theme.magenta(0),
            "magenta(1)" => theme.magenta(1),
            "magenta(2)" => theme.magenta(2),
            "magenta(3)" => theme.magenta(3),
            "redpink(0)" => theme.redpink(0),
            "redpink(1)" => theme.redpink(1),
            "redpink(2)" => theme.redpink(2),
            "redpink(3)" => theme.redpink(3),
            "primary(0)" => theme.primary(0),
            "primary(1)" => theme.primary(1),
            "primary(2)" => theme.primary(2),
            "primary(3)" => theme.primary(3),
            "secondary(0)" => theme.primary(0),
            "secondary(1)" => theme.primary(1),
            "secondary(2)" => theme.primary(2),
            "secondary(3)" => theme.primary(3),
            "black" => Style::new().on_black(),
            "red" => Style::new().on_red(),
            "green" => Style::new().on_green(),
            "yellow" => Style::new().on_yellow(),
            "blue" => Style::new().on_blue(),
            "magenta" => Style::new().on_magenta(),
            "cyan" => Style::new().on_cyan(),
            "gray" => Style::new().on_gray(),
            "dark gray" => Style::new().on_dark_gray(),
            "light red" => Style::new().on_light_red(),
            "light green" => Style::new().on_light_green(),
            "light yellow" => Style::new().on_light_yellow(),
            "light blue" => Style::new().on_light_blue(),
            "light magenta" => Style::new().on_light_magenta(),
            "light cyan" => Style::new().on_light_cyan(),
            "white" => Style::new().on_white(),
            _ => {
                if s.len() == 6 && !s.contains(' ') {
                    debug!("16 color {}", s);
                    if let Ok(mut c) = u32::from_str_radix(s, 16) {
                        let b = c % 256;
                        c = c / 256;
                        let g = c % 256;
                        c = c / 256;
                        let r = c % 256;
                        Style::new().bg(Color::Rgb(r as u8, g as u8, b as u8))
                    } else {
                        return Err(anyhow!("invalid color {}", s));
                    }
                } else {
                    let r;
                    let g;
                    let b;

                    debug!("triple color {}", s);

                    let mut si = s.split(" ");
                    if let Some(v) = si.next() {
                        r = if let Ok(w) = v.parse::<u32>() {
                            w
                        } else {
                            return Err(anyhow!("invalid color {}", s));
                        }
                    } else {
                        return Err(anyhow!("invalid color {}", s));
                    }
                    if let Some(v) = si.next() {
                        g = if let Ok(w) = v.parse::<u32>() {
                            w
                        } else {
                            return Err(anyhow!("invalid color {}", s));
                        }
                    } else {
                        return Err(anyhow!("invalid color {}", s));
                    }
                    if let Some(v) = si.next() {
                        b = if let Ok(w) = v.parse::<u32>() {
                            w
                        } else {
                            return Err(anyhow!("invalid color {}", s));
                        }
                    } else {
                        return Err(anyhow!("invalid color {}", s));
                    }
                    Style::new().bg(Color::Rgb(r as u8, g as u8, b as u8))
                }
            }
        };

        Ok(r)
    }

    fn rule(s: &str) -> (u16, u16) {
        let mut state = 0;
        let mut live = 0;
        let mut birth = 0;
        for c in s.chars() {
            if c.is_ascii_digit() {
                let d = c as u8 - b'0';
                if state == 0 {
                    live += 1u16 << d;
                } else {
                    birth += 1u16 << d;
                }
            } else if c == '/' {
                state = 1;
            } else {
                // noop
            }
        }

        (live, birth)
    }

    pub fn load_life(file: &Path, theme: &DarkTheme) -> Result<LifeGameState, Error> {
        let mut ini = Ini::new();
        if let Err(e) = ini.load(file) {
            return Err(anyhow!("{}", e));
        }

        let name = file.file_stem().expect("name").to_string_lossy();
        let rule = rule(&ini.get("life", "rules").unwrap_or("23/3".into()));
        let one = ini.get("life", "one").unwrap_or("1Xx".into());
        let one_color = color(
            &ini.get("life", "one.color").unwrap_or("cccccc".into()),
            theme,
        )?;
        let zero_color = color(
            &ini.get("life", "zero.color").unwrap_or("000000".into()),
            theme,
        )?;

        let mut height = 0;
        let mut width = 0;
        loop {
            if let Some(v) = ini.get("data", &format!("{}", height)) {
                let v = v.trim_matches('"').trim_matches('\'');
                width = max(width, v.chars().count() as u16);
            } else {
                break;
            }
            height += 1;
        }
        let mut world_0 = vec![0; width as usize * height as usize];
        for row in 0..height {
            if let Some(d) = ini.get("data", &format!("{}", row)) {
                let d = d.trim_matches('"').trim_matches('\'');
                for (col, c) in d.chars().enumerate() {
                    if col >= width as usize {
                        break;
                    }

                    let pos = row as usize * width as usize + col;
                    world_0[pos] = if one.contains(c) { 1 } else { 0 };
                }
            }
        }

        let mut game = LifeGameState {
            style1: one_color,
            style0: zero_color,
            name: name.to_string(),
            area_0: Rect::new(0, 0, width, height),
            world_0,
            area: Rect::new(0, 0, width, height),
            world: vec![0; width as usize * height as usize],
            new_world: vec![0; width as usize * height as usize],
            live: rule.0,
            birth: rule.1,
            round: 0,
        };

        game.restart();

        Ok(game)
    }
}

fn setup_logging() -> Result<(), Error> {
    _ = fs::remove_file("log.log");
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
        .apply()?;
    Ok(())
}
