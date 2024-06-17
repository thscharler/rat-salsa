#![allow(clippy::collapsible_if)]

use crate::_private::NonExhaustive;
use crate::event::{DoubleClick, DoubleClickOutcome, EditKeys, EditOutcome};
use crate::selection::{CellSelection, RowSelection, RowSetSelection};
use crate::table::data::{DataRepr, DataReprIter};
use crate::textdata::{Row, TextTableData};
use crate::util::{revert_style, transfer_buffer};
use crate::{FTableContext, TableData, TableDataIter, TableSelection};
#[allow(unused_imports)]
use log::debug;
#[cfg(debug_assertions)]
use log::warn;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, FocusKeys, HandleEvent, MouseOnly, Outcome};
use rat_focus::FocusFlag;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
#[cfg(debug_assertions)]
use ratatui::style::Stylize;
#[cfg(debug_assertions)]
use ratatui::text::Text;
use ratatui::widgets::{Block, StatefulWidget, StatefulWidgetRef, Widget};
use std::cell::Cell;
use std::cmp::{max, min};
use std::collections::HashSet;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;
use std::rc::Rc;

/// FTable widget.
///
/// Can be used like a ratatui::Table, but the benefits only
/// show if you use [FTable::data] or [FTable::iter] to set the table data.
///
/// See [FTable::data] and [FTable::iter] for an example.
#[derive(Debug, Default)]
pub struct FTable<'a, Selection> {
    data: DataRepr<'a>,
    no_row_count: bool,

    header: Option<Row<'a>>,
    footer: Option<Row<'a>>,

    widths: Vec<Constraint>,
    flex: Flex,
    column_spacing: u16,
    layout_width: Option<u16>,

    block: Option<Block<'a>>,

    header_style: Option<Style>,
    footer_style: Option<Style>,
    style: Style,

    select_row_style: Option<Style>,
    show_row_focus: bool,
    select_column_style: Option<Style>,
    show_column_focus: bool,
    select_cell_style: Option<Style>,
    show_cell_focus: bool,
    select_header_style: Option<Style>,
    show_header_focus: bool,
    select_footer_style: Option<Style>,
    show_footer_focus: bool,

    focus_style: Option<Style>,

    scroll_gap: Cell<bool>,

    debug: bool,

    _phantom: PhantomData<Selection>,
}

mod data {
    use crate::textdata::TextTableData;
    use crate::{FTableContext, TableData, TableDataIter};
    #[allow(unused_imports)]
    use log::debug;
    #[allow(unused_imports)]
    use log::warn;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Style, Stylize};
    use std::fmt::{Debug, Formatter};

    #[derive(Default)]
    pub(super) enum DataRepr<'a> {
        #[default]
        None,
        Text(TextTableData<'a>),
        Data(Box<dyn TableData<'a> + 'a>),
        Iter(Box<dyn TableDataIter<'a> + 'a>),
    }

    impl<'a> DataRepr<'a> {
        pub(super) fn into_iter(self) -> DataReprIter<'a, 'a> {
            match self {
                DataRepr::None => DataReprIter::None,
                DataRepr::Text(v) => DataReprIter::IterText(v, None),
                DataRepr::Data(v) => DataReprIter::IterData(v, None),
                DataRepr::Iter(v) => DataReprIter::IterIter(v),
            }
        }

        pub(super) fn iter<'b>(&'b self) -> DataReprIter<'a, 'b> {
            match self {
                DataRepr::None => DataReprIter::None,
                DataRepr::Text(v) => DataReprIter::IterDataRef(v, None),
                DataRepr::Data(v) => DataReprIter::IterDataRef(v.as_ref(), None),
                DataRepr::Iter(v) => {
                    // TableDataIter might not implement a valid cloned().
                    if let Some(v) = v.cloned() {
                        DataReprIter::IterIter(v)
                    } else {
                        DataReprIter::Invalid(None)
                    }
                }
            }
        }
    }

    impl<'a> Debug for DataRepr<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Data").finish()
        }
    }

    #[derive(Default)]
    pub(super) enum DataReprIter<'a, 'b> {
        #[default]
        None,
        Invalid(Option<usize>),
        IterText(TextTableData<'a>, Option<usize>),
        IterData(Box<dyn TableData<'a> + 'a>, Option<usize>),
        IterDataRef(&'b dyn TableData<'a>, Option<usize>),
        IterIter(Box<dyn TableDataIter<'a> + 'a>),
    }

    impl<'a, 'b> TableDataIter<'a> for DataReprIter<'a, 'b> {
        fn rows(&self) -> Option<usize> {
            match self {
                DataReprIter::None => Some(0),
                DataReprIter::Invalid(_) => Some(1),
                DataReprIter::IterText(v, _) => Some(v.rows.len()),
                DataReprIter::IterData(v, _) => Some(v.rows()),
                DataReprIter::IterDataRef(v, _) => Some(v.rows()),
                DataReprIter::IterIter(v) => v.rows(),
            }
        }

        fn nth(&mut self, n: usize) -> bool {
            let incr = |row: &mut Option<usize>, rows: usize| match *row {
                None => {
                    *row = Some(n);
                    n < rows
                }
                Some(w) => {
                    *row = Some(w + n + 1);
                    w + n + 1 < rows
                }
            };

            match self {
                DataReprIter::None => false,
                DataReprIter::Invalid(row) => incr(row, 1),
                DataReprIter::IterText(v, row) => incr(row, v.rows.len()),
                DataReprIter::IterData(v, row) => incr(row, v.rows()),
                DataReprIter::IterDataRef(v, row) => incr(row, v.rows()),
                DataReprIter::IterIter(v) => v.nth(n),
            }
        }

        /// Row height.
        fn row_height(&self) -> u16 {
            match self {
                DataReprIter::None => 1,
                DataReprIter::Invalid(_) => 1,
                DataReprIter::IterText(v, n) => v.row_height(n.expect("row")),
                DataReprIter::IterData(v, n) => v.row_height(n.expect("row")),
                DataReprIter::IterDataRef(v, n) => v.row_height(n.expect("row")),
                DataReprIter::IterIter(v) => v.row_height(),
            }
        }

        fn row_style(&self) -> Option<Style> {
            match self {
                DataReprIter::None => None,
                DataReprIter::Invalid(_) => Some(Style::new().white().on_red()),
                DataReprIter::IterText(v, n) => v.row_style(n.expect("row")),
                DataReprIter::IterData(v, n) => v.row_style(n.expect("row")),
                DataReprIter::IterDataRef(v, n) => v.row_style(n.expect("row")),
                DataReprIter::IterIter(v) => v.row_style(),
            }
        }

        /// Render the cell given by column/row.
        fn render_cell(&self, ctx: &FTableContext, column: usize, area: Rect, buf: &mut Buffer) {
            match self {
                DataReprIter::None => {}
                DataReprIter::Invalid(_) => {
                    if column == 0 {
                        #[cfg(debug_assertions)]
                        warn!("FTable::render_ref - TableDataIter must implement a valid cloned() for this to work.");

                        buf.set_string(
                            area.x,
                            area.y,
                            "TableDataIter must implement a valid cloned() for this",
                            Style::default(),
                        );
                    }
                }
                DataReprIter::IterText(v, n) => {
                    v.render_cell(ctx, column, n.expect("row"), area, buf)
                }
                DataReprIter::IterData(v, n) => {
                    v.render_cell(ctx, column, n.expect("row"), area, buf)
                }
                DataReprIter::IterDataRef(v, n) => {
                    v.render_cell(ctx, column, n.expect("row"), area, buf)
                }
                DataReprIter::IterIter(v) => v.render_cell(ctx, column, area, buf),
            }
        }
    }
}

