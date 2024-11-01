use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{layout_grid, run_ui, setup_logging, MiniSalsaState};
use map_range_int::MapRange;
use rat_event::{ct_event, ConsumedEvent, HandleEvent, Regular};
use rat_focus::{Focus, FocusBuilder};
use rat_menu::event::MenuOutcome;
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_widget::event::Outcome;
use rat_widget::range_op::RangeOp;
use rat_widget::slider::{Slider, SliderState};
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{StatefulWidget, Widget};
use ratatui::Frame;
use std::cmp::max;

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        direction: Direction::Vertical,
        alignment: Alignment::Center,
        v_width: 1,
        c1: SliderState::<u8>::new(),
        c2: SliderState::<EnumSlide>::new_range((EnumSlide::A, EnumSlide::K), 1),
        menu: MenuLineState::named("menu"),
    };
    state.c1.set_value(Some(0));
    state.c1.set_long_step(10);

    state.c2.set_value(Some(EnumSlide::C));

    run_ui(
        "slider1",
        handle_input,
        repaint_input,
        &mut data,
        &mut state,
    )
}

struct Data {}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum EnumSlide {
    #[default]
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
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
        match value {
            EnumSlide::A => 0,
            EnumSlide::B => 1,
            EnumSlide::C => 2,
            EnumSlide::D => 3,
            EnumSlide::E => 4,
            EnumSlide::F => 5,
            EnumSlide::G => 6,
            EnumSlide::H => 7,
            EnumSlide::I => 8,
            EnumSlide::J => 9,
            EnumSlide::K => 10,
        }
    }
}

impl MapRange<u16> for EnumSlide {
    fn map_range(self, range: (Self, Self), o_range: (u16, u16)) -> Option<u16> {
        let v = u8::from(self);
        let l = u8::from(range.0);
        let u = u8::from(range.1);
        v.map_range((l, u), o_range)
    }

    fn map_range_unchecked(self, range: (Self, Self), o_range: (u16, u16)) -> u16 {
        let v = u8::from(self);
        let l = u8::from(range.0);
        let u = u8::from(range.1);
        v.map_range_unchecked((l, u), o_range)
    }
}

impl MapRange<EnumSlide> for u16 {
    fn map_range(self, range: (Self, Self), o_range: (EnumSlide, EnumSlide)) -> Option<EnumSlide> {
        let l = u8::from(o_range.0);
        let u = u8::from(o_range.1);
        let o = self.map_range(range, (l, u));
        o.map(|v| EnumSlide::from(v))
    }

    fn map_range_unchecked(
        self,
        range: (Self, Self),
        o_range: (EnumSlide, EnumSlide),
    ) -> EnumSlide {
        let l = u8::from(o_range.0);
        let u = u8::from(o_range.1);
        let o = self.map_range_unchecked(range, (l, u));
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
    menu: MenuLineState,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

    let lg = layout_grid::<3, 3>(
        l1[0],
        Layout::horizontal([
            Constraint::Length(21), //
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
        .style(THEME.text_input())
        .knob_style(THEME.secondary(1))
        .bounds_style(THEME.gray(2))
        .focus_style(THEME.focus())
        .lower_bound_str(a)
        .upper_bound_str(b)
        .direction(state.direction)
        .align_bounds(state.alignment)
        .render(slider_area, frame.buffer_mut(), &mut state.c1);

    let mut slider_area = lg[2][1];
    slider_area.height = 1;
    let knob = format!("||{:?}||", state.c2.value.expect("some"));
    Slider::new()
        .style(THEME.text_input())
        .knob_style(THEME.secondary(1))
        .bounds_style(THEME.gray(2))
        .focus_style(THEME.focus())
        .lower_bound_str(">")
        .upper_bound_str("<")
        .knob_str(knob)
        .direction(Direction::Horizontal)
        .render(slider_area, frame.buffer_mut(), &mut state.c2);

    Span::from(format!("{:?} of {:?}", state.c1.value, state.c1.range))
        .render(lg[0][1], frame.buffer_mut());

    let menu1 = MenuLine::new()
        .title("~~~ swoosh ~~~")
        .item_parsed("_Quit")
        .styles(THEME.menu_style());
    frame.render_stateful_widget(menu1, l1[1], &mut state.menu);

    Ok(())
}

fn focus(state: &mut State) -> Focus {
    let mut fb = FocusBuilder::new(None);
    fb.widget(&state.menu);
    fb.widget(&state.c1);
    fb.widget(&state.c2);
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
    let f = focus.handle(event, Regular);

    let r = Outcome::Continue;
    let r = r.or_else(|| state.c1.handle(event, Regular));
    let r = r.or_else(|| state.c2.handle(event, Regular));

    let r = r.or_else(|| match event {
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

    let r = r.or_else(|| match state.menu.handle(event, Regular) {
        MenuOutcome::Activated(v) => {
            match v {
                0 => {
                    istate.quit = true;
                    return Outcome::Changed;
                }
                _ => {}
            }
            Outcome::Changed
        }
        r => r.into(),
    });

    Ok(max(f, r))
}
