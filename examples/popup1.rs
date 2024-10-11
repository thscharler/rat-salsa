#![allow(dead_code)]
#![allow(unreachable_pub)]

use crate::blue::{Blue, BlueState};
use crate::mini_salsa::{layout_grid, run_ui, setup_logging, MiniSalsaState};
use crate::popup_focus::{PopFoc, PopFocState};
use crate::popup_nonfocus::{PopAct, PopActState};
use rat_event::{ct_event, HandleEvent, Outcome, Regular};
use rat_focus::{Focus, FocusBuilder, HasFocus};
use rat_popup::event::PopupOutcome;
use rat_popup::Placement;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::StatefulWidget;
use ratatui::Frame;
use std::cmp::max;

mod mini_salsa;

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

        popfoc: PopFocState::new(),
        popact: PopActState::new(),
    };

    run_ui(handle_stuff, repaint_stuff, &mut data, &mut state)
}

struct Data {}

struct State {
    area: Rect,
    left: Rect,
    right: Rect,

    popfoc: PopFocState,
    popact: PopActState,

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
    if state.which_blue == 1 {
        blue = blue
            .style(Style::new().on_red())
            .focus_style(Style::new().on_light_red());
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
        0 => PopFoc.render(
            Rect::new(0, 0, 13, 5),
            frame.buffer_mut(),
            &mut state.popfoc,
        ),
        1 => PopAct.render(
            Rect::new(0, 0, 11, 5),
            frame.buffer_mut(),
            &mut state.popact,
        ),
        _ => {}
    }

    Ok(())
}

impl HasFocus for State {
    #[allow(clippy::single_match)]
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.not_blue);
        builder.widget(&self.blue);

        match self.which_blue {
            0 => {
                builder.container(&self.popfoc);
            }
            _ => {}
        }
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
        PopupOutcome::Hidden => {
            focus.next();
            Outcome::Changed
        }
        r => r.into(),
    };
    let r1 = Outcome::from(state.popact.handle(event, Regular));

    let r2 = match event {
        ct_event!(keycode press F(1)) => {
            state.which_blue = (state.which_blue + 1) % 2;
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
                    state.popfoc.placement = placement;
                    focus.focus(&state.popfoc);
                    Outcome::Changed
                }
                1 => {
                    state.popact.show(placement);
                    Outcome::Changed
                }
                _ => Outcome::Continue,
            }
        }
        ct_event!(mouse down Right for x,y) if state.right.contains((*x, *y).into()) => {
            // relative to mouse
            let placement = Placement::Position(*x, *y);
            focus.focus(&state.popfoc);
            match state.which_blue {
                0 => {
                    state.popfoc.placement = placement;
                    focus.focus(&state.popfoc);
                    Outcome::Changed
                }
                1 => {
                    state.popact.show(placement);
                    Outcome::Changed
                }
                _ => Outcome::Continue,
            }
        }
        _ => Outcome::Continue,
    };

    Ok(max(f, max(r0, max(r1, r2))))
}

mod blue {
    use rat_focus::{FocusFlag, HasFocusFlag};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Style, Stylize};
    use ratatui::widgets::StatefulWidget;

    #[derive(Debug)]
    pub struct Blue {
        style: Style,
        focus_style: Style,
    }

    impl Blue {
        pub fn new() -> Self {
            Self {
                style: Style::new().on_blue(),
                focus_style: Style::new().on_light_blue(),
            }
        }

        pub fn style(mut self, style: Style) -> Self {
            self.style = style;
            self
        }

        pub fn focus_style(mut self, style: Style) -> Self {
            self.focus_style = style;
            self
        }
    }

    impl StatefulWidget for Blue {
        type State = BlueState;

        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            state.area = area;
            if state.focus.is_focused() {
                buf.set_style(area, self.focus_style);
            } else {
                buf.set_style(area, self.style);
            }
        }
    }

    #[derive(Debug, Default)]
    pub struct BlueState {
        pub area: Rect,
        pub focus: FocusFlag,
    }

    impl BlueState {
        pub fn new() -> Self {
            Self {
                area: Default::default(),
                focus: FocusFlag::named("blue"),
            }
        }

        pub fn named(name: &str) -> Self {
            Self {
                area: Default::default(),
                focus: FocusFlag::named(name),
            }
        }
    }

    impl HasFocusFlag for BlueState {
        fn focus(&self) -> FocusFlag {
            self.focus.clone()
        }

        fn area(&self) -> Rect {
            self.area
        }
    }
}

mod popup_focus {
    use rat_event::{ct_event, HandleEvent, Regular};
    use rat_focus::{ContainerFlag, FocusBuilder, FocusFlag, HasFocus, HasFocusFlag, Navigation};
    use rat_popup::event::PopupOutcome;
    use rat_popup::{Placement, Popup, PopupState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Style, Stylize};
    use ratatui::text::Span;
    use ratatui::widgets::{Block, StatefulWidget, Widget};
    use std::cmp::max;

    #[derive(Debug)]
    pub struct PopFoc;