/// Combined style.
#[derive(Debug)]
pub struct FTableStyle {
    pub style: Style,
    pub header_style: Option<Style>,
    pub footer_style: Option<Style>,

    pub select_row_style: Option<Style>,
    pub select_column_style: Option<Style>,
    pub select_cell_style: Option<Style>,
    pub select_header_style: Option<Style>,
    pub select_footer_style: Option<Style>,

    pub show_row_focus: bool,
    pub show_column_focus: bool,
    pub show_cell_focus: bool,
    pub show_header_focus: bool,
    pub show_footer_focus: bool,

    pub focus_style: Option<Style>,

    pub non_exhaustive: NonExhaustive,
}

/// FTable state.
#[derive(Debug, Clone)]
pub struct FTableState<Selection> {
    /// Current focus state.
    pub focus: FocusFlag,

    /// Total area.
    pub area: Rect,

    /// Total header area.
    pub header_area: Rect,
    /// Total table area.
    pub table_area: Rect,
    /// Area per visible row.
    pub row_areas: Vec<Rect>,
    /// Area per visible column, also contains the following spacer if any.
    /// Good for click-hit checks.
    pub column_areas: Vec<Rect>,
    /// Area for each defined column without the spacer.
    /// Columns not visible will have width 0.
    pub base_column_areas: Vec<Rect>,
    /// Total footer area.
    pub footer_area: Rect,

    /// Row count.
    pub rows: usize,
    // debug info
    pub _counted_rows: usize,
    /// Column count.
    pub columns: usize,

    /// Current row offset. Automatically capped at rows-1.
    pub row_offset: usize,
    /// Current column offset. Automatically capped at columns-1.
    pub col_offset: usize,

    /// Current page len as items.
    pub row_page_len: usize,
    /// Current page width as columns.
    pub col_page_len: usize,

    /// Maximum offset for row-scrolling. Can be set higher, but this
    /// offset guarantees a full page display.
    pub max_row_offset: usize,
    /// Maximum offset for column-scrolling. Can be set higher, but this
    /// offset guarantees a full page display.
    pub max_col_offset: usize,

    /// Selection data.
    pub selection: Selection,

    /// Helper for mouse interactions.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl<'a, Selection> FTable<'a, Selection> {
    /// New, empty Table.
    pub fn new() -> Self
    where
        Selection: Default,
    {
        Self::default()
    }

    /// Create a new FTable with preformatted data. For compatibility
    /// with ratatui.
    ///
    /// Use of [FTable::data] is preferred.
    pub fn new_ratatui<R, C>(rows: R, widths: C) -> Self
    where
        R: IntoIterator,
        R::Item: Into<Row<'a>>,
        C: IntoIterator,
        C::Item: Into<Constraint>,
        Selection: Default,
    {
        let widths = widths.into_iter().map(|v| v.into()).collect::<Vec<_>>();
        let data = TextTableData {
            rows: rows.into_iter().map(|v| v.into()).collect(),
        };
        Self {
            data: DataRepr::Text(data),
            widths,
            ..Default::default()
        }
    }

    /// Set preformatted row-data. For compatibility with ratatui.
    ///
    /// Use of [FTable::data] is preferred.
    pub fn rows<T>(mut self, rows: T) -> Self
    where
        T: IntoIterator<Item = Row<'a>>,
    {
        let rows = rows.into_iter().collect();
        self.data = DataRepr::Text(TextTableData { rows });
        self
    }

    /// Set a reference to the TableData facade to your data.
    ///
    /// The way to go is to define a small struct that contains just a
    /// reference to your data. Then implement TableData for this struct.
    ///
    /// ```rust
    /// use ratatui::buffer::Buffer;
    /// use ratatui::layout::Rect;
    /// use ratatui::prelude::Style;
    /// use ratatui::text::Span;
    /// use ratatui::widgets::{StatefulWidget, Widget};
    /// use rat_ftable::{FTable, FTableContext, FTableState, TableData};
    ///
    /// # struct SampleRow;
    /// # let area = Rect::default();
    /// # let mut buf = Buffer::empty(area);
    /// # let buf = &mut buf;
    ///
    /// struct Data1<'a>(&'a [SampleRow]);
    ///
    /// impl<'a> TableData<'a> for Data1<'a> {
    ///     fn rows(&self) -> usize {
    ///         self.0.len()
    ///     }
    ///
    ///     fn row_height(&self, row: usize) -> u16 {
    ///         // to some calculations ...
    ///         1
    ///     }
    ///
    ///     fn row_style(&self, row: usize) -> Style {
    ///         // to some calculations ...
    ///         Style::default()
    ///     }
    ///
    ///     fn render_cell(&self, ctx: &FTableContext, column: usize, row: usize, area: Rect, buf: &mut Buffer) {
    ///         if let Some(data) = self.0.get(row) {
    ///             let rend = match column {
    ///                 0 => Span::from("column1"),
    ///                 1 => Span::from("column2"),
    ///                 2 => Span::from("column3"),
    ///                 _ => return
    ///             };
    ///             rend.render(area, buf);
    ///         }
    ///     }
    /// }
    ///
    /// // When you are creating the table widget you hand over a reference
    /// // to the facade struct.
    ///
    /// let my_data_somewhere_else = vec![SampleRow;999999];
    /// let mut table_state_somewhere_else = FTableState::default();
    ///
    /// // ...
    ///
    /// let table1 = FTable::default().data(Data1(&my_data_somewhere_else));
    /// table1.render(area, buf, &mut table_state_somewhere_else);
    /// ```
    #[inline]
    pub fn data(mut self, data: impl TableData<'a> + 'a) -> Self {
        self.widths = data.widths();
        self.header = data.header();
        self.footer = data.footer();
        self.data = DataRepr::Data(Box::new(data));
        self
    }

