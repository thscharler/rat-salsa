use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use rat_event::Regular;
use rat_theme4::{StyleName, WidgetStyle};
use rat_widget::event::{HandleEvent, Outcome, event_flow};
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_widget::toolbar::{Toolbar, ToolbarOutcome, ToolbarState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        tools: ToolbarState::default(),
        anticipate: false,
        beatification: false,
        chosen: 0,
        text_0: Default::default(),
    };

    run_ui("button1", mock_init, event, render, &mut state)
}

struct State {
    tools: ToolbarState,

    anticipate: bool,
    beatification: bool,
    chosen: usize,

    text_0: TextInputMockState,
}

impl HasFocus for State {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.tools);
        builder.widget(&self.text_0);
    }

    fn focus(&self) -> FocusFlag {
        todo!()
    }

    fn area(&self) -> Rect {
        todo!()
    }
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::vertical([
        Constraint::Length(1), //
        Constraint::Fill(1),
    ])
    .split(area);

    let (w, p) = Toolbar::new()
        .text("ABC")
        .button("F1", "Function")
        .hotkey(KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE))
        .button("F2", "Function")
        .hotkey(KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE))
        .checkbox("A", "nticipate", state.anticipate)
        .hotkey(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE))
        .button("F3", "Function")
        .hotkey(KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE))
        .checkbox("B", "eatification", state.beatification)
        .hotkey(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE))
        .choice("F7", ["Choose 1", "Choose 2", "Choose 3"], state.chosen)
        .hotkey(KeyEvent::new(KeyCode::F(7), KeyModifiers::NONE))
        .styles(ctx.theme.style(WidgetStyle::TOOLBAR))
        .into_widgets(l0[0], &mut state.tools);

    w.render(l0[0], buf, &mut state.tools);

    let mut txt_area = l0[1];
    txt_area.x += 4;
    txt_area.y += 4;
    txt_area.width = 20;
    txt_area.height = 1;
    TextInputMock::default()
        .sample("...")
        .style(ctx.theme.style_style(Style::INPUT))
        .focus_style(ctx.theme.style_style(Style::INPUT_FOCUS))
        .render(txt_area, buf, &mut state.text_0);

    p.render(l0[0], buf, &mut state.tools);

    Ok(())
}

fn event(
    event: &Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    ctx.focus = Some(FocusBuilder::rebuild_for(state, ctx.focus.take()));
    ctx.handle_focus(event);

    match state.tools.handle(event, Regular) {
        ToolbarOutcome::Pressed(0) => event_flow!({
            ctx.status[0] = "button 1".into();
            Outcome::Changed
        }),
        ToolbarOutcome::Pressed(1) => event_flow!({
            ctx.status[0] = "button 2".into();
            Outcome::Changed
        }),
        ToolbarOutcome::Checked(2, v) => event_flow!({
            state.anticipate = v;
            ctx.status[0] = format!("checked {} {}", 2, v);
            Outcome::Changed
        }),
        ToolbarOutcome::Pressed(3) => event_flow!({
            ctx.status[0] = "button 3".into();
            Outcome::Changed
        }),
        ToolbarOutcome::Checked(4, v) => event_flow!({
            state.beatification = v;
            ctx.status[0] = format!("checked {} {}", 4, v);
            Outcome::Changed
        }),
        ToolbarOutcome::Selected(5, m) => event_flow!({
            ctx.status[0] = format!("selected {} {}", 5, m);
            state.chosen = m;
            Outcome::Changed
        }),
        r => event_flow!(r),
    }

    Ok(Outcome::Continue)
}
