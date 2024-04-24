use crate::app::Example;
use crate::data::ExData;
use crate::state::ExState;
use rat_salsa::{run_tui, RunConfig};
use std::time::SystemTime;

type Control = rat_salsa::ControlUI<ExAction, anyhow::Error>;

fn main() -> Result<(), anyhow::Error> {
    setup_logger()?;

    let mut data = ExData {
        datum: Default::default(),
    };
    let mut state = ExState::default();

    run_tui(
        &Example,
        &mut data,
        &mut state,
        RunConfig {
            n_threats: 1,
            log_timing: false,
        },
    )?;

    Ok(())
}

fn setup_logger() -> Result<(), anyhow::Error> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("example1.log")?)
        .apply()?;
    Ok(())
}

#[derive(Debug)]
pub enum ExAction {}

pub mod data {
    use chrono::NaiveDate;

    #[derive(Debug)]
    pub struct ExData {
        pub datum: NaiveDate,
    }
}

pub mod state {
    use crate::theme::{Theme, ONEDARK};
    use rat_salsa::widget::button::ButtonStyle;
    use rat_salsa::widget::input::{TextInputState, TextInputStyle};
    use rat_salsa::widget::mask_input::{MaskedInputState, MaskedInputStyle};
    use rat_salsa::widget::message::{StatusDialogState, StatusDialogStyle, StatusLineState};
    use rat_salsa::{Repaint, Timers};
    use ratatui::prelude::{Color, Stylize};
    use ratatui::style::Style;

    #[derive(Debug, Default)]
    pub struct ExState {
        pub g: GeneralState,
        pub repaint: Repaint,
        pub timers: Timers,
        pub mask0: Mask0,
    }

    #[derive(Debug)]
    pub struct GeneralState {
        pub theme: &'static Theme,
        pub status: StatusLineState,
        pub error_dlg: StatusDialogState,
    }

    #[derive(Debug)]
    pub struct Mask0 {
        pub input_0: MaskedInputState,
        pub input_1: TextInputState,
        pub timer_1: usize,
        pub roll: usize,
    }

    impl Default for GeneralState {
        fn default() -> Self {
            Self {
                theme: &ONEDARK,
                status: Default::default(),
                error_dlg: Default::default(),
            }
        }
    }

    impl GeneralState {
        pub fn status_style(&self) -> Style {
            Style::default().fg(self.theme.white).bg(self.theme.one_bg3)
        }

        pub fn input_style(&self) -> TextInputStyle {
            TextInputStyle {
                style: Style::default().fg(self.theme.black).bg(self.theme.base05),
                focus: Style::default().fg(self.theme.black).bg(self.theme.green),
                select: Style::default().fg(self.theme.black).bg(self.theme.base0e),
                cursor: None,
                ..TextInputStyle::default()
            }
        }

        pub fn input_mask_style(&self) -> MaskedInputStyle {
            MaskedInputStyle {
                style: Style::default().fg(self.theme.black).bg(self.theme.base05),
                focus: Style::default().fg(self.theme.black).bg(self.theme.green),
                select: Style::default().fg(self.theme.black).bg(self.theme.base0e),
                cursor: None,
                invalid: Some(Style::default().fg(Color::White).bg(Color::Red)),
                ..MaskedInputStyle::default()
            }
        }

        pub fn button_style(&self) -> ButtonStyle {
            ButtonStyle {
                style: Style::default()
                    .fg(self.theme.black)
                    .bg(self.theme.purple)
                    .bold(),
                focus: Style::default()
                    .fg(self.theme.black)
                    .bg(self.theme.green)
                    .bold(),
                armed: Style::default()
                    .fg(self.theme.black)
                    .bg(self.theme.orange)
                    .bold(),
            }
        }
        pub fn status_dialog_style(&self) -> StatusDialogStyle {
            StatusDialogStyle {
                style: self.status_style(),
                button: self.button_style(),
            }
        }
    }

    impl Default for Mask0 {
        fn default() -> Self {
            let mut s = Self {
                input_0: Default::default(),
                input_1: Default::default(),
                timer_1: 0,
                roll: 0,
            };
            s.input_0.focus.set();
            s.input_0.set_mask("99.99.9999").expect("mask");
            s.input_0.set_display_mask("TT.MM.YYYY");
            s.input_0.select_all();
            s
        }
    }
}

// -----------------------------------------------------------------------

