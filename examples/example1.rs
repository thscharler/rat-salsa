use crate::app::Example;
use crate::data::ExData;
use crate::state::ExState;
use rat_salsa::{run_tui, ControlUI};
use std::time::SystemTime;

type Control = ControlUI<ExAction, anyhow::Error>;

fn main() -> Result<(), anyhow::Error> {
    setup_logger()?;

    let mut data = ExData {
        datum: Default::default(),
    };
    let mut state = ExState::default();

    run_tui(&Example, &mut data, &mut state, 1)?;

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
    use rat_salsa::button::ButtonStyle;
    use rat_salsa::input::{InputState, InputStyle};
    use rat_salsa::mask_input::{InputMaskState, InputMaskStyle};
    use rat_salsa::message::{StatusDialogState, StatusDialogStyle, StatusLineState};
    use ratatui::prelude::{Color, Stylize};
    use ratatui::style::Style;

    #[derive(Debug)]
    pub struct ExState {
        pub g: GeneralState,

        pub input_0: InputMaskState,
        pub input_1: InputState,
    }

    impl Default for ExState {
        fn default() -> Self {
            let mut s = Self {
                g: Default::default(),
                input_0: Default::default(),
                input_1: Default::default(),
            };
            s.input_0.focus.set();
            s.input_0.set_mask("99.99.9999");
            s.input_0.set_display_mask("TT.MM.YYYY");
            s.input_0.select_all();

            s
        }
    }

    #[derive(Debug)]
    pub struct GeneralState {
        pub theme: &'static Theme,
        pub status: StatusLineState,
        pub error_dlg: StatusDialogState,
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

        pub fn input_style(&self) -> InputStyle {
            InputStyle {
                style: Style::default().fg(self.theme.black).bg(self.theme.base05),
                focus: Style::default().fg(self.theme.black).bg(self.theme.green),
                select: Style::default().fg(self.theme.black).bg(self.theme.base0e),
                cursor: None,
                ..InputStyle::default()
            }
        }

        pub fn input_mask_style(&self) -> InputMaskStyle {
            InputMaskStyle {
                style: Style::default().fg(self.theme.black).bg(self.theme.base05),
                focus: Style::default().fg(self.theme.black).bg(self.theme.green),
                select: Style::default().fg(self.theme.black).bg(self.theme.base0e),
                cursor: None,
                invalid: Some(Style::default().fg(Color::White).bg(Color::Red)),
                ..InputMaskStyle::default()
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
}

// -----------------------------------------------------------------------

pub mod app {
    use crate::data::ExData;
    use crate::state::ExState;
    use crate::{Control, ExAction};
    use chrono::NaiveDate;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    #[allow(unused_imports)]
    use log::debug;
    use rat_salsa::focus::Focus;
    use rat_salsa::input::Input;
    use rat_salsa::layout::{layout_edit, EditConstraint};
    use rat_salsa::mask_input::InputMask;
    use rat_salsa::message::{StatusDialog, StatusLine};
    use rat_salsa::{
        cut, validate, yeet, HandleEvent, RenderFrameWidget, TaskSender, ThreadPool, TuiApp,
    };
    use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
    use ratatui::text::Span;
    use ratatui::Frame;

    #[derive(Debug)]
    pub struct Example;

    impl TuiApp for Example {
        type Data = ExData;
        type State = ExState;
        type Task = ExAction;
        type Action = ExAction;
        type Error = anyhow::Error;

        fn repaint(
            &self,
            frame: &mut Frame<'_>,
            data: &mut ExData,
            uistate: &mut ExState,
        ) -> Control {
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

            _ = yeet!(repaint_mask0(frame, layout[0], data, uistate));

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

        fn handle_event(&self, evt: Event, data: &mut ExData, uistate: &mut ExState) -> Control {
            cut!(match &evt {
                Event::Resize(_, _) => Control::Changed,
                Event::Key(KeyEvent {
                    kind: KeyEventKind::Press,
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => Control::Break,
                _ => Control::Continue,
            });

            cut!({
                let error_dlg = &mut uistate.g.error_dlg;
                if error_dlg.active {
                    error_dlg.handle(&evt)
                } else {
                    Control::Continue
                }
            });

            cut!(handle_mask0(&evt, data, uistate));

            Control::Continue
        }

        fn run_action(
            &self,
            _action: Self::Action,
            _data: &mut Self::Data,
            _uistate: &mut Self::State,
        ) -> Control {
            // match action {}

            Control::Continue
        }

        fn start_task(
            &self,
            action: ExAction,
            _data: &ExData,
            _uistate: &ExState,
            worker: &ThreadPool<Self>,
        ) -> Control {
            worker.send(action)
        }

        fn run_task(&self, _task: Self::Task, _send: &TaskSender<Self>) -> Control {
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
            Control::Changed
        }
    }

    fn repaint_mask0(
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
            ],
        )
        .split(work);

        let l_edit0 = layout_edit(
            work[0],
            [
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
            [EditConstraint::Label("Text"), EditConstraint::Widget(15)],
        );

        let label_edit = Span::from("Datum");

        let edit = InputMask::default().style(uistate.g.input_mask_style());
        let label_parsed = Span::from("Parsed");
        let parsed = Span::from(data.datum.format("%d.%m.%Y").to_string());
        let label_compact = Span::from("No spaces");
        let compact = Span::from(uistate.input_0.compact_value());
        frame.render_widget(label_edit, l_edit0.label[0]);
        frame.render_frame_widget(edit, l_edit0.widget[0], &mut uistate.input_0);
        frame.render_widget(label_parsed, l_edit0.label[1]);
        frame.render_widget(parsed, l_edit0.widget[1]);
        frame.render_widget(label_compact, l_edit0.label[2]);
        frame.render_widget(compact, l_edit0.widget[2]);
        let label_mask = Span::from("Mask");
        frame.render_widget(label_mask, l_edit0.label[3]);
        let mask = Span::from(uistate.input_0.mask());
        frame.render_widget(mask, l_edit0.widget[3]);

        let label_edit = Span::from("Text");
        let edit = Input::default().style(uistate.g.input_style());
        frame.render_widget(label_edit, l_edit1.label[0]);
        frame.render_frame_widget(edit, l_edit1.widget[0], &mut uistate.input_1);

        Control::Continue
    }

    fn focus_mask0(uistate: &ExState) -> Focus<'_> {
        Focus::new([
            (&uistate.input_0.focus, uistate.input_0.area),
            (&uistate.input_1.focus, uistate.input_1.area),
        ])
    }

    fn handle_mask0(evt: &Event, data: &mut ExData, uistate: &mut ExState) -> Control {
        let f = focus_mask0(uistate).handle(evt);

        // validation and reformat on focus lost.
        validate!(uistate.input_0 =>
        if let Ok(d) = NaiveDate::parse_from_str(uistate.input_0.compact_value().as_str(), "%d.%m.%Y") {
            data.datum = d;
            let v = data.datum.format("%d.%m.%Y").to_string();
            uistate.input_0.set_value(v);
            uistate.input_0.select_all();
            true
        } else {
            data.datum = NaiveDate::default();
            false
        });

        cut!({
            let r = uistate.input_0.handle(evt);
            // quick validation for every change
            r.on_changed_do(|| {
                let str = uistate.input_0.compact_value();
                let r = NaiveDate::parse_from_str(str.as_str(), "%d.%m.%Y");
                uistate.input_0.is_valid = r.is_ok();
            });
            r
        });

        cut!(uistate.input_1.handle(evt));

        cut!(match evt {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if let Ok(d) =
                    NaiveDate::parse_from_str(uistate.input_0.compact_value().as_str(), "%d.%m.%Y")
                {
                    uistate.input_0.is_valid = true;
                    data.datum = d;
                } else {
                    uistate.input_0.is_valid = false;
                    data.datum = NaiveDate::default();
                }
                let v = data.datum.format("%d.%m.%Y").to_string();
                uistate.input_0.set_value(v);
                Control::Changed
            }
            _ => Control::Continue,
        });

        f.into_control()
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
