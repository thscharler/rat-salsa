use crate::base46::Base46;
use crate::show_tabs::ShowTabs;
use crate::{Config, Global, base46, show_tabs};
use anyhow::Error;
use rat_theme4::WidgetStyle;
use rat_widget::event::{HandleEvent, Outcome, Regular, event_flow};
use rat_widget::focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_widget::tabbed::{Tabbed, TabbedState};
use rat_widget::text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType, StatefulWidget};

// mark tabs
#[derive(Debug)]
pub struct ShowOrBase46 {
    pub tabs: TabbedState,
    pub show: ShowTabs,
    pub base46: Base46,
}

impl ShowOrBase46 {
    pub fn new(cfg: &Config) -> Self {
        Self {
            tabs: Default::default(),
            show: ShowTabs::new(cfg.loc),
            base46: Base46::default(),
        }
    }

    pub fn show_focused(&mut self, focus: &Focus) {
        match self.tabs.selected() {
            Some(0) => {
                self.show.show_focused(focus);
            }
            Some(1) => {
                self.base46.form.show_focused(focus);
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
                builder.widget(&self.base46);
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
            Some(2) => self.base46.screen_cursor(),
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
        .tabs(["Preview", "Base46"])
        .block(Block::bordered().border_type(BorderType::Rounded))
        .styles(ctx.theme.style(WidgetStyle::TABBED))
        .render(area, buf, &mut state.tabs);

    match state.tabs.selected() {
        Some(0) => {
            show_tabs::render(state.tabs.widget_area, buf, &mut state.show, ctx)?;
        }
        Some(1) => {
            let mut area = state.tabs.widget_area;
            area.width += 1;
            base46::render(area, buf, &mut state.base46, ctx)?;
        }
        _ => {}
    };
    Ok(())
}

pub fn event(
    event: &crossterm::event::Event,
    state: &mut ShowOrBase46,
    ctx: &mut Global,
) -> Result<Outcome, Error> {
    event_flow!(match state.tabs.selected() {
        Some(0) => {
            show_tabs::event(event, &mut state.show, ctx)?
        }
        Some(1) => {
            base46::event(event, &mut state.base46, ctx)?
        }
        _ => {
            Outcome::Continue
        }
    });
    event_flow!(state.tabs.handle(event, Regular));
    Ok(Outcome::Continue)
}
