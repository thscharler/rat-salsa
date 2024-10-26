use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{ct_event, try_flow};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_widget::event::Outcome;
use rat_widget::layout::layout_middle;
use rat_widget::msgdialog;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::StatefulWidget;
use ratatui::Frame;
use std::iter::repeat_with;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        menu: Default::default(),
        msg: Default::default(),
    };

    run_ui(
        "menu_status1",
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {}

struct State {
    pub(crate) menu: MenuLineState,
    pub(crate) msg: MsgDialogState,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    MenuLine::new()
        .title("Sample")
        .item_parsed("Choose _1")
        .item_parsed("Choose _2")
        .item_parsed("Choose _3")
        .item_parsed("_Message|F1")
        .item_parsed("_Quit")
        .styles(THEME.menu_style())
        .render(l1[1], frame.buffer_mut(), &mut state.menu);

    if state.msg.active() {
        let l_msg = layout_middle(
            l1[0],
            Constraint::Percentage(19),
            Constraint::Percentage(19),
            Constraint::Percentage(19),
            Constraint::Percentage(19),
        );
        MsgDialog::new()
            .styles(THEME.msg_dialog_style())
            // .block(Block::bordered().style(THEME.gray(3)))
            // .style(THEME.gray(3))
            // .button_style(ButtonStyle {
            //     style: THEME.secondary(2),
            //     focus: Some(THEME.primary(3)),
            //     armed: Some(THEME.primary(1)),
            //     ..Default::default()
            // })
            .render(l_msg, frame.buffer_mut(), &mut state.msg);
    }

    Ok(())
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(msgdialog::handle_dialog_events(&mut state.msg, event));

    try_flow!(match event {
        ct_event!(keycode press F(1)) => {
            state.msg.append(
                &repeat_with(|| "Hello world!\n------------\n")
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
                istate.status[0] = format!("Selected {}", v);
                Outcome::Changed
            }
            MenuOutcome::Activated(v) => {
                istate.status[0] = format!("Activated {}", v);
                match v {
                    3 => {
                        state.msg.append(
                            &repeat_with(|| "Hello world!\n------------\n")
                                .take(20)
                                .collect::<String>(),
                        );
                        state.msg.set_active(true);
                        return Ok(Outcome::Changed);
                    }
                    4 => {
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
