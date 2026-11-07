//!
//! Example for [TableData]
//!
//!

mod widget;

use crate::editor::{RowEditor, RowEditorState};
use crate::mini_salsa::mock_init;
use crate::mini_salsa::{MiniSalsaState, run_ui, setup_logging};
use crate::styles::table_style;
use format_num_pattern::NumberFormat;
use rat_cursor::HasScreenCursor;
use rat_event::{HandleEvent, Regular, event_flow};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_ftable2::edit::{TableEdit, TableEditState};
use rat_ftable2::event::Outcome;
use rat_scrolled::Scroll;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Flex, Layout, Rect};
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;

mod data;
mod mini_salsa;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        table: TableEditState::named("t0", RowEditorState::default()).with(data::DATA.iter().map(
            |v| Sample {
                text: (*v).into(),
                num1: rand::random(),
                num2: rand::random(),
                check: rand::random(),
            },
        )),
    };

    run_ui("slice", mock_init, event, render, &mut state)
}

#[derive(Default, Clone)]
struct Sample {
    text: String,
    num1: f32,
    num2: f32,
    check: bool,
}

struct State {
    table: TableEditState<Sample, RowEditorState>,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l0 = Layout::horizontal([Constraint::Percentage(61)])
        .flex(Flex::Center)
        .split(area);

    let row_fmt = NumberFormat::new("000000").expect("fmt");
    let num1_fmt = NumberFormat::new("####0.00").expect("fmt");
    let num2_fmt = NumberFormat::new("####0.00").expect("fmt");

    TableEdit::<'_, Sample, RowEditor>::new(RowEditor)
        .widths([
            Constraint::Length(6),
            Constraint::Length(20),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(3),
        ])
        .render_cell(0, |_v, area, buf, ctx| {
            row_fmt.fmt_u(ctx.row).render(area, buf)
        })
        .render_cell(1, |v, area, buf, _ctx| v.text.as_str().render(area, buf))
        .render_cell(2, |v, area, buf, _ctx| {
            num1_fmt.fmt_u(v.num1).render(area, buf)
        })
        .render_cell(3, |v, area, buf, _ctx| {
            num2_fmt.fmt_u(v.num2).render(area, buf)
        })
        .render_cell(4, |v, area, buf, _ctx| {
            let cc = if v.check { "\u{2622}" } else { "\u{2623}" };
            cc.render(area, buf);
        })
        .column_spacing(1)
        .header(vec!["Nr", "Text", "Val1", "Val2", "State"])
        .footer(vec!["a", "b", "c", "d", "e"])
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("tabledata"),
        )
        .vscroll(Scroll::new())
        .styles(table_style(&ctx.theme))
        .render(l0[0], buf, &mut state.table);

    ctx.cursor = state.table.screen_cursor();

    Ok(())
}

fn focus(state: &mut State) -> Focus {
    let mut fb = FocusBuilder::new(None);
    fb.widget(&state.table);
    fb.widget(&FocusFlag::new());
    fb.build()
}

fn event(
    event: &Event,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    ctx.focus_outcome = focus(state).handle(event, Regular);

    event_flow!(state.table.handle(event, &())?);

    Ok(Outcome::Continue)
}

mod editor {
    use crate::Sample;
    use crate::widget::textinputf::{TextInputF, TextInputFState};
    use rat_cursor::impl_screen_cursor;
    use rat_event::{HandleEvent, Regular, event_flow};
    use rat_focus::{FocusBuilder, impl_has_focus, match_focus};
    use rat_ftable2::edit::{TableEditor, TableEditorState};
    use rat_ftable2::event::Outcome;
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;
    use ratatui_core::style::Style;
    use ratatui_core::widgets::StatefulWidget;
    use ratatui_crossterm::crossterm::event::Event;
    use std::cmp::max;

    pub struct RowEditor;

    #[derive(Default)]
    pub struct RowEditorState {
        value: Sample,

        text: TextInputFState,
        num1: TextInputFState,
        num2: TextInputFState,
        check: TextInputFState,
    }

    impl TableEditor<Sample> for RowEditor {
        type State = RowEditorState;

