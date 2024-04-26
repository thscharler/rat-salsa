#![allow(unused_variables)]

use crossbeam::channel::Sender;
use crossterm::event::Event;
use rat_salsa::widget::button::ButtonStyle;
use rat_salsa::widget::input::TextInputStyle;
use rat_salsa::widget::mask_input::MaskedInputStyle;
use rat_salsa::widget::menuline::{MenuLine, MenuLineState, MenuStyle};
use rat_salsa::widget::message::{
    StatusDialog, StatusDialogState, StatusDialogStyle, StatusLine, StatusLineState,
};
use rat_salsa::{
    check_break, run_tui, tr, ControlUI, DefaultKeys, HandleCrossterm, Repaint, RepaintEvent,
    RunConfig, Timed, Timers, TuiApp,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Style};
use ratatui::style::Stylize;
use ratatui::Frame;
use std::time::SystemTime;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = MinimalData::default();
    let mut state = MinimalState::default();

    // Start the event loop
    run_tui(
        &MinimalApp, // application logic
        &mut data,
        &mut state,
        RunConfig {
            n_threats: 1, // background threads
            log_timing: true,
            ..RunConfig::default()
        },
    )?;

    Ok(())
}

// -----------------------------------------------------------------------

// type alias for the control-flow enum for the app.
type Control = ControlUI<MinimalAction, anyhow::Error>;

// plain data
#[derive(Debug, Default)]
pub struct MinimalData {}

// triggered actions and background actions
#[derive(Debug)]
pub enum MinimalAction {}

// ui state
#[derive(Debug, Default)]
pub struct MinimalState {
    pub g: GeneralState, // collection of global stuff.
    pub mask0: Mask0,    // state of widgets
}

#[derive(Debug)]
pub struct GeneralState {
    pub theme: &'static Theme,        // theme colors and styles
    pub repaint: Repaint,             // extra repaint flag.
    pub timers: Timers,               // generates timer events
    pub status: StatusLineState,      // status line
    pub error_dlg: StatusDialogState, // status dialog
}

#[derive(Debug)]
pub struct Mask0 {
    pub menu: MenuLineState<u16>,
}

impl Default for GeneralState {
    fn default() -> Self {
        Self {
            theme: &ONEDARK,
            repaint: Default::default(),
            timers: Default::default(),
            status: Default::default(),
            error_dlg: Default::default(),
        }
    }
}

impl Default for Mask0 {
    fn default() -> Self {
        let s = Self {
            menu: Default::default(),
        };
        s.menu.focus.set();
        s
    }
}

// -----------------------------------------------------------------------

// Collection of application functions.
// It needs no state of its own, everything is passed as arguments.
#[derive(Debug)]
pub struct MinimalApp;

// utility struct for the base layout of the app.
#[derive(Debug, Clone, Copy)]
pub struct MinimalAppLayout {
    area: Rect,
    menu: Rect,
    status: Rect,
}

impl TuiApp for MinimalApp {
    // define app data, state, action and error types.
    type Data = MinimalData;
    type State = MinimalState;
    type Action = MinimalAction;
    type Error = anyhow::Error;

