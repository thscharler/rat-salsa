#![allow(dead_code)]
use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
#[allow(unused_imports)]
use log::debug;
use rat_event::{ct_event, try_flow, ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder, HasFocusFlag};
use rat_scrolled::Scroll;
use rat_widget::event::Outcome;
use rat_widget::list::selection::RowSelection;
use rat_widget::list::{List, ListState};
use rat_widget::menuline::{MenuLine, MenuLineState, MenuOutcome};
use rat_widget::splitter::{Split, SplitResize, SplitState, SplitType};
use rat_widget::statusline::StatusLineState;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, StatefulWidget, Widget};
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        dir: Direction::Horizontal,
        split_type: Default::default(),
        border_type: None,
        inner_border_type: None,
        resize: Default::default(),
        split: Default::default(),
        left: Default::default(),
        right: Default::default(),
        rightright: Default::default(),
        menu: Default::default(),
        status: Default::default(),
    };
    state.menu.focus.set(true);

    run_ui(handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    pub(crate) dir: Direction,
    pub(crate) split_type: SplitType,
    pub(crate) border_type: Option<BorderType>,
    pub(crate) inner_border_type: Option<BorderType>,
    pub(crate) resize: SplitResize,
    pub(crate) split: SplitState,
    pub(crate) left: ListState<RowSelection>,
    pub(crate) right: ListState<RowSelection>,
    pub(crate) rightright: ListState<RowSelection>,
    pub(crate) menu: MenuLineState,
    pub(crate) status: StatusLineState,
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

    let mut split = Split::new()
        .styles(THEME.split_style(state.split_type))
        .direction(state.dir)
        .split_type(state.split_type)
        .resize(state.resize)
        .mark_offset(1)
        .constraints([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]);
    if let Some(blk) = state.border_type {
        split = split.block(
            Block::bordered()
                .border_type(blk)
                .border_style(THEME.gray(3)),
        );
        split = split.join_1(blk);
        split = split.join_0(blk);
    }
    let (split, split_overlay) = split.into_widgets();
    split.render(l2[1], frame.buffer_mut(), &mut state.split);

    let mut w_left = List::<RowSelection>::new([
        "L-0", "L-1", "L-2", "L-3", "L-4", "L-5", "L-6", "L-7", "L-8", "L-9", //
        "L-10", "L-11", "L-12", "L-13", "L-14", "L-15", "L-16", "L-17", "L-18", "L-19", //
        "L-20", "L-21", "L-22", "L-23", "L-24", "L-25", "L-26", "L-27", "L-28", "L-29", //
    ])
    .style(THEME.gray(3));
    match state.split_type {
        SplitType::FullEmpty
        | SplitType::FullPlain
        | SplitType::FullDouble
        | SplitType::FullThick
        | SplitType::FullQuadrantInside
        | SplitType::FullQuadrantOutside => {}
        SplitType::Scroll => {
            if let Some(inner_border) = state.inner_border_type {
                w_left = w_left.block(
                    Block::bordered()
                        .title("inner block")
                        .border_style(THEME.gray(3))
                        .border_type(inner_border),
                );
            }
            let mut scroll_left = Scroll::new().styles(THEME.scrolled_style());
            if state.dir == Direction::Horizontal {
                scroll_left = scroll_left.start_margin(2);
            }
            w_left = w_left.scroll(scroll_left);
        }
        SplitType::Widget => {
            if let Some(inner_border) = state.inner_border_type {
                w_left = w_left.block(
                    Block::bordered()
                        .title("inner block")
                        .border_style(THEME.gray(3))
                        .border_type(inner_border),
                );
            }
        }
    }
    w_left.render(
        state.split.widget_areas[0],
        frame.buffer_mut(),
        &mut state.left,
    );

    let w_right = List::<RowSelection>::new([
        "R-0", "R-1", "R-2", "R-3", "R-4", "R-5", "R-6", "R-7", "R-8", "R-9", //
        "R-10", "R-11", "R-12", "R-13", "R-14", "R-15", "R-16", "R-17", "R-18", "R-19", //
        "R-20", "R-21", "R-22", "R-23", "R-24", "R-25", "R-26", "R-27", "R-28", "R-29", //
    ])
    .style(THEME.gray(3));
    w_right.render(
        state.split.widget_areas[1],
        frame.buffer_mut(),
        &mut state.right,
    );

    let w_right2 = List::<RowSelection>::new([
        "W-0", "W-1", "W-2", "W-3", "W-4", "W-5", "W-6", "W-7", "W-8", "W-9", //
        "W-10", "W-11", "W-12", "W-13", "W-14", "W-15", "W-16", "W-17", "W-18", "W-19", //
        "W-20", "W-21", "W-22", "W-23", "W-24", "W-25", "W-26", "W-27", "W-28", "W-29", //
    ])
    .style(THEME.gray(3));
    w_right2.render(
        state.split.widget_areas[2],
        frame.buffer_mut(),
        &mut state.rightright,
    );

    // There might be an overlay, if the stars are right.
    split_overlay.render(l2[1], frame.buffer_mut(), &mut state.split);

    let mut area = Rect::new(l2[0].x, l2[0].y, l2[0].width, 1);

    Line::from("F1: horizontal")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F2: vertical")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F3: toggle")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F4: type")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F5: border")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F6: left border")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F7: resize")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F12: key-nav")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    area.y += 1;

    Line::from(format!(
        "area {},{}+{}+{}",
        state.split.inner.x, state.split.inner.y, state.split.inner.width, state.split.inner.height
    ))
    .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("areas").render(area, frame.buffer_mut());
    area.y += 1;
    for a in &state.split.widget_areas {
        Line::from(format!("{},{}+{}+{}", a.x, a.y, a.width, a.height))
            .render(area, frame.buffer_mut());
        area.y += 1;
    }
    // area.y += 1;
    // Line::from("split").render(area, frame.buffer_mut());
    // area.y += 1;
    // for s in &state.split.splitline_areas {
    //     Line::from(format!("{},{}+{}+{}", s.x, s.y, s.width, s.height))
    //         .render(area, frame.buffer_mut());
    //     area.y += 1;
    // }
    // Line::from("mark").render(area, frame.buffer_mut());
    // area.y += 1;
    // for s in &state.split.splitline_mark_position {
    //     Line::from(format!("{},{}", s.x, s.y)).render(area, frame.buffer_mut());
    //     area.y += 1;
    // }

    use std::fmt::Write;
    let txt = state
        .split
        .area_lengths()
        .iter()
        .fold(String::from("Length "), |mut v, w| {
            _ = write!(v, "{}, ", *w);
            v
        });
    Line::from(txt).render(area, frame.buffer_mut());
    area.y += 1;
    Line::from(format!("Drag {:?}", state.split.mouse.drag.get())).render(area, frame.buffer_mut());
    area.y += 1;
    Line::from(format!("{:?}", state.split.split_type)).render(area, frame.buffer_mut());
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
    let mut fb = FocusBuilder::default();
    fb.widget(&state.split)
        .widget(&state.left)
        .widget(&state.right)
        .widget(&state.rightright)
        .widget(&state.menu);
    fb.build()
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(match event {
        ct_event!(keycode press F(1)) => {
            state.dir = Direction::Horizontal;
            Outcome::Changed
        }
        ct_event!(keycode press F(2)) => {
            state.dir = Direction::Vertical;
            Outcome::Changed
        }
        ct_event!(keycode press F(3)) => {
            if state.dir == Direction::Horizontal {
                state.dir = Direction::Vertical;
            } else {
                state.dir = Direction::Horizontal;
            }
            Outcome::Changed
        }

        ct_event!(keycode press F(4)) => {
            state.split_type = match state.split_type {
                SplitType::FullEmpty => SplitType::Scroll,
                SplitType::Scroll => SplitType::Widget,
                SplitType::Widget => SplitType::FullPlain,
                SplitType::FullPlain => SplitType::FullDouble,
                SplitType::FullDouble => SplitType::FullThick,
                SplitType::FullThick => SplitType::FullQuadrantInside,
                SplitType::FullQuadrantInside => SplitType::FullQuadrantOutside,
                SplitType::FullQuadrantOutside => SplitType::FullEmpty,
            };
            Outcome::Changed
        }
        ct_event!(keycode press SHIFT-F(4)) => {
            state.split_type = match state.split_type {
                SplitType::FullEmpty => SplitType::FullQuadrantOutside,
                SplitType::Scroll => SplitType::FullEmpty,
                SplitType::Widget => SplitType::Scroll,
                SplitType::FullPlain => SplitType::Widget,
                SplitType::FullDouble => SplitType::FullPlain,
                SplitType::FullThick => SplitType::FullDouble,
                SplitType::FullQuadrantInside => SplitType::FullThick,
                SplitType::FullQuadrantOutside => SplitType::FullQuadrantInside,
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
        ct_event!(keycode press F(6)) => {
            state.inner_border_type = match state.inner_border_type {
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
        ct_event!(keycode press SHIFT-F(6)) => {
            state.inner_border_type = match state.inner_border_type {
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
        ct_event!(keycode press F(7)) => {
            state.resize = match state.resize {
                SplitResize::Neighbours => SplitResize::Full,
                SplitResize::Full => SplitResize::Neighbours,
            };
            Outcome::Changed
        }
        ct_event!(keycode press F(12)) => {
            if state.split.is_focused() {
                state.split.focus.set(false);
            } else {
                state.split.focus.set(true);
            }
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    let f = focus(state).handle(event, Regular);
    let r = f.and_try(|| {
        try_flow!(HandleEvent::handle(&mut state.split, event, Regular));
        try_flow!(match state.left.handle(event, Regular) {
            Outcome::Changed => {
                debug!("sel left {:?}", state.left.selected());
                Outcome::Changed
            }
            r => r,
        });
        try_flow!(state.right.handle(event, Regular));
        try_flow!(state.rightright.handle(event, Regular));
        try_flow!(match state.menu.handle(event, Regular) {
            MenuOutcome::Activated(0) => {
                istate.quit = true;
                Outcome::Changed
            }
            _ => {
                Outcome::Continue
            }
        });
        Ok(Outcome::Continue)
    });
    r
}
