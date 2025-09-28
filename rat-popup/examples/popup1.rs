#![allow(dead_code)]
#![allow(unreachable_pub)]

use crate::adapter::blue::{Blue, BlueState};
use crate::mini_salsa::{MiniSalsaState, layout_grid, mock_init, run_ui, setup_logging};
use crate::variants::popup_edit::{PopEditGreen, PopEditGreenState};
use crate::variants::popup_focus::{PopFocusBlue, PopFocusBlueState};
use crate::variants::popup_lock_edit::{PopLockMagenta, PopLockMagentaState};
use crate::variants::popup_nonfocus::{PopNonFocusRed, PopNonFocusRedState};
use log::debug;
use rat_cursor::HasScreenCursor;
use rat_event::{HandleEvent, Outcome, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_popup::PopupConstraint;
use rat_popup::event::PopupOutcome;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::StatefulWidget;

mod adapter;
mod mini_salsa;
mod variants;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        area: Default::default(),
        left: Default::default(),
        right: Default::default(),

        which_blue: 0,
        blue: BlueState::named("blue"),
        not_blue: BlueState::named("not_blue"),

        popfoc: PopFocusBlueState::new(),
        popact: PopNonFocusRedState::new(),
        popedit: PopEditGreenState::default(),
        poplock: PopLockMagentaState::default(),
    };

    run_ui("popup1", mock_init, event, render, &mut data, &mut state)
}

struct Data {}

struct State {
    area: Rect,
    left: Rect,
    right: Rect,

    popfoc: PopFocusBlueState,
    popact: PopNonFocusRedState,
    popedit: PopEditGreenState,
    poplock: PopLockMagentaState,

    which_blue: u8,
    blue: BlueState,
    not_blue: BlueState,
}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    istate.status[0] = "Ctrl-Q to Quit; F1 chg pop; F2 chg place".into();

    let l = layout_grid::<4, 3>(
        area,
        Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(3),
        ]),
        Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]),
    );

    state.area = area;
    state.left = l[0][0].union(l[2][2]);
    state.right = l[3][0].union(l[3][2]);

    // two test regions:
    // for placement relative to a rect.
    let mut blue = Blue::new();
    match state.which_blue {
        0 => {
            blue = blue
                .style(Style::new().on_blue())
                .focus_style(Style::new().on_light_blue());
        }
        1 => {
            blue = blue
                .style(Style::new().on_red())
                .focus_style(Style::new().on_light_red());
        }
        2 => {
            blue = blue
                .style(Style::new().on_green())
                .focus_style(Style::new().on_light_green());
        }
        3 => {
            blue = blue
                .style(Style::new().on_magenta())
                .focus_style(Style::new().on_light_magenta());
        }
        _ => {}
    }
    blue.render(l[1][1], frame.buffer_mut(), &mut state.blue);

    Blue::new()
        .style(Style::new().on_dark_gray())
        .focus_style(Style::new().on_gray())
        .render(l[0][0], frame.buffer_mut(), &mut state.not_blue);

    // for placement near the mouse cursor.
    frame
        .buffer_mut()
        .set_style(l[3][0].union(l[3][2]), Style::new().on_dark_gray());

    match state.which_blue {
        0 => PopFocusBlue.render(
            Rect::new(0, 0, 13, 5),
            frame.buffer_mut(),
            &mut state.popfoc,
        ),
        1 => PopNonFocusRed.render(
            Rect::new(0, 0, 11, 5),
            frame.buffer_mut(),
            &mut state.popact,
        ),
        2 => PopEditGreen.render(
            Rect::new(0, 0, 11, 5),
            frame.buffer_mut(),
            &mut state.popedit,
        ),
        3 => PopLockMagenta.render(
            Rect::new(0, 0, 11, 5),
            frame.buffer_mut(),
            &mut state.poplock,
        ),
        _ => {}
    }

    match state.which_blue {
        0 => None,
        1 => None,
        2 => state.popedit.screen_cursor(),
        3 => state.poplock.screen_cursor(),
        _ => None,
    }
    .map(|p| frame.set_cursor_position(p));

    Ok(())
}

impl HasFocus for State {
    #[allow(clippy::single_match)]
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.not_blue);
        builder.widget(&self.blue);

        _ = match self.which_blue {
            0 => builder.widget(&self.popfoc),
            1 => builder,
            2 => builder.widget(&self.popedit),
            3 => builder.widget(&self.poplock),
            _ => builder,
        };
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not a widget")
    }

    fn area(&self) -> Rect {
        unimplemented!("not a widget")
    }
}

fn focus(state: &mut State) -> Focus {
    let f = FocusBuilder::build_for(state);
    f.enable_log();
    f
}