    // the extra repaint flag is optional.
    fn get_repaint<'b>(&self, uistate: &'b Self::State) -> Option<&'b Repaint> {
        Some(&uistate.g.repaint)
    }

    // timers are optional.
    fn get_timers<'b>(&self, uistate: &'b Self::State) -> Option<&'b Timers> {
        Some(&uistate.g.timers)
    }

    fn init(
        &self,
        data: &mut Self::Data,
        uistate: &mut Self::State,
        send: &Sender<Self::Action>,
    ) -> Result<(), Self::Error> {
        // TODO: init before event-loop. maybe start some workers.
        Ok(())
    }

    // paint one frame.
    fn repaint(
        &self,
        event: RepaintEvent, // gives the cause for the repaint
        frame: &mut Frame<'_>,
        data: &mut Self::Data,
        uistate: &mut Self::State,
    ) -> Control {
        let area = frame.size();

        let layout = {
            let r = Layout::new(
                Direction::Vertical,
                [
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ],
            )
            .split(area);

            MinimalAppLayout {
                area: r[0],
                menu: r[1],
                status: r[2],
            }
        };

        // call out for more ui
        tr!(repaint_mask0(&event, frame, layout, data, uistate), _);

        // dialog use a flag for control
        if uistate.g.error_dlg.active {
            let err = StatusDialog::new().style(uistate.g.theme.status_dialog_style());
            frame.render_stateful_widget(err, layout.area, &mut uistate.g.error_dlg);
        }

        let status = StatusLine::new().style(uistate.g.theme.status_style());
        frame.render_stateful_widget(status, layout.status, &mut uistate.g.status);

        Control::Continue
    }

    // called for application timers.
    fn handle_timer(
        &self,
        event: Timed,
        data: &mut Self::Data,
        uistate: &mut Self::State,
    ) -> Control {
        // TODO: timer
        Control::Continue
    }

    // called for events.
    fn handle_event(
        &self,
        event: Event,
        data: &mut Self::Data,
        uistate: &mut Self::State,
    ) -> Control {
        use crossterm::event::*;

        // some global event handling.
        check_break!(match &event {
            Event::Resize(_, _) => {
                // triggers a repaint
                Control::Change
            }
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                // break the control loop
                Control::Break
            }
            _ => Control::Continue,
        });

        // break event processing and repeat the loop if the event is handled by
        // this component. everything but ControlUI::Continue is considered processing the
        // event.
        check_break!({
            if uistate.g.error_dlg.active {
                uistate.g.error_dlg.handle(&event, DefaultKeys)
            } else {
                Control::Continue
            }
        });

        // hand out processing
        check_break!(handle_mask0(&event, data, uistate));

        Control::Continue
    }

    // run some activity
    fn run_action(
        &self,
        action: Self::Action,
        data: &mut Self::Data,
        uistate: &mut Self::State,
        send: &Sender<Self::Action>,
    ) -> Control {
        // TODO: actions
        Control::Continue
    }

    // run as a background task. the resulting ControlUI is sent back to the eventloop
    // for further processing.
    fn run_task(
        &self,
        task: Self::Action,
        send: &Sender<Control>, // Access to the back channel. Can send more results from the
                                // background process.
    ) -> Control {
        // TODO: tasks
        Control::Continue
    }

    // any error ends here.
    fn report_error(
        &self,
        error: Self::Error,
        data: &mut Self::Data,
        uistate: &mut Self::State,
    ) -> Control {
        uistate.g.error_dlg.log(format!("{:?}", &*error).as_str());
        Control::Change
    }
}

// more painting ...
fn repaint_mask0(
    event: &RepaintEvent,
    frame: &mut Frame<'_>,
    layout: MinimalAppLayout,
    data: &mut MinimalData,
    uistate: &mut MinimalState,
) -> Control {
    // TODO: repaint_mask

    // paint the menu-line
    let menu = MenuLine::new()
        .style(uistate.g.theme.menu_style())
        .add("_Quit", 0u16); // menu item, _ denotes a shortcut.
                             // 0u16 is the item action. available as type parameter.
    frame.render_stateful_widget(menu, layout.menu, &mut uistate.mask0.menu);

    Control::Continue
}

// more event processing ...
fn handle_mask0(event: &Event, data: &mut MinimalData, uistate: &mut MinimalState) -> Control {
    let mask0 = &mut uistate.mask0;

    // TODO: handle_mask

    // process events. and_then calls the continuation if the state is ControlUI::Action.
    // there are more of those:
    //      - and_do: returns something else but a ControlUI
    //      - or_else: run on ControlUI::Continue
    //      - on_change/on_no_change: run on ControlUI::Change/ControlUI::NoChange
    //      ...
    // check_break returns early if the result differs from ControlUI::Continue
    check_break!(mask0
        .menu
        .handle(event, DefaultKeys)
        .on_action(|a| match a {
            0 => {
                Control::Break
            }
            _ => Control::Continue,
        }));

    Control::Continue
}

// -----------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct Theme {
    pub name: &'static str,
    pub dark_theme: bool,

    pub white: Color,
    pub darker_black: Color,
    pub black: Color,
    pub black2: Color,
    pub one_bg: Color,
    pub one_bg2: Color,
    pub one_bg3: Color,
    pub grey: Color,
    pub grey_fg: Color,
    pub grey_fg2: Color,
    pub light_grey: Color,
    pub red: Color,
    pub baby_pink: Color,
    pub pink: Color,
    pub line: Color,
    pub green: Color,
    pub vibrant_green: Color,
    pub nord_blue: Color,
    pub blue: Color,
    pub yellow: Color,
    pub sun: Color,
    pub purple: Color,
    pub dark_purple: Color,
    pub teal: Color,
    pub orange: Color,
    pub cyan: Color,
    pub statusline_bg: Color,
    pub lightbg: Color,
    pub pmenu_bg: Color,
    pub folder_bg: Color,

    pub base00: Color,
    pub base01: Color,
    pub base02: Color,
    pub base03: Color,
    pub base04: Color,
    pub base05: Color,
    pub base06: Color,
    pub base07: Color,
    pub base08: Color,
    pub base09: Color,
    pub base0a: Color,
    pub base0b: Color,
    pub base0c: Color,
    pub base0d: Color,
    pub base0e: Color,
    pub base0f: Color,
}

