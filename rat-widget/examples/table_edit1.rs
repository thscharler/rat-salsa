//!
//! Example for [TableData]
//!

use crate::mini_salsa::{MiniSalsaState, THEME, mock_init, run_ui, setup_logging};
use anyhow::Error;
use format_num_pattern::{NumberFmtError, NumberFormat, NumberSymbols};
use pure_rust_locales::Locale;
use pure_rust_locales::Locale::de_AT_euro;
use rat_event::{HandleEvent, Outcome, Regular, flow, try_flow};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, match_focus};
use rat_ftable::edit::table::{EditableTable, EditableTableState};
use rat_ftable::edit::{TableEditor, TableEditorState};
use rat_ftable::event::EditOutcome;
use rat_ftable::textdata::{Cell, Row};
use rat_ftable::{Table, TableContext, TableData};
use rat_scrolled::Scroll;
use rat_text::HasScreenCursor;
use rat_text::number_input::{NumberInput, NumberInputState};
use rat_text::text_input::{TextInput, TextInputState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Flex, Layout, Rect};
use ratatui_core::terminal::Frame;
use ratatui_core::text::Span;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;

mod data {
    pub(crate) static TINY_DATA: [&str; 10] = [
        "Lorem",
        "ipsum",
        "dolor",
        "sit",
        "amet,",
        "consetetur",
        "sadipscing",
        "elitr,",
        "sed",
        "diam",
    ];
}

mod mini_salsa;

fn main() -> Result<(), Error> {
    setup_logging()?;

    let mut data = Data {
        table_data: data::TINY_DATA
            .iter()
            .map(|v| Sample {
                text: v.to_string(),
                num1: rand::random(),
                num2: rand::random(),
                num3: rand::random(),
            })
            .collect(),
    };

    let mut state = State {
        loc: de_AT_euro,
        table: EditableTableState::new(SampleEditorState::new(de_AT_euro)?),
        text1: Default::default(),
        text2: Default::default(),
    };
    state.table.table.select(Some(0));

    run_ui(
        "table_edit1",
        mock_init,
        event,
        render,
        &mut data,
        &mut state,
    )
}

#[derive(Debug, Clone, Default)]
struct Sample {
    pub(crate) text: String,
    pub(crate) num1: f32,
    pub(crate) num2: u32,
    pub(crate) num3: u32,
}

struct Data {
    table_data: Vec<Sample>,
}

struct State {
    loc: Locale,

    table: EditableTableState<SampleEditorState>,
    text1: TextInputState,
    text2: TextInputState,
}

impl HasFocus for State {
    fn build(&self, builder: &mut FocusBuilder) {
        builder
            .widget(&self.text1)
            .widget(&self.table)
            .widget(&self.text2);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("silent container")
    }

    fn area(&self) -> Rect {
        unimplemented!("silent container")
    }
}

impl HasScreenCursor for State {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.text1
            .screen_cursor()
            .or(self.table.screen_cursor())
            .or(self.text2.screen_cursor())
    }
}

struct TableData1<'a> {
    data: &'a [Sample],
    fmt1: NumberFormat,
    fmt2: NumberFormat,
}

impl<'a> TableData<'a> for TableData1<'a> {
    fn rows(&self) -> usize {
        self.data.len()
    }

    fn header(&self) -> Option<Row<'a>> {
        Some(
            Row::new([
                Cell::from("Text"),
                Cell::from("Float"),
                Cell::from("Int"),
                Cell::from("Int"),
            ])
            .style(Some(THEME.table_header())),
        )
    }

    fn widths(&self) -> Vec<Constraint> {
        vec![
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ]
    }

    fn render_cell(
        &self,
        _ctx: &TableContext,
        column: usize,
        row: usize,
        area: Rect,
        buf: &mut Buffer,
    ) {
        if let Some(d) = self.data.get(row) {
            match column {
                0 => Span::from(&d.text).render(area, buf),
                1 => Span::from(self.fmt1.fmt_u(d.num1)).render(area, buf),
                2 => Span::from(self.fmt2.fmt_u(d.num2)).render(area, buf),
                3 => Span::from(self.fmt2.fmt_u(d.num3)).render(area, buf),
                _ => {}
            }
        }
    }
}

fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), Error> {
    let l0 = Layout::horizontal([
        Constraint::Length(10),
        Constraint::Percentage(61),
        Constraint::Length(10),
    ])
    .margin(1)
    .flex(Flex::Center)
    .split(area);

    let l1 = Layout::vertical([Constraint::Length(1)])
        .flex(Flex::Center)
        .split(area);

    TextInput::new().styles(istate.theme.text_style()).render(
        Rect::new(l0[0].x, l1[0].y, l0[0].width, l1[0].height),
        frame.buffer_mut(),
        &mut state.text1,
    );

    EditableTable::new(
        Table::default()
            .data(TableData1 {
                data: &data.table_data,
                fmt1: NumberFormat::news("###,##0.0", NumberSymbols::numeric(state.loc))?,
                fmt2: NumberFormat::news("##########", NumberSymbols::numeric(state.loc))?,
            })
            .column_spacing(1)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(istate.theme.container_border())
                    .title("tabledata"),
            )
            .vscroll(Scroll::new().style(istate.theme.container_arrow()))
            .styles(istate.theme.table_style()),
        SampleEditor,
    )
    .render(l0[1], frame.buffer_mut(), &mut state.table);

    TextInput::new().styles(istate.theme.text_style()).render(
        Rect::new(l0[2].x, l1[0].y, l0[2].width, l1[0].height),
        frame.buffer_mut(),
        &mut state.text2,
    );

    if let Some(cursor) = state.screen_cursor() {
        frame.set_cursor_position(cursor);
    }

    Ok(())
}

