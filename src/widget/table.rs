///
/// Extensions for [ratatui::widgets::Table]
///
use crate::util::{next, next_opt, prev, prev_opt};
use crate::widget::selected::{NoSelection, Selection, SingleSelection};
use crate::widget::{ActionTrigger, HasVerticalScroll};
use crate::FocusFlag;
use crate::{ControlUI, HasFocusFlag};
use crate::{DefaultKeys, HandleCrossterm, Input, MouseOnly};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Position, Rect};
use ratatui::prelude::*;
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, HighlightSpacing, Row, Table, TableState};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem;

/// Add some minor fixes to [ratatui::widgets::Table]
#[derive()]
pub struct TableExt<'a, SEL> {
    ///
    pub rows: Vec<Row<'a>>,
    ///
    pub table: Table<'a>,
    /// Row count
    pub len: usize,

    /// Base style
    pub base_style: Style,
    /// Style for selected + not focused.
    pub select_style: Style,
    /// Style for selected + focused.
    pub focus_style: Style,

    pub _phantom: PhantomData<SEL>,
}

impl<'a, SEL> Debug for TableExt<'a, SEL> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableExt")
            .field("rows", &self.rows)
            .field("table", &self.table)
            .field("len", &self.len)
            .field("base_style", &self.base_style)
            .field("select_style", &self.select_style)
            .field("focus_style", &self.focus_style)
            .finish()
    }
}

impl<'a, SEL> Default for TableExt<'a, SEL> {
    fn default() -> Self {
        Self {
            rows: Default::default(),
            table: Default::default(),
            len: 0,
            base_style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            _phantom: Default::default(),
        }
    }
}

/// Combined style.
#[derive(Debug, Default)]
pub struct TableExtStyle {
    pub style: Style,
    pub select_style: Style,
    pub focus_style: Style,
}

impl<'a, SEL> TableExt<'a, SEL> {
    pub fn new<R, C>(rows: R, widths: C) -> Self
    where
        R: IntoIterator,
        R::Item: Into<Row<'a>>,
        C: IntoIterator,
        C::Item: Into<Constraint>,
    {
        let rows = rows.into_iter().map(|v| v.into()).collect::<Vec<_>>();
        let len = rows.len();

        Self {
            rows,
            table: Table::default().widths(widths),
            len,
            base_style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            _phantom: Default::default(),
        }
    }

    pub fn rows<T>(mut self, rows: T) -> Self
    where
        T: IntoIterator<Item = Row<'a>>,
    {
        let rows = rows.into_iter().collect::<Vec<_>>();
        self.len = rows.len();
        self.rows = rows;
        self
    }

    pub fn header(mut self, header: Row<'a>) -> Self {
        self.table = self.table.header(header);
        self
    }

    pub fn footer(mut self, footer: Row<'a>) -> Self {
        self.table = self.table.footer(footer);
        self
    }

    pub fn widths<I>(mut self, widths: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        self.table = self.table.widths(widths);
        self
    }

    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.table = self.table.column_spacing(spacing);
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.table = self.table.block(block);
        self
    }

    pub fn styles(mut self, styles: TableExtStyle) -> Self {
        self.base_style = styles.style;
        self.select_style = styles.select_style;
        self.focus_style = styles.focus_style;
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.base_style = style.into();
        self
    }

    pub fn select_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_style = select_style.into();
        self
    }

    pub fn focus_style<S: Into<Style>>(mut self, focus_style: S) -> Self {
        self.focus_style = focus_style.into();
        self
    }

    pub fn select_symbol<T: Into<Text<'a>>>(mut self, select_symbol: T) -> Self {
        self.table = self.table.highlight_symbol(select_symbol);
        self
    }

    pub fn select_spacing(mut self, value: HighlightSpacing) -> Self {
        self.table = self.table.highlight_spacing(value);
        self
    }

    pub fn flex(mut self, flex: Flex) -> Self {
        self.table = self.table.flex(flex);
        self
    }
}

impl<'a, SEL: Selection> StatefulWidget for TableExt<'a, SEL> {
    type State = TableExtState<SEL>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.len = self.len;

        // if state.gained_focus() {
        //     if state.table_state.selected().is_none() {
        //         state.table_state.select(Some(0));
        //     }
        // }

        let selection = &state.selection;