pub mod app {
    use crate::data::ExData;
    use crate::state::{ExState, Mask0};
    use crate::{Control, ExAction};
    use chrono::NaiveDate;
    use crossbeam::channel::Sender;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    #[allow(unused_imports)]
    use log::debug;
    use rat_salsa::layout::{layout_edit, EditConstraint};
    use rat_salsa::widget::input::TextInput;
    use rat_salsa::widget::mask_input::MaskedInput;
    use rat_salsa::widget::message::{StatusDialog, StatusLine};
    use rat_salsa::{
        check_break, on_lost, tr, ControlUI, HasValidFlag, Timed, TimerDef, Timers, TuiApp,
    };
    use rat_salsa::{DefaultKeys, HandleCrossterm, RenderFrameWidget, Repaint};
    use rat_salsa::{Focus, RepaintEvent};
    use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
    use ratatui::text::Span;
    use ratatui::Frame;
    use std::time::Duration;

    #[derive(Debug)]
    pub struct Example;

    impl TuiApp for Example {
        type Data = ExData;
        type State = ExState;
        type Action = ExAction;
        type Error = anyhow::Error;

        fn get_repaint<'a, 'b>(&'a self, uistate: &'b Self::State) -> Option<&'b Repaint> {
            Some(&uistate.repaint)
        }

