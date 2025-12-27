use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use rat_event::{ct_event, try_flow};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_theme4::WidgetStyle;
use rat_widget::event::Outcome;
use rat_widget::layout::layout_middle;
use rat_widget::msgdialog;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
use std::iter::repeat_with;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        menu: Default::default(),
        msg: Default::default(),
    };

    run_ui("menu_status1", mock_init, event, render, &mut state)
}

struct State {
    pub(crate) menu: MenuLineState,
    pub(crate) msg: MsgDialogState,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
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
        .styles(ctx.theme.style(WidgetStyle::MENU))
        .render(l1[1], buf, &mut state.menu);

    if state.msg.active() {
        let l_msg = layout_middle(
            l1[0],
            Constraint::Percentage(19),
            Constraint::Percentage(19),
            Constraint::Percentage(19),
            Constraint::Percentage(19),
        );
        MsgDialog::new()
            .styles(ctx.theme.style(WidgetStyle::MSG_DIALOG))
            .render(l_msg, buf, &mut state.msg);
    }

    Ok(())
}

fn event(
    event: &Event,
    ctx: &mut MiniSalsaState,
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
                ctx.status[0] = format!("Selected {}", v);
                Outcome::Changed
            }
            MenuOutcome::Activated(v) => {
                ctx.status[0] = format!("Activated {}", v);
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
                        ctx.quit = true;
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
