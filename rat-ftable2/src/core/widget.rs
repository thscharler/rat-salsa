use crate::_private::NonExhaustive;
use crate::core::TableCoreState;
use crate::selection::TableSelection;
use crate::selection::row::RowSelection;
use crate::style::TableStyle;
use crate::util::{fallback_select_style, revert_style, transfer_buffer};
use crate::{LayoutWidth, RenderContext};
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Flex, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::text::Text;
use ratatui_core::widgets::Widget;
use ratatui_widgets::block::Block;
use std::cmp::max;
use std::marker::PhantomData;
use std::rc::Rc;

/// The table-core.
///
/// Holds all the render-parameters and does the actual
/// rendering of the table.
///
/// It is used as the backend for the user-facing widgets.
pub struct TableCore<'a, T: Copy, Selection = RowSelection> {
    header: Vec<Text<'a>>,
    footer: Vec<Text<'a>>,
    row_height_fn: Option<Box<dyn FnMut(T, &RenderContext) -> u16 + 'a>>,
    render_row_fn: Option<Box<dyn FnMut(T, Rect, &mut Buffer, &RenderContext) -> bool + 'a>>,
    render_cell_fn: Vec<Option<Box<dyn FnMut(T, Rect, &mut Buffer, &RenderContext) + 'a>>>,

    widths: Vec<Constraint>,
    varying_heights: bool,
    flex: Flex,
    column_spacing: u16,
    layout_width: LayoutWidth,

    style: Style,
    block: Option<Block<'a>>,
    hscroll: Option<Scroll<'a>>,
    vscroll: Option<Scroll<'a>>,
    header_style: Option<Style>,
    footer_style: Option<Style>,
    focus_style: Option<Style>,

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

    _phantom: PhantomData<Selection>,
}

impl<'a, T: Copy, Selection> Default for TableCore<'a, T, Selection> {
    fn default() -> Self {
        Self {
            header: Default::default(),
            footer: Default::default(),
            row_height_fn: Default::default(),
            render_row_fn: Default::default(),
            render_cell_fn: Default::default(),
            widths: Default::default(),
            varying_heights: Default::default(),
            flex: Default::default(),
            column_spacing: Default::default(),
            layout_width: Default::default(),
            style: Default::default(),
            block: Default::default(),
            hscroll: Default::default(),
            vscroll: Default::default(),
            header_style: Default::default(),
            footer_style: Default::default(),
            focus_style: Default::default(),
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
            _phantom: Default::default(),
        }
    }
}

impl<'a, T: Copy, Selection> TableCore<'a, T, Selection> {
    /// New, empty Table.
    pub fn new() -> Self
    where
        Selection: Default,
    {
        Self::default()
    }

    /// Set the closure that calculates the row-height for a value.
    #[inline]
    pub fn row_height(mut self, row_height_fn: impl FnMut(T, &RenderContext) -> u16 + 'a) -> Self {
        self.row_height_fn = Some(Box::new(row_height_fn));
        self
    }

    /// Set a closure for row-level rendering.
    #[inline]
    pub fn render_row(
        mut self,
        render_row_fn: impl FnMut(T, Rect, &mut Buffer, &RenderContext) -> bool + 'a,
    ) -> Self {
        self.render_row_fn = Some(Box::new(render_row_fn));
        self
    }

    /// Set a closure for rendering one column.
    #[inline]
    pub fn render_cell(
        mut self,
        column: usize,
        render_cell_fn: impl FnMut(T, Rect, &mut Buffer, &RenderContext) + 'a,
    ) -> Self {
        while self.render_cell_fn.len() <= column {
            self.render_cell_fn.push(None);
        }
        self.render_cell_fn[column] = Some(Box::new(render_cell_fn));
        self
    }

