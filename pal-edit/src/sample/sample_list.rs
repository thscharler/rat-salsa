use crate::Global;
use anyhow::Error;
use rat_theme4::WidgetStyle;
use rat_widget::event::{HandleEvent, Outcome, Regular, event_flow};
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_widget::list::{List, ListState};
use rat_widget::scrolled::Scroll;
use rat_widget::text::HasScreenCursor;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;

#[derive(Debug)]
pub struct SampleList {
    pub list: ListState,
}

impl HasFocus for SampleList {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.list);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not available")
    }

    fn area(&self) -> Rect {
        unimplemented!("not available")
    }
}

impl HasScreenCursor for SampleList {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.list.screen_cursor()
    }
}

impl Default for SampleList {
    fn default() -> Self {
        Self {
            list: ListState::named("list"),
        }
    }
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut SampleList,
    ctx: &mut Global,
) -> Result<(), Error> {
    let entries = [
        "Backpacks: A portable bag with straps for carrying personal items, commonly used for school or travel.",
        "Books: Written or printed works consisting of pages bound together along one side, used for reading and learning.",
        "Bicycles: Human-powered vehicles with two wheels, used for transportation and recreation.",
        "Coffee Makers: Appliances designed to brew coffee from ground beans, commonly found in homes and offices.",
        "Smartphones: Portable devices combining a mobile phone with advanced computing capabilities, including internet access and apps.",
        "Gardens: Plots of land cultivated for growing plants, flowers, or vegetables, often for aesthetic or practical purposes.",
        "Music Boxes: Mechanical devices that play music through a rotating cylinder with pins, often used as decorative items.",
        "Pens: Writing instruments that dispense ink, used for writing or drawing.",
        "Laptops: Portable computers with integrated screen, keyboard, and battery, designed for mobile computing.",
        "Dogs: Domesticated mammals commonly kept as pets, known for loyalty and companionship.",
    ];
    List::new(entries)
        .scroll(Scroll::new())
        .styles(ctx.show_theme.style(WidgetStyle::LIST))
        .render(area, buf, &mut state.list);

    Ok(())
}

pub fn event(event: &Event, state: &mut SampleList, _ctx: &mut Global) -> Result<Outcome, Error> {
    event_flow!(state.list.handle(event, Regular));
    Ok(Outcome::Continue)
}
