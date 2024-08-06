use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use rat_event::{ct_event, ConsumedEvent};
use rat_widget::event::Outcome;
use rat_widget::layout::layout_grid;
use rat_widget::menuline::MenuOutcome;
use rat_widget::popup_menu;
use rat_widget::popup_menu::{Placement, PopupMenu, PopupMenuState, Separator};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, StatefulWidget};
use ratatui::Frame;
mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        area: Default::default(),
        left: Default::default(),
        right: Default::default(),
        placement: Placement::Top,
        popup_area: Default::default(),
        popup: PopupMenuState::default(),
    };

    run_ui(handle_stuff, repaint_stuff, &mut data, &mut state)
}

struct Data {}

struct State {
    pub(crate) area: Rect,
    pub(crate) left: Rect,
    pub(crate) right: Rect,

    pub(crate) placement: Placement,
    pub(crate) popup_area: Rect,
    pub(crate) popup: PopupMenuState,
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

    state.area = l[1][1];
    state.left = l[0][0].union(l[2][2]);
    state.right = l[3][0].union(l[3][2]);

    frame
        .buffer_mut()
        .set_style(l[1][1], Style::new().on_blue());
    frame
        .buffer_mut()
        .set_style(l[3][0].union(l[3][2]), Style::new().on_dark_gray());

    if state.popup.active() {
        PopupMenu::new()
            .style(Style::new().black().on_cyan())
            .block(Block::bordered().title("Nice popup"))
            .placement(state.placement)
            .add_str("Item _1")
            .add_sep(Separator::Plain)
            .add_str("Item _2")
            .add_str("Item _3")
            .add_str("Item _4")
            .boundary(area)
            .render(state.popup_area, frame.buffer_mut(), &mut state.popup);
    }

    Ok(())
}

fn handle_stuff(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let r1 = match popup_menu::handle_popup_events(&mut state.popup, event) {
        MenuOutcome::Activated(n) => {
            istate.status[0] = format!("Selected {}", n);
            Outcome::Changed
        }
        r => r.into(),
    };
    if r1.is_consumed() {
        return Ok(r1);
    }

    let r2 = match event {
        ct_event!(mouse down Left for x,y) if state.left.contains((*x, *y).into()) => {
            state.popup_area = state.area;
            state.popup.set_active(true);

            if *x < state.area.left() {
                state.placement = Placement::Left;
            } else if *x >= state.area.right() {
                state.placement = Placement::Right;
            } else if *y < state.area.top() {
                state.placement = Placement::Top;
            } else if *y >= state.area.bottom() {
                state.placement = Placement::Bottom;
            }
            Outcome::Changed
        }
        ct_event!(mouse down Left for x,y) if state.right.contains((*x, *y).into()) => {
            state.popup_area = Rect::new(*x, *y, 0, 0);
            state.popup.set_active(true);
            state.placement = Placement::Right;
            Outcome::Changed
        }
        _ => Outcome::Continue,
    };
    Ok(r2)
}