fn event(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);

    istate.focus_outcome = focus.handle(event, Regular);

    try_flow!(match state.popfoc.handle(event, Regular) {
        PopupOutcome::Hide => {
            state.popfoc.hide(&focus);
            Outcome::Changed
        }
        r => r.into(),
    });
    try_flow!(match state.popact.handle(event, Regular) {
        PopupOutcome::Hide => {
            state.popact.hide();
            Outcome::Changed
        }
        r => r.into(),
    });
    try_flow!(match state.popedit.handle(event, Regular) {
        PopupOutcome::Hide => {
            state.popedit.hide(&mut focus);
            Outcome::Changed
        }
        r => r.into(),
    });
    try_flow!(match state.poplock.handle(event, Regular) {
        PopupOutcome::Hide => {
            state.poplock.hide(&mut focus);
            Outcome::Changed
        }
        r => r.into(),
    });

    try_flow!(match event {
        ct_event!(keycode press F(1)) => {
            debug!("change blue");
            state.which_blue = (state.which_blue + 1) % 4;
            match state.which_blue {
                0 => state.popfoc.hide(&focus),
                1 => state.popact.hide(),
                2 => state.popedit.hide(&mut focus),
                3 => state.poplock.hide(&focus),
                _ => {}
            }
            Outcome::Changed
        }
        ct_event!(keycode press F(2)) => {
            let mut placement;
            let placement = match state.which_blue {
                0 => &mut state.popfoc.placement,
                1 => &mut state.popact.placement,
                2 => &mut state.popedit.placement,
                3 => &mut state.poplock.placement,
                _ => {
                    placement = PopupConstraint::None;
                    &mut placement
                }
            };
            *placement = match *placement {
                PopupConstraint::Above(a, r) => PopupConstraint::Right(a, r),
                PopupConstraint::Right(a, r) => PopupConstraint::Below(a, r),
                PopupConstraint::Below(a, r) => PopupConstraint::Left(a, r),
                PopupConstraint::Left(a, r) => PopupConstraint::Above(a, r),
                v => v,
            };
            Outcome::Changed
        }
        ct_event!(keycode press SHIFT-F(2)) => {
            let mut placement;
            let placement = match state.which_blue {
                0 => &mut state.popfoc.placement,
                1 => &mut state.popact.placement,
                2 => &mut state.popedit.placement,
                3 => &mut state.poplock.placement,
                _ => {
                    placement = PopupConstraint::None;
                    &mut placement
                }
            };
            *placement = match *placement {
                PopupConstraint::Above(a, r) => PopupConstraint::Left(a, r),
                PopupConstraint::Left(a, r) => PopupConstraint::Below(a, r),
                PopupConstraint::Below(a, r) => PopupConstraint::Right(a, r),
                PopupConstraint::Right(a, r) => PopupConstraint::Above(a, r),
                v => v,
            };
            Outcome::Changed
        }
        ct_event!(mouse down Right for x,y)
            if state.left.contains((*x, *y).into())
                && !state.blue.area.contains((*x, *y).into()) =>
        {
            // placement relative to rect
            let placement = if *x < state.blue.area.left() {
                PopupConstraint::Left(Alignment::Left, state.blue.area)
            } else if *x >= state.blue.area.right() {
                PopupConstraint::Right(Alignment::Left, state.blue.area)
            } else if *y < state.blue.area.top() {
                PopupConstraint::Above(Alignment::Left, state.blue.area)
            } else if *y >= state.blue.area.bottom() {
                PopupConstraint::Below(Alignment::Left, state.blue.area)
            } else {
                unreachable!()
            };
            match state.which_blue {
                0 => {
                    state.popfoc.show(placement, &focus);
                    Outcome::Changed
                }
                1 => {
                    state.popact.show(placement);
                    Outcome::Changed
                }
                2 => {
                    state.popedit.show(placement, &mut focus);
                    Outcome::Changed
                }
                3 => {
                    state.poplock.show(placement, &focus);
                    Outcome::Changed
                }
                _ => Outcome::Continue,
            }
        }
        ct_event!(mouse down Right for x,y) if state.right.contains((*x, *y).into()) => {
            // relative to mouse
            let placement = PopupConstraint::Position(*x, *y);
            match state.which_blue {
                0 => {
                    state.popfoc.show(placement, &mut focus);
                    Outcome::Changed
                }
                1 => {
                    state.popact.show(placement);
                    Outcome::Changed
                }
                2 => {
                    state.popedit.show(placement, &mut focus);
                    Outcome::Changed
                }
                3 => {
                    state.poplock.show(placement, &mut focus);
                    Outcome::Changed
                }
                _ => Outcome::Continue,
            }
        }
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}
