use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use crate::substratum1::{Substratum, SubstratumState};
use rat_event::{ConsumedEvent, HandleEvent, Outcome, Regular};
use rat_focus::{Focus, HasFocus};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Block;
use ratatui::Frame;

mod adapter;
mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut data = Data {};

    let mut state = State {
        sub1: Default::default(),
        sub2: Default::default(),
        sub3: Default::default(),
        sub4: Default::default(),
    };
    focus_input(&mut state).next();

    run_ui(handle_input, repaint_input, &mut data, &mut state)
}

struct Data {}

struct State {
    pub(crate) sub1: SubstratumState,
    pub(crate) sub2: SubstratumState,
    pub(crate) sub3: SubstratumState,
    pub(crate) sub4: SubstratumState,
}

fn repaint_input(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([
        Constraint::Length(25),
        Constraint::Length(25),
        Constraint::Fill(1),
    ])
    .split(area);

    let l00 = Layout::vertical([Constraint::Length(8), Constraint::Length(8)]).split(l0[0]);

    let w1 = Substratum::new().block(Block::bordered().title("First"));
    frame.render_stateful_widget(w1, l00[0], &mut state.sub1);
    let w2 = Substratum::new().block(Block::bordered().title("Second"));
    frame.render_stateful_widget(w2, l00[1], &mut state.sub2);

    let l11 = Layout::vertical([Constraint::Length(8), Constraint::Length(8)]).split(l0[1]);

    let w3 = Substratum::new().block(Block::bordered().title("Third"));
    frame.render_stateful_widget(w3, l11[0], &mut state.sub3);
    let w4 = Substratum::new().block(Block::bordered().title("Forth"));
    frame.render_stateful_widget(w4, l11[1], &mut state.sub4);

    let cursor = state.sub1.screen_cursor().or_else(|| {
        state.sub2.screen_cursor().or_else(|| {
            state
                .sub3
                .screen_cursor()
                .or_else(|| state.sub4.screen_cursor())
        })
    });
    if let Some((x, y)) = cursor {
        frame.set_cursor_position((x, y));
    }

    Ok(())
}

fn focus_input(state: &mut State) -> Focus {
    let mut f = Focus::default();
    f.add_focus(state.sub1.focus());
    f.add_focus(state.sub2.focus());
    f.add_focus(state.sub3.focus());
    f.add_focus(state.sub4.focus());
    f
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let f = focus_input(state).handle(event, Regular);
    let f = f.and(|| {
        (state.sub1.handle(event, Regular))
            .or_else(|| state.sub2.handle(event, Regular))
            .or_else(|| state.sub3.handle(event, Regular))
            .or_else(|| state.sub4.handle(event, Regular))
    });
    Ok(f)
}

pub mod substratum1 {
    use crate::adapter::textinputf::{TextInputF, TextInputFState};
    use crate::mini_salsa::layout_grid;
    use crate::mini_salsa::theme::THEME;
    use rat_event::{ConsumedEvent, HandleEvent, Outcome, Regular};
    use rat_focus::{ContainerFlag, Focus, HasFocus, HasFocusFlag};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Layout, Rect};
    use ratatui::prelude::{BlockExt, Span, StatefulWidget, Style};
    use ratatui::widgets::{Block, Widget};

    #[derive(Debug, Default)]
    pub struct Substratum<'a> {
        block: Option<Block<'a>>,
    }

    impl<'a> Substratum<'a> {
        pub fn new() -> Self {
            Self {
                block: Default::default(),
            }
        }

        pub fn block(mut self, block: Block<'a>) -> Self {
            self.block = Some(block);
            self
        }
    }

    #[derive(Debug, Default)]
    pub struct SubstratumState {
        pub focus: ContainerFlag,
        pub area: Rect,
        pub input1: TextInputFState,
        pub input2: TextInputFState,
        pub input3: TextInputFState,
        pub input4: TextInputFState,
    }

    impl<'a> StatefulWidget for Substratum<'a> {
        type State = SubstratumState;

        fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            let inner = self.block.inner_if_some(area);
            state.area = area;

            self.block = if state.focus.get() {
                if let Some(block) = self.block {
                    Some(block.border_style(Style::default().fg(THEME.secondary[3])))
                } else {
                    self.block
                }
            } else {
                self.block
            };

            self.block.render(area, buf);

            let l_grid = layout_grid::<2, 4>(
                inner,
                Layout::horizontal([Constraint::Length(10), Constraint::Length(20)]),
                Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]),
            );

            Span::from("Text 1").render(l_grid[0][0], buf);
            let input1 = TextInputF::default()
                .style(THEME.text_input())
                .focus_style(THEME.text_input_focus());
            input1.render(l_grid[1][0], buf, &mut state.input1);

            Span::from("Text 2").render(l_grid[0][1], buf);
            let input2 = TextInputF::default()
                .style(THEME.text_input())
                .focus_style(THEME.text_input_focus());
            input2.render(l_grid[1][1], buf, &mut state.input2);

            Span::from("Text 3").render(l_grid[0][2], buf);
            let input3 = TextInputF::default()
                .style(THEME.text_input())
                .focus_style(THEME.text_input_focus());
            input3.render(l_grid[1][2], buf, &mut state.input3);

            Span::from("Text 4").render(l_grid[0][3], buf);
            let input4 = TextInputF::default()
                .style(THEME.text_input())
                .focus_style(THEME.text_input_focus());
            input4.render(l_grid[1][3], buf, &mut state.input4);
        }
    }

    impl SubstratumState {
        pub fn screen_cursor(&self) -> Option<(u16, u16)> {
            if self.input1.is_focused() {
                self.input1.screen_cursor()
            } else if self.input2.is_focused() {
                self.input2.screen_cursor()
            } else if self.input3.is_focused() {
                self.input3.screen_cursor()
            } else if self.input4.is_focused() {
                self.input4.screen_cursor()
            } else {
                None
            }
        }
    }

    impl HasFocus for SubstratumState {
        fn focus(&self) -> Focus {
            Focus::new_container_list(
                self.focus.clone(),
                self.area,
                &[&self.input1, &self.input2, &self.input3, &self.input4],
            )
        }

        fn container(&self) -> Option<ContainerFlag> {
            Some(self.focus.clone())
        }

        fn area(&self) -> Rect {
            self.area
        }
    }

    impl HandleEvent<crossterm::event::Event, Regular, Outcome> for SubstratumState {
        fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
            (self.input1.handle(event, Regular))
                .or_else(|| self.input2.handle(event, Regular))
                .or_else(|| self.input3.handle(event, Regular))
                .or_else(|| self.input4.handle(event, Regular))
        }
    }
}