        for (i, r) in self.rows.iter_mut().enumerate() {
            let style = if state.focus.get() {
                if selection.is_selected(i) {
                    self.focus_style
                } else {
                    self.base_style
                }
            } else {
                if selection.is_selected(i) {
                    self.select_style
                } else {
                    self.base_style
                }
            };

            *r = mem::take(r).style(style);
        }

        let table = self.table.style(self.base_style).rows(self.rows);

        StatefulWidget::render(table, area, buf, &mut state.table_state);
    }
}

impl<'a, SEL> Styled for TableExt<'a, SEL> {
    type Item = TableExt<'a, SEL>;

    fn style(&self) -> Style {
        <Table<'_> as Styled>::style(&self.table)
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.table = self.table.set_style(style);
        self
    }
}

impl<'a, SEL, Item> FromIterator<Item> for TableExt<'a, SEL>
where
    Item: Into<Row<'a>>,
{
    fn from_iter<U: IntoIterator<Item = Item>>(iter: U) -> Self {
        let rows = iter.into_iter().map(|v| v.into()).collect::<Vec<_>>();
        let len = rows.len();

        Self {
            rows,
            table: Table::default(),
            len,
            base_style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            _phantom: Default::default(),
        }
    }
}

/// Extended TableState, contains a [ratatui::widgets::TableState].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TableExtState<SEL> {
    pub focus: FocusFlag,
    pub area: Rect,
    pub trigger: ActionTrigger,
    pub len: usize,
    pub table_state: TableState,
    pub selection: SEL,
}

impl<SEL: Selection> HasFocusFlag for TableExtState<SEL> {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl<SEL: Selection> HasVerticalScroll for TableExtState<SEL> {
    fn vlen(&self) -> usize {
        self.len
    }

    fn voffset(&self) -> usize {
        self.table_state.offset()
    }

    fn set_voffset(&mut self, offset: usize) {
        *self.table_state.offset_mut() = offset;
    }

    fn vpage(&self) -> usize {
        self.area.height as usize
    }
}

impl<SEL: Selection> TableExtState<SEL> {
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.table_state = self.table_state.with_offset(offset);
        self
    }

    pub fn offset(&self) -> usize {
        self.table_state.offset()
    }

    pub fn offset_mut(&mut self) -> &mut usize {
        self.table_state.offset_mut()
    }

    pub fn selection(&self) -> &SEL {
        &self.selection
    }

    pub fn selection_mut(&mut self) -> &mut SEL {
        &mut self.selection
    }

    /// Scroll to selected.
    pub fn adjust_view(&mut self) {
        if let Some(selected) = self.selection.lead_selection() {
            if self.table_state.offset() + (self.area.height as usize) < selected {
                *self.table_state.offset_mut() = selected - (self.area.height as usize);
            }
            if self.table_state.offset() > selected {
                *self.table_state.offset_mut() = selected;
            }
        }
    }
}

#[derive(Debug)]
pub enum InputRequest {
    /// Select first row
    First,
    /// Select last row
    Last,
    /// Select new row
    Down(usize),
    /// Select prev row
    Up(usize),
    /// Select by mouse click
    Select(usize),
    /// Mouse page up
    ScrollDown(usize),
    /// Mouse page down
    ScrollUp(usize),
}