        fn get_timers<'a, 'b>(&'a self, uistate: &'b Self::State) -> Option<&'b Timers> {
            Some(&uistate.timers)
        }

        fn init(
            &self,
            _data: &mut Self::Data,
            _uistate: &mut Self::State,
            _send: &Sender<Self::Action>,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn repaint(
            &self,
            event: RepaintEvent,
            frame: &mut Frame<'_>,
            data: &mut Self::Data,
            uistate: &mut Self::State,
        ) -> ControlUI<Self::Action, Self::Error> {
            //
            let area = frame.size();

            let layout = Layout::new(
                Direction::Vertical,
                [
                    Constraint::Fill(1),
                    Constraint::Length(2),
                    Constraint::Length(1),
                ],
            )
            .split(area);

            tr!(repaint_mask0(event, frame, layout[0], data, uistate), _);

            let statusdialog = StatusDialog::new().style(uistate.g.status_dialog_style());
            let mut err_dialog = &mut uistate.g.error_dlg;
            if err_dialog.active {
                frame.render_stateful_widget(statusdialog, layout[0], &mut err_dialog);
            }
            let status = StatusLine::new().style(uistate.g.status_style());
            let mut msg = &mut uistate.g.status;
            frame.render_stateful_widget(status, layout[2], &mut msg);

            Control::Continue
        }

        fn handle_timer(
            &self,
            _event: Timed,
            _data: &mut Self::Data,
            _uistate: &mut Self::State,
        ) -> ControlUI<Self::Action, Self::Error> {
            Control::Continue
        }

        fn handle_event(&self, evt: Event, data: &mut ExData, uistate: &mut ExState) -> Control {
            check_break!(match &evt {
                Event::Resize(_, _) => Control::Change,
                Event::Key(KeyEvent {
                    kind: KeyEventKind::Press,
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => Control::Break,
                _ => Control::Continue,
            });

            check_break!({
                let error_dlg = &mut uistate.g.error_dlg;
                if error_dlg.active {
                    error_dlg.handle(&evt, DefaultKeys)
                } else {
                    Control::Continue
                }
            });

            check_break!(handle_mask0(&evt, data, uistate));

            Control::Continue
        }

        fn run_action(
            &self,
            _action: Self::Action,
            _data: &mut Self::Data,
            _uistate: &mut Self::State,
            _send: &Sender<Self::Action>,
        ) -> Control {
            // match action {}
            Control::Continue
        }

        fn run_task(&self, _task: Self::Action, _send: &Sender<Control>) -> Control {
            // match task {}
            Control::Continue
        }

        fn report_error(
            &self,
            error: anyhow::Error,
            _data: &mut ExData,
            uistate: &mut ExState,
        ) -> Control {
            uistate.g.error_dlg.log(format!("{:?}", &*error).as_str());
            Control::Change
        }
    }

    fn repaint_mask0(
        event: RepaintEvent,
        frame: &mut Frame<'_>,
        area: Rect,
        data: &mut ExData,
        uistate: &mut ExState,
    ) -> Control {
        let work = area.inner(&Margin::new(5, 1));

        let work = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Length(25),
                Constraint::Length(25),
                Constraint::Length(25),
                Constraint::Length(25),
            ],
        )
        .split(work);

        let l_edit0 = layout_edit(
            work[0],
            &[
                EditConstraint::Label("Datum"),
                EditConstraint::Widget(15),
                EditConstraint::Label("Parsed"),
                EditConstraint::Widget(10),
                EditConstraint::Label("No spaces"),
                EditConstraint::Widget(10),
                EditConstraint::Label("Mask"),
                EditConstraint::Widget(10),
            ],
        );
        let l_edit1 = layout_edit(
            work[1],
            &[EditConstraint::Label("Text"), EditConstraint::Widget(15)],
        );

        let l_edit2 = layout_edit(
            work[2],
            &[EditConstraint::Label("Rolling banners are nice :-) ")],
        );

        let label_edit = Span::from("Datum");

        let edit = MaskedInput::default().style(uistate.g.input_mask_style());
        let label_parsed = Span::from("Parsed");
        let parsed = Span::from(data.datum.format("%d.%m.%Y").to_string());
        let label_compact = Span::from("No spaces");
        let compact = Span::from(uistate.mask0.input_0.compact_value());
        frame.render_widget(label_edit, l_edit0.label(0));
        frame.render_frame_widget(edit, l_edit0.widget(0), &mut uistate.mask0.input_0);
        frame.render_widget(label_parsed, l_edit0.label(1));
        frame.render_widget(parsed, l_edit0.widget(1));
        frame.render_widget(label_compact, l_edit0.label(2));
        frame.render_widget(compact, l_edit0.widget(2));
        let label_mask = Span::from("Mask");
        frame.render_widget(label_mask, l_edit0.label(3));
        let mask = Span::from(uistate.mask0.input_0.mask());
        frame.render_widget(mask, l_edit0.widget(3));

        let label_edit = Span::from("Text");
        let edit = TextInput::default().style(uistate.g.input_style());
        frame.render_widget(label_edit, l_edit1.label(0));
        frame.render_frame_widget(edit, l_edit1.widget(0), &mut uistate.mask0.input_1);

        if uistate.mask0.timer_1 == 0 {
            uistate.mask0.timer_1 = uistate.timers.add(
                TimerDef::new()
                    .repeat(usize::MAX)
                    .repaint(true)
                    .timer(Duration::from_millis(500)),
            );
        }

        if let RepaintEvent::Timer(t) = event {
            if t.tag == uistate.mask0.timer_1 {
                uistate.mask0.roll = t.counter % 29;
            }
        }
        let txt_roll = "Rolling banners are nice :-) ";
        let (txt_roll1, txt_roll2) = txt_roll.split_at(uistate.mask0.roll);
        let label_roll = Span::from(format!("{}{}", txt_roll2, txt_roll1).to_string());
        frame.render_widget(label_roll, l_edit2.label(0));

        Control::Continue
    }

    #[derive(Debug)]
    pub struct ExKeys;

    impl<'a> HandleCrossterm<ControlUI<bool, ()>, ExKeys> for Focus<'a> {
        fn handle(&mut self, event: &Event, _: ExKeys) -> ControlUI<bool, ()> {
            use crossterm::event::*;
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::F(2),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    if self.next() {
                        ControlUI::Change
                    } else {
                        ControlUI::Continue
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::F(3),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    if self.prev() {
                        ControlUI::Change
                    } else {
                        ControlUI::Continue
                    }
                }
                _ => ControlUI::Continue,
            }
        }
    }

    fn focus_mask0(state: &Mask0) -> Focus<'_> {
        Focus::new([
            (&state.input_0.focus, state.input_0.area),
            (&state.input_1.focus, state.input_1.area),
        ])
    }

    fn handle_mask0(evt: &Event, data: &mut ExData, uistate: &mut ExState) -> Control {
        let state = &mut uistate.mask0;

        focus_mask0(state)
            .handle(evt, DefaultKeys)
            .and_do(|_| uistate.repaint.set());

        // alternate focus keys
        focus_mask0(state)
            .handle(evt, ExKeys)
            .and_do(|_| uistate.repaint.set());

        // validation and reformat on focus lost.
        on_lost!(
            state.input_0 => {
                if let Ok(d) = NaiveDate::parse_from_str(state.input_0.compact_value().as_str(), "%d.%m.%Y") {
                    data.datum = d;
                    let v = data.datum.format("%d.%m.%Y").to_string();
                    state.input_0.set_value(v);
                    state.input_0.select_all();
                } else {
                    data.datum = NaiveDate::default();
                }
            },
            state.input_1 => {}
        );

        check_break!({
            state.input_0.handle(evt, DefaultKeys).on_change(|| {
                // quick validation for every change
                let str = state.input_0.compact_value();
                let val = NaiveDate::parse_from_str(str.as_str(), "%d.%m.%Y");
                state.input_0.set_valid_from(val);
                Control::Change
            })
        });
        check_break!(state.input_1.handle(evt, DefaultKeys));

        Control::Continue
    }
}

pub mod theme {
    use ratatui::style::Color;

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
}
