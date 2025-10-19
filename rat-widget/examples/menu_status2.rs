use crate::mini_salsa::theme::ShellTheme;
use crate::mini_salsa::{MiniSalsaState, RADIUM, run_ui, setup_logging};
use rat_event::{ct_event, try_flow};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_widget::event::Outcome;
use rat_widget::layout::layout_middle;
use rat_widget::msgdialog;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline_stacked::{SLANT_BL_TR, SLANT_TL_BR, StatusLineStacked};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{StatefulWidget, Widget};
use std::iter::repeat_with;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        menu: Default::default(),
        msg: Default::default(),
        msg_count: Default::default(),
        mode: Default::default(),
        status_styling: Default::default(),
    };

    run_ui(
        "menu_status2", //
        init,
        event,
        render,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    menu: MenuLineState,
    msg: MsgDialogState,
    msg_count: u32,

    mode: i32,
    status_styling: i32,
}

fn init(
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    _state: &mut State,
) -> Result<(), anyhow::Error> {
    static THEME: ShellTheme = ShellTheme::new(RADIUM.name, RADIUM);
    istate.theme = &THEME;
    istate.hide_status = true;
    Ok(())
}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([
        Constraint::Length(1), //
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .split(area);

    MenuLine::new()
        .item_parsed(mtxt("STATE\u{FF3F}_1", 0, state).as_str())
        .item_parsed(mtxt("STATE\u{FF3F}_2", 1, state).as_str())
        .item_parsed(mtxt("STATE\u{FF3F}_3", 2, state).as_str())
        .item_parsed(mtxt2("_MESSAGE|F1", "_MESSAGE", 3, state).as_str())
        .item_parsed(mtxt("_STYLE", 4, state).as_str())
        .item_parsed(mtxt("_QUIT", 5, state).as_str())
        .styles(istate.theme.menu_style())
        .focus_style(istate.theme.primary(7))
        .render(l1[0], frame.buffer_mut(), &mut state.menu);

    match state.status_styling {
        0 => stacked_1(frame, l1[2], _data, istate, state),
        1 => stacked_2(frame, l1[2], _data, istate, state),
        _ => unreachable!(),
    }

    if state.msg.active() {
        let l_msg = layout_middle(
            l1[1],
            Constraint::Percentage(19),
            Constraint::Percentage(19),
            Constraint::Percentage(19),
            Constraint::Percentage(19),
        );
        MsgDialog::new()
            .styles(istate.theme.msg_dialog_style())
            .render(l_msg, frame.buffer_mut(), &mut state.msg);
    }

    Ok(())
}

fn mtxt(txt: &str, idx: usize, state: &mut State) -> String {
    if state.menu.selected == Some(idx) {
        format!("[{}]", txt).to_string()
    } else {
        txt.to_string()
    }
}

fn mtxt2(txt: &str, txt_sel: &str, idx: usize, state: &mut State) -> String {
    if state.menu.selected == Some(idx) {
        format!("[{}]", txt_sel).to_string()
    } else {
        txt.to_string()
    }
}

fn stacked_1(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) {
    let pal = istate.theme.palette();
    let color_0 = pal.gray[3];
    let color_1 = match state.mode {
        0 => pal.green[3],
        1 => pal.yellow[3],
        2 => pal.red[3],
        _ => unreachable!(),
    };
    let color_3 = pal.cyan[0];
    let color_4 = pal.cyan[7];

    let mode_str = match state.mode {
        0 => " OPERATIONAL ",
        1 => " DIRE ",
        2 => " EVACUATE ",
        _ => unreachable!(),
    };

    StatusLineStacked::new()
        .style(istate.theme.status_base())
        .start(
            Span::from(" WESTINGHOUSE[STATUS]2 ")
                .style(Style::new().fg(pal.text_black).bg(color_0)),
            Span::from(SLANT_TL_BR).style(Style::new().fg(color_0).bg(color_1)),
        )
        .start(
            Span::from(mode_str).style(Style::new().fg(pal.text_black).bg(color_1)),
            Span::from(SLANT_TL_BR).style(Style::new().fg(color_1)),
        )
        .center_margin(1)
        .center(istate.status[0].as_str())
        .end(
            Span::from(format!("R[{}][{:.0?} ", istate.frame, istate.last_render))
                .style(Style::new().fg(pal.text_black).bg(color_3)),
            Span::from(SLANT_BL_TR).style(Style::new().fg(color_3).bg(color_4)),
        )
        .end(
            "",
            Span::from(SLANT_BL_TR).style(Style::new().fg(color_4).bg(color_3)),
        )
        .end(
            Span::from(format!("E[{}][{:.0?}", istate.event_cnt, istate.last_event))
                .style(Style::new().fg(pal.text_black).bg(color_3)),
            Span::from(SLANT_BL_TR).style(Style::new().fg(color_3).bg(color_4)),
        )
        .end(
            "",
            Span::from(SLANT_BL_TR).style(Style::new().fg(color_4).bg(color_3)),
        )
        .end(
            Span::from(format!("MSG[{}", state.msg_count))
                .style(Style::new().fg(pal.text_black).bg(color_3)),
            Span::from(SLANT_BL_TR).style(Style::new().fg(color_3).bg(color_4)),
        )
        .end("", Span::from(SLANT_BL_TR).style(Style::new().fg(color_4)))
        .render(area, frame.buffer_mut());
}

fn stacked_2(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) {
    let pal = istate.theme.palette();
    let color_0 = pal.gray[3];
    let color_1 = match state.mode {
        0 => pal.green[3],
        1 => pal.yellow[3],
        2 => pal.red[3],
        _ => unreachable!(),
    };
    let color_3 = pal.cyan[0];
    let color_4 = pal.gray[6];

    let mode_str = match state.mode {
        0 => " [OPERATIONAL] ",
        1 => " [DIRE] ",
        2 => " [EVACUATE] ",
        _ => unreachable!(),
    };

    StatusLineStacked::new()
        .style(Style::new().fg(pal.text_light).bg(color_4))
        .start_bare(
            Span::from(" WESTINGHOUSE[STATUS]2 ")
                .style(Style::new().fg(pal.text_light).bg(color_0)),
        )
        .start_bare(Span::from(mode_str).style(Style::new().fg(pal.text_black).bg(color_1)))
        .center_margin(1)
        .center(istate.status[0].as_str())
        .end_bare(
            Span::from(format!("R[{}][{:.0?}] ", istate.frame, istate.last_render))
                .style(Style::new().fg(pal.text_black).bg(color_3)),
        )
        .end_bare(
            Span::from(format!(
                "E[{}][{:.0?}] ",
                istate.event_cnt, istate.last_event
            ))
            .style(Style::new().fg(pal.text_black).bg(color_3)),
        )
        .end_bare(
            Span::from(format!(" MSG[{}] ", state.msg_count))
                .style(Style::new().fg(pal.text_black).bg(color_3)),
        )
        .render(area, frame.buffer_mut());
}

fn event(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(msgdialog::handle_dialog_events(&mut state.msg, event));

    try_flow!(match event {
        ct_event!(keycode press F(1)) => {
            state.msg_count += 1;
            state.msg.append(
                &repeat_with(|| "ПРИВІТ РЕАКТОР!\n------------\n")
                    .take(20)
                    .collect::<String>(),
            );
            state.msg.set_active(true);
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    try_flow!(
        match menuline::handle_events(&mut state.menu, true, event) {
            MenuOutcome::Selected(v) => {
                istate.status[0] = format!("SELECT {}", v);
                Outcome::Changed
            }
            MenuOutcome::Activated(0) => {
                state.mode = 0;
                Outcome::Changed
            }
            MenuOutcome::Activated(1) => {
                state.mode = 1;
                Outcome::Changed
            }
            MenuOutcome::Activated(2) => {
                state.mode = 2;
                Outcome::Changed
            }
            MenuOutcome::Activated(v) => {
                istate.status[0] = format!("ACTIVATE {}", v);
                match v {
                    3 => {
                        state.msg.append(
                            &repeat_with(|| "HELLO REACTOR!\n------------\n")
                                .take(20)
                                .collect::<String>(),
                        );
                        state.msg_count += 1;
                        state.msg.set_active(true);
                        return Ok(Outcome::Changed);
                    }
                    4 => {
                        state.status_styling = (state.status_styling + 1) % 2;
                    }
                    5 => {
                        istate.quit = true;
                        return Ok(Outcome::Changed);
                    }
                    _ => {}
                }
                Outcome::Changed
            }
            r => r.into(),
        }
    );

    Ok(Outcome::Continue)
}
