use crate::sample_data_input::SampleDataInput;
use crate::sample_other::SampleOther;
use crate::sample_readability::SampleReadability;
use crate::{Global, sample_data_input, sample_other, sample_readability};
use anyhow::Error;
use pure_rust_locales::Locale;
use rat_theme4::{StyleName, WidgetStyle, dark_theme, shell_theme};
use rat_widget::choice::{Choice, ChoiceState};
use rat_widget::event::{ChoiceOutcome, HandleEvent, Outcome, Popup, Regular, event_flow};
use rat_widget::focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_widget::tabbed::{Tabbed, TabbedState};
use rat_widget::text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Position, Rect};
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Borders, StatefulWidget, Widget};
use std::iter::once;

// mark tabs
#[derive(Debug)]
pub struct ShowSample {
    pub themes: ChoiceState<String>,
    pub tabs: TabbedState,
    pub input: SampleDataInput,
    pub readability: SampleReadability,
    pub other: SampleOther,
}

impl ShowSample {
    pub fn new(loc: Locale) -> Self {
        Self {
            themes: ChoiceState::named("themes"),
            tabs: Default::default(),
            input: SampleDataInput::new(loc),
            readability: SampleReadability::default(),
            other: SampleOther::default(),
        }
    }

    pub fn show_focused(&mut self, focus: &Focus) {
        match self.tabs.selected() {
            Some(0) => {
                self.input.form.show_focused(focus);
            }
            Some(1) => { /*noop*/ }
            Some(2) => {
                self.other.form.show_focused(focus);
            }
            _ => {}
        }
    }
}

impl HasFocus for ShowSample {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.tabs);
        builder.widget(&self.themes);
        match self.tabs.selected() {
            Some(0) => {
                builder.widget(&self.input);
            }
            Some(1) => {
                builder.widget(&self.readability);
            }
            Some(2) => {
                builder.widget(&self.other);
            }
            _ => {}
        }
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not available")
    }

    fn area(&self) -> Rect {
        unimplemented!("not available")
    }
}

impl HasScreenCursor for ShowSample {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        match self.tabs.selected() {
            Some(0) => self.input.screen_cursor(),
            Some(1) => self.readability.screen_cursor(),
            Some(2) => self.other.screen_cursor(),
            _ => None,
        }
    }
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut ShowSample,
    ctx: &mut Global,
) -> Result<(), Error> {
    let l0 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .spacing(1)
    .split(area);

    let base = ctx.show_theme.style_style(Style::CONTAINER_BASE);
    for r in area.top()..area.bottom() {
        for c in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut(Position::new(c, r)) {
                cell.reset();
                cell.set_style(base);
            }
        }
    }

    Text::from("Preview")
        .alignment(Alignment::Center)
        .style(ctx.show_theme.style_style(Style::TITLE))
        .render(l0[1], buf);

    let l_function = Layout::horizontal([
        Constraint::Length(2), //
        Constraint::Length(12),
    ])
    .spacing(1)
    .split(l0[2]);
    let (choice, choice_theme) = Choice::new()
        .items(
            once("")
                .chain([
                    "Dark",     //
                    "Shell",    //
                    "Fallback", //
                ])
                .map(|v| (v.to_string(), v.to_string())),
        )
        .styles(ctx.show_theme.style(WidgetStyle::CHOICE))
        .into_widgets();
    choice.render(l_function[1], buf, &mut state.themes);

    Tabbed::new()
        .tabs(["Input", "Text", "Other"])
        .block(
            Block::bordered()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded),
        )
        .styles(ctx.show_theme.style(WidgetStyle::TABBED))
        .render(l0[3], buf, &mut state.tabs);

    match state.tabs.selected() {
        Some(0) => {
            let mut area = state.tabs.widget_area;
            area.width += 1;
            sample_data_input::render(area, buf, &mut state.input, ctx)?;
        }
        Some(1) => {
            sample_readability::render(state.tabs.widget_area, buf, &mut state.readability, ctx)?;
        }
        Some(2) => {
            sample_other::render(state.tabs.widget_area, buf, &mut state.other, ctx)?;
        }
        _ => {}
    };

    choice_theme.render(l_function[1], buf, &mut state.themes);

    Ok(())
}

pub fn event(
    event: &crossterm::event::Event,
    state: &mut ShowSample,
    ctx: &mut Global,
) -> Result<Outcome, Error> {
    event_flow!(match state.themes.handle(event, Popup) {
        ChoiceOutcome::Value => {
            let pal = ctx.show_theme.p;
            ctx.show_theme = match state.themes.value().as_str() {
                "Shell" => shell_theme("Shell", pal),
                // "Fallback" => fallback_theme("Fallback", palette),
                _ => dark_theme("Dark", pal),
            };
            Outcome::Changed
        }
        r => r.into(),
    });

    event_flow!(match state.tabs.selected() {
        Some(0) => {
            sample_data_input::event(event, &mut state.input, ctx)?
        }
        Some(1) => {
            sample_readability::event(event, &mut state.readability, ctx)?
        }
        Some(2) => {
            sample_other::event(event, &mut state.other, ctx)?
        }
        _ => {
            Outcome::Continue
        }
    });
    event_flow!(state.tabs.handle(event, Regular));
    Ok(Outcome::Continue)
}