    /// Set the table-header.
    #[inline]
    pub fn header<I>(mut self, header: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Text<'a>>,
    {
        self.header = header.into_iter().map(|v| v.into()).collect();
        self
    }

    /// Set the table-footer.
    #[inline]
    pub fn footer<I>(mut self, footer: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Text<'a>>,
    {
        self.footer = footer.into_iter().map(|v| v.into()).collect();
        self
    }

    /// Column widths as Constraints.
    #[inline]
    pub fn widths<I>(mut self, widths: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        self.widths = widths.into_iter().map(|v| v.into()).collect();
        self
    }

    /// If your table uses varying row-heights for each row
    /// all page calculations will fail (somewhat).
    /// If you enable this flag, the row-heights for
    /// each row will be stored in the state to allow
    /// accurate page-calculations.
    pub fn varying_heights(mut self, varying: bool) -> Self {
        self.varying_heights = varying;
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

    /// Set the display width of the table.
    /// If this is not set, the width of the rendered area is used.
    #[inline]
    pub fn layout_width(mut self, width: LayoutWidth) -> Self {
        self.layout_width = width;
        self
    }

    /// Draws a block around the table widget.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Sets the border-style for the Block, if any.
    pub fn border_style(mut self, style: Style) -> Self {
        self.block = self.block.map(|v| v.border_style(style));
        self
    }

    /// Sets the title-style for the Block, if any.
    pub fn title_style(mut self, style: Style) -> Self {
        self.block = self.block.map(|v| v.title_style(style));
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
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if let Some(border_style) = styles.border_style {
            self.block = self.block.map(|v| v.border_style(border_style));
        }
        if let Some(title_style) = styles.title_style {
            self.block = self.block.map(|v| v.title_style(title_style));
        }
        self.block = self.block.map(|v| v.style(self.style));

        if let Some(styles) = styles.scroll {
            self.hscroll = self.hscroll.map(|v| v.styles(styles.clone()));
            self.vscroll = self.vscroll.map(|v| v.styles(styles));
        }
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

    /// Inherent width.
    pub fn width(&self) -> u16 {
        let sa = ScrollArea::new()
            .style(self.style)
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        let padding = sa.padding();
        let width = total_width(self, 0);

        width + padding.left + padding.right
    }

    /// Inherent height.
    pub fn height(&self) -> u16 {
        let sa = ScrollArea::new()
            .style(self.style)
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        let padding = sa.padding();
        let header = self
            .header
            .iter()
            .map(|v| v.height() as u16)
            .max()
            .unwrap_or(0);
        let footer = self
            .footer
            .iter()
            .map(|v| v.height() as u16)
            .max()
            .unwrap_or(0);

        1 + header + footer + padding.top + padding.bottom
    }
}

fn total_width<T: Copy, Selection>(widget: &TableCore<'_, T, Selection>, area_width: u16) -> u16 {
    match widget.layout_width {
        LayoutWidth::Length(len) => len,
        LayoutWidth::Columns => {
            let mut width = 0;
            for w in widget.widths.iter().copied() {
                match w {
                    Constraint::Min(v) => width += v + widget.column_spacing,
                    Constraint::Max(v) => width += v + widget.column_spacing,
                    Constraint::Length(v) => width += v + widget.column_spacing,
                    Constraint::Percentage(p) => {
                        width += (((area_width as u32) * (p as u32)) / 100) as u16;
                    }
                    Constraint::Ratio(n, d) => {
                        width += (((area_width as u32) * n) / d) as u16;
                    }
                    Constraint::Fill(_) => {
                        /* unsupported, use dummy */
                        width += 10;
                    }
                }
            }
            max(width, area_width)
        }
        LayoutWidth::Area => area_width,
    }
}

// Do the column-layout. Fill in missing columns, if necessary.
fn layout_columns<T: Copy, Selection>(
    widget: &TableCore<'_, T, Selection>,
    area_width: u16,
) -> (u16, Rc<[Rect]>, Rc<[Rect]>) {
    let width = total_width(widget, area_width);
    let area = Rect::new(0, 0, width, 0);

    let (layout, spacers) = Layout::horizontal(&widget.widths)
        .flex(widget.flex)
        .spacing(widget.column_spacing)
        .split_with_spacers(area);

    (width, layout, spacers)
}

// Layout header/table/footer
#[inline]
fn layout_areas<T: Copy, Selection>(
    widget: &TableCore<'_, T, Selection>,
    area: Rect,
) -> Rc<[Rect]> {
    let heights = vec![
        Constraint::Length(
            widget
                .header
                .iter()
                .map(|v| v.height() as u16)
                .max()
                .unwrap_or(0),
        ),
        Constraint::Fill(1),
        Constraint::Length(
            widget
                .footer
                .iter()
                .map(|v| v.height() as u16)
                .max()
                .unwrap_or(0),
        ),
    ];

    Layout::vertical(heights).split(area)
}

impl<'a, T: Copy, Selection> TableCore<'a, T, Selection>
where
    Selection: TableSelection,
{
    pub fn render(
        self,
        data: Option<&mut dyn Iterator<Item = T>>,
        area: Rect,
        buf: &mut Buffer,
        state: &mut TableCoreState<Selection>,
    ) {
        if let Some(data) = data {
            render_table(self, Some(data), area, buf, state);
        } else {
            render_table(self, None, area, buf, state);
        }
    }
}

fn render_table<'a, T: Copy, Selection: TableSelection>(
    mut widget: TableCore<'a, T, Selection>,
    mut data: Option<&mut dyn Iterator<Item = T>>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut TableCoreState<Selection>,
) {
    state.rows = data.as_mut().map(|v| v.size_hint().0).unwrap_or(0);
    state.columns = widget.widths.len();
    state.area = area;

    let sa = ScrollArea::new()
        .style(widget.style)
        .block(widget.block.as_ref())
        .h_scroll(widget.hscroll.as_ref())
        .v_scroll(widget.vscroll.as_ref());
    state.inner = sa.inner(area, Some(&state.hscroll), Some(&state.vscroll));

    let l_rows = layout_areas(&widget, state.inner);
    state.header_area = l_rows[0];
    state.table_area = l_rows[1];
    state.footer_area = l_rows[2];

    // horizontal layout
    let (width, l_columns, l_spacers) = layout_columns(&widget, state.table_area.width);
    calculate_column_areas(state.columns, l_columns.as_ref(), l_spacers.as_ref(), state);

    // render block only
    sa.render_block(area, buf);

    // render header & footer
    render_header(
        &widget,
        state.columns,
        width,
        l_columns.as_ref(),
        l_spacers.as_ref(),
        state.header_area,
        buf,
        state,
    );
    render_footer(
        &widget,
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
    state.row_heights.clear();
    state.row_height = 0;
    state.vscroll.set_page_len(0);
    state.hscroll.set_page_len(area.width as usize);

    let mut ctx = RenderContext {
        row: 0,
        focus: state.focus.get(),
        selected_cell: false,
        selected_row: false,
        selected_column: false,
        style: widget.style,
        select_style: None,
        column_layout: &state.column_layout,
        space_area: Default::default(),
        row_area: Default::default(),
        non_exhaustive: NonExhaustive,
    };

    'll: {
        let Some(data) = data else {
            break 'll;
        };

        // iterate to offset row.
        let mut row = None;
        let mut value;
        let mut row_y = state.table_area.y;
        let mut row_heights = Vec::new();
        let mut row_buf = Buffer::empty(Rect::new(0, 0, width, 1));

        // skip to offset.
        if widget.varying_heights {
            loop {
                value = if let Some(v) = data.nth(0) {
                    v
                } else {
                    break 'll;
                };

                if let Some(rr) = row {
                    if rr + 1 < state.vscroll.offset() {
                        ctx.row = rr;
                        let row_height = widget
                            .row_height_fn
                            .as_mut()
                            .map_or(1, |f| (*f)(value, &ctx));
                        state.row_heights.push(row_height)
                    }
                    row = Some(rr + 1);
                } else {
                    if 0 < state.vscroll.offset() {
                        ctx.row = 0;
                        let row_height = widget
                            .row_height_fn
                            .as_mut()
                            .map_or(1, |f| (*f)(value, &ctx));
                        state.row_heights.push(row_height)
                    }
                    row = Some(0);
                }

                if row.expect("row") >= state.vscroll.offset() {
                    break;
                }
            }
        } else {
            value = if let Some(v) = data.nth(state.vscroll.offset()) {
                v
            } else {
                break 'll;
            };
            row = Some(state.vscroll.offset());
        }

        // render-loop
        loop {
            ctx.row = row.expect("row");
            ctx.selected_cell = false;
            ctx.selected_row = false;
            ctx.selected_column = false;
            ctx.select_style = None;
            ctx.focus = false;
            ctx.row_area = Rect::default();
            ctx.space_area = Rect::default();

            // Render each row to a temporary buffer.
            let row_height = widget
                .row_height_fn
                .as_mut()
                .map_or(1, |f| (*f)(value, &ctx));
            let render_row_area = Rect::new(0, 0, width, row_height);
            ctx.row_area = render_row_area;
            row_buf.resize(render_row_area);
            row_heights.push(row_height);
            if widget.varying_heights {
                state.row_heights.push(row_height);
            } else {
                state.row_height = row_height;
            }
            // Target area for the finished row.
            let visible_row_area = Rect::new(
                state.table_area.x,
                row_y,
                state.table_area.width,
                render_row_area.height,
            )
            .intersection(state.table_area);
            state.row_areas.push(visible_row_area);
            state.vscroll.page_len += 1;

            if state.selection.is_selected_row(row.expect("row")) {
                ctx.selected_row = true;
                ctx.select_style = patch_select(
                    widget.select_row_style,
                    widget.focus_style,
                    state.focus.get(),
                    widget.show_row_focus,
                );
            }

            let render_cells = if render_row_area.height > 0 {
                if let Some(row_style) = ctx.select_style {
                    row_buf.set_style(render_row_area, row_style);
                } else {
                    row_buf.set_style(render_row_area, widget.style);
                }

                if let Some(render_row) = widget.render_row_fn.as_mut() {
                    render_row(value, render_row_area, &mut row_buf, &ctx)
                } else {
                    true
                }
            } else {
                true
            };

            // can skip this entirely
            if render_cells && render_row_area.height > 0 {
                let mut col = 0;
                loop {
                    if col >= state.columns {
                        break;
                    }

                    ctx.selected_cell = false;
                    ctx.selected_column = false;
                    ctx.select_style = None;

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

                    if state.selection.is_selected_column(col) {
                        ctx.selected_column = true;
                        ctx.select_style = patch_select(
                            widget.select_column_style,
                            widget.focus_style,
                            state.focus.get(),
                            widget.show_column_focus,
                        );
                    }

                    if state.selection.is_selected_cell(col, row.expect("row")) {
                        ctx.selected_cell = true;
                        ctx.select_style = patch_select(
                            widget.select_cell_style,
                            widget.focus_style,
                            state.focus.get(),
                            widget.show_cell_focus,
                        );
                    }

                    // at least partially visible?
                    if render_cell_area.right() > state.hscroll.offset as u16
                        || render_cell_area.left() < state.hscroll.offset as u16 + area.width
                    {
                        if let Some(select_style) = ctx.select_style {
                            row_buf.set_style(render_cell_area, select_style);
                            row_buf.set_style(ctx.space_area, select_style);
                        }

                        if let Some(Some(render_cell)) = widget.render_cell_fn.get_mut(col) {
                            render_cell(value, render_cell_area, &mut row_buf, &ctx);
                        }
                    }

                    col += 1;
                }

                // render shifted and clipped row.
                // this moves the cells from row_buf to the screen buf.
                transfer_buffer(
                    &mut row_buf,
                    state.hscroll.offset() as u16,
                    visible_row_area,
                    buf,
                );
            }

            value = if let Some(value) = data.nth(0) {
                value
            } else {
                break 'll;
            };

            row = Some(row.expect("row").saturating_add(1));
            row_y += render_row_area.height;

            if visible_row_area.bottom() >= state.table_area.bottom() {
                break;
            }
        }

        if !widget.varying_heights {
            // skip to a guess for the last page.
            // the guess uses row-height is 1, which may read a few more lines than
            // absolutely necessary.
            let skip_rows = state
                .rows
                .saturating_sub(row.map_or(0, |v| v + 1))
                .saturating_sub(state.table_area.height as usize);
            // if we can still skip some rows, then the data so far is useless.
            if skip_rows > 0 {
                row_heights.clear();
            }

            value = if let Some(value) = data.nth(skip_rows) {
                row = Some(row.map_or(skip_rows, |row| row + skip_rows + 1));
                value
            } else {
                break 'll;
            }
        }

        let mut sum_height = row_heights.iter().sum::<u16>();
        loop {
            let row_height = widget
                .row_height_fn
                .as_mut()
                .map_or(1, |f| (*f)(value, &ctx));
            if widget.varying_heights {
                state.row_heights.push(row_height);
            }
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

            value = if let Some(value) = data.nth(0) {
                value
            } else {
                break;
            };

            row = Some(row.expect("row") + 1);
        }

        state.rows = row.map_or(0, |v| v + 1);
        // have we got a page worth of data?
        if let Some(last_page) = calc_last_page(state.table_area.height, row_heights) {
            state.vscroll.set_max_offset(state.rows - last_page);
        } else {
            // we don't have enough data to establish the last page.
            // either there are not enough rows or the given row-count
            // was off. make a guess.
            state
                .vscroll
                .set_max_offset(state.rows.saturating_sub(state.table_area.height as usize));
        }
        state
            .hscroll
            .set_max_offset(width.saturating_sub(state.table_area.width) as usize);
        state.selection.validate_rows(state.rows);
        state.selection.validate_cols(state.columns);
    }

    // render only the scrollbars.
    ScrollArea::new()
        .style(widget.style)
        .block(widget.block.as_ref())
        .h_scroll(widget.hscroll.as_ref())
        .v_scroll(widget.vscroll.as_ref())
        .render_scrollbars(
            area,
            buf,
            &mut ScrollAreaState::new()
                .h_scroll(&mut state.hscroll)
                .v_scroll(&mut state.vscroll),
        );
}

#[allow(clippy::too_many_arguments)]
fn render_footer<T: Copy, Selection: TableSelection>(
    widget: &TableCore<'_, T, Selection>,
    columns: usize,
    width: u16,
    l_columns: &[Rect],
    l_spacers: &[Rect],
    area: Rect,
    buf: &mut Buffer,
    state: &mut TableCoreState<Selection>,
) {
    let height = widget
        .footer
        .iter()
        .map(|v| v.height() as u16)
        .max()
        .unwrap_or(0);

    if height > 0 {
        let render_row_area = Rect::new(0, 0, width, height);
        let mut row_buf = Buffer::empty(render_row_area);

        row_buf.set_style(render_row_area, widget.style);
        if let Some(footer_style) = widget.footer_style {
            row_buf.set_style(render_row_area, footer_style);
        }

        for (col, text) in widget.footer.iter().enumerate() {
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
                if let Some(selected_style) = patch_select(
                    widget.select_footer_style,
                    widget.focus_style,
                    state.focus.get(),
                    widget.show_footer_focus,
                ) {
                    row_buf.set_style(render_cell_area, selected_style);
                    row_buf.set_style(render_space_area, selected_style);
                }
            };

            // partially visible?
            if render_cell_area.right() > state.hscroll.offset as u16
                || render_cell_area.left() < state.hscroll.offset as u16 + area.width
            {
                text.render(render_cell_area, &mut row_buf);
            }
        }

        // render shifted and clipped row.
        transfer_buffer(&mut row_buf, state.hscroll.offset() as u16, area, buf);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_header<T: Copy, Selection: TableSelection>(
    widget: &TableCore<'_, T, Selection>,
    columns: usize,
    width: u16,
    l_columns: &[Rect],
    l_spacers: &[Rect],
    area: Rect,
    buf: &mut Buffer,
    state: &mut TableCoreState<Selection>,
) {
    let height = widget
        .header
        .iter()
        .map(|v| v.height() as u16)
        .max()
        .unwrap_or(0);

    if height > 0 {
        let render_row_area = Rect::new(0, 0, width, height);
        let mut row_buf = Buffer::empty(render_row_area);

        row_buf.set_style(render_row_area, widget.style);
        if let Some(header_style) = widget.header_style {
            row_buf.set_style(render_row_area, header_style);
        }

        for (col, text) in widget.header.iter().enumerate() {
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
                if let Some(selected_style) = patch_select(
                    widget.select_header_style,
                    widget.focus_style,
                    state.focus.get(),
                    widget.show_header_focus,
                ) {
                    row_buf.set_style(render_cell_area, selected_style);
                    row_buf.set_style(render_space_area, selected_style);
                }
            };

            // partially visible?
            if render_cell_area.right() > state.hscroll.offset as u16
                || render_cell_area.left() < state.hscroll.offset as u16 + area.width
            {
                text.render(render_cell_area, &mut row_buf);
            }
        }

        // render shifted and clipped row.
        transfer_buffer(&mut row_buf, state.hscroll.offset() as u16, area, buf);
    }
}

fn calculate_column_areas<Selection>(
    columns: usize,
    l_columns: &[Rect],
    l_spacers: &[Rect],
    state: &mut TableCoreState<Selection>,
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
        let cell_x2 = (l_columns[col].x + l_columns[col].width + l_spacers[col + 1].width) as isize;

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

fn patch_select(
    style: Option<Style>,
    focus_style: Option<Style>,
    focus: bool,
    show_focus: bool,
) -> Option<Style> {
    if let Some(style) = style {
        if let Some(focus_style) = focus_style {
            if focus && show_focus {
                Some(style.patch(focus_style))
            } else {
                Some(fallback_select_style(style))
            }
        } else {
            if focus && show_focus {
                Some(revert_style(style))
            } else {
                Some(fallback_select_style(style))
            }
        }
    } else {
        None
    }
}

fn calc_last_page(height: u16, mut row_heights: Vec<u16>) -> Option<usize> {
    let mut sum_heights = 0;
    let mut n_rows = 0;
    while let Some(h) = row_heights.pop() {
        sum_heights += h;
        n_rows += 1;
        if sum_heights >= height {
            break;
        }
    }

    if sum_heights < height {
        None
    } else {
        Some(n_rows)
    }
}