    ///
    /// Alternative representation for the data as a kind of Iterator.
    /// It uses interior iteration, which fits quite nice for this and
    /// avoids handing out lifetime bound results of the actual iterator.
    /// Which is a bit nightmarish to get right.
    ///
    ///
    /// Caution: If you can't give the number of rows, the table will iterate over all
    /// the data. See [FTable::no_row_count].
    ///
    /// ```rust
    /// use std::iter::{Enumerate};
    /// use std::slice::Iter;
    /// use format_num_pattern::NumberFormat;
    /// use ratatui::buffer::Buffer;
    /// use ratatui::layout::{Constraint, Rect};
    /// use ratatui::prelude::Color;
    /// use ratatui::style::{Style, Stylize};
    /// use ratatui::text::Span;
    /// use ratatui::widgets::{Widget, StatefulWidget};
    /// use rat_ftable::{FTable, FTableContext, FTableState, TableDataIter};
    ///
    /// # struct Data {
    /// #     table_data: Vec<Sample>
    /// # }
    /// #
    /// # struct Sample {
    /// #     pub text: String
    /// # }
    /// #
    /// # let data = Data {
    /// #     table_data: vec![],
    /// # };
    /// # let area = Rect::default();
    /// # let mut buf = Buffer::empty(area);
    /// # let buf = &mut buf;
    ///
    /// struct RowIter1<'a> {
    ///     iter: Enumerate<Iter<'a, Sample>>,
    ///     item: Option<(usize, &'a Sample)>,
    /// }
    ///
    /// impl<'a> TableDataIter<'a> for RowIter1<'a> {
    ///     fn rows(&self) -> Option<usize> {
    ///         // If you can, give the length. Otherwise,
    ///         // the table will iterate all to find out a length.
    ///         None
    ///         // Some(100_000)
    ///     }
    ///
    ///     /// Select the nth element from the current position.
    ///     fn nth(&mut self, n: usize) -> bool {
    ///         self.item = self.iter.nth(n);
    ///         self.item.is_some()
    ///     }
    ///
    ///     /// Row height.
    ///     fn row_height(&self) -> u16 {
    ///         1
    ///     }
    ///
    ///     /// Row style.
    ///     fn row_style(&self) -> Style {
    ///         Style::default()
    ///     }
    ///
    ///     /// Render one cell.
    ///     fn render_cell(&self,
    ///                     ctx: &FTableContext,
    ///                     column: usize,
    ///                     area: Rect,
    ///                     buf: &mut Buffer)
    ///     {
    ///         let row = self.item.expect("data");
    ///         match column {
    ///             0 => {
    ///                 let row_fmt = NumberFormat::new("000000").expect("fmt");
    ///                 let span = Span::from(row_fmt.fmt_u(row.0));
    ///                 buf.set_style(area, Style::new().black().bg(Color::from_u32(0xe7c787)));
    ///                 span.render(area, buf);
    ///             }
    ///             1 => {
    ///                 let span = Span::from(&row.1.text);
    ///                 span.render(area, buf);
    ///             }
    ///             _ => {}
    ///         }
    ///     }
    /// }
    ///
    /// let mut rit = RowIter1 {
    ///     iter: data.table_data.iter().enumerate(),
    ///     item: None,
    /// };
    ///
    /// let table1 = FTable::default()
    ///     .iter(&mut rit)
    ///     .widths([
    ///         Constraint::Length(6),
    ///         Constraint::Length(20)
    ///     ]);
    ///
    /// let mut table_state_somewhere_else = FTableState::default();
    ///
    /// table1.render(area, buf, &mut table_state_somewhere_else);
    /// ```
    ///
    #[inline]
    pub fn iter(mut self, data: impl TableDataIter<'a> + 'a) -> Self {
        #[cfg(debug_assertions)]
        if data.rows().is_none() {
            warn!("FTable::iter - rows is None, this will be slower");
        }
        self.header = data.header();
        self.footer = data.footer();
        self.widths = data.widths();
        self.data = DataRepr::Iter(Box::new(data));
        self
    }

    /// If you work with an TableDataIter to fill the table, and
    /// if you don't return a count with rows(), FTable will run
    /// through all your iterator to find the actual number of rows.
    ///
    /// This may take its time.
    ///
    /// If you set no_row_count(true), this part will be skipped, and
    /// the row count will be set to an estimate of usize::MAX.
    /// This will destroy your ability to jump to the end of the data,
    /// but otherwise it's fine.
    /// You can still page-down through the data, and if you ever
    /// reach the end, the correct row-count can be established.
    ///
    /// _Extra info_: This might be only useful if you have a LOT of data.
    /// In my test it changed from 1.5ms to 150µs for about 100.000 rows.
    /// And 1.5ms is still not that much ... so you probably want to
    /// test without this first and then decide.
    pub fn no_row_count(mut self, no_row_count: bool) -> Self {
        self.no_row_count = no_row_count;
        self
    }

    /// Set the table-header.
    #[inline]
    pub fn header(mut self, header: Row<'a>) -> Self {
        self.header = Some(header);
        self
    }

    /// Set the table-footer.
    #[inline]
    pub fn footer(mut self, footer: Row<'a>) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Column widths as Constraints.
    pub fn widths<I>(mut self, widths: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        self.widths = widths.into_iter().map(|v| v.into()).collect();
        self
    }

    /// Flex for layout.
    #[inline]
    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }

    /// Spacing between columns.
    #[inline]
    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

    /// Overrides the width of the rendering area for layout purposes.
    /// Layout uses this width, even if it means that some columns are
    /// not visible.
    #[inline]
    pub fn layout_width(mut self, width: u16) -> Self {
        self.layout_width = Some(width);
        self
    }

    /// Draws a block around the table widget.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set all styles as a bundle.
    #[inline]
    pub fn styles(mut self, styles: FTableStyle) -> Self {
        self.style = styles.style;
        self.header_style = styles.header_style;
        self.footer_style = styles.footer_style;

        self.select_row_style = styles.select_row_style;
        self.show_row_focus = styles.show_row_focus;
        self.select_column_style = styles.select_column_style;
        self.show_column_focus = styles.show_column_focus;
        self.select_cell_style = styles.select_cell_style;
        self.show_cell_focus = styles.show_cell_focus;
        self.select_header_style = styles.select_header_style;
        self.show_header_focus = styles.show_header_focus;
        self.select_footer_style = styles.select_footer_style;
        self.show_footer_focus = styles.show_footer_focus;

        self.focus_style = styles.focus_style;
        self
    }

    /// Base style for the table.
    #[inline]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Base style for the table.
    #[inline]
    pub fn header_style(mut self, style: Option<Style>) -> Self {
        self.header_style = style;
        self
    }

    /// Base style for the table.
    #[inline]
    pub fn footer_style(mut self, style: Option<Style>) -> Self {
        self.footer_style = style;
        self
    }

    /// Style for a selected row. The chosen selection must support
    /// row-selection for this to take effect.
    #[inline]
    pub fn select_row_style(mut self, select_style: Option<Style>) -> Self {
        self.select_row_style = select_style;
        self
    }

    /// Add the focus-style to the row-style if the table is focused.
    #[inline]
    pub fn show_row_focus(mut self, show: bool) -> Self {
        self.show_row_focus = show;
        self
    }

    /// Style for a selected column. The chosen selection must support
    /// column-selection for this to take effect.
    #[inline]
    pub fn select_column_style(mut self, select_style: Option<Style>) -> Self {
        self.select_column_style = select_style;
        self
    }

