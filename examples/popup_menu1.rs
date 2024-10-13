#![allow(dead_code)]
#![allow(unreachable_pub)]

use crate::blue::{Blue, BlueState};
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{ct_event, ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder};
use rat_menu::event::MenuOutcome;
use rat_menu::menuitem::Separator;
use rat_menu::popup_menu;
use rat_menu::popup_menu::{Placement, PopupMenu, PopupMenuState};
use rat_widget::event::Outcome;
use rat_widget::layout::layout_grid;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, StatefulWidget};
use ratatui::Frame;
use std::cmp::max;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        left: Default::default(),
        right: Default::default(),
        blue: BlueState::named("blue"),
        not_blue: BlueState::named("not_blue"),
        placement: Placement::None,
        offset: (0, 0),
        popup_area: Default::default(),
        popup: PopupMenuState::default(),
    };

    run_ui(handle_stuff, repaint_stuff, &mut data, &mut state)
}

struct Data {}

struct State {
    left: Rect,
    right: Rect,

    blue: BlueState,
    not_blue: BlueState,

    placement: Placement,
    offset: (i16, i16),
    popup_area: Rect,
    popup: PopupMenuState,
}

fn repaint_stuff(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
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

    // state.area = l[1][1];
    state.left = l[0][0].union(l[2][2]);
    state.right = l[3][0].union(l[3][2]);

    // two test regions:
    // for placement relative to a rect.
    Blue::new()
        .style(Style::new().on_blue())
        .focus_style(Style::new().on_light_blue())
        .render(l[1][1], frame.buffer_mut(), &mut state.blue);
    Blue::new()
        .style(Style::new().on_yellow())
        .focus_style(Style::new().on_light_yellow())
        .render(l[0][1], frame.buffer_mut(), &mut state.not_blue);

    // for placement near the mouse cursor.
    frame
        .buffer_mut()
        .set_style(l[3][0].union(l[3][2]), Style::new().on_dark_gray());

    if state.popup.is_active() {
        PopupMenu::new()
            .item_parsed("Item _1")
            .separator(Separator::Plain)
            .item_parsed("Item _2")
            .item_parsed("Item _3")
            .item_parsed("Item _4")
            .style(Style::new().black().on_cyan())
            .block(
                Block::bordered()
                    .style(Style::new().black().on_cyan())
                    .title("Nice popup"),
            )
            .placement(state.placement)
            .offset(state.offset)
            .render(state.popup_area, frame.buffer_mut(), &mut state.popup);
    }

    Ok(())
}

fn focus(state: &mut State) -> Focus {
    let mut fb = FocusBuilder::new(None);
    fb.widget(&state.blue);
    fb.widget(&state.not_blue);
    fb.build()
}

fn handle_stuff(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let f = focus(state).handle(event, Regular);
    if f == Outcome::Changed {
        state.popup.set_active(false);
    }

    let r = match popup_menu::handle_popup_events(&mut state.popup, event) {
        MenuOutcome::Hide => {
            match event {
                ct_event!(mouse down Left for _x, _y) => {
                    // reposition. later.
                    Outcome::Continue
                }
                _ => {
                    state.popup.set_active(false);
                    Outcome::Changed
                }
            }
        }
        MenuOutcome::Activated(n) => {
            istate.status[0] = format!("Selected {}", n);
            Outcome::Changed
        }
        r => r.into(),
    };

    let r = r.or_else(|| match event {
        ct_event!(key press CONTROL-' ') => {
            // placement relative to cursor
            state.placement = Placement::AboveLeft(state.blue.area);
            state.offset = (-2, 0);
            state.popup.set_active(true);
            Outcome::Changed
        }
        ct_event!(mouse down Left for x,y) if state.left.contains((*x, *y).into()) => {
            // placement relative to rect
            if *x < state.blue.area.left() {
                state.placement = Placement::LeftTop(state.blue.area);
                state.offset = (0, -1);
            } else if *x >= state.blue.area.right() {
                state.placement = Placement::RightTop(state.blue.area);
                state.offset = (0, -1);
            } else if *y < state.blue.area.top() {
                state.placement = Placement::AboveLeft(state.blue.area);
                state.offset = (-2, 0);
            } else if *y >= state.blue.area.bottom() {
                state.placement = Placement::BelowLeft(state.blue.area);
                state.offset = (-2, 0);
            }
            state.popup.set_active(true);
            Outcome::Changed
        }
        ct_event!(mouse down Left for x,y) if state.right.contains((*x, *y).into()) => {
            // placement relative to cursor
            state.placement = Placement::Position(*x, *y);
            state.offset = (-1, -1);
            state.popup.set_active(true);
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(max(f, r))
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
