use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use crate::substratum1::{Substratum, SubstratumState};
use crate::substratum2::{Substratum2, Substratum2State};
use rat_event::{ConsumedEvent, HandleEvent, Outcome, Regular};
use rat_focus::{Focus, FocusBuilder};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use std::cmp::max;

mod adapter;
mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        sub1: Substratum2State::named("sub1"),
        sub3: SubstratumState::named("sub3"),
        sub4: SubstratumState::named("sub4"),
    };
    focus_input(&mut state).next();

    run_ui("focus_recursive2", mock_init, event, render, &mut state)
}

struct State {
    pub(crate) sub1: Substratum2State,
    pub(crate) sub3: SubstratumState,
    pub(crate) sub4: SubstratumState,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([
        Constraint::Length(25),
        Constraint::Length(25),
        Constraint::Fill(1),
    ])
    .split(area);

    Substratum2::new(&ctx.theme)
        .block(Block::bordered().title("First"))
        .render(l0[0], buf, &mut state.sub1);

    let l11 = Layout::vertical([Constraint::Length(8), Constraint::Length(8)]).split(l0[1]);
    Substratum::new(&ctx.theme)
        .block(Block::bordered().title("Third"))
        .render(l11[0], buf, &mut state.sub3);
    Substratum::new(&ctx.theme)
        .block(Block::bordered().title("Forth"))
        .render(l11[1], buf, &mut state.sub4);

    let cursor = state.sub1.screen_cursor().or_else(|| {
        state
            .sub3
            .screen_cursor()
            .or_else(|| state.sub4.screen_cursor())
    });
    if let Some((x, y)) = cursor {
        ctx.cursor = Some((x, y));
    }
    Ok(())
}

fn focus_input(state: &mut State) -> Focus {
    let mut fb = FocusBuilder::default();
    fb.widget(&state.sub1)
        .widget(&state.sub3)
        .widget(&state.sub4);
    fb.build()
}

fn event(
    event: &Event,
    _ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    let f = focus_input(state).handle(event, Regular);
    let r = state
        .sub1
        .handle(event, Regular)
        .or_else(|| state.sub3.handle(event, Regular))
        .or_else(|| state.sub4.handle(event, Regular));
    Ok(max(f, r))
}

pub mod substratum2 {
    use crate::substratum1::{Substratum, SubstratumState};
    use rat_event::{ConsumedEvent, HandleEvent, Outcome, Regular};
    use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_theme4::theme::SalsaTheme;
    use rat_theme4::{RatWidgetColor, StyleName};
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Constraint, Layout, Rect};
    use ratatui_core::style::{Color, Style};
    use ratatui_core::widgets::{StatefulWidget, Widget};
    use ratatui_crossterm::crossterm::event::Event;
    use ratatui_widgets::block::{Block, BlockExt};

    #[derive(Debug)]
    pub struct Substratum2<'a> {
        theme: &'a SalsaTheme,
        block: Option<Block<'a>>,
    }

    impl<'a> Substratum2<'a> {
        pub fn new(theme: &'a SalsaTheme) -> Self {
            Self {
                theme,
                block: Default::default(),
            }
        }

        pub fn block(mut self, block: Block<'a>) -> Self {
            self.block = Some(block);
            self
        }
    }

    #[derive(Debug, Default)]
    pub struct Substratum2State {
        pub container_focus: FocusFlag,
        pub area: Rect,
        pub stratum1: SubstratumState,
        pub stratum2: SubstratumState,
    }

    impl Substratum2State {
        pub fn named(name: &str) -> Self {
            Self {
                container_focus: FocusFlag::new().with_name(name),
                area: Default::default(),
                stratum1: SubstratumState::named(format!("{}.1", name).as_str()),
                stratum2: SubstratumState::named(format!("{}.2", name).as_str()),
            }
        }
    }

    impl<'a> StatefulWidget for Substratum2<'a> {
        type State = Substratum2State;

        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            state.area = area;

            let inner = self.block.inner_if_some(area);

            let mut block = if let Some(block) = self.block {
                block
            } else {
                Block::bordered()
            }
            .style(self.theme.style_style(Style::CONTAINER_BASE))
            .border_style(self.theme.style_style(Style::CONTAINER_BORDER_FG));
            if state.container_focus.get() {
                block = block.border_style(self.theme.p.fg_style_alias(Color::FOCUS_BG))
            }
            block.render(area, buf);

            let ll = Layout::vertical([Constraint::Length(8), Constraint::Length(8)]).split(inner);

            let ss1 = Substratum::new(self.theme).block(Block::bordered().title("Primus"));
            ss1.render(ll[0], buf, &mut state.stratum1);

            let ss2 = Substratum::new(self.theme).block(Block::bordered().title("Secundus"));
            ss2.render(ll[1], buf, &mut state.stratum2);
        }
    }

    impl Substratum2State {
        pub fn screen_cursor(&self) -> Option<(u16, u16)> {
            if self.stratum1.is_focused() {
                self.stratum1.screen_cursor()
            } else if self.stratum2.is_focused() {
                self.stratum2.screen_cursor()
            } else {
                None
            }
        }
    }

    impl HasFocus for Substratum2State {
        fn build(&self, builder: &mut FocusBuilder) {
            let tag = builder.start(self);
            builder.widget(&self.stratum1);
            builder.widget(&self.stratum2);
            builder.end(tag);
        }

        fn focus(&self) -> FocusFlag {
            self.container_focus.clone()
        }

        fn area(&self) -> Rect {
            self.area
        }
    }

    impl HandleEvent<Event, Regular, Outcome> for Substratum2State {
        fn handle(&mut self, event: &Event, _keymap: Regular) -> Outcome {
            self.stratum1
                .handle(event, Regular) //
                .or_else(|| self.stratum2.handle(event, Regular))
        }
    }
}

