use crate::mini_salsa::{MiniSalsaState, layout_grid, run_ui, setup_logging};
use map_range_int::MapRange;
use rat_event::{HandleEvent, Regular, ct_event, try_flow};
use rat_focus::{Focus, FocusBuilder};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_widget::event::Outcome;
use rat_widget::range_op::RangeOp;
use rat_widget::slider::{Slider, SliderState};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, StatefulWidget, Widget};

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        direction: Direction::Vertical,
        alignment: Alignment::Center,
        v_width: 3,
        c1: SliderState::<u8>::new(),
        c2: SliderState::<EnumSlide>::new_range((EnumSlide::A, EnumSlide::K), 1),
        r: SliderState::default(),
        g: SliderState::default(),
        b: SliderState::default(),
        menu: MenuLineState::named("menu"),
    };
    state.c1.set_value(0);
    state.c1.set_long_step(10);

    state.c2.set_value(EnumSlide::C);

    run_ui(
        "slider1",
        |_, _, _| {},
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
enum EnumSlide {
    #[default]
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
    I = 8,
    J = 9,
    K = 10,
}

impl From<u8> for EnumSlide {
    fn from(value: u8) -> Self {
        match value {
            0 => EnumSlide::A,
            1 => EnumSlide::B,
            2 => EnumSlide::C,
            3 => EnumSlide::D,
            4 => EnumSlide::E,
            5 => EnumSlide::F,
            6 => EnumSlide::G,
            7 => EnumSlide::H,
            8 => EnumSlide::I,
            9 => EnumSlide::J,
            10 => EnumSlide::K,
            _ => panic!("oob"),
        }
    }
}

impl From<EnumSlide> for u8 {
    fn from(value: EnumSlide) -> Self {
        value as u8
    }
}

impl MapRange<u16> for EnumSlide {
    fn map_range_unchecked(self, bounds: (Self, Self), o_range: (u16, u16)) -> u16 {
        let v = u8::from(self);
        let l = u8::from(bounds.0);
        let u = u8::from(bounds.1);
        v.map_range_unchecked((l, u), o_range)
    }
}

impl MapRange<EnumSlide> for u16 {
    fn map_range_unchecked(
        self,
        bounds: (Self, Self),
        o_range: (EnumSlide, EnumSlide),
    ) -> EnumSlide {
        let l = u8::from(o_range.0);
        let u = u8::from(o_range.1);
        let o = self.map_range_unchecked(bounds, (l, u));
        EnumSlide::from(o)
    }
}

impl RangeOp for EnumSlide {
    type Step = u8;

    fn add_clamp(self, delta: Self::Step, bounds: (Self, Self)) -> Self {
        let v = u8::from(self);
        let l = u8::from(bounds.0);
        let u = u8::from(bounds.1);

        let v2 = v.add_clamp(delta, (l, u));

        Self::from(v2)
    }

    fn sub_clamp(self, delta: Self::Step, bounds: (Self, Self)) -> Self {
        let v = u8::from(self);
        let l = u8::from(bounds.0);
        let u = u8::from(bounds.1);

        let v2 = v.sub_clamp(delta, (l, u));

        Self::from(v2)
    }
}

struct State {
    direction: Direction,
    alignment: Alignment,
    v_width: u16,

    c1: SliderState<u8>,
    c2: SliderState<EnumSlide>,

    r: SliderState<u8>,
    g: SliderState<u8>,
    b: SliderState<u8>,

    menu: MenuLineState,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    let lg = layout_grid::<4, 3>(
        l1[0],
        Layout::horizontal([
            Constraint::Length(21), //
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Length(20),
        ])
        .spacing(1)
        .flex(Flex::Start),
        Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(20),
            Constraint::Fill(1),
        ])
        .spacing(1),
    );

    let (a, b) = match state.direction {
        Direction::Horizontal => ("A[", "]B"),
        Direction::Vertical => ("Arbitrary\nBound\nHere", "Limit\nis\nthis"),
    };

    let mut slider_area = lg[1][1];
    match state.direction {
        Direction::Horizontal => {
            slider_area.height = state.v_width;
        }
        Direction::Vertical => {
            slider_area.width = state.v_width;
        }
    }
    Slider::new()
        .styles(istate.theme.slider_style())
        .lower_bound(a)
        .upper_bound(b)
        .direction(state.direction)
        .text_align(state.alignment)
        .render(slider_area, frame.buffer_mut(), &mut state.c1);

    let mut slider_area = lg[2][1];
    slider_area.height = 3;
    let knob = format!("||{:?}||", state.c2.value);
    Slider::new()
        .styles(istate.theme.slider_style())
        .track_char("-")
        .horizontal_knob(knob)
        .block(Block::bordered().border_type(BorderType::Rounded))
        .direction(Direction::Horizontal)
        .render(slider_area, frame.buffer_mut(), &mut state.c2);

    Span::from(format!("{:?} of {:?}", state.c1.value, state.c1.range))
        .render(lg[0][1], frame.buffer_mut());

    let mut slider_area = lg[3][1];
    slider_area.height = 1;
    Slider::new()
        .styles(istate.theme.slider_style())
        .direction(Direction::Horizontal)
        .horizontal_knob("|")
        .long_step(16)
        .lower_bound(format!("R {:02x} ", state.r.value()))
        .upper_bound("")
        .render(slider_area, frame.buffer_mut(), &mut state.r);
    slider_area.y += 1;
    Slider::new()
        .styles(istate.theme.slider_style())
        .direction(Direction::Horizontal)
        .horizontal_knob("|")
        .long_step(16)
        .lower_bound(format!("G {:02x} ", state.g.value()))
        .upper_bound("")
        .render(slider_area, frame.buffer_mut(), &mut state.g);
    slider_area.y += 1;
    Slider::new()
        .styles(istate.theme.slider_style())
        .direction(Direction::Horizontal)
        .horizontal_knob("|")
        .long_step(16)
        .lower_bound(format!("B {:02x} ", state.b.value()))
        .upper_bound("")
        .render(slider_area, frame.buffer_mut(), &mut state.b);

    let menu1 = MenuLine::new()
        .title("~~~ swoosh ~~~")
        .item_parsed("_Quit")
        .styles(istate.theme.menu_style());
    frame.render_stateful_widget(menu1, l1[1], &mut state.menu);

    Ok(())
}

