use crate::util::{next_opt, next_pg_opt, prev_opt, prev_pg_opt};
use crate::widget::ActionTrigger;
use crate::FocusFlag;
use crate::{ControlUI, HasFocusFlag};
use crate::{DefaultKeys, HandleCrossterm, Input, MouseOnly};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Position, Rect};
use ratatui::prelude::*;
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, HighlightSpacing, Row, Table, TableState};

/// Add some minor fixes to [ratatui::widgets::Table]
#[derive(Debug, Default)]
pub struct TableExt<'a> {
    pub table: Table<'a>,
    /// Row count
    pub row_count: usize,
    /// Style for selected + not focused.
    pub select_style: Style,
    /// Style for selected + focused.
    pub focus_style: Style,
}

/// Combined style.
#[derive(Debug, Default)]
pub struct TableExtStyle {
    pub style: Style,
    pub select_style: Style,
    pub focus_style: Style,
}

impl<'a> TableExt<'a> {
    pub fn new<R, C>(rows: R, widths: C) -> Self
    where
        R: IntoIterator,
        R::Item: Into<Row<'a>>,
        C: IntoIterator,
        C::Item: Into<Constraint>,
    {
        let rows = rows.into_iter().collect::<Vec<_>>();
        let row_count = rows.len();

        Self {
            table: Table::new(rows, widths),
            row_count,
            select_style: Default::default(),
            focus_style: Default::default(),
        }
    }

    pub fn rows<T>(mut self, rows: T) -> Self
    where
        T: IntoIterator<Item = Row<'a>>,
    {
        let rows = rows.into_iter().collect::<Vec<_>>();
        self.row_count = rows.len();
        self.table = self.table.rows(rows);
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

    pub fn style(mut self, styles: TableExtStyle) -> Self {
        self.table = self.table.style(styles.style);
        self.select_style = styles.select_style;
        self.focus_style = styles.focus_style;
        self
    }

    pub fn base_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.table = self.table.style(style);
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

impl<'a> StatefulWidget for TableExt<'a> {
    type State = TableExtState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.row_count = self.row_count;

        if state.gained_focus() {
            if state.table_state.selected().is_none() {
                state.table_state.select(Some(0));
            }
        }

        let table = if state.focus.get() {
            self.table.highlight_style(self.focus_style)
        } else {
            self.table.highlight_style(self.select_style)
        };

        StatefulWidget::render(table, area, buf, &mut state.table_state);
    }
}

impl<'a> Styled for TableExt<'a> {
    type Item = TableExt<'a>;

    fn style(&self) -> Style {
        <Table<'_> as Styled>::style(&self.table)
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.table = self.table.set_style(style);
        self
    }
}

impl<'a, Item> FromIterator<Item> for TableExt<'a>
where
    Item: Into<Row<'a>>,
{
    fn from_iter<U: IntoIterator<Item = Item>>(iter: U) -> Self {
        let rows = iter.into_iter().map(|v| v.into()).collect::<Vec<_>>();
        let row_count = rows.len();

        Self {
            table: Table::from_iter(rows),
            row_count,
            select_style: Default::default(),
            focus_style: Default::default(),
        }
    }
}

/// Extended TableState, contains a [ratatui::widgets::TableState].
#[derive(Debug)]
pub struct TableExtState {
    pub focus: FocusFlag,
    pub area: Rect,
    pub trigger: ActionTrigger,
    pub row_count: usize,
    pub table_state: TableState,
}

impl Default for TableExtState {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            table_state: TableState::default().with_selected(0),
            row_count: 0,
            area: Default::default(),
            trigger: Default::default(),
        }
    }
}

impl HasFocusFlag for TableExtState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl TableExtState {
    pub fn offset(&self) -> usize {
        self.table_state.offset()
    }

    pub fn selected(&self) -> Option<usize> {
        self.table_state.selected()
    }

    pub fn select(&mut self, select: Option<usize>) {
        self.table_state.select(select);
    }