    /// Add the focus-style to the column-style if the table is focused.
    #[inline]
    pub fn show_column_focus(mut self, show: bool) -> Self {
        self.show_column_focus = show;
        self
    }

    /// Style for a selected cell. The chosen selection must support
    /// cell-selection for this to take effect.
    #[inline]
    pub fn select_cell_style(mut self, select_style: Option<Style>) -> Self {
        self.select_cell_style = select_style;
        self
    }

    /// Add the focus-style to the cell-style if the table is focused.
    #[inline]
    pub fn show_cell_focus(mut self, show: bool) -> Self {
        self.show_cell_focus = show;
        self
    }

    /// Style for a selected header cell. The chosen selection must
    /// support column-selection for this to take effect.
    #[inline]
    pub fn select_header_style(mut self, select_style: Option<Style>) -> Self {
        self.select_header_style = select_style;
        self
    }

    /// Add the focus-style to the header-style if the table is focused.
    #[inline]
    pub fn show_header_focus(mut self, show: bool) -> Self {
        self.show_header_focus = show;
        self
    }

    /// Style for a selected footer cell. The chosen selection must
    /// support column-selection for this to take effect.
    #[inline]
    pub fn select_footer_style(mut self, select_style: Option<Style>) -> Self {
        self.select_footer_style = select_style;
        self
    }

    /// Add the footer-style to the table-style if the table is focused.
    #[inline]
    pub fn show_footer_focus(mut self, show: bool) -> Self {
        self.show_footer_focus = show;
        self
    }

    /// This style will be patched onto the selection to indicate that
    /// the widget has the input focus.
    ///
    /// The selection must support some kind of selection for this to
    /// be effective.
    #[inline]
    pub fn focus_style(mut self, focus_style: Option<Style>) -> Self {
        self.focus_style = focus_style;
        self
    }

    /// Just some utility to help with debugging. Usually does nothing.
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
}

impl<'a, Selection> FTable<'a, Selection> {
    /// Does this table need scrollbars?
    /// Returns (horizontal, vertical)
    pub fn need_scroll(&self, area: Rect) -> (bool, bool) {
        //
        // Attention: This must be kept in sync with the actual rendering.
        //
        let inner_area = self.block.inner_if_some(area);
        let l_rows = self.layout_areas(inner_area);
        let table_area = l_rows[1];

        let vertical = match &self.data {
            DataRepr::None => false,
            DataRepr::Text(v) => self.need_scroll_tabledata(v, area),
            DataRepr::Data(v) => self.need_scroll_tabledata(v.as_ref(), area),
            DataRepr::Iter(v) => self.need_scroll_tableiter(v.as_ref(), area),
        };

        // Hack to get some spacing between the last column and
        // the scrollbar.
        if vertical {
            self.scroll_gap.set(true);
        }

        // horizontal layout
        let (width, _, _) = self.layout_columns(table_area.width);
        let horizontal = width > table_area.width;

        (horizontal, vertical)
    }

    fn need_scroll_tableiter(&self, _data: &dyn TableDataIter<'a>, _area: Rect) -> bool {
        // can't iterate data here, have to guess.
        // the guess is `true`.
        true
    }

    #[allow(clippy::let_and_return)]
    fn need_scroll_tabledata(&self, data: &dyn TableData<'a>, area: Rect) -> bool {
        let rows = data.rows();

        // vertical layout
        let inner_area = self.block.inner_if_some(area);
        let l_rows = self.layout_areas(inner_area);
        let table_area = l_rows[1];

        // maximum offsets
        let vertical = 'f: {
            let mut page_height = 0;
            for r in 0..rows {
                let row_height = data.row_height(r);
                if page_height + row_height >= table_area.height {
                    break 'f true;
                }
                page_height += row_height;
            }
            false
        };

        vertical
    }

    // area_width or layout_width
    #[inline]
    fn total_width(&self, area_width: u16) -> u16 {
        let w = if let Some(layout_width) = self.layout_width {
            layout_width
        } else {
            area_width
        };
        if self.scroll_gap.get() {
            w.saturating_sub(1)
        } else {
            w
        }
    }

    // Do the column-layout. Fill in missing columns, if necessary.
    #[inline]
    fn layout_columns(&self, width: u16) -> (u16, Rc<[Rect]>, Rc<[Rect]>) {
        let widths;
        let widths = if self.widths.is_empty() {
            widths = vec![Constraint::Fill(1); 0];
            widths.as_slice()
        } else {
            self.widths.as_slice()
        };

        let width = self.total_width(width);
        let area = Rect::new(0, 0, width, 0);

        let (layout, spacers) = Layout::horizontal(widths)
            .flex(self.flex)
            .spacing(self.column_spacing)
            .split_with_spacers(area);

        (width, layout, spacers)
    }

    // Layout header/table/footer
    #[inline]
    fn layout_areas(&self, area: Rect) -> Rc<[Rect]> {
        let heights = vec![
            Constraint::Length(self.header.as_ref().map(|v| v.height).unwrap_or(0)),
            Constraint::Fill(1),
            Constraint::Length(self.footer.as_ref().map(|v| v.height).unwrap_or(0)),
        ];

        Layout::vertical(heights).split(area)
    }
}

impl<'a, Selection> StatefulWidgetRef for FTable<'a, Selection>
where
    Selection: TableSelection,
{
    type State = FTableState<Selection>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let iter = self.data.iter();
        self.render_iter(iter, area, buf, state);
    }
}

impl<'a, Selection> StatefulWidget for FTable<'a, Selection>
where
    Selection: TableSelection,
{
    type State = FTableState<Selection>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let iter = mem::take(&mut self.data).into_iter();
        self.render_iter(iter, area, buf, state);
    }
}

