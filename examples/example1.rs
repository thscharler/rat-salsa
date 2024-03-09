use crate::app::Example;
use crate::data::ExData;
use crate::state::ExState;
use rat_salsa::{run_tui, ControlUI};

type Control = ControlUI<ExAction, anyhow::Error>;

fn main() {
    // ...

    let mut data = ExData {};
    let mut state = ExState::default();

    let res = run_tui(&Example, &mut data, &mut state, 1);

    _ = dbg!(res);
}

#[derive(Debug)]
pub enum ExAction {}

pub mod data {
    #[derive(Debug)]
    pub struct ExData {}
}

pub mod state {
    use crate::theme::Theme;
    use rat_salsa::button::ButtonStyle;
    use rat_salsa::message::{StatusDialogState, StatusDialogStyle, StatusLineState};
    use ratatui::prelude::Stylize;
    use ratatui::style::Style;

    #[derive(Debug, Default)]
    pub struct ExState {
        pub g: GeneralState,
    }

    #[derive(Debug, Default)]
    pub struct GeneralState {
        pub theme: Theme,
        pub status: StatusLineState,
        pub error_dlg: StatusDialogState,
    }

    impl GeneralState {
        pub fn status_style(&self) -> Style {
            Style::default().fg(self.theme.white).bg(self.theme.one_bg3)
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
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use rat_salsa::message::{StatusDialog, StatusLine};
    use rat_salsa::{cut, HandleEvent, TaskSender, ThreadPool, TuiApp};
    use ratatui::layout::{Constraint, Direction, Layout};
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
            _data: &mut ExData,
            uistate: &mut ExState,
        ) -> Result<(), anyhow::Error> {
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

            // TODO fan out

            {
                let statusdialog = StatusDialog::new().style(uistate.g.status_dialog_style());
                let mut err_dialog = &mut uistate.g.error_dlg;
                if err_dialog.active {
                    frame.render_stateful_widget(statusdialog, layout[0], &mut err_dialog);
                }
            }
            {
                let status = StatusLine::new().style(uistate.g.status_style());
                let mut msg = &mut uistate.g.status;
                frame.render_stateful_widget(status, layout[2], &mut msg);
            }

            Ok(())
        }

        fn handle_event(&self, evt: Event, _data: &mut ExData, uistate: &mut ExState) -> Control {
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
