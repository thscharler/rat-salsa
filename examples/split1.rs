use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
#[allow(unused_imports)]
use log::debug;
use rat_event::{ct_event, flow_ok, FocusKeys, HandleEvent, MouseOnly};
use rat_widget::event::Outcome;
use rat_widget::menuline::{MenuLine, MenuLineState};
use rat_widget::splitter::{Split, SplitState, SplitStyle, SplitWidget};
use rat_widget::statusline::StatusLineState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{StatefulWidget, Widget};
use ratatui::Frame;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        dir: Direction::Horizontal,
        minimal: Default::default(),
        split: Default::default(),
        menu: Default::default(),
        status: Default::default(),
    };
    state.menu.focus.set(true);

    run_ui(handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    pub(crate) dir: Direction,
    pub(crate) minimal: SplitStyle,
    pub(crate) split: SplitState<()>,
    pub(crate) menu: MenuLineState,
    pub(crate) status: StatusLineState,
}

struct Split1;

impl SplitWidget for Split1 {
    type State = ();

    fn render(&self, n: usize, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        match n {
            0 => Line::from("LEFT")
                .style(Style::default().on_dark_gray())
                .render(area, buf),
            1 => Line::from("RIGHT")
                .style(Style::default().on_dark_gray())
                .render(area, buf),
            _ => {}
        }
    }
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
    ])
    .split(area);

    let l2 = Layout::horizontal([
        Constraint::Length(25),
        Constraint::Fill(1),
        Constraint::Length(15),
    ])
    .split(l1[1]);

    Split::new(Split1)
        .direction(state.dir)
        .render_split(state.minimal)
        .constraints([Constraint::Fill(1), Constraint::Fill(1)])
        .style(Style::default().on_blue())
        .drag_style(Style::default().on_light_blue())
        .render(l2[1], frame.buffer_mut(), &mut state.split);

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
    Line::from("F4: minimize")
        .yellow()
        .render(area, frame.buffer_mut());
    area.y += 1;
    Line::from("F5: key-nav")
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
    for a in &state.split.areas {
        Line::from(format!("{},{}+{}+{}", a.x, a.y, a.width, a.height))
            .render(area, frame.buffer_mut());
        area.y += 1;
    }
    area.y += 1;
    Line::from("split").render(area, frame.buffer_mut());
    area.y += 1;
    for s in &state.split.split {
        Line::from(format!("{},{}+{}+{}", s.x, s.y, s.width, s.height))
            .render(area, frame.buffer_mut());
        area.y += 1;
    }
    Line::from(format!("drag {:?}", state.split.mouse.drag.get())).render(area, frame.buffer_mut());
    area.y += 1;

    let menu1 = MenuLine::new()
        .title("||||")
        .add_str("_Quit")
        .title_style(Style::default().black().on_yellow())
        .style(Style::default().black().on_dark_gray());
    frame.render_stateful_widget(menu1, l1[2], &mut state.menu);

    Ok(())
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    flow_ok!(match event {
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
            state.split = Default::default();
            state.minimal = match state.minimal {
                SplitStyle::Full => SplitStyle::Minimal,
                SplitStyle::Minimal => SplitStyle::None,
                SplitStyle::None => SplitStyle::Full,
            };
            Outcome::Changed
        }

        ct_event!(keycode press F(5)) => {
            if state.split.key_nav.is_none() {
                state.split.key_nav = Some(0);
            } else {
                state.split.key_nav = None;
            }
            Outcome::Changed
        }
        _ => Outcome::NotUsed,
    });

    flow_ok!(HandleEvent::handle(&mut state.split, event, MouseOnly));

    flow_ok!(match state.menu.handle(event, FocusKeys) {
        _ => {
            Outcome::NotUsed
        }
    });

    Ok(Outcome::NotUsed)
}