impl<'a, Selection> FTable<'a, Selection>
where
    Selection: TableSelection,
{
    /// Render an Iterator over TableRowData.
    ///
    /// rows: If the row number is known, this can help.
    ///
    fn render_iter<'b>(
        &self,
        mut data: DataReprIter<'a, 'b>,
        area: Rect,
        buf: &mut Buffer,
        state: &mut FTableState<Selection>,
    ) {
        if let Some(rows) = data.rows() {
            state.rows = rows;
        }
        state.columns = self.widths.len();
        state.area = area;

        // offset validity
        if let Some(rows) = data.rows() {
            if state.row_offset >= rows {
                state.row_offset = rows.saturating_sub(1);
            }
        } else {
            // no validity check here.
            // do it with the first skip.
        }
        if state.col_offset >= state.columns {
            state.col_offset = state.columns.saturating_sub(1);
        }

        // render block
        self.block.render(area, buf);

        // vertical layout
        let inner_area = self.block.inner_if_some(area);
        let l_rows = self.layout_areas(inner_area);
        state.header_area = l_rows[0];
        state.table_area = l_rows[1];
        state.footer_area = l_rows[2];

        // horizontal layout
        let (width, l_columns, l_spacers) = self.layout_columns(state.table_area.width);
        self.calculate_column_areas(state.columns, l_columns.as_ref(), l_spacers.as_ref(), state);

        // render header & footer
        self.render_header(
            state.columns,
            l_columns.as_ref(),
            l_spacers.as_ref(),
            buf,
            state,
        );
        self.render_footer(
            state.columns,
            l_columns.as_ref(),
            l_spacers.as_ref(),
            buf,
            state,
        );

        // render table
        buf.set_style(state.table_area, self.style);

        state.row_areas.clear();
        state.row_page_len = 0;

        let mut row_buf = Buffer::empty(Rect::new(0, 0, width, 1));
        let mut row = None;
        let mut row_y = state.table_area.y;
        let mut row_heights = Vec::new();
        #[cfg(debug_assertions)]
        let mut insane_offset = false;

        let mut ctx = FTableContext {
            focus: state.focus.get(),
            selected_cell: false,
            selected_row: false,
            selected_column: false,
            style: self.style,
            row_style: None,
            select_style: None,
            space_area: Default::default(),
            non_exhaustive: NonExhaustive,
        };

        if data.nth(state.row_offset) {
            row = Some(state.row_offset);
            loop {
                ctx.row_style = data.row_style();
                // We render each row to a temporary buffer.
                // For ease of use we start each row at 0,0.
                // We still only render at least partially visible cells.
                let row_area = Rect::new(0, 0, width, max(data.row_height(), 1));
                // resize should work fine unless the row-heights vary wildly.
                row_buf.resize(row_area);

                if let Some(row_style) = ctx.row_style {
                    row_buf.set_style(row_area, row_style);
                } else {
                    row_buf.set_style(row_area, self.style);
                }

                row_heights.push(row_area.height);

                // Target area for the finished row.
                let visible_row = Rect::new(
                    state.table_area.x,
                    row_y,
                    state.table_area.width,
                    max(data.row_height(), 1),
                )
                .intersection(state.table_area);

                state.row_areas.push(visible_row);
                state.row_page_len += 1;

                // todo: render_row

                let mut col = state.col_offset;
                loop {
                    if col >= state.columns {
                        break;
                    }

                    let cell_area =
                        Rect::new(l_columns[col].x, 0, l_columns[col].width, row_area.height);
                    ctx.space_area = Rect::new(
                        l_spacers[col + 1].x,
                        0,
                        l_spacers[col + 1].width,
                        row_area.height,
                    );

                    ctx.select_style = if state.selection.is_selected_cell(col, row.expect("row")) {
                        ctx.selected_cell = true;
                        ctx.selected_row = false;
                        ctx.selected_column = false;
                        self.patch_select(
                            self.select_cell_style,
                            state.focus.get(),
                            self.show_cell_focus,
                        )
                    } else if state.selection.is_selected_row(row.expect("row")) {
                        ctx.selected_cell = false;
                        ctx.selected_row = true;
                        ctx.selected_column = false;
                        // use a fallback if no row-selected style is set.
                        if self.select_row_style.is_some() {
                            self.patch_select(
                                self.select_row_style,
                                state.focus.get(),
                                self.show_row_focus,
                            )
                        } else {
                            self.patch_select(
                                Some(revert_style(self.style)),
                                state.focus.get(),
                                self.show_row_focus,
                            )
                        }
                    } else if state.selection.is_selected_column(col) {
                        ctx.selected_cell = false;
                        ctx.selected_row = false;
                        ctx.selected_column = true;
                        self.patch_select(
                            self.select_column_style,
                            state.focus.get(),
                            self.show_column_focus,
                        )
                    } else {
                        ctx.selected_cell = false;
                        ctx.selected_row = false;
                        ctx.selected_column = false;
                        None
                    };
                    if let Some(select_style) = ctx.select_style {
                        row_buf.set_style(cell_area, select_style);
                        row_buf.set_style(ctx.space_area, select_style);
                    }

                    data.render_cell(&ctx, col, cell_area, &mut row_buf);

                    if cell_area.right() >= state.table_area.right() {
                        break;
                    }
                    col += 1;
                }

                // render shifted and clipped row.
                transfer_buffer(
                    &mut row_buf,
                    l_columns[state.col_offset].x,
                    visible_row,
                    buf,
                );

                if visible_row.bottom() >= state.table_area.bottom() {
                    break;
                }
                if !data.nth(0) {
                    break;
                }
                row = Some(row.expect("row") + 1);
                row_y += row_area.height;
            }
        } else {
            // can only guess whether the skip failed completely or partially.
            // so don't alter row here.

            // if this first skip fails all bets are off.
            if data.rows().is_none() || data.rows() == Some(0) {
                // this is ok
            } else {
                #[cfg(debug_assertions)]
                {
                    insane_offset = true;
                }
            }
        }

        // maximum offsets
        {
            if let Some(rows) = data.rows() {
                // skip to a guess for the last page.
                // the guess uses row-height is 1, which may read a few more lines than
                // absolutely necessary.
                let skip_rows = rows
                    .saturating_sub(row.map_or(0, |v| v + 1))
                    .saturating_sub(state.table_area.height as usize);
                // if we can still skip some rows, then the data so far is useless.
                if skip_rows > 0 {
                    row_heights.clear();
                }
                let nth_row = skip_rows;
                // collect the remaining row-heights.
                if data.nth(nth_row) {
                    row = Some(row.map_or(nth_row, |row| row + nth_row + 1));
                    loop {
                        row_heights.push(data.row_height());
                        // don't need more.
                        if row_heights.len() > state.table_area.height as usize {
                            row_heights.remove(0);
                        }
                        if !data.nth(0) {
                            break;
                        }
                        row = Some(row.expect("row") + 1);
                        // if the given number of rows is too small, we would overshoot here.
                        if row.expect("row") > rows {
                            break;
                        }
                    }
                    // we break before to have an accurate last page.
                    // but we still want to report an error, if the count is off.
                    while data.nth(0) {
                        row = Some(row.expect("row") + 1);
                    }
                } else {
                    // skip failed, maybe again?
                    // leave everything as is and report later.
                }

                state.rows = rows;
                state._counted_rows = row.map_or(0, |v| v + 1);
            } else if self.no_row_count {
                // We need to feel out a bit beyond the page, otherwise
                // we can't really stabilize the row count and the
                // display starts flickering.
                if row.is_some() {
                    if data.nth(0) {
                        // try one past page
                        row = Some(row.expect("row") + 1);
                        if data.nth(0) {
                            // have an unknown number of rows left.
                            row = Some(usize::MAX - 1);
                        }
                    }
                }

                state.rows = row.map_or(0, |v| v + 1);
                state._counted_rows = row.map_or(0, |v| v + 1);
            } else {
                // Read all the rest to establish the exact row-count.
                while data.nth(0) {
                    row_heights.push(data.row_height());
                    // don't need more info. drop the oldest.
                    if row_heights.len() > state.table_area.height as usize {
                        row_heights.remove(0);
                    }
                    row = Some(row.map_or(0, |v| v + 1));
                }

                state.rows = row.map_or(0, |v| v + 1);
                state._counted_rows = row.map_or(0, |v| v + 1);
            }

            let mut sum_heights = 0;
            let mut n_rows = 0;
            while let Some(h) = row_heights.pop() {
                sum_heights += h;
                n_rows += 1;
                if sum_heights >= state.table_area.height {
                    break;
                }
            }

            // have we got a page worth of data?
            if sum_heights < state.table_area.height {
                state.max_row_offset = 0;
            } else {
                state.max_row_offset = state.rows - n_rows;
            }
        }
        {
            state.max_col_offset = 0;
            let max_right = l_columns.last().map(|v| v.right()).unwrap_or(0);
            for c in (0..state.columns).rev() {
                if max_right - l_columns[c].left() >= state.table_area.width {
                    state.max_col_offset = c;
                    break;
                }
            }
        }

        #[cfg(debug_assertions)]
        {
            use std::fmt::Write;
            let mut msg = String::new();
            if insane_offset {
                _= write!(msg,
                    "FTable::render:\n        offset {}\n        rows {}\n        iter-rows {}max\n    don't match up\n",
                    state.row_offset, state.rows, state._counted_rows
                );
            }
            if state.rows != state._counted_rows {
                _ = write!(
                    msg,
                    "FTable::render:\n    rows {} don't match\n    iterated rows {}",
                    state.rows, state._counted_rows
                );
            }
            if !msg.is_empty() {
                warn!("{}", &msg);
                Text::from(msg)
                    .white()
                    .on_red()
                    .render(state.table_area, buf);
            }
        }
    }

    fn render_footer(
        &self,
        columns: usize,
        l_columns: &[Rect],
        l_spacers: &[Rect],
        buf: &mut Buffer,
        state: &mut FTableState<Selection>,
    ) {
        if let Some(footer) = &self.footer {
            if let Some(footer_style) = footer.style {
                buf.set_style(state.footer_area, footer_style);
            } else if let Some(footer_style) = self.footer_style {
                buf.set_style(state.footer_area, footer_style);
            } else {
                buf.set_style(state.footer_area, self.style);
            }

            let mut col = state.col_offset;
            loop {
                if col >= columns {
                    break;
                }

                let cell_area = Rect::new(
                    state.footer_area.x + l_columns[col].x - l_columns[state.col_offset].x,
                    state.footer_area.y,
                    l_columns[col].width,
                    state.footer_area.height,
                )
                .intersection(state.footer_area);

                let space_area = Rect::new(
                    state.footer_area.x + l_spacers[col + 1].x - l_columns[state.col_offset].x,
                    state.footer_area.y,
                    l_spacers[col + 1].width,
                    state.footer_area.height,
                )
                .intersection(state.footer_area);

                if state.selection.is_selected_column(col) {
                    if let Some(selected_style) = self.patch_select(
                        self.select_footer_style,
                        state.focus.get(),
                        self.show_footer_focus,
                    ) {
                        buf.set_style(cell_area, selected_style);
                        buf.set_style(space_area, selected_style);
                    }
                };

                if let Some(cell) = footer.cells.get(col) {
                    if let Some(cell_style) = cell.style {
                        buf.set_style(cell_area, cell_style);
                    }
                    cell.content.clone().render(cell_area, buf);
                }

                if cell_area.right() >= state.footer_area.right() {
                    break;
                }

                col += 1;
            }
        }
    }

    fn render_header(
        &self,
        columns: usize,
        l_columns: &[Rect],
        l_spacers: &[Rect],
        buf: &mut Buffer,
        state: &mut FTableState<Selection>,
    ) {
        if let Some(header) = &self.header {
            if let Some(header_style) = header.style {
                buf.set_style(state.header_area, header_style);
            } else if let Some(header_style) = self.header_style {
                buf.set_style(state.header_area, header_style);
            } else {
                buf.set_style(state.header_area, self.style);
            }

            let mut col = state.col_offset;
            loop {
                if col >= columns {
                    break;
                }

                let cell_area = Rect::new(
                    state.header_area.x + l_columns[col].x - l_columns[state.col_offset].x,
                    state.header_area.y,
                    l_columns[col].width,
                    state.header_area.height,
                )
                .intersection(state.header_area);

                let space_area = Rect::new(
                    state.header_area.x + l_spacers[col + 1].x - l_columns[state.col_offset].x,
                    state.header_area.y,
                    l_spacers[col + 1].width,
                    state.header_area.height,
                )
                .intersection(state.header_area);

                if state.selection.is_selected_column(col) {
                    if let Some(selected_style) = self.patch_select(
                        self.select_header_style,
                        state.focus.get(),
                        self.show_header_focus,
                    ) {
                        buf.set_style(cell_area, selected_style);
                        buf.set_style(space_area, selected_style);
                    }
                };

                if let Some(cell) = header.cells.get(col) {
                    if let Some(cell_style) = cell.style {
                        buf.set_style(cell_area, cell_style);
                    }
                    cell.content.clone().render(cell_area, buf);
                }

                if cell_area.right() >= state.header_area.right() {
                    break;
                }

                col += 1;
            }
        }
    }

    fn calculate_column_areas(
        &self,
        columns: usize,
        l_columns: &[Rect],
        l_spacers: &[Rect],
        state: &mut FTableState<Selection>,
    ) {
        state.column_areas.clear();
        state.col_page_len = 0;

        let mut col = state.col_offset;
        loop {
            if col >= columns {
                break;
            }

            // merge the column + the folling spacer as the
            // column area.
            let mut column_area = Rect::new(
                state.table_area.x + l_columns[col].x - l_columns[state.col_offset].x,
                state.table_area.y,
                l_columns[col].width,
                state.table_area.height,
            );
            state
                .base_column_areas
                .push(column_area.intersection(state.table_area));

            column_area.width += l_spacers[col + 1].width;
            state
                .column_areas
                .push(column_area.intersection(state.table_area));

            state.col_page_len += 1;

            if column_area.right() >= state.table_area.right() {
                break;
            }

            col += 1;
        }

        // Base areas for every column.
        state.base_column_areas.clear();
        for col in 0..columns {
            let column_area = if col < state.col_offset {
                Rect::new(0, state.table_area.y, 0, state.table_area.height)
            } else {
                Rect::new(
                    state.table_area.x + l_columns[col].x - l_columns[state.col_offset].x,
                    state.table_area.y,
                    l_columns[col].width,
                    state.table_area.height,
                )
            };
            let column_area = column_area.intersection(state.table_area);
            state.base_column_areas.push(column_area);
        }
    }

    fn patch_select(&self, style: Option<Style>, focus: bool, show: bool) -> Option<Style> {
        if let Some(style) = style {
            if let Some(focus_style) = self.focus_style {
                if focus && show {
                    Some(style.patch(focus_style))
                } else {
                    Some(style)
                }
            } else {
                Some(style)
            }
        } else {
            None
        }
    }
}

