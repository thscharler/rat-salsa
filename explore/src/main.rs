use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use crate::toolbar::{Toolbar, ToolbarKeys, ToolbarOutcome, ToolbarState, ToolbarStyles};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rat_theme4::StyleName;
use rat_widget::button::ButtonStyle;
use rat_widget::checkbox::{CheckboxCheck, CheckboxStyle};
use rat_widget::choice::{ChoiceClose, ChoiceFocus, ChoiceSelect, ChoiceStyle};
use rat_widget::event::{HandleEvent, Outcome, event_flow};
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_widget::popup::{Placement, PopupStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::StatefulWidget;

mod mini_salsa;
mod toolbar;

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        tools: ToolbarState::default(),
        anticipate: false,
        beatificate: false,
        text_0: Default::default(),
    };

    run_ui("button1", mock_init, event, render, &mut state)
}

struct State {
    tools: ToolbarState,

    anticipate: bool,
    beatificate: bool,

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

    let style = ToolbarStyles {
        style: ctx.theme.style_style(Style::STATUS_BASE),
        key_style: Some(ctx.theme.style_style(Style::KEY_BINDING)),
        button: Some(ButtonStyle {
            style: ctx.theme.style_style(Style::BUTTON_BASE),
            armed: Some(ctx.theme.style_style(Style::SELECT)),
            hover: Some(ctx.theme.style_style(Style::HOVER)),
            ..Default::default()
        }),
        checkbox: Some(CheckboxStyle {
            style: ctx.theme.style_style(Style::BUTTON_BASE),
            behave_check: Some(CheckboxCheck::SingleClick),
            ..Default::default()
        }),
        choice: Some(ChoiceStyle {
            style: ctx.theme.style_style(Style::BUTTON_BASE),
            button: Some(ctx.theme.style_style(Style::BUTTON_BASE)),
            select: Some(ctx.theme.style_style(Style::SELECT)),
            focus: Some(ctx.theme.style_style(Style::BUTTON_BASE)),
            popup: PopupStyle {
                placement: Some(Placement::BelowOrAbove),
                ..Default::default()
            },
            popup_style: Some(ctx.theme.style_style(Style::POPUP_BASE)),
            popup_border: Some(ctx.theme.style_style(Style::POPUP_BORDER_FG)),
            behave_focus: Some(ChoiceFocus::OpenOnFocusGained),
            behave_select: Some(ChoiceSelect::MouseClick),
            behave_close: Some(ChoiceClose::SingleClick),
            ..Default::default()
        }),
        ..Default::default()
    };

    let (w, p) = Toolbar::new()
        .text("ABC")
        .collapsed_buttons("...")
        .button("F1", "Function", true)
        .button("F2", "Function", true)
        .checkbox("A", "nticipate", state.anticipate)
        .button("F3", "Function", true)
        .checkbox("B", "eatificate", state.beatificate)
        .choice("F7", ["Choose 1", "Choose 2", "Choose 3"])
        .styles(style)
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
    event: &crossterm::event::Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    ctx.focus = Some(FocusBuilder::rebuild_for(state, ctx.focus.take()));
    ctx.handle_focus(event);

    match state.tools.handle(
        event,
        ToolbarKeys {
            focus: ctx.focus(),
            keys: [
                Some(KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE)),
                Some(KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE)),
                Some(KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE)),
                Some(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT)),
                Some(KeyEvent::new(KeyCode::Char('B'), KeyModifiers::SHIFT)),
                Some(KeyEvent::new(KeyCode::F(7), KeyModifiers::NONE)),
            ],
        },
    ) {
        ToolbarOutcome::Checked(2, v) => event_flow!({
            state.anticipate = v;
            ctx.status[0] = format!("checked {} {}", 2, v);
            Outcome::Changed
        }),
        ToolbarOutcome::Checked(4, v) => event_flow!({
            state.beatificate = v;
            ctx.status[0] = format!("checked {} {}", 4, v);
            Outcome::Changed
        }),
        ToolbarOutcome::Pressed(n) => event_flow!({
            ctx.status[0] = format!("pressed {}", n);
            Outcome::Changed
        }),
        ToolbarOutcome::Selected(n, m) => event_flow!({
            ctx.status[0] = format!("selected {} {}", n, m);
            Outcome::Changed
        }),
        r => event_flow!(r),
    }

    Ok(Outcome::Continue)
}
