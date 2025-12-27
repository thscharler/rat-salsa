use crate::foreign::Foreign;
use crate::sample::show_sample;
use crate::sample::show_sample::ShowSample;
use crate::{Config, Global, foreign};
use anyhow::Error;
use rat_theme4::WidgetStyle;
use rat_widget::event::{HandleEvent, Outcome, Regular, event_flow};
use rat_widget::focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_widget::tabbed::{Tabbed, TabbedState};
use rat_widget::text::HasScreenCursor;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;

// mark tabs
#[derive(Debug)]
pub struct ShowOrBase46 {
    pub tabs: TabbedState,
    pub show: ShowSample,
    pub foreign: Foreign,
}

impl ShowOrBase46 {
    pub fn new(cfg: &Config) -> Self {
        Self {
            tabs: Default::default(),
            show: ShowSample::new(cfg.loc),
            foreign: Foreign::default(),
        }
    }

    pub fn show_focused(&mut self, focus: &Focus) {
        match self.tabs.selected() {
            Some(0) => {
                self.show.show_focused(focus);
            }
            Some(1) => {
                self.foreign.form.show_focused(focus);
            }
            _ => {}
        }
    }
}

impl HasFocus for ShowOrBase46 {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.tabs);
        match self.tabs.selected() {
            Some(0) => {
                builder.widget(&self.show);
            }
            Some(1) => {
                builder.widget(&self.foreign);
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

impl HasScreenCursor for ShowOrBase46 {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        match self.tabs.selected() {
            Some(0) => self.show.screen_cursor(),
            Some(1) => self.foreign.screen_cursor(),
            _ => None,
        }
    }
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut ShowOrBase46,
    ctx: &mut Global,
) -> Result<(), Error> {
    Tabbed::new()
        .tabs([
            "Preview".to_string(),
            format!("Foreign {}", state.foreign.name),
        ])
        .block(Block::bordered().border_type(BorderType::Rounded))
        .styles(ctx.theme.style(WidgetStyle::TABBED))
        .render(area, buf, &mut state.tabs);

    match state.tabs.selected() {
        Some(0) => {
            show_sample::render(state.tabs.widget_area, buf, &mut state.show, ctx)?;
        }
        Some(1) => {
            let mut area = state.tabs.widget_area;
            area.width += 1;
            foreign::render(area, buf, &mut state.foreign, ctx)?;
        }
        _ => {}
    };
    Ok(())
}

pub fn event(event: &Event, state: &mut ShowOrBase46, ctx: &mut Global) -> Result<Outcome, Error> {
    event_flow!(match state.tabs.selected() {
        Some(0) => {
            show_sample::event(event, &mut state.show, ctx)?
        }
        Some(1) => {
            foreign::event(event, &mut state.foreign, ctx)?
        }
        _ => {
            Outcome::Continue
        }
    });
    event_flow!(state.tabs.handle(event, Regular));
    Ok(Outcome::Continue)
}