impl Default for FTableStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            header_style: None,
            footer_style: None,
            select_row_style: None,
            select_column_style: None,
            select_cell_style: None,
            select_header_style: None,
            select_footer_style: None,
            show_row_focus: false,
            show_column_focus: false,
            show_cell_focus: false,
            show_header_focus: false,
            show_footer_focus: false,
            focus_style: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection: Default> Default for FTableState<Selection> {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            area: Default::default(),
            header_area: Default::default(),
            table_area: Default::default(),
            footer_area: Default::default(),
            row_areas: Default::default(),
            column_areas: Default::default(),
            base_column_areas: Default::default(),
            rows: 0,
            _counted_rows: 0,
            columns: 0,
            row_offset: 0,
            col_offset: 0,
            row_page_len: 0,
            col_page_len: 0,
            max_row_offset: 0,
            max_col_offset: 0,
            selection: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection> FTableState<Selection> {
    /// Renders the widget in focused style.
    ///
    /// This flag is not used for event-handling.
    #[inline]
    pub fn set_focused(&mut self, focus: bool) {
        self.focus.focus.set(focus);
    }

    /// Renders the widget in focused style.
    ///
    /// This flag is not used for event-handling.
    #[inline]
    pub fn is_focused(&mut self) -> bool {
        self.focus.focus.get()
    }

    /// Number of rows.
    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Number of columns.
    #[inline]
    pub fn columns(&self) -> usize {
        self.columns
    }

    /// Returns the whole row-area and the cell-areas for the
    /// given row, if it is visible.
    ///
    /// Attention: These areas might be 0-length if the column is scrolled
    /// beyond the table-area.
    ///
    /// See: [FTableState::scroll_to]
    pub fn row_cells(&self, row: usize) -> Option<(Rect, Vec<Rect>)> {
        if row < self.row_offset || row >= self.row_offset + self.row_page_len {
            return None;
        }

        let mut areas = Vec::new();

        let r = self.row_areas[row];
        for c in &self.base_column_areas {
            areas.push(Rect::new(c.x, r.y, c.width, r.height));
        }

        Some((r, areas))
    }

    /// Cell at given position.
    pub fn cell_at_clicked(&self, pos: (u16, u16)) -> Option<(usize, usize)> {
        let col = self.column_at_clicked(pos);
        let row = self.row_at_clicked(pos);

        match (col, row) {
            (Some(col), Some(row)) => Some((col, row)),
            _ => None,
        }
    }

    /// Column at given position.
    pub fn column_at_clicked(&self, pos: (u16, u16)) -> Option<usize> {
        rat_event::util::column_at_clicked(&self.column_areas, pos.0).map(|v| self.col_offset + v)
    }

    /// Row at given position.
    pub fn row_at_clicked(&self, pos: (u16, u16)) -> Option<usize> {
        rat_event::util::row_at_clicked(&self.row_areas, pos.1).map(|v| self.row_offset + v)
    }

    /// Cell when dragging. Can go outside the area.
    pub fn cell_at_drag(&self, pos: (u16, u16)) -> (usize, usize) {
        let col = self.column_at_drag(pos);
        let row = self.row_at_drag(pos);

        (col, row)
    }

    /// Row when dragging. Can go outside the area.
    pub fn row_at_drag(&self, pos: (u16, u16)) -> usize {
        match rat_event::util::row_at_drag(self.table_area, &self.row_areas, pos.1) {
            Ok(v) => self.row_offset + v,
            Err(v) if v <= 0 => self.row_offset.saturating_sub((-v) as usize),
            Err(v) => self.row_offset + self.row_areas.len() + v as usize,
        }
    }

    /// Column when dragging. Can go outside the area.
    pub fn column_at_drag(&self, pos: (u16, u16)) -> usize {
        match rat_event::util::column_at_drag(self.table_area, &self.column_areas, pos.0) {
            Ok(v) => self.col_offset + v,
            Err(v) if v <= 0 => self.col_offset.saturating_sub(1),
            Err(_v) => self.col_offset + self.column_areas.len() + 1,
        }
    }

    /// Sets both offsets to 0.
    pub fn clear_offset(&mut self) {
        self.row_offset = 0;
        self.col_offset = 0;
    }
}

impl<Selection: TableSelection> FTableState<Selection> {
    /// Scroll to selected.
    pub fn scroll_to_selected(&mut self) {
        if let Some(selected) = self.selection.lead_selection() {
            self.scroll_to(selected)
        }
    }

    /// Scroll to position.
    pub fn scroll_to(&mut self, pos: (usize, usize)) {
        if self.row_offset + self.row_page_len <= pos.1 {
            self.set_vertical_offset(pos.1 - self.row_page_len + 1);
        }
        if self.row_offset > pos.1 {
            self.set_vertical_offset(pos.1);
        }

        if self.col_offset + self.col_page_len <= pos.0 {
            self.set_horizontal_offset(pos.0 - self.col_page_len + 1);
        }
        if self.col_offset > pos.0 {
            self.set_horizontal_offset(pos.0);
        }
    }
}

impl<Selection: TableSelection> FTableState<Selection> {
    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    pub fn vertical_max_offset(&self) -> usize {
        self.max_row_offset
    }

    /// Current vertical offset.
    pub fn vertical_offset(&self) -> usize {
        self.row_offset
    }

    /// Vertical page-size at the current offset.
    pub fn vertical_page(&self) -> usize {
        self.row_page_len
    }

    /// Suggested scroll per scroll-event.
    pub fn vertical_scroll(&self) -> usize {
        max(self.vertical_page() / 10, 1)
    }

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    pub fn horizontal_max_offset(&self) -> usize {
        self.max_col_offset
    }

    /// Current horizontal offset.
    pub fn horizontal_offset(&self) -> usize {
        self.col_offset
    }

    /// Horizontal page-size at the current offset.
    pub fn horizontal_page(&self) -> usize {
        self.col_page_len
    }

    /// Suggested scroll per scroll-event.
    pub fn horizontal_scroll(&self) -> usize {
        max(self.horizontal_page() / 10, 1)
    }

    /// Change the vertical offset.
    ///
    /// Due to overscroll it's possible that this is an invalid offset for the widget.
    /// The widget must deal with this situation.
    ///
    /// The widget returns true if the offset changed at all.
    pub fn set_vertical_offset(&mut self, offset: usize) -> bool {
        let old_offset = self.row_offset;
        if offset >= self.rows {
            self.row_offset = self.rows;
        } else {
            self.row_offset = offset;
        }
        old_offset != self.row_offset
    }

    /// Change the horizontal offset.
    ///
    /// Due to overscroll it's possible that this is an invalid offset for the widget.
    /// The widget must deal with this situation.
    ///
    /// The widget returns true if the offset changed at all.
    pub fn set_horizontal_offset(&mut self, offset: usize) -> bool {
        let old_offset = self.col_offset;
        if offset >= self.columns {
            self.col_offset = self.columns;
        } else {
            self.col_offset = offset;
        }
        old_offset != self.col_offset
    }

    /// Scroll up by n items.
    /// The widget returns true if the offset changed at all.
    pub fn scroll_up(&mut self, n: usize) -> bool {
        self.set_vertical_offset(self.vertical_offset().saturating_sub(n))
    }

    /// Scroll down by n items.
    /// The widget returns true if the offset changed at all.
    pub fn scroll_down(&mut self, n: usize) -> bool {
        self.set_vertical_offset(min(self.vertical_offset() + n, self.vertical_max_offset()))
    }

    /// Scroll up by n items.
    /// The widget returns true if the offset changed at all.
    pub fn scroll_left(&mut self, n: usize) -> bool {
        self.set_horizontal_offset(self.horizontal_offset().saturating_sub(n))
    }

    /// Scroll down by n items.
    /// The widget returns true if the offset changed at all.
    pub fn scroll_right(&mut self, n: usize) -> bool {
        self.set_horizontal_offset(min(
            self.horizontal_offset() + n,
            self.horizontal_max_offset(),
        ))
    }
}

impl FTableState<RowSelection> {
    /// Scroll selection instead of offset.
    #[inline]
    pub fn set_scroll_selection(&mut self, scroll: bool) {
        self.selection.set_scroll_selected(scroll);
    }

    /// Scroll selection instead of offset.
    #[inline]
    pub fn scroll_selection(&self) -> bool {
        self.selection.scroll_selected()
    }

    /// Clear offsets and selection.
    #[inline]
    pub fn clear(&mut self) {
        self.clear_offset();
        self.clear_selection();
    }

    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.selection.has_selection()
    }

    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.selection.selected()
    }

    #[inline]
    pub fn select(&mut self, row: Option<usize>) {
        self.selection.select(row);
    }

    /// Select a row, clamp between 0 and maximum.
    #[inline]
    pub fn select_clamped(&mut self, select: usize, maximum: usize) {
        self.selection.select_clamped(select, maximum);
    }
}