fn event(
    event: &Event,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, Error> {
    istate.focus_outcome = FocusBuilder::build_for(state).handle(event, Regular);

    try_flow!(state.text1.handle(event, Regular));
    try_flow!({
        handle_table(event, data, istate, state).unwrap_or_else(|e| {
            istate.status[0] = format!("{}", e);
            Outcome::Changed
        })
    });
    try_flow!(state.text2.handle(event, Regular));

    Ok(Outcome::Continue)
}

fn handle_table(
    event: &Event,
    data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, Error> {
    try_flow!(match state.table.handle(event, istate) {
        EditOutcome::Edit => {
            if let Some(sel_row) = state.table.table.selected_checked() {
                state
                    .table
                    .editor
                    .set_value(data.table_data[sel_row].clone(), istate)?;
                state.table.edit(0, sel_row);
            }
            Outcome::Changed
        }
        EditOutcome::Cancel => {
            if let Some(sel) = state.table.table.selected_checked() {
                cancel_edit(data, sel, istate, state)?;
                Outcome::Changed
            } else {
                Outcome::Continue
            }
        }
        EditOutcome::Commit => {
            if let Some(sel) = state.table.table.selected_checked() {
                commit_edit(data, sel, istate, state)?;
                Outcome::Changed
            } else {
                Outcome::Continue
            }
        }
        EditOutcome::CommitAndAppend => {
            if let Some(sel) = state.table.table.selected_checked() {
                commit_edit(data, sel, istate, state)?;
            }
            if let Some(sel) = state.table.table.selected_checked() {
                let value = state.table.editor.create_value(istate)?;
                state.table.editor.set_value(value.clone(), istate)?;
                data.table_data.insert(sel + 1, value);
                state.table.edit_new(sel + 1);
            }
            Outcome::Changed
        }
        EditOutcome::CommitAndEdit => {
            if let Some(sel_row) = state.table.table.selected_checked() {
                commit_edit(data, sel_row, istate, state)?;
                state
                    .table
                    .editor
                    .set_value(data.table_data[sel_row + 1].clone(), istate)?;
                state.table.edit(0, sel_row + 1);
            }
            Outcome::Changed
        }
        EditOutcome::Insert => {
            if let Some(sel) = state.table.table.selected_checked() {
                let value = state.table.editor.create_value(istate)?;
                state.table.editor.set_value(value.clone(), istate)?;
                data.table_data.insert(sel, value);
                state.table.edit_new(sel);
            }
            Outcome::Changed
        }
        EditOutcome::Append => {
            let value = state.table.editor.create_value(istate)?;
            state.table.editor.set_value(value.clone(), istate)?;
            data.table_data.push(value);
            state.table.edit_new(data.table_data.len() - 1);
            Outcome::Changed
        }
        EditOutcome::Remove => {
            if let Some(sel) = state.table.table.selected_checked() {
                if sel < data.table_data.len() {
                    data.table_data.remove(sel);
                    state.table.remove(sel);
                }
            }
            Outcome::Changed
        }

        r => Outcome::from(r),
    });

    Ok(Outcome::Continue)
}

fn cancel_edit(
    data: &mut Data,
    sel: usize,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), Error> {
    if state.table.is_insert() {
        data.table_data.remove(sel);
    }
    state.table.cancel();
    Ok(())
}

fn commit_edit(
    data: &mut Data,
    sel: usize,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), Error> {
    if let Some(value) = state.table.editor.value(istate)? {
        data.table_data[sel] = value;
        state.table.commit();
        Ok(())
    } else {
        cancel_edit(data, sel, istate, state)?;
        Ok(())
    }
}

// -------------------------------------------------------------

#[derive(Debug, Default)]
struct SampleEditor;

#[derive(Debug)]
struct SampleEditorState {
    text: TextInputState,
    num1: NumberInputState,
    num2: NumberInputState,
    num3: NumberInputState,
}

impl TableEditor for SampleEditor {
    type State = SampleEditorState;

    fn render(&self, _area: Rect, cell_areas: &[Rect], buf: &mut Buffer, state: &mut Self::State) {
        TextInput::new()
            .styles(THEME.text_style())
            .render(cell_areas[0], buf, &mut state.text);
        NumberInput::new()
            .styles(THEME.text_style())
            .render(cell_areas[1], buf, &mut state.num1);
        NumberInput::new()
            .styles(THEME.text_style())
            .render(cell_areas[2], buf, &mut state.num2);
        NumberInput::new()
            .styles(THEME.text_style())
            .render(cell_areas[3], buf, &mut state.num3);
    }
}

impl SampleEditorState {
    fn new(loc: Locale) -> Result<Self, NumberFmtError> {
        Ok(Self {
            text: Default::default(),
            num1: NumberInputState::named("num1").with_loc_pattern("###,##0.0", loc)?,
            num2: NumberInputState::named("num2").with_loc_pattern("##########", loc)?,
            num3: NumberInputState::named("num3").with_loc_pattern("##########", loc)?,
        })
    }
}

impl TableEditorState for SampleEditorState {
    type Context<'a> = MiniSalsaState;
    type Value = Sample;
    type Err = Error;

    fn create_value(&self, _ctx: &Self::Context<'_>) -> Result<Self::Value, Self::Err> {
        Ok(Sample::default())
    }

    fn set_value(&mut self, data: Sample, _ctx: &Self::Context<'_>) -> Result<(), Error> {
        self.text.set_text(&data.text);
        self.num1.set_value(data.num1)?;
        self.num2.set_value(data.num2)?;
        self.num3.set_value(data.num3)?;
        Ok(())
    }

    fn value(&mut self, _ctx: &Self::Context<'_>) -> Result<Option<Sample>, Error> {
        if self.text.text().is_empty() {
            return Ok(None);
        }

        let mut data = Sample::default();
        data.text = self.text.text().to_string();
        data.num1 = self.num1.value()?;
        data.num2 = self.num2.value()?;
        data.num3 = self.num3.value()?;
        Ok(Some(data))
    }

    fn focused_col(&self) -> Option<usize> {
        match_focus!(
            self.text => Some(0),
            self.num1 => Some(1),
            self.num2 => Some(2),
            self.num3 => Some(3),
            else => None
        )
    }

    fn set_focused_col(&self, _col: usize) {
        // noop
    }
}

impl HasFocus for SampleEditorState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder
            .widget(&self.text)
            .widget(&self.num1)
            .widget(&self.num2)
            .widget(&self.num3);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("silent container")
    }

    fn area(&self) -> Rect {
        unimplemented!("silent container")
    }
}

impl HasScreenCursor for SampleEditorState {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.text
            .screen_cursor()
            .or(self.num1.screen_cursor())
            .or(self.num2.screen_cursor())
            .or(self.num3.screen_cursor())
    }
}

impl<'a> HandleEvent<Event, &'a MiniSalsaState, EditOutcome> for SampleEditorState {
    fn handle(&mut self, event: &Event, ctx: &'a MiniSalsaState) -> EditOutcome {
        ctx.focus_outcome_cell
            .set(FocusBuilder::build_for(self).handle(event, Regular));

        flow!(Outcome::from(self.text.handle(event, Regular)));
        flow!(Outcome::from(self.num1.handle(event, Regular)));
        flow!(Outcome::from(self.num2.handle(event, Regular)));
        flow!(Outcome::from(self.num3.handle(event, Regular)));

        EditOutcome::Continue
    }
}
