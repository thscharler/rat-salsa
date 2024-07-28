#![allow(dead_code)]
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
#[allow(unused_imports)]
use log::debug;
use rat_event::{ct_event, flow_ok, HandleEvent, Regular};
use rat_focus::{Focus, HasFocusFlag};
use rat_scrolled::Scroll;
use rat_widget::event::Outcome;
use rat_widget::list::selection::RowSelection;
use rat_widget::list::{List, ListState};
use rat_widget::menuline::{MenuLine, MenuLineState, MenuOutcome};
use rat_widget::statusline::StatusLineState;
use rat_widget::tabbed::attached::AttachedTabs;
use rat_widget::tabbed::glued::GluedTabs;
use rat_widget::tabbed::{TabPlacement, Tabbed, TabbedState};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Line;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, BorderType, StatefulWidget, Widget};
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        border_type: None,
        placement: Default::default(),
        style: 0,
        close: false,
        tabbed: Default::default(),
        tabs: Default::default(),
        menu: Default::default(),
        status: Default::default(),
    };
    state.menu.focus.set(true);

    run_ui(handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    border_type: Option<BorderType>,
    placement: TabPlacement,
    style: usize,
    close: bool,

    tabbed: TabbedState,
    tabs: [ListState<RowSelection>; 3],

    menu: MenuLineState,
    status: StatusLineState,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    let l2 = Layout::horizontal([
        Constraint::Length(25),
        Constraint::Fill(1),
        Constraint::Length(15),
    ])
    .split(l1[1]);

    let mut tab = Tabbed::new();
    tab = match state.style {
        0 => tab.tab_type(GluedTabs::new().placement(state.placement)),
        1 => tab.tab_type(
            AttachedTabs::new()
                .placement(state.placement)
                .join(state.border_type.unwrap_or_default()),
        ),
        _ => unreachable!(),
    };
    tab = tab
        .style(THEME.black(2))
        .select_style(THEME.orange(2))
        .tab_style(THEME.limegreen(0));
    if state.close {
        tab = tab.closeable(true);
    }
    if let Some(border_type) = state.border_type {
        tab = tab.block(Block::bordered().border_type(border_type));
    }
    tab = tab.tabs(["Tabbed 1", "Tabbed 2", "Tabbed 3"]);
    tab.render(l2[1], frame.buffer_mut(), &mut state.tabbed);

    match state.tabbed.selected {
        0 => {
            List::<RowSelection>::new([
                "L-0", "L-1", "L-2", "L-3", "L-4", "L-5", "L-6", "L-7", "L-8", "L-9", //
                "L-10", "L-11", "L-12", "L-13", "L-14", "L-15", "L-16", "L-17", "L-18",
                "L-19", //
                "L-20", "L-21", "L-22", "L-23", "L-24", "L-25", "L-26", "L-27", "L-28",
                "L-29", //
            ])
            .style(THEME.gray(3))
            .scroll(Scroll::new().styles(THEME.scrolled_style()))
            .render(
                state.tabbed.inner_area,
                frame.buffer_mut(),
                &mut state.tabs[0],
            );
        }
        1 => {
            List::<RowSelection>::new([
                "R-0", "R-1", "R-2", "R-3", "R-4", "R-5", "R-6", "R-7", "R-8", "R-9", //
                "R-10", "R-11", "R-12", "R-13", "R-14", "R-15", "R-16", "R-17", "R-18",
                "R-19", //
                "R-20", "R-21", "R-22", "R-23", "R-24", "R-25", "R-26", "R-27", "R-28",
                "R-29", //
            ])
            .style(THEME.gray(3))
            .block(Block::bordered().style(THEME.block()))
            .scroll(Scroll::new().styles(THEME.scrolled_style()))
            .render(
                state.tabbed.inner_area,
                frame.buffer_mut(),
                &mut state.tabs[1],
            );
        }
        2 => "nothing".render(state.tabbed.inner_area, frame.buffer_mut()),
        _ => {}
    }

    let mut area = Rect::new(l2[0].x, l2[0].y, l2[0].width, 1);

    Line::from("F1: close")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F2: type")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F3: alignment")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F5: border")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F12: key-nav")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    area.y += 1;

    let menu1 = MenuLine::new()
        .title("||||")
        .add_str("_Quit")
        .title_style(Style::default().black().on_yellow())
        .style(Style::default().black().on_dark_gray());
    frame.render_stateful_widget(menu1, l1[3], &mut state.menu);

    Ok(())
}

fn focus(state: &State) -> Focus {
    let mut f = Focus::default();
    f.add(&state.tabbed);
    f.add(&state.tabs[state.tabbed.selected]);
    f.add(&state.menu);
    f
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let f = focus(state).handle(event, Regular);

    flow_ok!(match event {
        ct_event!(keycode press F(1)) => {
            state.close = !state.close;
            Outcome::Changed
        }
        ct_event!(keycode press F(2)) => {
            state.style = match state.style {
                0 => 1,
                1 => 0,
                _ => 0,
            };
            Outcome::Changed
        }
        ct_event!(keycode press SHIFT-F(2)) => {
            state.style = match state.style {
                0 => 1,
                1 => 0,
                _ => 0,
            };
            Outcome::Changed
        }
        ct_event!(keycode press F(3)) => {
            state.placement = match state.placement {
                TabPlacement::Top => TabPlacement::Right,
                TabPlacement::Right => TabPlacement::Bottom,
                TabPlacement::Bottom => TabPlacement::Left,
                TabPlacement::Left => TabPlacement::Top,
            };
            Outcome::Changed
        }
        ct_event!(keycode press SHIFT-F(3)) => {
            state.placement = match state.placement {
                TabPlacement::Top => TabPlacement::Left,
                TabPlacement::Right => TabPlacement::Top,
                TabPlacement::Bottom => TabPlacement::Right,
                TabPlacement::Left => TabPlacement::Bottom,
            };
            Outcome::Changed
        }
        ct_event!(keycode press F(5)) => {
            state.border_type = match state.border_type {
                None => Some(BorderType::Plain),
                Some(BorderType::Plain) => Some(BorderType::Double),
                Some(BorderType::Double) => Some(BorderType::Rounded),
                Some(BorderType::Rounded) => Some(BorderType::Thick),
                Some(BorderType::Thick) => Some(BorderType::QuadrantInside),
                Some(BorderType::QuadrantInside) => Some(BorderType::QuadrantOutside),
                Some(BorderType::QuadrantOutside) => None,
            };
            Outcome::Changed
        }
        ct_event!(keycode press SHIFT-F(5)) => {
            state.border_type = match state.border_type {
                None => Some(BorderType::QuadrantOutside),
                Some(BorderType::Plain) => None,
                Some(BorderType::Double) => Some(BorderType::Plain),
                Some(BorderType::Rounded) => Some(BorderType::Double),
                Some(BorderType::Thick) => Some(BorderType::Rounded),
                Some(BorderType::QuadrantInside) => Some(BorderType::Thick),
                Some(BorderType::QuadrantOutside) => Some(BorderType::QuadrantInside),
            };
            Outcome::Changed
        }
        _ => Outcome::NotUsed,
    });

    flow_ok!(HandleEvent::handle(&mut state.tabbed, event, Regular), consider f);
    match state.tabbed.selected {
        0 => flow_ok!(state.tabs[0].handle(event, Regular), consider f),
        1 => flow_ok!(state.tabs[1].handle(event, Regular), consider f),
        _ => {}
    }
    flow_ok!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            istate.quit = true;
            Outcome::Changed
        }
        _ => {
            Outcome::NotUsed
        }
    }, consider f);

    Ok(f)
}
