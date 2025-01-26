#![allow(clippy::collapsible_if)]

use crate::_private::NonExhaustive;
use crate::event::{DoubleClick, DoubleClickOutcome};
use crate::selection::{CellSelection, RowSelection, RowSetSelection};
use crate::table::data::{DataRepr, DataReprIter};
use crate::textdata::{Row, TextTableData};
use crate::util::{fallback_select_style, revert_style, transfer_buffer};
use crate::{TableContext, TableData, TableDataIter, TableSelection};
use rat_event::util::MouseFlags;
use rat_event::{ct_event, HandleEvent};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::{relocate_area, relocate_areas, RelocatableState};
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::cmp::{max, min};
use std::collections::HashSet;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;
use std::rc::Rc;

/// Table widget.
///
/// Can be used as a drop-in replacement for the ratatui table. But
/// that's not the point of this widget.
///
/// This widget uses the [TableData](crate::TableData) trait instead
/// of rendering all the table-cells and putting them into a Vec.
/// This way rendering time only depends on the screen-size not on
/// the size of your data.
///
/// There is a second trait [TableDataIter](crate::TableDataIter) that
/// works better if you only have an Iterator over your data.
///
/// See [Table::data] and [Table::iter] for an example.
#[derive(Debug)]
pub struct Table<'a, Selection> {
    data: DataRepr<'a>,
    no_row_count: bool,

    header: Option<Row<'a>>,
    footer: Option<Row<'a>>,

    widths: Vec<Constraint>,
    flex: Flex,
    column_spacing: u16,
    layout_width: Option<u16>,
    auto_layout_width: bool,

    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,

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

    debug: bool,

    _phantom: PhantomData<Selection>,
}

mod data {
    use crate::textdata::TextTableData;
    use crate::{TableContext, TableData, TableDataIter};
    #[cfg(debug_assertions)]
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
        // TODO: maybe add an Owned where data is kept in the state?
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

        #[cfg(feature = "unstable-widget-ref")]
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

