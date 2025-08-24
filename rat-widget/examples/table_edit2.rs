//!
//! Example for [TableData]
//!

use crate::mini_salsa::theme::THEME;
use crate::mini_salsa::{run_ui, setup_logging, MiniSalsaState};
use anyhow::Error;
use format_num_pattern::{NumberFmtError, NumberFormat, NumberSymbols};
use pure_rust_locales::Locale;
use pure_rust_locales::Locale::de_AT_euro;
use rat_event::{ConsumedEvent, HandleEvent, Outcome, Regular};
use rat_focus::{match_focus, FocusBuilder, FocusFlag, HasFocus};
use rat_ftable::edit::vec::{EditableTableVec, EditableTableVecState};
use rat_ftable::edit::{TableEditor, TableEditorState};
use rat_ftable::textdata::{Cell, Row};
use rat_ftable::{Table, TableContext, TableData};
use rat_scrolled::Scroll;
use rat_text::number_input::{NumberInput, NumberInputState};
use rat_text::text_input::{TextInput, TextInputState};
use rat_text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::{block, Block, StatefulWidget, Widget};
use ratatui::Frame;
use std::cmp::max;

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

    let mut data = Data {};

    let mut state = State {
        loc: de_AT_euro,
        table: EditableTableVecState::new(SampleEditorState::new(de_AT_euro)?),
        text1: Default::default(),
        text2: Default::default(),
    };

    state.table.set_value(
        data::TINY_DATA
            .iter()
            .map(|v| Sample {
                text: v.to_string(),
                num1: rand::random(),
                num2: rand::random(),
                num3: rand::random(),
            })
            .collect(),
    );
    state.table.table.select(Some(0));

    run_ui(
        "table_edit2",
        handle_table,
        repaint_table,
        &mut data,
        &mut state,
    )
}

#[derive(Debug, Default, Clone)]
struct Sample {
    pub(crate) text: String,
    pub(crate) num1: f32,
    pub(crate) num2: u32,
    pub(crate) num3: u32,
}

struct Data {}

struct State {
    loc: Locale,

    table: EditableTableVecState<SampleEditorState>,
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

fn repaint_table(
    frame: &mut Frame<'_>,
    area: Rect,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
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

    TextInput::new().styles(THEME.input_style()).render(
        Rect::new(l0[0].x, l1[0].y, l0[0].width, l1[0].height),
        frame.buffer_mut(),
        &mut state.text1,
    );

    EditableTableVec::new(
        |data: &[Sample]| {
            Table::default()
                .data(TableData1 {
                    data: Default::default(),
                    fmt1: NumberFormat::news("###,##0.0", NumberSymbols::numeric(state.loc))
                        .expect("fmt"),
                    fmt2: NumberFormat::news("##########", NumberSymbols::numeric(state.loc))
                        .expect("fmt"),
                })
                .column_spacing(1)
                .auto_layout_width()
                .block(
                    Block::bordered()
                        .border_type(block::BorderType::Rounded)
                        .border_style(THEME.block())
                        .title("tabledata"),
                )
                .vscroll(Scroll::new().style(THEME.block()))
                .styles(THEME.table_style())
        },
        SampleEditor,
    )
    .render(l0[1], frame.buffer_mut(), &mut state.table);

    TextInput::new().styles(THEME.input_style()).render(
        Rect::new(l0[2].x, l1[0].y, l0[2].width, l1[0].height),
        frame.buffer_mut(),
        &mut state.text2,
    );

    if let Some(cursor) = state.screen_cursor() {
        frame.set_cursor_position(cursor);
    }

    Ok(())
}

fn handle_table(
    event: &crossterm::event::Event,
    _data: &mut Data,
    istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, Error> {
    let f = FocusBuilder::build_for(state).handle(event, Regular);

    let mut r = Outcome::Continue;
    r = r.or_else(|| state.text1.handle(event, Regular).into());
    r = r.or_else(|| {
        state.table.handle(event, istate).unwrap_or_else(|e| {
            istate.status[0] = format!("{}", e);
            Outcome::Changed
        })
    });
    r = r.or_else(|| state.text2.handle(event, Regular).into());

    Ok(max(Outcome::from(r), f))
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
            .styles(THEME.input_style())
            .render(cell_areas[0], buf, &mut state.text);
        NumberInput::new()
            .styles(THEME.input_style())
            .render(cell_areas[1], buf, &mut state.num1);
        NumberInput::new()
            .styles(THEME.input_style())
            .render(cell_areas[2], buf, &mut state.num2);
        NumberInput::new()
            .styles(THEME.input_style())
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

    fn create_value(&self, _ctx: &MiniSalsaState) -> Result<Sample, Error> {
        Ok(Sample {
            text: Default::default(),
            num1: Default::default(),
            num2: Default::default(),
            num3: Default::default(),
        })
    }

    fn set_value(&mut self, value: Sample, _ctx: &MiniSalsaState) -> Result<(), Error> {
        self.text.set_text(&value.text);
        self.num1.set_value(value.num1)?;
        self.num2.set_value(value.num2)?;
        self.num3.set_value(value.num3)?;
        Ok(())
    }

    fn value(&mut self, _ctx: &MiniSalsaState) -> Result<Option<Sample>, Error> {
        if self.text.text().is_empty() {
            return Ok(None);
        }

        let mut value = Sample::default();
        value.text = self.text.text().to_string();
        value.num1 = self.num1.value()?;
        value.num2 = self.num2.value()?;
        value.num3 = self.num3.value()?;
        Ok(Some(value))
    }

    fn focused_col(&self) -> Option<usize> {
        match_focus!(
            self.text => Some(0),
            self.num1 => Some(1),
            self.num2 => Some(2),
            self.num3 => Some(3)
            , _ => None
        )
    }

    fn set_focused_col(&self, col: usize) {
        let focus = FocusBuilder::build_for(self);
        match col {
            0 => focus.focus(&self.text),
            1 => focus.focus(&self.num1),
            2 => focus.focus(&self.num1),
            3 => focus.focus(&self.num1),
            _ => {}
        }
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

impl HandleEvent<crossterm::event::Event, &MiniSalsaState, Result<Outcome, Error>>
    for SampleEditorState
{
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        _ctx: &MiniSalsaState,
    ) -> Result<Outcome, Error> {
        let f = FocusBuilder::build_for(self).handle(event, Regular);

        let mut r = Outcome::Continue;
        r = r.or_else(|| self.text.handle(event, Regular).into());
        r = r.or_else(|| self.num1.handle(event, Regular).into());
        r = r.or_else(|| self.num2.handle(event, Regular).into());
        r = r.or_else(|| self.num3.handle(event, Regular).into());

        Ok(max(f, r))
    }
}