fn focus(state: &mut State) -> Focus {
    let mut fb = FocusBuilder::new(None);
    fb.widget(&state.menu);
    fb.widget(&state.c1);
    fb.widget(&state.c2);
    fb.widget(&state.r);
    fb.widget(&state.g);
    fb.widget(&state.b);
    let f = fb.build();
    f.enable_log();
    f
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let mut focus = focus(state);

    istate.focus_outcome = focus.handle(event, Regular);

    try_flow!(state.c1.handle(event, Regular));
    try_flow!(state.c2.handle(event, Regular));
    try_flow!(state.r.handle(event, Regular));
    try_flow!(state.g.handle(event, Regular));
    try_flow!(state.b.handle(event, Regular));

    try_flow!(match event {
        ct_event!(keycode press F(2)) => {
            state.direction = match state.direction {
                Direction::Horizontal => Direction::Vertical,
                Direction::Vertical => Direction::Horizontal,
            };
            Outcome::Changed
        }
        ct_event!(keycode press F(3)) => {
            state.alignment = match state.alignment {
                Alignment::Left => Alignment::Center,
                Alignment::Center => Alignment::Right,
                Alignment::Right => Alignment::Left,
            };
            Outcome::Changed
        }
        ct_event!(keycode press F(1)) => {
            state.v_width = match state.v_width {
                v if v < 15 => v + 1,
                _ => 1,
            };
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    try_flow!(match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(v) => {
            match v {
                0 => {
                    istate.quit = true;
                    Outcome::Changed
                }
                _ => Outcome::Changed,
            }
        }
        r => r.into(),
    });

    Ok(Outcome::Continue)
}