impl FTableState<RowSetSelection> {
    /// Clear offsets and selection.
    #[inline]
    pub fn clear(&mut self) {
        self.clear_offset();
        self.clear_selection();
    }

    /// Clear the selection.
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.selection.has_selection()
    }

    #[inline]
    pub fn selected(&self) -> HashSet<usize> {
        self.selection.selected()
    }

    #[inline]
    pub fn set_lead(&mut self, lead: Option<usize>, extend: bool) {
        self.selection.set_lead(lead, extend);
    }

    /// Set a new lead, at the same time limit the lead to max.
    #[inline]
    pub fn set_lead_clamped(&mut self, lead: usize, max: usize, extend: bool) {
        self.selection.set_lead_clamped(lead, max, extend);
    }

    /// Current lead.
    #[inline]
    pub fn lead(&self) -> Option<usize> {
        self.selection.lead()
    }

    /// Current anchor.
    #[inline]
    pub fn anchor(&self) -> Option<usize> {
        self.selection.anchor()
    }

    /// Add to selection.
    #[inline]
    pub fn add_selected(&mut self, idx: usize) {
        self.selection.add(idx);
    }

    /// Remove from selection. Only works for retired selections, not for the
    /// active anchor-lead range.
    #[inline]
    pub fn remove_selected(&mut self, idx: usize) {
        self.selection.remove(idx);
    }
}

