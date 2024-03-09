use crate::focus::FocusFlag;
use crate::util::{next_opt, next_pg_opt, prev_opt, prev_pg_opt};
use crate::{ControlUI, HandleEvent};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Position, Rect};
use ratatui::prelude::*;
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, HighlightSpacing, Row, Table, TableState};

#[derive(Debug)]
pub struct TableExt<'a> {
    pub table: Table<'a>,
    pub row_count: usize,
    pub select_style: Style,
    pub focus_style: Style,
}

#[derive(Debug, Default)]
pub struct TableExtStyle {
    pub style: Style,
    pub select_style: Style,
    pub focus_style: Style,
}

impl<'a> Default for TableExt<'a> {
    fn default() -> Self {
        Self {
            table: Default::default(),
            row_count: 0,
            select_style: Default::default(),
            focus_style: Default::default(),
        }
    }
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

impl<'a> Widget for TableExt<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Widget::render(self.table, area, buf);
    }
}

impl<'a> StatefulWidget for TableExt<'a> {
    type State = TableExtState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.row_count = self.row_count;

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
        <Table as Styled>::style(&self.table)
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

/// Extended TableState, contains a ratatui::TableState.
#[derive(Debug)]
pub struct TableExtState {
    pub focus: FocusFlag,
    pub area: Rect,
    pub click: bool,
    pub row_count: usize,
    pub table_state: TableState,
}

impl Default for TableExtState {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            click: false,
            table_state: TableState::default().with_selected(0),
            row_count: 0,
            area: Default::default(),
        }
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

impl TableExtState {
    pub fn handle_doubleclick<A, E>(&mut self, evt: &Event, action: A) -> ControlUI<A, E> {
        match evt {
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => 'f: {
                if self.click {
                    // ignore selection click itself.
                    self.click = false;
                    break 'f ControlUI::Continue;
                }

                if self.area.contains(Position::new(*column, *row)) {
                    let rr = row - self.area.y;
                    let sel = self.table_state.offset() + rr as usize;

                    if self.table_state.selected() == Some(sel) {
                        break 'f ControlUI::Action(action);
                    }
                }

                ControlUI::Continue
            }

            _ => ControlUI::Continue,
        }
    }
}

impl<'a, A, E> HandleEvent<A, E> for TableExtState {
    fn handle(&mut self, evt: &Event) -> ControlUI<A, E> {
        match evt {
            Event::Key(KeyEvent {
                code: cc,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.focus.get() {
                    match cc {
                        KeyCode::Down => {
                            self.scroll_down();
                            ControlUI::Changed
                        }
                        KeyCode::Up => {
                            self.scroll_up();
                            ControlUI::Changed
                        }
                        KeyCode::PageDown => {
                            self.scroll_pg_down();
                            ControlUI::Changed
                        }
                        KeyCode::PageUp => {
                            self.scroll_pg_up();
                            ControlUI::Changed
                        }

                        _ => ControlUI::Continue,
                    }
                } else {
                    ControlUI::Continue
                }
            }

            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_scr_down();
                    ControlUI::Changed
                } else {
                    ControlUI::Continue
                }
            }

            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_scr_up();
                    ControlUI::Changed
                } else {
                    ControlUI::Continue
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
                            self.click = true;
                            self.table_state.select(Some(sel));
                            ControlUI::Changed
                        } else {
                            ControlUI::Unchanged
                        }
                    } else {
                        ControlUI::Unchanged
                    }
                } else {
                    ControlUI::Continue
                }
            }

            _ => ControlUI::Continue,
        }
    }
}