pub mod substratum1 {
    use crate::adapter::textinputf::{TextInputF, TextInputFState};
    use crate::mini_salsa::layout_grid;
    use rat_event::{ConsumedEvent, HandleEvent, Outcome, Regular};
    use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_theme4::theme::SalsaTheme;
    use rat_theme4::{RatWidgetColor, StyleName};
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Constraint, Layout, Rect};
    use ratatui_core::style::{Color, Style};
    use ratatui_core::text::Span;
    use ratatui_core::widgets::{StatefulWidget, Widget};
    use ratatui_crossterm::crossterm::event::Event;
    use ratatui_widgets::block::{Block, BlockExt};

    #[derive(Debug)]
    pub struct Substratum<'a> {
        theme: &'a SalsaTheme,
        block: Option<Block<'a>>,
    }

    impl<'a> Substratum<'a> {
        pub fn new(theme: &'a SalsaTheme) -> Self {
            Self {
                theme,
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
        pub container_focus: FocusFlag,
        pub area: Rect,
        pub input1: TextInputFState,
        pub input2: TextInputFState,
        pub input3: TextInputFState,
        pub input4: TextInputFState,
    }

    impl SubstratumState {
        pub fn named(name: &str) -> Self {
            Self {
                container_focus: FocusFlag::new().with_name(name),
                area: Default::default(),
                input1: Default::default(),
                input2: Default::default(),
                input3: Default::default(),
                input4: Default::default(),
            }
        }
    }

    impl<'a> StatefulWidget for Substratum<'a> {
        type State = SubstratumState;

        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            state.area = area;

            let inner = self.block.inner_if_some(area);

            let mut block = if let Some(block) = self.block {
                block
            } else {
                Block::bordered()
            }
            .style(self.theme.style_style(Style::CONTAINER_BASE))
            .border_style(self.theme.style_style(Style::CONTAINER_BORDER_FG));
            if state.container_focus.get() {
                block = block.border_style(self.theme.p.fg_style_alias(Color::FOCUS_BG))
            }
            block.render(area, buf);

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
            TextInputF::default()
                .style(self.theme.style_style(Style::INPUT))
                .focus_style(self.theme.style_style(Style::FOCUS))
                .render(l_grid[1][0], buf, &mut state.input1);

            Span::from("Text 2").render(l_grid[0][1], buf);
            TextInputF::default()
                .style(self.theme.style_style(Style::INPUT))
                .focus_style(self.theme.style_style(Style::FOCUS))
                .render(l_grid[1][1], buf, &mut state.input2);

            Span::from("Text 3").render(l_grid[0][2], buf);
            TextInputF::default()
                .style(self.theme.style_style(Style::INPUT))
                .focus_style(self.theme.style_style(Style::FOCUS))
                .render(l_grid[1][2], buf, &mut state.input3);

            Span::from("Text 4").render(l_grid[0][3], buf);
            TextInputF::default()
                .style(self.theme.style_style(Style::INPUT))
                .focus_style(self.theme.style_style(Style::FOCUS))
                .render(l_grid[1][3], buf, &mut state.input4);
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
        fn build(&self, builder: &mut FocusBuilder) {
            let tag = builder.start(self);
            builder
                .widget(&self.input1)
                .widget(&self.input2)
                .widget(&self.input3)
                .widget(&self.input4);
            builder.end(tag);
        }

        fn focus(&self) -> FocusFlag {
            self.container_focus.clone()
        }

        fn area(&self) -> Rect {
            self.area
        }
    }

    impl HandleEvent<Event, Regular, Outcome> for SubstratumState {
        fn handle(&mut self, event: &Event, _keymap: Regular) -> Outcome {
            self.input1
                .handle(event, Regular) //
                .or_else(|| self.input2.handle(event, Regular))
                .or_else(|| self.input3.handle(event, Regular))
                .or_else(|| self.input4.handle(event, Regular))
        }
    }
}