    impl StatefulWidget for PopFoc {
        type State = PopFocState;

        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            if state.popup.is_active() {
                Popup::new()
                    .placement(state.placement)
                    .block(Block::bordered())
                    .render(area, buf, &mut state.popup);

                state.area = state.popup.area;

                buf.set_style(state.popup.widget_area, Style::new().on_yellow());
                Span::from(" p-o-p-u-p ").render(state.popup.widget_area, buf);
            } else {
                state.popup.clear_areas();
                state.area = Rect::default();
            }
        }
    }

    #[derive(Debug, Default)]
    pub struct PopFocState {
        pub area: Rect,

        /// Where to place the popup
        pub placement: Placement,
        pub popup: PopupState,

        pub focus: FocusFlag,
    }

    impl PopFocState {
        pub fn new() -> Self {
            Self {
                area: Default::default(),
                placement: Default::default(),
                popup: PopupState::named("foc-popup"),
                focus: FocusFlag::named("foc"),
            }
        }

        pub fn is_active(&self) -> bool {
            self.is_focused()
        }
    }

    impl HasFocusFlag for PopFocState {
        fn focus(&self) -> FocusFlag {
            self.focus.clone()
        }

        fn area(&self) -> Rect {
            self.area
        }

        fn navigable(&self) -> Navigation {
            Navigation::Leave
        }
    }

    impl HasFocus for PopFocState {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.widget(self);
        }

        fn container(&self) -> Option<ContainerFlag> {
            Some(self.popup.container.clone())
        }

        fn area(&self) -> Rect {
            self.popup.container.area()
        }
    }

    impl HandleEvent<crossterm::event::Event, Regular, PopupOutcome> for PopFocState {
        fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> PopupOutcome {
            let r0 = self.popup.handle(event, Regular);

            let r1 = match event {
                ct_event!(keycode press F(2)) => {
                    self.placement = match self.placement {
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
                    PopupOutcome::Changed
                }
                ct_event!(keycode press SHIFT-F(2)) => {
                    self.placement = match self.placement {
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
                    PopupOutcome::Changed
                }
                _ => PopupOutcome::Continue,
            };

            max(r0, r1)
        }
    }
}

mod popup_nonfocus {
    use rat_event::{ct_event, HandleEvent, Regular};
    use rat_popup::event::PopupOutcome;
    use rat_popup::{Placement, Popup, PopupState};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Style, Stylize};
    use ratatui::text::Span;
    use ratatui::widgets::{Block, BorderType, StatefulWidget, Widget};
    use std::cmp::max;

    #[derive(Debug, Default)]
    pub struct PopAct;

    impl StatefulWidget for PopAct {
        type State = PopActState;

        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            if state.popup.is_active() {
                let (dx, dy) = match state.placement {
                    Placement::None => (0, 0),
                    Placement::AboveLeft(_) => (-1, 0),
                    Placement::AboveCenter(_) => (0, 0),
                    Placement::AboveRight(_) => (1, 0),
                    Placement::LeftTop(_) => (0, -1),
                    Placement::LeftMiddle(_) => (0, 0),
                    Placement::LeftBottom(_) => (0, 1),
                    Placement::RightTop(_) => (0, -1),
                    Placement::RightMiddle(_) => (0, 0),
                    Placement::RightBottom(_) => (0, 1),
                    Placement::BelowLeft(_) => (-1, 0),
                    Placement::BelowCenter(_) => (0, 0),
                    Placement::BelowRight(_) => (1, 0),
                    Placement::Position(_, _) => (-1, -1),
                    _ => {
                        unimplemented!()
                    }
                };

                Popup::new()
                    .placement(state.placement)
                    .offset((dx, dy))
                    .block(Block::bordered().border_type(BorderType::Rounded))
                    .render(area, buf, &mut state.popup);

                buf.set_style(state.popup.widget_area, Style::new().on_cyan());
                Span::from("***").render(state.popup.widget_area, buf);
            }
        }
    }

    #[derive(Debug, Default)]
    pub struct PopActState {
        /// Where to place the popup
        pub placement: Placement,
        /// Internalized popup state.
        pub popup: PopupState,
    }

    impl PopActState {
        pub fn new() -> Self {
            Self {
                placement: Default::default(),
                popup: PopupState::named("act-popup"),
            }
        }

        pub fn is_active(&self) -> bool {
            self.popup.is_active()
        }

        pub fn show(&mut self, placement: Placement) {
            self.placement = placement;
            self.popup.set_active(true);
        }

        pub fn hide(&mut self) {
            self.popup.set_active(false);
        }
    }

    impl HandleEvent<crossterm::event::Event, Regular, PopupOutcome> for PopActState {
        fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> PopupOutcome {
            let r0 = self.popup.handle(event, Regular);

            let r1 = match event {
                ct_event!(keycode press F(2)) => {
                    self.placement = match self.placement {
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
                    PopupOutcome::Changed
                }
                ct_event!(keycode press SHIFT-F(2)) => {
                    self.placement = match self.placement {
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
                    PopupOutcome::Changed
                }
                _ => PopupOutcome::Continue,
            };

            max(r0, r1)
        }
    }
}