impl FTableState<CellSelection> {
    /// Clear offsets and selection.
    #[inline]
    pub fn clear(&mut self) {
        self.clear_offset();
        self.clear_selection();
    }

    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Selected cell.
    #[inline]
    pub fn selected(&self) -> Option<(usize, usize)> {
        self.selection.selected()
    }

    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.selection.has_selection()
    }

    /// Select a cell.
    #[inline]
    pub fn select_cell(&mut self, select: Option<(usize, usize)>) -> bool {
        self.selection.select_cell(select)
    }

    /// Select a row. Column stays the same.
    #[inline]
    pub fn select_row(&mut self, select: Option<usize>) -> bool {
        self.selection.select_row(select)
    }

    /// Select a column, row stays the same.
    #[inline]
    pub fn select_column(&mut self, select: Option<usize>) -> bool {
        self.selection.select_column(select)
    }

    /// Select a cell, clamp between 0 and maximum.
    #[inline]
    pub fn select_clamped(&mut self, select: (usize, usize), maximum: (usize, usize)) -> bool {
        self.selection.select_clamped(select, maximum)
    }
}

impl<Selection> HandleEvent<crossterm::event::Event, DoubleClick, DoubleClickOutcome>
    for FTableState<Selection>
{
    /// Handles double-click events on the table.
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        _keymap: DoubleClick,
    ) -> DoubleClickOutcome {
        match event {
            ct_event!(mouse any for m) if self.mouse.doubleclick(self.table_area, m) => {
                if let Some((col, row)) = self.cell_at_clicked((m.column, m.row).into()) {
                    DoubleClickOutcome::ClickClick(col, row)
                } else {
                    DoubleClickOutcome::NotUsed
                }
            }
            _ => DoubleClickOutcome::NotUsed,
        }
    }
}

/// Handle all events for recognizing double-clicks.
pub fn handle_doubleclick_events<Selection: TableSelection>(
    state: &mut FTableState<Selection>,
    event: &crossterm::event::Event,
) -> DoubleClickOutcome {
    state.handle(event, DoubleClick)
}

impl<Selection: TableSelection> HandleEvent<crossterm::event::Event, EditKeys, EditOutcome>
    for FTableState<Selection>
where
    Self: HandleEvent<crossterm::event::Event, FocusKeys, Outcome>,
{
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: EditKeys) -> EditOutcome {
        match event {
            ct_event!(keycode press Insert) => EditOutcome::Insert,
            ct_event!(keycode press Delete) => EditOutcome::Remove,
            ct_event!(keycode press Enter) => EditOutcome::Edit,
            ct_event!(keycode press Down) => {
                if let Some((_column, row)) = self.selection.lead_selection() {
                    if row == self.rows().saturating_sub(1) {
                        return EditOutcome::Append;
                    }
                }
                <Self as HandleEvent<_, FocusKeys, Outcome>>::handle(self, event, FocusKeys).into()
            }

            ct_event!(keycode release  Insert)
            | ct_event!(keycode release Delete)
            | ct_event!(keycode release Enter)
            | ct_event!(keycode release Down) => EditOutcome::Unchanged,

            _ => {
                <Self as HandleEvent<_, FocusKeys, Outcome>>::handle(self, event, FocusKeys).into()
            }
        }
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_edit_events<Selection: TableSelection>(
    state: &mut FTableState<Selection>,
    focus: bool,
    event: &crossterm::event::Event,
) -> EditOutcome
where
    FTableState<Selection>: HandleEvent<crossterm::event::Event, FocusKeys, Outcome>,
    FTableState<Selection>: HandleEvent<crossterm::event::Event, MouseOnly, Outcome>,
{
    if focus {
        state.handle(event, EditKeys)
    } else {
        let r = state.handle(event, MouseOnly);
        r.into()
    }
}