impl Theme {
    pub fn status_style(&self) -> Style {
        Style::default().fg(self.white).bg(self.one_bg3)
    }

    pub fn input_style(&self) -> TextInputStyle {
        TextInputStyle {
            style: Style::default().fg(self.black).bg(self.base05),
            focus: Style::default().fg(self.black).bg(self.green),
            select: Style::default().fg(self.black).bg(self.base0e),
            cursor: None,
            ..TextInputStyle::default()
        }
    }

    pub fn input_mask_style(&self) -> MaskedInputStyle {
        MaskedInputStyle {
            style: Style::default().fg(self.black).bg(self.base05),
            focus: Style::default().fg(self.black).bg(self.green),
            select: Style::default().fg(self.black).bg(self.base0e),
            cursor: None,
            invalid: Some(Style::default().fg(Color::White).bg(Color::Red)),
            ..MaskedInputStyle::default()
        }
    }

    pub fn button_style(&self) -> ButtonStyle {
        ButtonStyle {
            style: Style::default().fg(self.black).bg(self.purple).bold(),
            focus: Style::default().fg(self.black).bg(self.green).bold(),
            armed: Style::default().fg(self.black).bg(self.orange).bold(),
        }
    }

    pub fn status_dialog_style(&self) -> StatusDialogStyle {
        StatusDialogStyle {
            style: self.status_style(),
            button: self.button_style(),
        }
    }

    pub fn menu_style(&self) -> MenuStyle {
        MenuStyle {
            style: Style::default().fg(self.white).bg(self.one_bg3).bold(),
            title: Style::default().fg(self.black).bg(self.base0a).bold(),
            select: Style::default().fg(self.black).bg(self.base0e).bold(),
            focus: Style::default().fg(self.black).bg(self.green).bold(),
        }
    }
}

pub static ONEDARK: Theme = Theme {
    name: "onedark",
    dark_theme: false,

    white: Color::from_u32(0xabb2bf),
    darker_black: Color::from_u32(0x1b1f27),
    black: Color::from_u32(0x1e222a), //  nvim bg
    black2: Color::from_u32(0x252931),
    one_bg: Color::from_u32(0x282c34), // real bg of onedark
    one_bg2: Color::from_u32(0x353b45),
    one_bg3: Color::from_u32(0x373b43),
    grey: Color::from_u32(0x42464e),
    grey_fg: Color::from_u32(0x565c64),
    grey_fg2: Color::from_u32(0x6f737b),
    light_grey: Color::from_u32(0x6f737b),
    red: Color::from_u32(0xe06c75),
    baby_pink: Color::from_u32(0xDE8C92),
    pink: Color::from_u32(0xff75a0),
    line: Color::from_u32(0x31353d), // for lines like vertsplit
    green: Color::from_u32(0x98c379),
    vibrant_green: Color::from_u32(0x7eca9c),
    nord_blue: Color::from_u32(0x81A1C1),
    blue: Color::from_u32(0x61afef),
    yellow: Color::from_u32(0xe7c787),
    sun: Color::from_u32(0xEBCB8B),
    purple: Color::from_u32(0xde98fd),
    dark_purple: Color::from_u32(0xc882e7),
    teal: Color::from_u32(0x519ABA),
    orange: Color::from_u32(0xfca2aa),
    cyan: Color::from_u32(0xa3b8ef),
    statusline_bg: Color::from_u32(0x22262e),
    lightbg: Color::from_u32(0x2d3139),
    pmenu_bg: Color::from_u32(0x61afef),
    folder_bg: Color::from_u32(0x61afef),

    base00: Color::from_u32(0x1e222a),
    base01: Color::from_u32(0x353b45),
    base02: Color::from_u32(0x3e4451),
    base03: Color::from_u32(0x545862),
    base04: Color::from_u32(0x565c64),
    base05: Color::from_u32(0xabb2bf),
    base06: Color::from_u32(0xb6bdca),
    base07: Color::from_u32(0xc8ccd4),
    base08: Color::from_u32(0xe06c75),
    base09: Color::from_u32(0xd19a66),
    base0a: Color::from_u32(0xe5c07b),
    base0b: Color::from_u32(0x98c379),
    base0c: Color::from_u32(0x56b6c2),
    base0d: Color::from_u32(0x61afef),
    base0e: Color::from_u32(0xc678dd),
    base0f: Color::from_u32(0xbe5046),
};

fn setup_logging() -> Result<(), anyhow::Error> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}]\n        {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("log.log")?)
        .apply()?;
    Ok(())
}
