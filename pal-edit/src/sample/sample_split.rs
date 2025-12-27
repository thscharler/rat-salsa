use crate::Global;
use anyhow::Error;
use rat_theme4::WidgetStyle;
use rat_widget::event::{HandleEvent, Outcome, Regular, event_flow};
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_widget::splitter::{Split, SplitState, SplitType};
use rat_widget::text::HasScreenCursor;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Direction, Rect};
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;

#[derive(Debug)]
pub struct SampleSplit {
    pub split: SplitState,
}

impl HasFocus for SampleSplit {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget_navigate(&self.split, Navigation::Regular);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not available")
    }

    fn area(&self) -> Rect {
        unimplemented!("not available")
    }
}

impl HasScreenCursor for SampleSplit {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        None
    }
}

impl Default for SampleSplit {
    fn default() -> Self {
        Self {
            split: SplitState::named("split"),
        }
    }
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut SampleSplit,
    ctx: &mut Global,
) -> Result<(), Error> {
    let (split_layout, split) = Split::new()
        .direction(Direction::Horizontal)
        .split_type(SplitType::FullPlain)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(60),
        ])
        .styles(ctx.show_theme.style(WidgetStyle::SPLIT))
        .into_widgets();
    split_layout.render(area, buf, &mut state.split);

    // todo

    split.render(area, buf, &mut state.split);

    Ok(())
}

pub fn event(event: &Event, state: &mut SampleSplit, _ctx: &mut Global) -> Result<Outcome, Error> {
    event_flow!(state.split.handle(event, Regular));
    Ok(Outcome::Continue)
}