        fn render(
            &self,
            _area: Rect,
            cell_areas: &[Rect],
            buf: &mut Buffer,
            state: &mut Self::State,
        ) {
            TextInputF::default()
                .focus_style(Style::new().underlined())
                .render(cell_areas[1], buf, &mut state.text);

            TextInputF::default()
                .focus_style(Style::new().underlined())
                .render(cell_areas[2], buf, &mut state.num1);

            TextInputF::default()
                .focus_style(Style::new().underlined())
                .render(cell_areas[3], buf, &mut state.num2);

            TextInputF::default()
                .focus_style(Style::new().underlined())
                .render(cell_areas[4], buf, &mut state.check);
        }
    }

    impl_has_focus!(text, num1, num2, check for RowEditorState);
    impl_screen_cursor!(text, num1, num2, check for RowEditorState);

    impl TableEditorState<Sample> for RowEditorState {
        type Context<'a> = ();
        type Err = anyhow::Error;

        // create a default row value
        fn create_value<'a>(&self, _ctx: &'a Self::Context<'a>) -> Result<Sample, Self::Err> {
            Ok(Sample {
                text: "".into(),
                num1: 0.0,
                num2: 0.0,
                check: false,
            })
        }

        // new editor value
        fn set_value<'a>(
            &mut self,
            value: Sample,
            _ctx: &'a Self::Context<'a>,
        ) -> Result<(), Self::Err> {
            self.value = value;
            Ok(())
        }

        // reconstruct the row value from its fields
        fn value<'a>(&mut self, _ctx: &'a Self::Context<'a>) -> Result<Option<Sample>, Self::Err> {
            Ok(Some(self.value.clone()))
        }

        // map editor focus to table column
        fn focused_col(&self) -> Option<usize> {
            match_focus!(
                self.text => Some(1),
                self.num1 => Some(2),
                self.num2 => Some(3),
                self.check => Some(4),
                else => None
            )
        }

        // start editing (a specific column)
        fn start<'a>(
            &mut self,
            col: Option<usize>,
            _ctx: &'a Self::Context<'a>,
        ) -> Result<(), Self::Err> {
            let f = FocusBuilder::build_for(self);
            match col {
                None => f.first(),
                Some(1) => f.focus(&self.text),
                Some(2) => f.focus(&self.num1),
                Some(3) => f.focus(&self.num2),
                Some(4) => f.focus(&self.check),
                _ => f.focus(&self.text),
            }
            Ok(())
        }

        // stop editing the row
        fn stop<'a>(&mut self, _ctx: &'a Self::Context<'a>) -> Result<(), Self::Err> {
            Ok(())
        }
    }

    impl HandleEvent<Event, &(), Result<Outcome, anyhow::Error>> for RowEditorState {
        fn handle(&mut self, event: &Event, _qualifier: &()) -> Result<Outcome, anyhow::Error> {
            let ff = FocusBuilder::build_for(self).handle(event, Regular);

            let rr: Outcome = 'l: {
                event_flow!(break 'l self.text.handle(event, Regular));
                event_flow!(break 'l self.num1.handle(event, Regular));
                event_flow!(break 'l self.num2.handle(event, Regular));
                event_flow!(break 'l self.check.handle(event, Regular));
                Outcome::Continue
            };

            Ok(max(ff, rr))
        }
    }
}

mod styles {
    use rat_scrolled::ScrollStyle;
    use rat_theme4::StyleName;
    use rat_theme4::theme::SalsaTheme;
    use ratatui_core::style::Style;

    pub fn table_style(th: &SalsaTheme) -> rat_ftable2::TableStyle {
        rat_ftable2::TableStyle {
            style: th.style(Style::CONTAINER_BASE),
            select_row: Some(th.style(Style::SELECT)),
            show_row_focus: true,
            focus_style: Some(th.style(Style::FOCUS)),
            border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
            scroll: Some(scroll_style(th)),
            header: Some(th.style(Style::HEADER)),
            footer: Some(th.style(Style::FOOTER)),
            ..Default::default()
        }
    }

    pub fn scroll_style(th: &SalsaTheme) -> ScrollStyle {
        ScrollStyle {
            thumb_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
            track_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
            min_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
            begin_style: Some(th.style(Style::CONTAINER_ARROW_FG)),
            end_style: Some(th.style(Style::CONTAINER_ARROW_FG)),
            ..Default::default()
        }
    }
}
