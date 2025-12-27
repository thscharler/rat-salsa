use crate::Global;
use anyhow::Error;
use rat_theme4::{StyleName, WidgetStyle};
use rat_widget::event::{HandleEvent, Outcome, Regular, event_flow};
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_widget::scrolled::Scroll;
use rat_widget::table::textdata::{Cell, Row};
use rat_widget::table::{Table, TableState, TableStyle};
use rat_widget::text::HasScreenCursor;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Rect};
use ratatui_core::style::Style;
use ratatui_core::widgets::StatefulWidget;
use ratatui_crossterm::crossterm::event::Event;

#[derive(Debug)]
pub struct SampleTable {
    pub table: TableState,
}

impl HasFocus for SampleTable {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.table);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not available")
    }

    fn area(&self) -> Rect {
        unimplemented!("not available")
    }
}

impl HasScreenCursor for SampleTable {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.table.screen_cursor()
    }
}

impl Default for SampleTable {
    fn default() -> Self {
        Self {
            table: TableState::named("table"),
        }
    }
}

pub fn render(
    doc: bool,
    area: Rect,
    buf: &mut Buffer,
    state: &mut SampleTable,
    ctx: &mut Global,
) -> Result<(), Error> {
    let mut style = ctx.show_theme.style::<TableStyle>(WidgetStyle::TABLE);
    if doc {
        style.style = ctx.show_theme.style_style(Style::DOCUMENT_BASE);
    }

    Table::new_ratatui(
        [
            Row::new([
                Cell::new("1"),
                Cell::new("67.9"),
                Cell::new("Female"),
                Cell::new("236.4"),
                Cell::new("129.8"),
                Cell::new("26.4"),
                Cell::new("Yes"),
                Cell::new("High"),
            ]),
            Row::new([
                Cell::new("2"),
                Cell::new("54.8"),
                Cell::new("Female"),
                Cell::new("256.3"),
                Cell::new("133.4"),
                Cell::new("28.4"),
                Cell::new("No"),
                Cell::new("Medium"),
            ]),
            Row::new([
                Cell::new("3"),
                Cell::new("68.4"),
                Cell::new("Male"),
                Cell::new("198.7"),
                Cell::new("158.5"),
                Cell::new("24.1"),
                Cell::new("Yes"),
                Cell::new("High"),
            ]),
            Row::new([
                Cell::new("4"),
                Cell::new("67.9"),
                Cell::new("Male"),
                Cell::new("205.0"),
                Cell::new("136.0"),
                Cell::new("19.9"),
                Cell::new("No"),
                Cell::new("Low"),
            ]),
            Row::new([
                Cell::new("5"),
                Cell::new("60.9"),
                Cell::new("Male"),
                Cell::new("207.7"),
                Cell::new("145.4"),
                Cell::new("26.7"),
                Cell::new("No"),
                Cell::new("Medium"),
            ]),
            Row::new([
                Cell::new("6"),
                Cell::new("44.9"),
                Cell::new("Female"),
                Cell::new("222.5"),
                Cell::new("130.6"),
                Cell::new("30.6"),
                Cell::new("Noe"),
                Cell::new("Low"),
            ]),
        ],
        [
            Constraint::Length(1),
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Length(11),
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Length(9),
        ],
    )
    .scroll(Scroll::new())
    .column_spacing(1)
    .header(Row::new([
        Cell::new("#"),
        Cell::new("Age"),
        Cell::new("Gender"),
        Cell::new("Cholesterol"),
        Cell::new("SystolicBP"),
        Cell::new("BMI"),
        Cell::new("Smoking"),
        Cell::new("Education"),
    ]))
    .styles(style)
    .layout_column_widths()
    .render(area, buf, &mut state.table);

    Ok(())
}

pub fn event(event: &Event, state: &mut SampleTable, _ctx: &mut Global) -> Result<Outcome, Error> {
    event_flow!(state.table.handle(event, Regular));
    Ok(Outcome::Continue)
}