    impl Debug for DataRepr<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Data").finish()
        }
    }

    #[derive(Default)]
    pub(super) enum DataReprIter<'a, 'b> {
        #[default]
        None,
        #[allow(dead_code)]
        Invalid(Option<usize>),
        IterText(TextTableData<'a>, Option<usize>),
        IterData(Box<dyn TableData<'a> + 'a>, Option<usize>),
        #[allow(dead_code)]
        IterDataRef(&'b dyn TableData<'a>, Option<usize>),
        IterIter(Box<dyn TableDataIter<'a> + 'a>),
    }

    impl<'a> TableDataIter<'a> for DataReprIter<'a, '_> {
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
                    *row < Some(rows)
                }
                Some(w) => {
                    *row = Some(w.saturating_add(n).saturating_add(1));
                    *row < Some(rows)
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
        fn render_cell(&self, ctx: &TableContext, column: usize, area: Rect, buf: &mut Buffer) {
            match self {
                DataReprIter::None => {}
                DataReprIter::Invalid(_) => {
                    if column == 0 {
                        #[cfg(debug_assertions)]
                        warn!("Table::render_ref - TableDataIter must implement a valid cloned() for this to work.");

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
pub struct TableStyle {
    pub style: Style,
    pub header: Option<Style>,
    pub footer: Option<Style>,

    pub select_row: Option<Style>,
    pub select_column: Option<Style>,
    pub select_cell: Option<Style>,
    pub select_header: Option<Style>,
    pub select_footer: Option<Style>,

    pub show_row_focus: bool,
    pub show_column_focus: bool,
    pub show_cell_focus: bool,
    pub show_header_focus: bool,
    pub show_footer_focus: bool,

    pub focus_style: Option<Style>,

    pub block: Option<Block<'static>>,
    pub border_style: Option<Style>,
    pub scroll: Option<ScrollStyle>,

    pub non_exhaustive: NonExhaustive,
}

/// Table state.
#[derive(Debug)]
pub struct TableState<Selection> {
    /// Current focus state.
    /// __read+write__
    pub focus: FocusFlag,

    /// Total area.
    /// __read only__ Renewed with each render.
    pub area: Rect,
    /// Area inside the border and scrollbars
    /// __read only__ Renewed with each render.
    pub inner: Rect,

    /// Total header area.
    /// __read only__ Renewed with each render.
    pub header_area: Rect,
    /// Total table area.
    /// __read only__ Renewed with each render.
    pub table_area: Rect,
    /// Area per visible row. The first element is at row_offset.
    /// __read only__ Renewed with each render.
    pub row_areas: Vec<Rect>,
    /// Area for each column plus the following spacer if any.
    /// Invisible columns have width 0, height is the height of the table_area.
    /// __read only__ Renewed with each render.
    pub column_areas: Vec<Rect>,
    /// Layout areas for each column plus the following spacer if any.
    /// Positions are 0-based, y and height are 0.
    /// __read only__ Renewed with each render.
    pub column_layout: Vec<Rect>,
    /// Total footer area.
    /// __read only__ Renewed with each render.
    pub footer_area: Rect,

    /// Row count.
    /// __read+write__ Renewed with each render anyway.
    pub rows: usize,
    // debug info
    pub _counted_rows: usize,
    /// Column count.
    /// __read only__ Renewed with each render.
    pub columns: usize,

    /// Row scrolling data.
    /// __read+write__ max_offset set with each render.
    pub vscroll: ScrollState,
    /// Column scrolling data.
    /// __read+write__ max_offset set with each render.
    pub hscroll: ScrollState,

    /// Selection data.
    /// __read+write__ selection model. selection is not bound by rows.
    pub selection: Selection,

    /// Helper for mouse interactions.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl<Selection> Default for Table<'_, Selection> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            no_row_count: Default::default(),
            header: Default::default(),
            footer: Default::default(),
            widths: Default::default(),
            flex: Default::default(),
            column_spacing: Default::default(),
            layout_width: Default::default(),
            auto_layout_width: Default::default(),
            block: Default::default(),
            hscroll: Default::default(),
            vscroll: Default::default(),
            header_style: Default::default(),
            footer_style: Default::default(),
            style: Default::default(),
            select_row_style: Default::default(),
            show_row_focus: true,
            select_column_style: Default::default(),
            show_column_focus: Default::default(),
            select_cell_style: Default::default(),
            show_cell_focus: Default::default(),
            select_header_style: Default::default(),
            show_header_focus: Default::default(),
            select_footer_style: Default::default(),
            show_footer_focus: Default::default(),
            focus_style: Default::default(),
            debug: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<'a, Selection> Table<'a, Selection> {
    /// New, empty Table.
    pub fn new() -> Self
    where
        Selection: Default,
    {
        Self::default()
    }

    /// Create a new Table with preformatted data. For compatibility
    /// with ratatui.
    ///
    /// Use of [Table::data] is preferred.
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
    /// Use of [Table::data] is preferred.
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
    /// use rat_ftable::{Table, TableContext, TableState, TableData};    ///
    /// #
    /// use rat_ftable::selection::RowSelection;
    ///
    /// struct SampleRow;
    /// # impl Clone for SampleRow {
    /// #     fn clone(&self) -> Self {
    /// #        SampleRow
    /// #     }
    /// # }
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
    ///     fn row_style(&self, row: usize) -> Option<Style> {
    ///         // to some calculations ...
    ///         None
    ///     }
    ///
    ///     fn render_cell(&self, ctx: &TableContext, column: usize, row: usize, area: Rect, buf: &mut Buffer) {
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
    /// let mut table_state_somewhere_else = TableState::<RowSelection>::default();
    ///
    /// // ...
    ///
    /// let table1 = Table::default().data(Data1(&my_data_somewhere_else));
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
    /// the data. See [Table::no_row_count].
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
    /// use rat_ftable::{Table, TableContext, TableState, TableDataIter};
    /// use rat_ftable::selection::RowSelection;
    ///
    /// struct Data {
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
    ///     fn row_style(&self) -> Option<Style> {
    ///         Some(Style::default())
    ///     }
    ///
    ///     /// Render one cell.
    ///     fn render_cell(&self,
    ///                     ctx: &TableContext,
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
    /// let table1 = Table::default()
    ///     .iter(rit)
    ///     .widths([
    ///         Constraint::Length(6),
    ///         Constraint::Length(20)
    ///     ]);
    ///
    /// let mut table_state_somewhere_else = TableState::<RowSelection>::default();
    ///
    /// table1.render(area, buf, &mut table_state_somewhere_else);
    /// ```
    ///
    #[inline]
    pub fn iter(mut self, data: impl TableDataIter<'a> + 'a) -> Self {
        #[cfg(debug_assertions)]
        if data.rows().is_none() {
            use log::warn;
            warn!("Table::iter - rows is None, this will be slower");
        }
        self.header = data.header();
        self.footer = data.footer();
        self.widths = data.widths();
        self.data = DataRepr::Iter(Box::new(data));
        self
    }

    /// If you work with an TableDataIter to fill the table, and
    /// if you don't return a count with rows(), Table will run
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

    /// Calculates the width from the given column-constraints.
    /// If a fixed layout_width() is set too, that one will win.
    ///
    /// Panic:
    /// Rendering will panic, if any constraint other than Constraint::Length(),
    /// Constraint::Min() or Constraint::Max() is used.
    #[inline]
    pub fn auto_layout_width(mut self) -> Self {
        self.auto_layout_width = true;
        self
    }

    /// Draws a block around the table widget.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Scrollbars
    pub fn scroll(mut self, scroll: Scroll<'a>) -> Self {
        self.hscroll = Some(scroll.clone().override_horizontal());
        self.vscroll = Some(scroll.override_vertical());
        self
    }

    /// Scrollbars
    pub fn hscroll(mut self, scroll: Scroll<'a>) -> Self {
        self.hscroll = Some(scroll.override_horizontal());
        self
    }

    /// Scrollbars
    pub fn vscroll(mut self, scroll: Scroll<'a>) -> Self {
        self.vscroll = Some(scroll.override_vertical());
        self
    }

    /// Set all styles as a bundle.
    #[inline]
    pub fn styles(mut self, styles: TableStyle) -> Self {
        self.style = styles.style;
        if styles.header.is_some() {
            self.header_style = styles.header;
        }
        if styles.footer.is_some() {
            self.footer_style = styles.footer;
        }
        if styles.select_row.is_some() {
            self.select_row_style = styles.select_row;
        }
        self.show_row_focus = styles.show_row_focus;
        if styles.select_column.is_some() {
            self.select_column_style = styles.select_column;
        }
        self.show_column_focus = styles.show_column_focus;
        if styles.select_cell.is_some() {
            self.select_cell_style = styles.select_cell;
        }
        self.show_cell_focus = styles.show_cell_focus;
        if styles.select_header.is_some() {
            self.select_header_style = styles.select_header;
        }
        self.show_header_focus = styles.show_header_focus;
        if styles.select_footer.is_some() {
            self.select_footer_style = styles.select_footer;
        }
        self.show_footer_focus = styles.show_footer_focus;
        if styles.focus_style.is_some() {
            self.focus_style = styles.focus_style;
        }
        // TODO: add border_style for other XXStyles too.
        if let Some(border_style) = styles.border_style {
            self.block = self.block.map(|v| v.border_style(border_style));
        }
        self.block = self.block.map(|v| v.style(self.style));
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if let Some(styles) = styles.scroll {
            self.hscroll = self.hscroll.map(|v| v.styles(styles.clone()));
            self.vscroll = self.vscroll.map(|v| v.styles(styles));
        }
        self
    }

    /// Base style for the table.
    #[inline]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self.block = self.block.map(|v| v.style(self.style));
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

impl<Selection> Table<'_, Selection> {
    // area_width or layout_width
    #[inline]
    fn total_width(&self, area_width: u16) -> u16 {
        if let Some(layout_width) = self.layout_width {
            layout_width
        } else if self.auto_layout_width {
            let mut width = 0;
            for w in &self.widths {
                match w {
                    Constraint::Min(v) => width += *v + self.column_spacing,
                    Constraint::Max(v) => width += *v + self.column_spacing,
                    Constraint::Length(v) => width += *v + self.column_spacing,
                    _ => unimplemented!("Invalid layout constraint."),
                }
            }
            width
        } else {
            area_width
        }
    }

    // Do the column-layout. Fill in missing columns, if necessary.
    #[inline]
    fn layout_columns(&self, width: u16) -> (u16, Rc<[Rect]>, Rc<[Rect]>) {
        let width = self.total_width(width);
        let area = Rect::new(0, 0, width, 0);

        let (layout, spacers) = Layout::horizontal(&self.widths)
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

#[cfg(feature = "unstable-widget-ref")]
impl<'a, Selection> StatefulWidgetRef for Table<'a, Selection>
where
    Selection: TableSelection,
{
    type State = TableState<Selection>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let iter = self.data.iter();
        self.render_iter(iter, area, buf, state);
    }
}

impl<Selection> StatefulWidget for Table<'_, Selection>
where
    Selection: TableSelection,
{
    type State = TableState<Selection>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let iter = mem::take(&mut self.data).into_iter();
        self.render_iter(iter, area, buf, state);
    }
}

impl<'a, Selection> Table<'a, Selection>
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
        state: &mut TableState<Selection>,
    ) {
        if let Some(rows) = data.rows() {
            state.rows = rows;
        }
        state.columns = self.widths.len();
        state.area = area;

        let sa = ScrollArea::new()
            .style(self.style)
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        state.inner = sa.inner(area, Some(&state.hscroll), Some(&state.vscroll));

        let l_rows = self.layout_areas(state.inner);
        state.header_area = l_rows[0];
        state.table_area = l_rows[1];
        state.footer_area = l_rows[2];

        // horizontal layout
        let (width, l_columns, l_spacers) = self.layout_columns(state.table_area.width);
        self.calculate_column_areas(state.columns, l_columns.as_ref(), l_spacers.as_ref(), state);

        // render block+scroll
        sa.render(
            area,
            buf,
            &mut ScrollAreaState::new()
                .h_scroll(&mut state.hscroll)
                .v_scroll(&mut state.vscroll),
        );

        // render header & footer
        self.render_header(
            state.columns,
            width,
            l_columns.as_ref(),
            l_spacers.as_ref(),
            state.header_area,
            buf,
            state,
        );
        self.render_footer(
            state.columns,
            width,
            l_columns.as_ref(),
            l_spacers.as_ref(),
            state.footer_area,
            buf,
            state,
        );

        // render table
        state.row_areas.clear();
        state.vscroll.set_page_len(0);
        state.hscroll.set_page_len(area.width as usize);

        let mut row_buf = Buffer::empty(Rect::new(0, 0, width, 1));
        let mut row = None;
        let mut row_y = state.table_area.y;
        let mut row_heights = Vec::new();
        #[cfg(debug_assertions)]
        let mut insane_offset = false;

        let mut ctx = TableContext {
            focus: state.focus.get(),
            selected_cell: false,
            selected_row: false,
            selected_column: false,
            style: self.style,
            row_style: None,
            select_style: None,
            space_area: Default::default(),
            row_area: Default::default(),
            non_exhaustive: NonExhaustive,
        };

        if data.nth(state.vscroll.offset()) {
            row = Some(state.vscroll.offset());
            loop {
                ctx.row_style = data.row_style();
                // We render each row to a temporary buffer.
                // For ease of use we start each row at 0,0.
                // We still only render at least partially visible cells.
                let render_row_area = Rect::new(0, 0, width, data.row_height());
                ctx.row_area = render_row_area;
                row_buf.resize(render_row_area);
                if let Some(row_style) = ctx.row_style {
                    row_buf.set_style(render_row_area, row_style);
                } else {
                    row_buf.set_style(render_row_area, self.style);
                }
                row_heights.push(render_row_area.height);

                // Target area for the finished row.
                let visible_row_area = Rect::new(
                    state.table_area.x,
                    row_y,
                    state.table_area.width,
                    render_row_area.height,
                )
                .intersection(state.table_area);
                state.row_areas.push(visible_row_area);
                // only count fully visible rows.
                if render_row_area.height == visible_row_area.height {
                    state.vscroll.set_page_len(state.vscroll.page_len() + 1);
                }

                // can skip this entirely
                if render_row_area.height > 0 {
                    let mut col = 0;
                    loop {
                        if col >= state.columns {
                            break;
                        }

                        let render_cell_area = Rect::new(
                            l_columns[col].x,
                            0,
                            l_columns[col].width,
                            render_row_area.height,
                        );
                        ctx.space_area = Rect::new(
                            l_spacers[col + 1].x,
                            0,
                            l_spacers[col + 1].width,
                            render_row_area.height,
                        );

                        if state.selection.is_selected_cell(col, row.expect("row")) {
                            ctx.selected_cell = true;
                            ctx.selected_row = false;
                            ctx.selected_column = false;
                            ctx.select_style = self.patch_select(
                                self.select_cell_style,
                                state.focus.get(),
                                self.show_cell_focus,
                            );
                        } else if state.selection.is_selected_row(row.expect("row")) {
                            ctx.selected_cell = false;
                            ctx.selected_row = true;
                            ctx.selected_column = false;
                            // use a fallback if no row-selected style is set.
                            ctx.select_style = if self.select_row_style.is_some() {
                                self.patch_select(
                                    self.select_row_style,
                                    state.focus.get(),
                                    self.show_row_focus,
                                )
                            } else {
                                self.patch_select(
                                    Some(self.style),
                                    state.focus.get(),
                                    self.show_row_focus,
                                )
                            };
                        } else if state.selection.is_selected_column(col) {
                            ctx.selected_cell = false;
                            ctx.selected_row = false;
                            ctx.selected_column = true;
                            ctx.select_style = self.patch_select(
                                self.select_column_style,
                                state.focus.get(),
                                self.show_column_focus,
                            );
                        } else {
                            ctx.selected_cell = false;
                            ctx.selected_row = false;
                            ctx.selected_column = false;
                            ctx.select_style = None;
                        }

                        // partially visible?
                        if render_cell_area.right() > state.hscroll.offset as u16
                            || render_cell_area.left() < state.hscroll.offset as u16 + area.width
                        {
                            if let Some(select_style) = ctx.select_style {
                                row_buf.set_style(render_cell_area, select_style);
                                row_buf.set_style(ctx.space_area, select_style);
                            }
                            data.render_cell(&ctx, col, render_cell_area, &mut row_buf);
                        }

                        col += 1;
                    }

                    // render shifted and clipped row.
                    transfer_buffer(
                        &mut row_buf,
                        state.hscroll.offset() as u16,
                        visible_row_area,
                        buf,
                    );
                }

                if visible_row_area.bottom() >= state.table_area.bottom() {
                    break;
                }
                if !data.nth(0) {
                    break;
                }
                row = Some(row.expect("row").saturating_add(1));
                row_y += render_row_area.height;
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
        #[allow(unused_variables)]
        let algorithm;
        #[allow(unused_assignments)]
        {
            if let Some(rows) = data.rows() {
                algorithm = 0;
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
                    let mut sum_height = row_heights.iter().sum::<u16>();
                    row = Some(row.map_or(nth_row, |row| row + nth_row + 1));
                    loop {
                        let row_height = data.row_height();
                        row_heights.push(row_height);

                        // Keep a rolling sum of the heights and drop unnecessary info.
                        // We don't need more info, and there will be a lot more otherwise.
                        sum_height += row_height;
                        if sum_height
                            .saturating_sub(row_heights.first().copied().unwrap_or_default())
                            > state.table_area.height
                        {
                            let lost_height = row_heights.remove(0);
                            sum_height -= lost_height;
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

                // have we got a page worth of data?
                if let Some(last_page) = state.calc_last_page(row_heights) {
                    state.vscroll.set_max_offset(state.rows - last_page);
                } else {
                    // we don't have enough data to establish the last page.
                    // either there are not enough rows or the given row-count
                    // was off. make a guess.
                    state.vscroll.set_max_offset(
                        state.rows.saturating_sub(state.table_area.height as usize),
                    );
                }
            } else if self.no_row_count {
                algorithm = 1;

                // We need to feel out a bit beyond the page, otherwise
                // we can't really stabilize the row count and the
                // display starts flickering.
                if row.is_some() {
                    if data.nth(0) {
                        // try one past page
                        row = Some(row.expect("row").saturating_add(1));
                        if data.nth(0) {
                            // have an unknown number of rows left.
                            row = Some(usize::MAX - 1);
                        }
                    }
                }

                state.rows = row.map_or(0, |v| v + 1);
                state._counted_rows = row.map_or(0, |v| v + 1);
                // rough estimate
                state.vscroll.set_max_offset(usize::MAX - 1);
                if state.vscroll.page_len() == 0 {
                    state.vscroll.set_page_len(state.table_area.height as usize);
                }
            } else {
                algorithm = 2;

                // Read all the rest to establish the exact row-count.
                let mut sum_height = row_heights.iter().sum::<u16>();
                while data.nth(0) {
                    let row_height = data.row_height();
                    row_heights.push(row_height);

                    // Keep a rolling sum of the heights and drop unnecessary info.
                    // We don't need more info, and there will be a lot more otherwise.
                    sum_height += row_height;
                    if sum_height.saturating_sub(row_heights.first().copied().unwrap_or_default())
                        > state.table_area.height
                    {
                        let lost_height = row_heights.remove(0);
                        sum_height -= lost_height;
                    }
                    row = Some(row.map_or(0, |v| v + 1));
                }

                state.rows = row.map_or(0, |v| v + 1);
                state._counted_rows = row.map_or(0, |v| v + 1);

                // have we got a page worth of data?
                if let Some(last_page) = state.calc_last_page(row_heights) {
                    state.vscroll.set_max_offset(state.rows - last_page);
                } else {
                    state.vscroll.set_max_offset(0);
                }
            }
        }
        {
            state
                .hscroll
                .set_max_offset(width.saturating_sub(state.table_area.width) as usize);
        }

        #[cfg(debug_assertions)]
        {
            use std::fmt::Write;
            let mut msg = String::new();
            if insane_offset {
                _= write!(msg,
                          "Table::render:\n        offset {}\n        rows {}\n        iter-rows {}max\n    don't match up\nCode X{}X\n",
                          state.vscroll.offset(), state.rows, state._counted_rows, algorithm
                );
            }
            if state.rows != state._counted_rows {
                _ = write!(
                    msg,
                    "Table::render:\n    rows {} don't match\n    iterated rows {}\nCode X{}X\n",
                    state.rows, state._counted_rows, algorithm
                );
            }
            if !msg.is_empty() {
                use log::warn;
                use ratatui::style::Stylize;
                use ratatui::text::Text;

                warn!("{}", &msg);
                Text::from(msg)
                    .white()
                    .on_red()
                    .render(state.table_area, buf);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_footer(
        &self,
        columns: usize,
        width: u16,
        l_columns: &[Rect],
        l_spacers: &[Rect],
        area: Rect,
        buf: &mut Buffer,
        state: &mut TableState<Selection>,
    ) {
        if let Some(footer) = &self.footer {
            let render_row_area = Rect::new(0, 0, width, footer.height);
            let mut row_buf = Buffer::empty(render_row_area);

            row_buf.set_style(render_row_area, self.style);
            if let Some(footer_style) = footer.style {
                row_buf.set_style(render_row_area, footer_style);
            } else if let Some(footer_style) = self.footer_style {
                row_buf.set_style(render_row_area, footer_style);
            }

            let mut col = 0;
            loop {
                if col >= columns {
                    break;
                }

                let render_cell_area =
                    Rect::new(l_columns[col].x, 0, l_columns[col].width, area.height);
                let render_space_area = Rect::new(
                    l_spacers[col + 1].x,
                    0,
                    l_spacers[col + 1].width,
                    area.height,
                );

                if state.selection.is_selected_column(col) {
                    if let Some(selected_style) = self.patch_select(
                        self.select_footer_style,
                        state.focus.get(),
                        self.show_footer_focus,
                    ) {
                        row_buf.set_style(render_cell_area, selected_style);
                        row_buf.set_style(render_space_area, selected_style);
                    }
                };

                // partially visible?
                if render_cell_area.right() > state.hscroll.offset as u16
                    || render_cell_area.left() < state.hscroll.offset as u16 + area.width
                {
                    if let Some(cell) = footer.cells.get(col) {
                        if let Some(cell_style) = cell.style {
                            row_buf.set_style(render_cell_area, cell_style);
                        }
                        cell.content.clone().render(render_cell_area, &mut row_buf);
                    }
                }

                col += 1;
            }

            // render shifted and clipped row.
            transfer_buffer(&mut row_buf, state.hscroll.offset() as u16, area, buf);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_header(
        &self,
        columns: usize,
        width: u16,
        l_columns: &[Rect],
        l_spacers: &[Rect],
        area: Rect,
        buf: &mut Buffer,
        state: &mut TableState<Selection>,
    ) {
        if let Some(header) = &self.header {
            let render_row_area = Rect::new(0, 0, width, header.height);
            let mut row_buf = Buffer::empty(render_row_area);

            row_buf.set_style(render_row_area, self.style);
            if let Some(header_style) = header.style {
                row_buf.set_style(render_row_area, header_style);
            } else if let Some(header_style) = self.header_style {
                row_buf.set_style(render_row_area, header_style);
            }

            let mut col = 0;
            loop {
                if col >= columns {
                    break;
                }

                let render_cell_area =
                    Rect::new(l_columns[col].x, 0, l_columns[col].width, area.height);
                let render_space_area = Rect::new(
                    l_spacers[col + 1].x,
                    0,
                    l_spacers[col + 1].width,
                    area.height,
                );

                if state.selection.is_selected_column(col) {
                    if let Some(selected_style) = self.patch_select(
                        self.select_header_style,
                        state.focus.get(),
                        self.show_header_focus,
                    ) {
                        row_buf.set_style(render_cell_area, selected_style);
                        row_buf.set_style(render_space_area, selected_style);
                    }
                };

                // partially visible?
                if render_cell_area.right() > state.hscroll.offset as u16
                    || render_cell_area.left() < state.hscroll.offset as u16 + area.width
                {
                    if let Some(cell) = header.cells.get(col) {
                        if let Some(cell_style) = cell.style {
                            row_buf.set_style(render_cell_area, cell_style);
                        }
                        cell.content.clone().render(render_cell_area, &mut row_buf);
                    }
                }

                col += 1;
            }

            // render shifted and clipped row.
            transfer_buffer(&mut row_buf, state.hscroll.offset() as u16, area, buf);
        }
    }

    fn calculate_column_areas(
        &self,
        columns: usize,
        l_columns: &[Rect],
        l_spacers: &[Rect],
        state: &mut TableState<Selection>,
    ) {
        state.column_areas.clear();
        state.column_layout.clear();

        let mut col = 0;
        let shift = state.hscroll.offset() as isize;
        loop {
            if col >= columns {
                break;
            }

            state.column_layout.push(Rect::new(
                l_columns[col].x,
                0,
                l_columns[col].width + l_spacers[col + 1].width,
                0,
            ));

            let cell_x1 = l_columns[col].x as isize;
            let cell_x2 =
                (l_columns[col].x + l_columns[col].width + l_spacers[col + 1].width) as isize;

            let squish_x1 = cell_x1.saturating_sub(shift);
            let squish_x2 = cell_x2.saturating_sub(shift);

            let abs_x1 = max(0, squish_x1) as u16;
            let abs_x2 = max(0, squish_x2) as u16;

            let v_area = Rect::new(
                state.table_area.x + abs_x1,
                state.table_area.y,
                abs_x2 - abs_x1,
                state.table_area.height,
            );
            state
                .column_areas
                .push(v_area.intersection(state.table_area));

            col += 1;
        }
    }

    #[expect(clippy::collapsible_else_if)]
    fn patch_select(&self, style: Option<Style>, focus: bool, show: bool) -> Option<Style> {
        if let Some(style) = style {
            if let Some(focus_style) = self.focus_style {
                if focus && show {
                    Some(style.patch(focus_style))
                } else {
                    Some(fallback_select_style(style))
                }
            } else {
                if focus && show {
                    Some(revert_style(style))
                } else {
                    Some(fallback_select_style(style))
                }
            }
        } else {
            None
        }
    }
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            header: None,
            footer: None,
            select_row: None,
            select_column: None,
            select_cell: None,
            select_header: None,
            select_footer: None,
            show_row_focus: true, // non standard
            show_column_focus: false,
            show_cell_focus: false,
            show_header_focus: false,
            show_footer_focus: false,
            focus_style: None,
            block: None,
            border_style: None,
            scroll: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection: Clone> Clone for TableState<Selection> {
    fn clone(&self) -> Self {
        Self {
            focus: FocusFlag::named(self.focus.name()),
            area: self.area,
            inner: self.inner,
            header_area: self.header_area,
            table_area: self.table_area,
            row_areas: self.row_areas.clone(),
            column_areas: self.column_areas.clone(),
            column_layout: self.column_layout.clone(),
            footer_area: self.footer_area,
            rows: self.rows,
            _counted_rows: self._counted_rows,
            columns: self.columns,
            vscroll: self.vscroll.clone(),
            hscroll: self.hscroll.clone(),
            selection: self.selection.clone(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection: Default> Default for TableState<Selection> {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            area: Default::default(),
            inner: Default::default(),
            header_area: Default::default(),
            table_area: Default::default(),
            row_areas: Default::default(),
            column_areas: Default::default(),
            column_layout: Default::default(),
            footer_area: Default::default(),
            rows: Default::default(),
            _counted_rows: Default::default(),
            columns: Default::default(),
            vscroll: Default::default(),
            hscroll: Default::default(),
            selection: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection> HasFocus for TableState<Selection> {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    #[inline]
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    #[inline]
    fn area(&self) -> Rect {
        self.area
    }
}

impl<Selection> RelocatableState for TableState<Selection> {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
        self.inner = relocate_area(self.inner, shift, clip);
        self.table_area = relocate_area(self.table_area, shift, clip);
        self.footer_area = relocate_area(self.footer_area, shift, clip);
        self.header_area = relocate_area(self.header_area, shift, clip);

        relocate_areas(self.row_areas.as_mut_slice(), shift, clip);
        relocate_areas(self.column_areas.as_mut_slice(), shift, clip);
        relocate_areas(self.column_layout.as_mut_slice(), shift, clip);

        self.hscroll.relocate(shift, clip);
        self.vscroll.relocate(shift, clip);
    }
}

impl<Selection> TableState<Selection> {
    fn calc_last_page(&self, mut row_heights: Vec<u16>) -> Option<usize> {
        let mut sum_heights = 0;
        let mut n_rows = 0;
        while let Some(h) = row_heights.pop() {
            sum_heights += h;
            n_rows += 1;
            if sum_heights >= self.table_area.height {
                break;
            }
        }

        if sum_heights < self.table_area.height {
            None
        } else {
            Some(n_rows)
        }
    }
}

// Baseline
impl<Selection> TableState<Selection>
where
    Selection: Default,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        Self {
            focus: FocusFlag::named(name),
            ..TableState::default()
        }
    }
}

// Baseline
impl<Selection> TableState<Selection> {
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
}

// Table areas
impl<Selection> TableState<Selection> {
    /// Returns the whole row-area and the cell-areas for the
    /// given row, if it is visible.
    ///
    /// Attention: These areas might be 0-length if the column is scrolled
    /// beyond the table-area.
    pub fn row_cells(&self, row: usize) -> Option<(Rect, Vec<Rect>)> {
        if row < self.vscroll.offset() || row >= self.vscroll.offset() + self.vscroll.page_len() {
            return None;
        }

        let mut areas = Vec::new();

        let r = self.row_areas[row - self.vscroll.offset()];
        for c in &self.column_areas {
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
        self.mouse.column_at(&self.column_areas, pos.0)
    }

    /// Row at given position.
    pub fn row_at_clicked(&self, pos: (u16, u16)) -> Option<usize> {
        self.mouse
            .row_at(&self.row_areas, pos.1)
            .map(|v| self.vscroll.offset() + v)
    }

    /// Cell when dragging. Position can be outside the table area.
    /// See [row_at_drag](TableState::row_at_drag), [col_at_drag](TableState::column_at_drag)
    pub fn cell_at_drag(&self, pos: (u16, u16)) -> (usize, usize) {
        let col = self.column_at_drag(pos);
        let row = self.row_at_drag(pos);

        (col, row)
    }

    /// Row when dragging. Position can be outside the table area.
    /// If the position is above the table-area this returns offset - #rows.
    /// If the position is below the table-area this returns offset + page_len + #rows.
    ///
    /// This doesn't account for the row-height of the actual rows outside
    /// the table area, just assumes '1'.
    pub fn row_at_drag(&self, pos: (u16, u16)) -> usize {
        match self
            .mouse
            .row_at_drag(self.table_area, &self.row_areas, pos.1)
        {
            Ok(v) => self.vscroll.offset() + v,
            Err(v) if v <= 0 => self.vscroll.offset().saturating_sub((-v) as usize),
            Err(v) => self.vscroll.offset() + self.row_areas.len() + v as usize,
        }
    }

    /// Column when dragging. Position can be outside the table area.
    /// If the position is left of the table area this returns offset - 1.
    /// If the position is right of the table area this returns offset + page_width + 1.
    pub fn column_at_drag(&self, pos: (u16, u16)) -> usize {
        match self
            .mouse
            .column_at_drag(self.table_area, &self.column_areas, pos.0)
        {
            Ok(v) => v,
            Err(v) if v <= 0 => self.hscroll.offset().saturating_sub((-v) as usize),
            Err(v) => self.hscroll.offset() + self.hscroll.page_len() + v as usize,
        }
    }
}

// Offset related.
impl<Selection: TableSelection> TableState<Selection> {
    /// Sets both offsets to 0.
    pub fn clear_offset(&mut self) {
        self.vscroll.set_offset(0);
        self.hscroll.set_offset(0);
    }

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    pub fn row_max_offset(&self) -> usize {
        self.vscroll.max_offset()
    }

    /// Current vertical offset.
    pub fn row_offset(&self) -> usize {
        self.vscroll.offset()
    }

    /// Change the vertical offset.
    ///
    /// Due to overscroll it's possible that this is an invalid offset for the widget.
    /// The widget must deal with this situation.
    ///
    /// The widget returns true if the offset changed at all.
    pub fn set_row_offset(&mut self, offset: usize) -> bool {
        self.vscroll.set_offset(offset)
    }

    /// Vertical page-size at the current offset.
    pub fn page_len(&self) -> usize {
        self.vscroll.page_len()
    }

    /// Suggested scroll per scroll-event.
    pub fn row_scroll_by(&self) -> usize {
        self.vscroll.scroll_by()
    }

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    pub fn x_max_offset(&self) -> usize {
        self.hscroll.max_offset()
    }

    /// Current horizontal offset.
    pub fn x_offset(&self) -> usize {
        self.hscroll.offset()
    }

    /// Change the horizontal offset.
    ///
    /// Due to overscroll it's possible that this is an invalid offset for the widget.
    /// The widget must deal with this situation.
    ///
    /// The widget returns true if the offset changed at all.
    pub fn set_x_offset(&mut self, offset: usize) -> bool {
        self.hscroll.set_offset(offset)
    }

    /// Horizontal page-size at the current offset.
    pub fn page_width(&self) -> usize {
        self.hscroll.page_len()
    }

    /// Suggested scroll per scroll-event.
    pub fn x_scroll_by(&self) -> usize {
        self.hscroll.scroll_by()
    }

    /// Ensures that the selected item is visible.
    /// Caveat: This doesn't work nicely if you have varying row-heights.
    pub fn scroll_to_selected(&mut self) -> bool {
        if let Some(selected) = self.selection.lead_selection() {
            let c = self.scroll_to_x(selected.0);
            let r = self.scroll_to_row(selected.1);
            r || c
        } else {
            false
        }
    }

    /// Ensures that the given row is visible.
    /// Caveat: This doesn't work nicely if you have varying row-heights.
    pub fn scroll_to_row(&mut self, pos: usize) -> bool {
        if pos >= self.rows {
            false
        } else if pos == self.row_offset().saturating_add(self.page_len()) {
            // the page might not fill the full area.
            let heights = self.row_areas.iter().map(|v| v.height).sum::<u16>();
            if heights < self.table_area.height {
                false
            } else {
                self.set_row_offset(pos.saturating_sub(self.page_len()).saturating_add(1))
            }
        } else if pos >= self.row_offset().saturating_add(self.page_len()) {
            self.set_row_offset(pos.saturating_sub(self.page_len()).saturating_add(1))
        } else if pos < self.row_offset() {
            self.set_row_offset(pos)
        } else {
            false
        }
    }

    /// Ensures that the given column is completely visible.
    pub fn scroll_to_col(&mut self, pos: usize) -> bool {
        if let Some(col) = self.column_layout.get(pos) {
            if (col.left() as usize) < self.x_offset() {
                self.set_x_offset(col.x as usize)
            } else if (col.right() as usize) >= self.x_offset().saturating_add(self.page_width()) {
                self.set_x_offset((col.right() as usize).saturating_sub(self.page_width()))
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Ensures that the given position is visible.
    pub fn scroll_to_x(&mut self, pos: usize) -> bool {
        if pos >= self.x_offset().saturating_add(self.page_width()) {
            self.set_x_offset(pos.saturating_sub(self.page_width()).saturating_add(1))
        } else if pos < self.x_offset() {
            self.set_x_offset(pos)
        } else {
            false
        }
    }

    /// Reduce the row-offset by n.
    pub fn scroll_up(&mut self, n: usize) -> bool {
        self.vscroll.scroll_up(n)
    }

    /// Increase the row-offset by n.
    pub fn scroll_down(&mut self, n: usize) -> bool {
        self.vscroll.scroll_down(n)
    }

    /// Reduce the col-offset by n.
    pub fn scroll_left(&mut self, n: usize) -> bool {
        self.hscroll.scroll_left(n)
    }

    /// Increase the col-offset by n.
    pub fn scroll_right(&mut self, n: usize) -> bool {
        self.hscroll.scroll_right(n)
    }
}

impl TableState<RowSelection> {
    /// Update the state to match adding items.
    /// This corrects the number of rows, offset and selection.
    pub fn items_added(&mut self, pos: usize, n: usize) {
        self.vscroll.items_added(pos, n);
        self.selection.items_added(pos, n);
        self.rows += n;
    }

    /// Update the state to match removing items.
    /// This corrects the number of rows, offset and selection.
    pub fn items_removed(&mut self, pos: usize, n: usize) {
        self.vscroll.items_removed(pos, n);
        self.selection
            .items_removed(pos, n, self.rows.saturating_sub(1));
        self.rows -= n;
    }

    /// When scrolling the table, change the selection instead of the offset.
    #[inline]
    pub fn set_scroll_selection(&mut self, scroll: bool) {
        self.selection.set_scroll_selected(scroll);
    }

    /// Clear the selection.
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Anything selected?
    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.selection.has_selection()
    }

    /// Selected row.
    /// The selected row is not constrained by the row-count.
    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.selection.selected()
    }

    /// Return the selected row and ensure it is in the
    /// range 0..rows.
    #[inline]
    #[allow(clippy::manual_filter)]
    pub fn selected_checked(&self) -> Option<usize> {
        if let Some(selected) = self.selection.selected() {
            if selected < self.rows {
                Some(selected)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Select the row.
    /// The selection is not constrained by the row-count.
    #[inline]
    pub fn select(&mut self, row: Option<usize>) -> bool {
        self.selection.select(row)
    }

    /// Scroll delivers a value between 0 and max_offset as offset.
    /// This remaps the ratio to the selection with a range 0..row_len.
    ///
    pub(crate) fn remap_offset_selection(&self, offset: usize) -> usize {
        if self.vscroll.max_offset() > 0 {
            (self.rows * offset) / self.vscroll.max_offset()
        } else {
            0 // ???
        }
    }

    /// Move the selection to the given row.
    /// Ensures the row is visible afterward.
    #[inline]
    pub fn move_to(&mut self, row: usize) -> bool {
        let r = self.selection.move_to(row, self.rows.saturating_sub(1));
        let s = self.scroll_to_row(self.selection.selected().expect("row"));
        r || s
    }

    /// Move the selection up n rows.
    /// Ensures the row is visible afterward.
    #[inline]
    pub fn move_up(&mut self, n: usize) -> bool {
        let r = self.selection.move_up(n, self.rows.saturating_sub(1));
        let s = self.scroll_to_row(self.selection.selected().expect("row"));
        r || s
    }

    /// Move the selection down n rows.
    /// Ensures the row is visible afterward.
    #[inline]
    pub fn move_down(&mut self, n: usize) -> bool {
        let r = self.selection.move_down(n, self.rows.saturating_sub(1));
        let s = self.scroll_to_row(self.selection.selected().expect("row"));
        r || s
    }
}

impl TableState<RowSetSelection> {
    /// Clear the selection.
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Anything selected?
    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.selection.has_selection()
    }

    /// Selected rows.
    #[inline]
    pub fn selected(&self) -> HashSet<usize> {
        self.selection.selected()
    }

    /// Change the lead-selection. Limits the value to the number of rows.
    /// If extend is false the current selection is cleared and both lead and
    /// anchor are set to the given value.
    /// If extend is true, the anchor is kept where it is and lead is changed.
    /// Everything in the range `anchor..lead` is selected. It doesn't matter
    /// if anchor < lead.
    #[inline]
    pub fn set_lead(&mut self, row: Option<usize>, extend: bool) -> bool {
        self.selection.set_lead(row, extend)
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

    /// Retire the current anchor/lead selection to the set of selected rows.
    /// Resets lead and anchor and starts a new selection round.
    #[inline]
    pub fn retire_selection(&mut self) {
        self.selection.retire_selection();
    }

    /// Add to selection. Only works for retired selections, not for the
    /// active anchor-lead range.
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

    /// Move the selection to the given row.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_to(&mut self, row: usize, extend: bool) -> bool {
        let r = self
            .selection
            .move_to(row, self.rows.saturating_sub(1), extend);
        let s = self.scroll_to_row(self.selection.lead().expect("row"));
        r || s
    }

    /// Move the selection up n rows.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_up(&mut self, n: usize, extend: bool) -> bool {
        let r = self
            .selection
            .move_up(n, self.rows.saturating_sub(1), extend);
        let s = self.scroll_to_row(self.selection.lead().expect("row"));
        r || s
    }

    /// Move the selection down n rows.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_down(&mut self, n: usize, extend: bool) -> bool {
        let r = self
            .selection
            .move_down(n, self.rows.saturating_sub(1), extend);
        let s = self.scroll_to_row(self.selection.lead().expect("row"));
        r || s
    }
}

impl TableState<CellSelection> {
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.selection.has_selection()
    }

    /// Selected cell.
    #[inline]
    pub fn selected(&self) -> Option<(usize, usize)> {
        self.selection.selected()
    }

    /// Select a cell.
    #[inline]
    pub fn select_cell(&mut self, select: Option<(usize, usize)>) -> bool {
        self.selection.select_cell(select)
    }

    /// Select a row. Column stays the same.
    #[inline]
    pub fn select_row(&mut self, row: Option<usize>) -> bool {
        if let Some(row) = row {
            self.selection
                .select_row(Some(min(row, self.rows.saturating_sub(1))))
        } else {
            self.selection.select_row(None)
        }
    }

    /// Select a column, row stays the same.
    #[inline]
    pub fn select_column(&mut self, column: Option<usize>) -> bool {
        if let Some(column) = column {
            self.selection
                .select_column(Some(min(column, self.columns.saturating_sub(1))))
        } else {
            self.selection.select_column(None)
        }
    }

    /// Select a cell, limit to maximum.
    #[inline]
    pub fn move_to(&mut self, select: (usize, usize)) -> bool {
        let r = self.selection.move_to(
            select,
            (self.columns.saturating_sub(1), self.rows.saturating_sub(1)),
        );
        let s = self.scroll_to_selected();
        r || s
    }

    /// Select a row, limit to maximum.
    #[inline]
    pub fn move_to_row(&mut self, row: usize) -> bool {
        let r = self.selection.move_to_row(row, self.rows.saturating_sub(1));
        let s = self.scroll_to_selected();
        r || s
    }

    /// Select a cell, clamp between 0 and maximum.
    #[inline]
    pub fn move_to_col(&mut self, col: usize) -> bool {
        let r = self
            .selection
            .move_to_col(col, self.columns.saturating_sub(1));
        let s = self.scroll_to_selected();
        r || s
    }

    /// Move the selection up n rows.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_up(&mut self, n: usize) -> bool {
        let r = self.selection.move_up(n, self.rows.saturating_sub(1));
        let s = self.scroll_to_selected();
        r || s
    }

    /// Move the selection down n rows.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_down(&mut self, n: usize) -> bool {
        let r = self.selection.move_down(n, self.rows.saturating_sub(1));
        let s = self.scroll_to_selected();
        r || s
    }

    /// Move the selection left n columns.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_left(&mut self, n: usize) -> bool {
        let r = self.selection.move_left(n, self.columns.saturating_sub(1));
        let s = self.scroll_to_selected();
        r || s
    }

    /// Move the selection right n columns.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_right(&mut self, n: usize) -> bool {
        let r = self.selection.move_right(n, self.columns.saturating_sub(1));
        let s = self.scroll_to_selected();
        r || s
    }
}

impl<Selection> HandleEvent<crossterm::event::Event, DoubleClick, DoubleClickOutcome>
    for TableState<Selection>
{
    /// Handles double-click events on the table.
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        _keymap: DoubleClick,
    ) -> DoubleClickOutcome {
        match event {
            ct_event!(mouse any for m) if self.mouse.doubleclick(self.table_area, m) => {
                if let Some((col, row)) = self.cell_at_clicked((m.column, m.row)) {
                    DoubleClickOutcome::ClickClick(col, row)
                } else {
                    DoubleClickOutcome::Continue
                }
            }
            _ => DoubleClickOutcome::Continue,
        }
    }
}

/// Handle all events for recognizing double-clicks.
pub fn handle_doubleclick_events<Selection: TableSelection>(
    state: &mut TableState<Selection>,
    event: &crossterm::event::Event,
) -> DoubleClickOutcome {
    state.handle(event, DoubleClick)
}
