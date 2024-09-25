use crate::app::{Scenery, SceneryState};
use crate::config::LifeConfig;
use crate::global::{GlobalState, PollTick};
use crate::message::LifeMsg;
use anyhow::Error;
use log::debug;
use rat_salsa::{run_tui, RunConfig};
use rat_theme::dark_theme::DarkTheme;
use rat_theme::scheme::IMPERIAL;
use std::fs;
use std::time::Duration;

type AppContext<'a> = rat_salsa::AppContext<'a, GlobalState, LifeMsg, Error>;
type RenderContext<'a> = rat_salsa::RenderContext<'a, GlobalState>;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let mut config = LifeConfig::default();

    // classic conway
    // (config.live, config.birth) = rule("23/3");
    // copy
    (config.live, config.birth) = rule("1357/1357");

    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = GlobalState::new(config, theme);

    let app = Scenery;
    let mut state = SceneryState::default();
    state.life.game = global::rat_state();

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollTick::new(
                Duration::from_secs(2),
                Duration::from_millis(500),
            ))
            .threads(1),
    )?;

    Ok(())
}

fn rule(rule: &str) -> (u16, u16) {
    let mut state = 0;
    let mut life = 0;
    let mut birth = 0;
    for c in rule.chars() {
        if c.is_ascii_digit() {
            let d = c as u8 - b'0';
            if state == 0 {
                life += 1u16 << d;
            } else {
                birth += 1u16 << d;
            }
        } else if c == '/' {
            state = 1;
        } else {
            // noop
        }
    }
    (life, birth)
}

/// Globally accessible data/state.
pub mod global {
    use crate::config::LifeConfig;
    use crate::life::LifeGameState;
    use rat_salsa::poll::PollEvents;
    use rat_salsa::{AppContext, AppState, Control};
    use rat_theme::dark_theme::DarkTheme;
    use rat_widget::msgdialog::MsgDialogState;
    use rat_widget::statusline::StatusLineState;
    use ratatui::layout::Rect;
    use std::fmt::Debug;
    use std::time::{Duration, SystemTime};

    pub fn block_state() -> LifeGameState {
        LifeGameState {
            area: Rect::new(0, 0, 6, 6),
            #[rustfmt::skip]
            world: vec![
                0,0,0,0,0,0,
                0,0,0,0,0,0,
                0,0,1,1,0,0,
                0,0,1,1,0,0,
                0,0,0,0,0,0,
                0,0,0,0,0,0,
            ],
            new_world: vec![0; 6 * 6],
        }
    }
    pub fn blink_state() -> LifeGameState {
        LifeGameState {
            area: Rect::new(0, 0, 6, 6),
            #[rustfmt::skip]
            world: vec![
                0,0,0,0,0,0,
                0,0,0,0,0,0,
                0,0,1,1,1,0,
                0,0,0,0,0,0,
                0,0,0,0,0,0,
                0,0,0,0,0,0,
            ],
            new_world: vec![0; 6 * 6],
        }
    }

    #[rustfmt::skip]
    pub fn rat_state() -> LifeGameState {
        LifeGameState {
            area: Rect::new(0,0,17,10),
            world: vec![
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
            new_world: vec![0; 16*16],
        }
    }

    #[derive(Debug)]
    pub struct GlobalState {
        pub cfg: LifeConfig,
        pub theme: DarkTheme,
        pub status: StatusLineState,
        pub error_dlg: MsgDialogState,
    }

    #[derive(Debug)]
    pub struct PollTick {
        tick: Duration,
        next: SystemTime,
    }

    impl PollTick {
        pub fn new(start: Duration, tick: Duration) -> Self {
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
                self.next += self.tick;
                state.tick(ctx)
            } else {
                Ok(Control::Continue)
            }
        }
    }

    impl GlobalState {
        pub fn new(cfg: LifeConfig, theme: DarkTheme) -> Self {
            Self {
                cfg,
                theme,
                status: Default::default(),
                error_dlg: Default::default(),
            }
        }
    }
}

