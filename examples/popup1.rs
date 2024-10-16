#![allow(dead_code)]
#![allow(unreachable_pub)]

use crate::adapter::blue::{Blue, BlueState};
use crate::mini_salsa::{layout_grid, run_ui, setup_logging, MiniSalsaState};
use crate::variants::popup_edit::{PopEditGreen, PopEditGreenState};
use crate::variants::popup_focus::{PopFocusBlue, PopFocusBlueState};
use crate::variants::popup_lock_edit::{PopLockMagenta, PopLockMagentaState};
use crate::variants::popup_nonfocus::{PopNonFocusRed, PopNonFocusRedState};
use rat_cursor::HasScreenCursor;
use rat_event::{ct_event, HandleEvent, Outcome, Regular};
use rat_focus::{Focus, FocusBuilder, FocusContainer};
use rat_popup::event::PopupOutcome;
use rat_popup::Placement;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::StatefulWidget;
use ratatui::Frame;

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

    run_ui("popup1", handle_stuff, repaint_stuff, &mut data, &mut state)
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

fn repaint_stuff(
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

impl FocusContainer for State {
    #[allow(clippy::single_match)]
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.not_blue);
        builder.widget(&self.blue);

        _ = match self.which_blue {
            0 => builder.container(&self.popfoc),
            1 => builder,
            2 => builder.container(&self.popedit),
            3 => builder.widget(&self.poplock),
            _ => builder,
        };
    }
}

fn focus(state: &mut State) -> Focus {
    let f = FocusBuilder::for_container(state);
    f.enable_log();
    f
}

fn handle_stuff(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);

    let f = focus.handle(event, Regular);

    let r0 = match state.popfoc.handle(event, Regular) {
        PopupOutcome::Hide => {
            focus.next();
            Outcome::Changed
        }
        r => r.into(),
    };
    let r1 = match state.popact.handle(event, Regular) {
        PopupOutcome::Hide => {
            state.popact.hide();
            Outcome::Changed
        }
        r => r.into(),
    };
    let r2 = match state.popedit.handle(event, Regular) {
        PopupOutcome::Hide => {
            state.popedit.hide(&mut focus);
            Outcome::Changed
        }
        r => r.into(),
    };
    let r3 = match state.poplock.handle(event, Regular) {
        PopupOutcome::Hide => {
            state.poplock.hide(&mut focus);
            Outcome::Changed
        }
        r => r.into(),
    };

    let r4 = match event {
        ct_event!(keycode press F(1)) => {
            state.which_blue = (state.which_blue + 1) % 4;
            match state.which_blue {
                0 => state.popfoc.hide(&mut focus),
                1 => state.popact.hide(),
                2 => state.popedit.hide(&mut focus),
                3 => state.poplock.hide(&mut focus),
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
                    placement = Placement::None;
                    &mut placement
                }
            };
            *placement = match *placement {
                Placement::AboveLeft(r) => Placement::AboveCenter(r),
                Placement::AboveCenter(r) => Placement::AboveRight(r),
                Placement::AboveRight(r) => Placement::RightTop(r),
                Placement::RightTop(r) => Placement::RightMiddle(r),
                Placement::RightMiddle(r) => Placement::RightBottom(r),
                Placement::RightBottom(r) => Placement::BelowRight(r),
                Placement::BelowRight(r) => Placement::BelowCenter(r),
                Placement::BelowCenter(r) => Placement::BelowLeft(r),
                Placement::BelowLeft(r) => Placement::LeftBottom(r),
                Placement::LeftBottom(r) => Placement::LeftMiddle(r),
                Placement::LeftMiddle(r) => Placement::LeftTop(r),
                Placement::LeftTop(r) => Placement::AboveLeft(r),
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
                    placement = Placement::None;
                    &mut placement
                }
            };
            *placement = match *placement {
                Placement::AboveLeft(r) => Placement::LeftTop(r),
                Placement::AboveCenter(r) => Placement::AboveLeft(r),
                Placement::AboveRight(r) => Placement::AboveCenter(r),
                Placement::RightTop(r) => Placement::AboveRight(r),
                Placement::RightMiddle(r) => Placement::RightTop(r),
                Placement::RightBottom(r) => Placement::RightMiddle(r),
                Placement::BelowRight(r) => Placement::RightBottom(r),
                Placement::BelowCenter(r) => Placement::BelowRight(r),
                Placement::BelowLeft(r) => Placement::BelowCenter(r),
                Placement::LeftBottom(r) => Placement::BelowLeft(r),
                Placement::LeftMiddle(r) => Placement::LeftBottom(r),
                Placement::LeftTop(r) => Placement::LeftMiddle(r),
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
                Placement::LeftTop(state.blue.area)
            } else if *x >= state.blue.area.right() {
                Placement::RightTop(state.blue.area)
            } else if *y < state.blue.area.top() {
                Placement::AboveLeft(state.blue.area)
            } else if *y >= state.blue.area.bottom() {
                Placement::BelowLeft(state.blue.area)
            } else {
                unreachable!()
            };
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
        ct_event!(mouse down Right for x,y) if state.right.contains((*x, *y).into()) => {
            // relative to mouse
            let placement = Placement::Position(*x, *y);
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
    };

    Ok([f, r0, r1, r2, r3, r4].iter().max().copied().expect("r"))
}