    pub fn adjust_view(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            if self.table_state.offset() + (self.area.height as usize) < selected {
                *self.table_state.offset_mut() = selected - (self.area.height as usize);
            }
        }
        if let Some(selected) = self.table_state.selected() {
            if self.table_state.offset() > selected {
                *self.table_state.offset_mut() = selected;
            }
        }
    }

    pub fn scroll_down(&mut self) {
        let next = next_opt(self.table_state.selected(), self.row_count);
        self.table_state.select(next);
        self.adjust_view();
    }

    pub fn scroll_up(&mut self) {
        let prev = prev_opt(self.table_state.selected());
        self.table_state.select(prev);
        self.adjust_view();
    }

    pub fn scroll_first(&mut self) {
        self.table_state.select(Some(0));
        self.adjust_view();
    }

    pub fn scroll_last(&mut self) {
        self.table_state.select(Some(self.row_count - 1));
        self.adjust_view();
    }

    pub fn scroll_pg_down(&mut self) {
        let next = next_pg_opt(
            self.table_state.selected(),
            self.area.height as usize / 2,
            self.row_count,
        );
        self.table_state.select(next);
        self.adjust_view();
    }

    pub fn scroll_pg_up(&mut self) {
        let prev = prev_pg_opt(self.table_state.selected(), self.area.height as usize / 2);
        self.table_state.select(prev);
        self.adjust_view();
    }

    pub fn scroll_scr_down(&mut self) {
        let next = next_pg_opt(
            self.table_state.selected(),
            self.area.height as usize / 5,
            self.row_count,
        );
        self.table_state.select(next);
        self.adjust_view();
    }

    pub fn scroll_scr_up(&mut self) {
        let prev = prev_pg_opt(self.table_state.selected(), self.area.height as usize / 5);
        self.table_state.select(prev);
        self.adjust_view();
    }
}

#[derive(Debug)]
pub enum InputRequest {
    First,
    Last,
    Down,
    Up,
    PageDown,
    PageUp,
    MouseScrollDown,
    MouseScrollUp,
    MouseSelect(usize),
    // MouseAction(usize, u64), // todo: ??? can't be done this way
}

impl<A, E> HandleCrossterm<ControlUI<A, E>> for TableExtState {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        let req = match event {
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.focus.get() {
                    Some(InputRequest::Down)
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
                    Some(InputRequest::Up)
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
                    Some(InputRequest::PageUp)
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
                    Some(InputRequest::PageDown)
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

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for TableExtState {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        let req = match event {
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    Some(InputRequest::MouseScrollDown)
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
                    Some(InputRequest::MouseScrollUp)
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
                    if (self.table_state.offset() + rr as usize) < self.row_count {
                        let sel = self.table_state.offset() + rr as usize;
                        if self.table_state.selected() != Some(sel) {
                            Some(InputRequest::MouseSelect(sel))
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
/// Returns `ControlUI::Action(true)` if a double click is detected.
///
/// ```rust ignore
/// uistate.page1.table1.handle(evt, DefaultKeys).or_else(|| {
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
///  })
/// ```
#[derive(Debug)]
pub struct DoubleClick;

impl<E> HandleCrossterm<ControlUI<(), E>, DoubleClick> for TableExtState {
    fn handle(&mut self, event: &Event, _: DoubleClick) -> ControlUI<(), E> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.focus.get() {
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
                    if self.table_state.selected() == Some(sel) {
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

#[derive(Debug)]
pub struct DeleteRow;

impl<E> HandleCrossterm<ControlUI<usize, E>, DeleteRow> for TableExtState {
    fn handle(&mut self, event: &Event, _: DeleteRow) -> ControlUI<usize, E> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Delete,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.focus.get() {
                    if let Some(selected) = self.table_state.selected() {
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

impl<A, E> Input<ControlUI<A, E>> for TableExtState {
    type Request = InputRequest;

    fn perform(&mut self, req: Self::Request) -> ControlUI<A, E> {
        match req {
            InputRequest::Down => {
                self.trigger.reset();
                self.scroll_down();
                ControlUI::Change
            }
            InputRequest::Up => {
                self.trigger.reset();
                self.scroll_up();
                ControlUI::Change
            }
            InputRequest::First => {
                self.trigger.reset();
                self.scroll_first();
                ControlUI::Change
            }
            InputRequest::Last => {
                self.trigger.reset();
                self.scroll_last();
                ControlUI::Change
            }
            InputRequest::PageDown => {
                self.trigger.reset();
                self.scroll_pg_down();
                ControlUI::Change
            }
            InputRequest::PageUp => {
                self.trigger.reset();
                self.scroll_pg_up();
                ControlUI::Change
            }
            InputRequest::MouseScrollDown => {
                self.trigger.reset();
                self.scroll_scr_down();
                ControlUI::Change
            }
            InputRequest::MouseScrollUp => {
                self.trigger.reset();
                self.scroll_scr_up();
                ControlUI::Change
            }
            InputRequest::MouseSelect(i) => {
                if self.table_state.selected() == Some(i) {
                    ControlUI::Continue
                } else {
                    self.trigger.reset();
                    self.table_state.select(Some(i));
                    ControlUI::Change
                }
            }
        }
    }
}