/// Configuration.
pub mod config {
    #[derive(Debug, Default)]
    pub struct LifeConfig {
        pub live: u16,
        pub birth: u16,
    }
}

/// Application wide messages.
pub mod message {
    #[derive(Debug)]
    pub enum LifeMsg {
        Message(String),
    }
}

pub mod app {
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
    use ratatui::widgets::StatefulWidget;
    use std::time::{Duration, SystemTime};

    #[derive(Debug)]
    pub struct Scenery;

    #[derive(Debug, Default)]
    pub struct SceneryState {
        pub life: LifeState,
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

            Life.render(area, buf, &mut state.life, ctx)?;

            if ctx.g.error_dlg.active() {
                let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
                err.render(layout[0], buf, &mut ctx.g.error_dlg);
            }

            let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
            ctx.g.status.status(1, format!("R {:.0?}", el).to_string());

            let status_layout =
                Layout::horizontal([Constraint::Fill(61), Constraint::Fill(39)]).split(layout[1]);
            let status = StatusLine::new()
                .layout([
                    Constraint::Fill(1),
                    Constraint::Length(8),
                    Constraint::Length(8),
                ])
                .styles(ctx.g.theme.statusline_style());
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
            ctx.g.status.status(2, format!("T {:.0?}", el).to_string());

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
            ctx.g.status.status(2, format!("H {:.0?}", el).to_string());

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
            ctx.g.status.status(2, format!("A {:.0?}", el).to_string());

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
            ctx.g.status.status(2, format!("T {:.0?}", el).to_string());

            Ok(r)
        }
    }
}

pub mod life {
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
    use std::fmt::{Debug, Formatter};
    use std::mem;

    #[derive(Debug)]
    pub(crate) struct Life;

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

            LifeGame {
                style: ctx.g.theme.limegreen(2),
            }
            .render(r[0], buf, &mut state.game);

            let menu = MenuLine::new()
                .styles(ctx.g.theme.menu_style())
                .title("--(  )>")
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
            // TODO: handle_mask

            let f = ctx.focus_mut().handle(event, Regular);
            ctx.queue(f);

            try_flow!(match self.menu.handle(event, Regular) {
                MenuOutcome::Activated(0) => {
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
            self.game.turn(ctx.g.cfg.live, ctx.g.cfg.birth);
            Ok(Control::Changed)
        }
    }

    #[derive(Debug)]
    pub struct LifeGame {
        pub style: Style,
    }

    #[derive(Default)]
    pub struct LifeGameState {
        pub area: Rect,

        pub world: Vec<u8>,
        pub new_world: Vec<u8>,
    }

    impl Debug for LifeGameState {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            use std::fmt::Write;

            let mut w0 = String::new();
            for y in 0..self.area.height {
                _ = write!(w0, "    ");
                for x in 0..self.area.width {
                    _ = write!(w0, "{}", self.world[(y * self.area.width + x) as usize]);
                }
                _ = write!(w0, "\n");
            }

            _ = writeln!(f, "LifeGame");
            _ = writeln!(f, "    {:?}", self.area);
            _ = writeln!(f, "{}", w0);

            Ok(())
        }
    }

    impl StatefulWidget for LifeGame {
        type State = LifeGameState;

        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            state.set_area(area);

            for y in 0..area.height {
                for x in 0..area.width {
                    let pos = (y * area.width + x) as usize;
                    if state.world[pos] == 1 {
                        if let Some(cell) = buf.cell_mut((x + area.x, y + area.y)) {
                            cell.set_style(self.style);
                            cell.set_symbol("\u{b7}");
                        }
                    }
                }
            }
        }
    }

    impl LifeGameState {
        pub fn set_area(&mut self, area: Rect) {
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

        /// Run a turn
        pub fn turn(&mut self, live: u16, birth: u16) {
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
                    if (birth & nb) != 0 {
                        self.new_world[pos] = 1;
                    } else if (live & nb) != 0 {
                        self.new_world[pos] = self.world[pos];
                    } else {
                        self.new_world[pos] = 0;
                    }
                }
            }

            mem::swap(&mut self.world, &mut self.new_world);
            // debug!("{:?}", self);
        }
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