impl<A, E> HandleCrossterm<ControlUI<A, E>> for TableExtState<SingleSelection> {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        let req = match event {
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.focus.get() {
                    Some(InputRequest::Down(1))
                } else {
                    None
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.focus.get() {
                    Some(InputRequest::Up(1))
                } else {
                    None
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::End,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.focus.get() {
                    Some(InputRequest::Last)
                } else {
                    None
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Home,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.focus.get() {
                    Some(InputRequest::First)
                } else {
                    None
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::PageUp,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.focus.get() {
                    Some(InputRequest::Up(self.vpage() / 2))
                } else {
                    None
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::PageDown,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.focus.get() {
                    Some(InputRequest::Down(self.vpage() / 2))
                } else {
                    None
                }
            }
            _ => return self.handle(event, MouseOnly),
        };

        if let Some(req) = req {
            self.perform(req)
        } else {
            ControlUI::Continue
        }
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for TableExtState<SingleSelection> {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        let req = match event {
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    Some(InputRequest::ScrollDown(self.vpage() / 5))
                } else {
                    None
                }
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    Some(InputRequest::ScrollUp(self.vpage() / 5))
                } else {
                    None
                }
            }
            Event::Mouse(
                MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    modifiers: KeyModifiers::NONE,
                }
                | MouseEvent {
                    kind: MouseEventKind::Drag(MouseButton::Left),
                    column,
                    row,
                    modifiers: KeyModifiers::NONE,
                },
            ) => {
                if self.area.contains(Position::new(*column, *row)) {
                    let rr = row - self.area.y;
                    if (self.table_state.offset() + rr as usize) < self.len {
                        let sel = self.table_state.offset() + rr as usize;
                        if self.selection.lead_selection() != Some(sel) {
                            Some(InputRequest::Select(sel))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            _ => None,
        };

        if let Some(req) = req {
            self.perform(req)
        } else {
            ControlUI::Continue
        }
    }
}

/// Extra mapping which does the double click a line in the table thing.
/// Returns `ControlUI::Action(())` if a double click is detected.
///
/// ```rust ignore
///      uistate.page1.table1.handle(evt, DoubleClick).and_then(|_| {
///          if let Some(file) = data.files[uistate.page1.table1.selected().unwrap()] {
///              let file_path = data
///                  .config
///                  .base_path
///                  .join(format!("{}.ods", &file));
///              Control::Action(RunCalc(file_path))
///          } else {
///              Control::Err(anyhow::Error::msg("Nothing to do."))
///          }
///      })
/// ```
#[derive(Debug)]
pub struct DoubleClick;

impl<E, SEL: Selection> HandleCrossterm<ControlUI<(), E>, DoubleClick> for TableExtState<SEL> {
    fn handle(&mut self, event: &Event, _: DoubleClick) -> ControlUI<(), E> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.is_focused() {
                    ControlUI::Run(())
                } else {
                    ControlUI::Continue
                }
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    let rr = row - self.area.y;
                    let sel = self.table_state.offset() + rr as usize;

                    // this cannot be accomplished otherwise. the return type is bitching.
                    if self.selection.lead_selection() == Some(sel) {
                        if self.trigger.pull(200) {
                            ControlUI::Run(())
                        } else {
                            ControlUI::NoChange
                        }
                    } else {
                        self.trigger.reset();
                        ControlUI::NoChange
                    }
                } else {
                    ControlUI::Continue
                }
            }
            _ => ControlUI::Continue,
        }
    }
}

/// Extra mapping that reacts to the delete-key in a table.
///
/// Returns [ControlUI::Run(usize)] for which row should be deleted.
#[derive(Debug)]
pub struct DeleteRow;

impl<E, SEL: Selection> HandleCrossterm<ControlUI<usize, E>, DeleteRow> for TableExtState<SEL> {
    fn handle(&mut self, event: &Event, _: DeleteRow) -> ControlUI<usize, E> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Delete,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.focus.get() {
                    if let Some(selected) = self.selection.lead_selection() {
                        ControlUI::Run(selected)
                    } else {
                        ControlUI::Continue
                    }
                } else {
                    ControlUI::Continue
                }
            }
            _ => ControlUI::Continue,
        }
    }
}

impl<A, E> Input<ControlUI<A, E>> for TableExtState<SingleSelection> {
    type Request = InputRequest;

    fn perform(&mut self, req: Self::Request) -> ControlUI<A, E> {
        match req {
            InputRequest::Down(n) => {
                let next = next_opt(self.table_state.selected(), n, self.len - 1);
                self.table_state.select(next);
                self.adjust_view();
                ControlUI::Change
            }
            InputRequest::Up(n) => {
                let prev = prev_opt(self.table_state.selected(), n);
                self.table_state.select(prev);
                self.adjust_view();
                ControlUI::Change
            }
            InputRequest::First => {
                self.table_state.select(Some(0));
                self.adjust_view();
                ControlUI::Change
            }
            InputRequest::Last => {
                self.table_state.select(Some(self.len - 1));
                self.adjust_view();
                ControlUI::Change
            }
            InputRequest::Select(n) => {
                if self.table_state.selected() == Some(n) {
                    ControlUI::Continue
                } else {
                    self.table_state.select(Some(n));
                    self.adjust_view();
                    ControlUI::Change
                }
            }

            InputRequest::ScrollDown(n) => {
                self.trigger.reset();
                let next = next(self.table_state.offset(), n, self.len - 1);
                *self.table_state.offset_mut() = next;
                ControlUI::Change
            }
            InputRequest::ScrollUp(n) => {
                self.trigger.reset();
                let prev = prev(self.table_state.offset(), n);
                *self.table_state.offset_mut() = prev;
                ControlUI::Change
            }
        }
    }
}
