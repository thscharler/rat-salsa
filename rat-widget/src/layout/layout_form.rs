use crate::layout::generic_layout::GenericLayout;
use crate::util::{block_padding, block_padding2};
use ratatui::layout::{Flex, Rect, Size};
use ratatui::widgets::{Block, Borders, Padding};
use std::borrow::Cow;
use std::cmp::{max, min};
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::mem;
use std::ops::Range;
use std::rc::Rc;

/// Label constraints for [LayoutForm].
///
/// Any given widths and heights will be reduced if there is not enough space.
#[derive(Debug, Default)]
pub enum FormLabel {
    /// No label, just the widget.
    #[default]
    None,
    /// Label by example.
    /// Line breaks in the text don't work. TODO.
    ///
    /// Will create a label area with the max width of all labels and a height of 1.
    /// The area will be top aligned with the widget.
    Str(&'static str),
    /// Label by example.
    /// Line breaks in the text don't work. TODO.
    ///
    /// Will create a label area with the max width of all labels and a height of 1.
    /// The area will be top aligned with the widget.
    String(String),
    /// Label by width.
    ///
    /// Will create a label area with the max width of all labels and a height of 1.
    /// The area will be top aligned with the widget.
    ///
    ///  __unit: cols__
    Width(u16),
    /// Label by height+width.
    ///
    /// Will create a label area with the max width of all labels and a height of rows.
    /// The area will be top aligned with the widget.
    ///
    ///  __unit: cols,rows__
    Size(u16, u16),
}

/// Widget constraints for [LayoutForm].
///
/// Any given widths and heights will be reduced if there is not enough space.
#[derive(Debug, Default)]
pub enum FormWidget {
    /// No widget, just a label.
    #[default]
    None,
    /// Widget aligned with the label.
    ///
    /// Will create an area with the given width and height 1.
    ///
    /// __unit: cols__
    Width(u16),
    /// Widget aligned with the label.
    ///
    /// Will create an area with the given width and height.
    ///
    /// __unit: cols,rows__
    Size(u16, u16),
    /// Widget aligned with the label.
    ///
    /// The widget will start with the given number of rows.
    /// If there is remaining vertical space left after a page-break this
    /// widget will get it. If there are more than one such widget
    /// the remainder will be evenly distributed.
    ///
    /// __unit: cols,rows__
    StretchY(u16, u16),
    /// Widget stacked below the label.
    ///
    /// Any label that is not FormLabel::None will be placed above
    /// the widget.
    ///
    /// __unit: cols,rows__
    Wide(u16, u16),
    /// Widget filling the maximum width.
    ///
    /// __unit: cols,rows__
    StretchX(u16, u16),
    /// Widget stacked below the label, with the maximum width.
    ///
    /// __unit: cols,rows__
    WideStretchX(u16, u16),
    /// Stretch the widget to the maximum width and height.
    ///
    /// The widget will start with the given number of rows.
    /// If there is remaining vertical space left after a page-break this
    /// widget will get it. If there are more than one such widget
    /// the remainder will be evenly distributed.
    ///
    /// __unit: cols,rows__
    StretchXY(u16, u16),

    /// Widget stacked below the label, with the maximum width and height.
    ///
    /// The widget will start with the given number of rows.
    /// If there is remaining vertical space left after a page-break this
    /// widget will get it. If there are more than one such widget
    /// the remainder will be evenly distributed.
    ///
    /// __unit: rows__
    WideStretchXY(u16, u16),
}

/// Create layouts for input widgets.
///
/// This takes the constraints for [FormLabel] and [FormWidget]
/// and creates a multi-column layout for them.
///
/// There are two branches of the layout:
///
/// - paged: Fill a target area with the widgets and start
///   a new page when the layout overflows. You can use [Form](crate::form::Form)
///   to render a multipage and navigate between the pages.
///
/// - endless: Just stack the widgets. You can use [Clipper](crate::clipper::Clipper)
///   to render such a layout and scroll through it.
///
/// ## Both variants
///
/// Can
/// - add Blocks for multiple widgets. Can stack them too.
/// - layout labels above or to the left of the widget.
/// - add manual column/page breaks.
/// - align widgets vertically.
/// - define line/column spacing.
/// - flex layout columns.
///
/// Can __not__
/// - forcibly shrink-fit the widgets to a layout size.
///
/// ## Paged layout
///
/// Can
/// - create n columns of widgets.
/// - stretch some widgets vertically to fill a column.
///
/// ## Endless layout
///
/// Can
/// - create n columns of widgets.
///     - todo: optimally fill all columns
///     - add manual breaks for now.
/// - __not__ usefully stretch the widgets vertically.
///
/// ```rust no_run
/// # use ratatui::buffer::Buffer;
/// # use ratatui::layout::{Flex, Rect};
/// # use ratatui::text::Span;
/// # use ratatui::widgets::{Padding, Widget, StatefulWidget};
/// # use rat_focus::{FocusFlag, HasFocus};
/// # use rat_text::text_input::{TextInput, TextInputState};
/// # use rat_widget::layout::{GenericLayout, LayoutForm};
/// #
/// # struct State {
/// #     layout: GenericLayout<usize>,
/// #     text1: TextInputState,
/// #     text2: TextInputState,
/// #     text3: TextInputState,
/// # }
/// #
/// # let mut state = State {layout: Default::default(),text1: Default::default(),text2: Default::default(),text3: Default::default()};
/// # let area = Rect::default();
/// # let mut buf = Buffer::empty(Rect::default());
/// # let buf = &mut buf;
///
/// if state.layout.area_changed(area) {
///     let mut ll = LayoutForm::new()
///             .spacing(1)
///             .line_spacing(1)
///             .column_spacing(1)
///             .padding(Padding::new(2,2,0,0))
///             .flex(Flex::Start)
///             .min_label(10);
///
///     use rat_widget::layout::{FormLabel as L, FormWidget as W};
///
///     // single row
///     ll.widget(state.text1.id(), L::Str("Text 1"), W::Width(22));
///     // stretch to form width. preferred width 15. 1 row high.
///     ll.widget(state.text2.id(), L::Str("Text 2"), W::StretchX(15, 1));
///     // stretch to the form-width and fill vertically.
///     // preferred width is 15 3 rows high.
///     ll.widget(state.text3.id(), L::Str("Text 3"), W::StretchXY(15, 3));
///
///     // calculate the layout and place it.
///     state.layout = ll.build_paged(area.as_size());
///  }
///
///  let ll = &state.layout;
///
///  // This is not really the intended use, but it works.
///  // In reality, you would use Clipper or Form.
///
///  let lbl1 = ll.label_for(state.text1.id());
///  Span::from(ll.label_str_for(state.text1.id())).render(lbl1, buf);
///
///  let txt1 = ll.widget_for(state.text1.id());
///  TextInput::new()
///     .render(txt1, buf, &mut state.text1);
///
///  let lbl2 = ll.label_for(state.text2.id());
///  Span::from(ll.label_str_for(state.text2.id())).render(lbl2, buf);
///
///  let txt2 = ll.widget_for(state.text2.id());
///  TextInput::new()
///     .render(txt2, buf, &mut state.text2);
///
///  let lbl3 = ll.label_for(state.text3.id());
///  Span::from(ll.label_str_for(state.text3.id())).render(lbl3, buf);
///
///  let txt3 = ll.widget_for(state.text3.id());
///  TextInput::new()
///     .render(txt3, buf, &mut state.text3);
///
/// ```
///
#[derive(Debug)]
pub struct LayoutForm<W>
where
    W: Eq + Hash + Clone + Debug,
{
    /// Column spacing.
    spacing: u16,
    /// Line spacing.
    line_spacing: u16,
    /// Column spacing
    column_spacing: u16,
    /// Page border.
    page_border: Padding,
    /// Mirror the borders between even/odd pages.
    mirror_border: bool,
    /// Layout as columns.
    columns: u16,
    /// Flex
    flex: Flex,
    /// Areas
    widgets: Vec<WidgetDef<W>>,
    /// Containers/Blocks
    blocks: VecDeque<BlockDef>,
    /// Page breaks.
    page_breaks: Vec<usize>,

    /// maximum width
    min_label: u16,
    min_widget: u16,

    /// maximum padding due to containers.
    max_left_padding: u16,
    max_right_padding: u16,

    /// current left indent.
    left_padding: u16,
    /// current right indent.
    right_padding: u16,
}

struct WidgetDef<W>
where
    W: Debug + Clone,
{
    // widget id
    id: W,
    // label constraint
    label: FormLabel,
    // label text
    label_str: Option<Cow<'static, str>>,
    // widget constraint
    widget: FormWidget,
}

/// Tag for a group/block.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct BlockTag(usize);

#[derive(Debug)]
struct BlockDef {
    id: BlockTag,
    // block
    block: Option<Block<'static>>,
    // padding due to block
    padding: Padding,
    // under construction
    constructing: bool,
    // range into the widget vec
    range: Range<usize>,
    // calculated container area.
    area: Rect,
}

#[derive(Debug)]
struct BlockOut {
    // block
    block: Option<Block<'static>>,
    // area
    area: Rect,
}

// effective positions for layout construction.
#[derive(Debug, Default, Clone, Copy)]
struct XPositions {
    // label position
    label_left: u16,
    // label width, max.
    label_width: u16,
    // widget position
    widget_left: u16,
    // widget width, max.
    widget_width: u16,
    // max widget position
    widget_right: u16,
    // left position for container blocks.
    container_left: u16,
    // right position for container blocks.
    container_right: u16,
    // total width label+spacing+widget
    total_width: u16,
}

// Const part of the page.
#[derive(Default, Clone)]
struct PageDef {
    // border
    page_border: Padding,
    // full width for all columns
    full_width: u16,
    // layout parameter flex
    flex: Flex,
    // max block padding
    max_left_padding: u16,
    // max block padding
    max_right_padding: u16,
    // max label
    max_label: u16,
    // max widget
    max_widget: u16,

    // page width
    #[allow(dead_code)]
    width: u16,
    // page height
    height: u16,
    // top border
    top: u16,
    // bottom border
    bottom: u16,
    // columns
    columns: u16,
    // column spacing
    column_spacing: u16,
    // label/widget spacing
    spacing: u16,
    // line spacing
    line_spacing: u16,
}

// Current page data
#[derive(Default, Clone)]
struct Page {
    // page def consts
    def: Rc<PageDef>,
    // page number
    page_no: u16,
    // page start y
    page_start: u16,
    // page end y
    page_end: u16,

    // current y
    y: u16,

    // top padding, accumulated
    top_padding: u16,
    // bottom padding, accumulated
    bottom_padding: u16,
    // bottom padding in case of page-break, accumulated
    bottom_padding_break: u16,
    // current line spacing.
    effective_line_spacing: u16,

    // current page x-positions
    x_pos: XPositions,
}

impl<W> Debug for WidgetDef<W>
where
    W: Clone + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WidgetDef {:?}: {:?} {:?} {:?}",
            self.id,
            self.label_str
                .as_ref()
                .map(|v| v.as_ref())
                .unwrap_or_default(),
            self.label,
            self.widget
        )?;

        Ok(())
    }
}

impl Debug for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Page: [{}x{}] +{}+{} _={}]",
            self.def.width, self.def.height, self.def.top, self.def.bottom, self.def.line_spacing
        )?;
        writeln!(
            f,
            "    page   {} {}..{}",
            self.page_no, self.page_start, self.page_end
        )?;
        writeln!(
            f,
            "    y      {} padding {}|{}|{}",
            self.y, self.top_padding, self.bottom_padding, self.bottom_padding_break
        )?;
        writeln!(
            f,
            "    label  {}+{}",
            self.x_pos.label_left, self.x_pos.label_width
        )?;
        writeln!(
            f,
            "    widget {}+{}",
            self.x_pos.widget_left, self.x_pos.widget_width
        )?;
        writeln!(
            f,
            "    block  {}..{}",
            self.x_pos.container_left, self.x_pos.container_right
        )?;
        write!(
            f, //
            "    total  {}",
            self.x_pos.total_width
        )?;
        Ok(())
    }
}

impl BlockDef {
    fn as_out(&self) -> BlockOut {
        BlockOut {
            block: self.block.clone(),
            area: self.area,
        }
    }
}

impl FormWidget {
    #[inline(always)]
    fn is_stretch_y(&self) -> bool {
        match self {
            FormWidget::None => false,
            FormWidget::Width(_) => false,
            FormWidget::Size(_, _) => false,
            FormWidget::Wide(_, _) => false,
            FormWidget::StretchX(_, _) => false,
            FormWidget::WideStretchX(_, _) => false,
            FormWidget::StretchXY(_, _) => true,
            FormWidget::WideStretchXY(_, _) => true,
            FormWidget::StretchY(_, _) => true,
        }
    }
}

impl<W> Default for LayoutForm<W>
where
    W: Eq + Clone + Hash + Debug,
{
    fn default() -> Self {
        Self {
            spacing: 1,
            line_spacing: Default::default(),
            column_spacing: 1,
            page_border: Default::default(),
            mirror_border: Default::default(),
            columns: 1,
            flex: Default::default(),
            widgets: Default::default(),
            page_breaks: Default::default(),
            min_label: Default::default(),
            min_widget: Default::default(),
            blocks: Default::default(),
            max_left_padding: Default::default(),
            max_right_padding: Default::default(),
            left_padding: Default::default(),
            right_padding: Default::default(),
        }
    }
}

impl<W> LayoutForm<W>
where
    W: Eq + Hash + Clone + Debug,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Spacing between label and widget.
    #[inline]
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Empty lines between widgets.
    #[inline]
    pub fn line_spacing(mut self, spacing: u16) -> Self {
        self.line_spacing = spacing;
        self
    }

    /// Empty space between columns.
    #[inline]
    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

    /// Page border.
    pub fn padding(mut self, border: Padding) -> Self {
        self.page_border = border;
        self
    }

    /// Page border.
    #[deprecated(since = "2.0.0", note = "use padding. is clearer.")]
    pub fn border(mut self, border: Padding) -> Self {
        self.page_border = border;
        self
    }

    /// Mirror the border given to layout between even and odd pages.
    /// The layout starts with page 0 which is even.
    #[inline]
    pub fn mirror_odd_border(mut self) -> Self {
        self.mirror_border = true;
        self
    }

    /// Layout as multiple columns.
    pub fn columns(mut self, columns: u8) -> Self {
        assert_ne!(columns, 0);
        self.columns = columns as u16;
        self
    }

    /// Flex.
    #[inline]
    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }

    /// Set a reference label width
    pub fn min_label(mut self, width: u16) -> Self {
        self.min_label = width;
        self
    }

    /// Set a reference widget width
    pub fn min_widget(mut self, width: u16) -> Self {
        self.min_widget = width;
        self
    }

    /// Start a group/block.
    ///
    /// This will create a block that covers all widgets added
    /// before calling `end()`.
    ///
    /// Groups/blocks can be stacked, but they cannot interleave.
    /// An inner group/block must be closed before an outer one.
    pub fn start(&mut self, block: Option<Block<'static>>) -> BlockTag {
        let max_idx = self.widgets.len();
        let padding = block_padding(&block);

        let tag = BlockTag(self.blocks.len());
        self.blocks.push_back(BlockDef {
            id: tag,
            block,
            padding,
            constructing: true,
            range: max_idx..max_idx,
            area: Rect::default(),
        });

        self.left_padding += padding.left;
        self.right_padding += padding.right;

        self.max_left_padding = max(self.max_left_padding, self.left_padding);
        self.max_right_padding = max(self.max_right_padding, self.right_padding);

        tag
    }

    /// Closes the group/block with the given tag.
    ///
    /// __Panic__
    /// Groups must be closed in reverse start order, otherwise
    /// this function will panic. It will also panic if there
    /// is no open group for the given tag.
    pub fn end(&mut self, tag: BlockTag) {
        let max = self.widgets.len();
        for cc in self.blocks.iter_mut().rev() {
            if cc.id == tag && cc.constructing {
                cc.range.end = max;
                cc.constructing = false;

                // might have been used by a widget.
                self.left_padding -= cc.padding.left;
                self.right_padding -= cc.padding.right;

                return;
            }
            if cc.constructing {
                panic!("Unclosed container {:?}", cc.id);
            }
        }

        panic!("No open container.");
    }

    /// Closes all open groups/blocks.
    pub fn end_all(&mut self) {
        let max = self.widgets.len();
        for cc in self.blocks.iter_mut().rev() {
            if cc.constructing {
                cc.range.end = max;
                cc.constructing = false;

                // might have been used by a widget.
                self.left_padding -= cc.padding.left;
                self.right_padding -= cc.padding.right;
            }
        }
    }

    /// Add a list of label + widget constraints.
    pub fn widgets(&mut self, list: impl IntoIterator<Item = (W, FormLabel, FormWidget)>) {
        for (k, l, w) in list {
            self.widget(k, l, w);
        }
    }

    /// Add label + widget constraint.
    /// Key must be a unique identifier.
    pub fn widget(&mut self, key: W, label: FormLabel, widget: FormWidget) {
        // split label by sample
        let (label, label_str) = match label {
            FormLabel::Str(s) => {
                let width = unicode_display_width::width(s) as u16;
                (FormLabel::Width(width), Some(Cow::Borrowed(s)))
            }
            FormLabel::String(s) => {
                let width = unicode_display_width::width(&s) as u16;
                (FormLabel::Width(width), Some(Cow::Owned(s)))
            }
            FormLabel::Width(w) => (FormLabel::Width(w), None),
            FormLabel::Size(w, h) => (FormLabel::Size(w, h), None),
            FormLabel::None => (FormLabel::None, None),
        };

        let w = match &label {
            FormLabel::None => 0,
            FormLabel::Str(_) | FormLabel::String(_) => {
                unreachable!()
            }
            FormLabel::Width(w) => *w,
            FormLabel::Size(w, _) => *w,
        };
        self.min_label = max(self.min_label, w);

        let w = match &widget {
            FormWidget::None => 0,
            FormWidget::Width(w) => *w,
            FormWidget::Size(w, _) => *w,
            FormWidget::StretchY(w, _) => *w,
            FormWidget::Wide(w, _) => *w,
            FormWidget::StretchX(w, _) => *w,
            FormWidget::WideStretchX(w, _) => *w,
            FormWidget::StretchXY(w, _) => *w,
            FormWidget::WideStretchXY(w, _) => *w,
        };
        self.min_widget = max(self.min_widget, w);

        self.widgets.push(WidgetDef {
            id: key,
            label,
            label_str,
            widget,
        });
    }

    /// Add a manual page break after the last widget.
    ///
    /// This will panic if the widget list is empty.
    pub fn page_break(&mut self) {
        self.page_breaks.push(self.widgets.len() - 1);
    }

    /// Add a manual column break after the last widget.
    /// This is a synonym for [page_break].
    ///
    /// This will panic if the widget list is empty.
    pub fn column_break(&mut self) {
        self.page_breaks.push(self.widgets.len() - 1);
    }

    fn validate_containers(&self) {
        for cc in self.blocks.iter() {
            if cc.constructing {
                panic!("Unclosed container {:?}", cc.id);
            }
        }
    }

    /// Calculate a layout without page-breaks using the given layout-width and padding.
    #[inline(always)]
    pub fn build_endless(self, width: u16) -> GenericLayout<W> {
        self.validate_containers();
        build_layout::<W, true>(self, Size::new(width, u16::MAX))
    }

    /// Calculate the layout for the given page size and padding.
    #[inline(always)]
    pub fn build_paged(self, page: Size) -> GenericLayout<W> {
        self.validate_containers();
        build_layout::<W, false>(self, page)
    }
}

impl XPositions {
    fn new(page: &Page, column: u16, mirror: bool) -> XPositions {
        let border = if mirror {
            Padding::new(
                page.def.page_border.right,
                page.def.page_border.left,
                page.def.page_border.top,
                page.def.page_border.bottom,
            )
        } else {
            page.def.page_border
        };

        let layout_width = page
            .def
            .full_width
            .saturating_sub(border.left)
            .saturating_sub(border.right);
        let column_width =
            (layout_width / page.def.columns).saturating_sub(page.def.column_spacing);
        let right_margin = page.def.full_width.saturating_sub(border.right);

        let offset;
        let label_left;
        let widget_left;
        let container_left;
        let container_right;
        let widget_right;

        match page.def.flex {
            Flex::Legacy => {
                offset = border.left + (column_width + page.def.column_spacing) * column;
                label_left = page.def.max_left_padding;
                widget_left = label_left + page.def.max_label + page.def.spacing;
                widget_right = column_width.saturating_sub(page.def.max_right_padding);

                container_left = 0;
                container_right = column_width;
            }
            Flex::Start => {
                let single_width = page.def.max_left_padding
                    + page.def.max_label
                    + page.def.spacing
                    + page.def.max_widget
                    + page.def.max_right_padding
                    + page.def.column_spacing;

                offset = border.left + single_width * column;
                label_left = page.def.max_left_padding;
                widget_left = label_left + page.def.max_label + page.def.spacing;
                widget_right = widget_left + page.def.max_widget;

                container_left = 0;
                container_right = widget_right + page.def.max_right_padding;
            }
            Flex::Center => {
                let single_width = page.def.max_left_padding
                    + page.def.max_label
                    + page.def.spacing
                    + page.def.max_widget
                    + page.def.max_right_padding
                    + page.def.column_spacing;
                let rest = layout_width
                    .saturating_sub(single_width * page.def.columns)
                    .saturating_add(page.def.column_spacing);

                offset = border.left + rest / 2 + single_width * column;
                label_left = page.def.max_left_padding;
                widget_left = label_left + page.def.max_label + page.def.spacing;
                widget_right = widget_left + page.def.max_widget;

                container_left = 0;
                container_right = widget_right + page.def.max_right_padding;
            }
            Flex::End => {
                let single_width = page.def.max_left_padding
                    + page.def.max_label
                    + page.def.spacing
                    + page.def.max_widget
                    + page.def.max_right_padding
                    + page.def.column_spacing;

                offset = right_margin
                    .saturating_sub(single_width * (page.def.columns - column))
                    .saturating_add(page.def.column_spacing);
                label_left = page.def.max_left_padding;
                widget_left = label_left + page.def.max_label + page.def.spacing;
                widget_right = widget_left + page.def.max_widget;

                container_left = 0;
                container_right = widget_right + page.def.max_right_padding;
            }
            Flex::SpaceAround => {
                let single_width = page.def.max_left_padding
                    + page.def.max_label
                    + page.def.spacing
                    + page.def.max_widget
                    + page.def.max_right_padding;
                let rest = layout_width.saturating_sub(single_width * page.def.columns);
                let spacing = rest / (page.def.columns + 1);

                offset = border.left + spacing + (single_width + spacing) * column;
                label_left = page.def.max_left_padding;
                widget_left = label_left + page.def.max_label + page.def.spacing;
                widget_right = widget_left + page.def.max_widget;

                container_left = 0;
                container_right = widget_right + page.def.max_right_padding;
            }
            Flex::SpaceBetween => {
                let single_width = page.def.max_left_padding
                    + page.def.max_label
                    + page.def.max_widget
                    + page.def.max_right_padding;
                let rest = layout_width.saturating_sub(single_width * page.def.columns);
                let spacing = if page.def.columns > 1 {
                    rest / (page.def.columns - 1)
                } else {
                    0
                };

                offset = border.left + (single_width + spacing) * column;
                label_left = page.def.max_left_padding;
                widget_left = label_left + page.def.max_label + page.def.spacing;
                widget_right = widget_left + page.def.max_widget;

                container_left = 0;
                container_right = widget_right + page.def.max_right_padding;
            }
        }

        XPositions {
            container_left: offset + container_left,
            label_left: offset + label_left,
            label_width: page.def.max_label,
            widget_left: offset + widget_left,
            widget_width: page.def.max_widget,
            container_right: offset + container_right,
            total_width: widget_right - label_left,
            widget_right: offset + widget_right,
        }
    }
}

impl Page {
    fn adjusted_widths<W>(layout: &LayoutForm<W>, page_size: Size) -> (u16, u16, u16)
    where
        W: Eq + Hash + Clone + Debug,
    {
        let layout_width = page_size
            .width
            .saturating_sub(layout.page_border.left)
            .saturating_sub(layout.page_border.right);
        let column_width = (layout_width / layout.columns).saturating_sub(layout.column_spacing);

        let mut max_label = layout.min_label;
        let mut max_widget = layout.min_widget;
        let mut spacing = layout.spacing;

        let nominal =
            layout.max_left_padding + max_label + spacing + max_widget + layout.max_right_padding;

        if nominal > column_width {
            let mut reduce = nominal - column_width;

            if spacing > reduce {
                spacing -= reduce;
                reduce = 0;
            } else {
                reduce -= spacing;
                spacing = 0;
            }
            if max_label > 5 {
                if max_label - 5 > reduce {
                    max_label -= reduce;
                    reduce = 0;
                } else {
                    reduce -= max_label - 5;
                    max_label = 5;
                }
            }
            if max_widget > 5 {
                if max_widget - 5 > reduce {
                    max_widget -= reduce;
                    reduce = 0;
                } else {
                    reduce -= max_widget - 5;
                    max_widget = 5;
                }
            }
            if max_label > reduce {
                max_label -= reduce;
                reduce = 0;
            } else {
                reduce -= max_label;
                max_label = 0;
            }
            if max_widget > reduce {
                max_widget -= reduce;
                // reduce = 0;
            } else {
                // reduce -= max_widget;
                max_widget = 0;
            }
        }

        (max_label, spacing, max_widget)
    }

    fn new<W>(layout: &LayoutForm<W>, page_size: Size) -> Self
    where
        W: Eq + Hash + Clone + Debug,
    {
        let (max_label, spacing, max_widget) = Self::adjusted_widths(layout, page_size);

        let def = PageDef {
            page_border: layout.page_border,
            full_width: page_size.width,
            flex: layout.flex,
            max_left_padding: layout.max_left_padding,
            max_right_padding: layout.max_right_padding,
            max_label,
            max_widget,
            width: page_size.width,
            height: page_size.height,
            top: layout.page_border.top,
            bottom: layout.page_border.bottom,
            columns: layout.columns,
            column_spacing: layout.column_spacing,
            spacing,
            line_spacing: layout.line_spacing,
        };
        let mut s = Self {
            def: Rc::new(def),
            page_no: 0,
            page_start: 0,
            page_end: page_size.height.saturating_sub(layout.page_border.bottom),
            y: layout.page_border.top,
            top_padding: 0,
            bottom_padding: 0,
            bottom_padding_break: 0,
            effective_line_spacing: 0,
            x_pos: Default::default(),
        };
        s.x_pos = XPositions::new(&s, 0, false);
        s
    }
}

// remove top/bottom border when squeezed.
fn adjust_blocks<W>(layout: &mut LayoutForm<W>, page_height: u16)
where
    W: Eq + Hash + Clone + Debug,
{
    if page_height == u16::MAX {
        return;
    }

    if page_height < 3 {
        for block_def in layout.blocks.iter_mut() {
            if let Some(block) = block_def.block.as_mut() {
                let padding = block_padding2(block);
                let borders = if padding.left > 0 {
                    Borders::LEFT
                } else {
                    Borders::NONE
                } | if padding.right > 0 {
                    Borders::RIGHT
                } else {
                    Borders::NONE
                };

                *block = mem::take(block).borders(borders);
                block_def.padding.top = 0;
                block_def.padding.bottom = 0;
            }
        }
    }
}

/// Calculate the layout for the given page size and padding.
fn build_layout<W, const ENDLESS: bool>(
    mut layout: LayoutForm<W>,
    page_size: Size,
) -> GenericLayout<W>
where
    W: Eq + Hash + Clone + Debug,
{
    let mut gen_layout = GenericLayout::with_capacity(
        layout.widgets.len(), //
        layout.blocks.len() * 2,
    );
    gen_layout.set_page_size(page_size);

    // clip Blocks if necessary
    adjust_blocks(&mut layout, page_size.height);

    // indexes into gen_layout for any generated areas that need y adjustment.
    let mut stretch = Vec::with_capacity(layout.widgets.len());
    let mut blocks_out = Vec::with_capacity(layout.blocks.len());

    let mut saved_page;
    let mut page = Page::new(&layout, page_size);

    for (idx, widget) in layout.widgets.iter_mut().enumerate() {
        // safe point
        saved_page = page.clone();

        let mut label_area;
        let mut widget_area;

        (label_area, widget_area) = next_widget(&mut page, &mut layout.blocks, widget, idx, false);
        next_blocks(&mut page, &mut layout.blocks, idx, &mut blocks_out);

        // page overflow induces page-break
        if !ENDLESS && page.y.saturating_add(page.bottom_padding_break) > page.page_end {
            // reset to safe-point
            page = saved_page;

            // page-break
            blocks_out.clear();
            page_break_blocks(&mut page, &mut layout.blocks, idx, &mut blocks_out);
            push_blocks(&mut blocks_out, &mut gen_layout);
            adjust_y_stretch(&page, &mut stretch, &mut gen_layout);
            page_break::<ENDLESS>(&mut page);
            assert!(stretch.is_empty());

            // redo current widget
            (label_area, widget_area) =
                next_widget(&mut page, &mut layout.blocks, widget, idx, true);
            next_blocks(&mut page, &mut layout.blocks, idx, &mut blocks_out);
        }

        // remember stretch widget.
        if !ENDLESS && widget.widget.is_stretch_y() {
            stretch.push(gen_layout.widget_len());
        }

        // add label + widget
        gen_layout.add(
            widget.id.clone(),
            widget_area,
            widget.label_str.take(),
            label_area,
        );

        push_blocks(&mut blocks_out, &mut gen_layout);

        // page-break after widget
        if layout.page_breaks.contains(&idx) {
            assert!(blocks_out.is_empty());
            page_break_blocks(&mut page, &mut layout.blocks, idx + 1, &mut blocks_out);
            push_blocks(&mut blocks_out, &mut gen_layout);
            if !ENDLESS {
                adjust_y_stretch(&page, &mut stretch, &mut gen_layout);
            }
            page_break::<ENDLESS>(&mut page);
            assert!(stretch.is_empty());
        }

        drop_blocks(&mut layout.blocks, idx);
        assert!(blocks_out.is_empty());
    }

    // modify layout to add y-stretch
    adjust_y_stretch(&page, &mut stretch, &mut gen_layout);

    gen_layout.set_page_count(((page.page_no + page.def.columns) / page.def.columns) as usize);
    gen_layout
}

// drop no longer used blocks. perf.
// there may be pathological cases, but otherwise this is fine.
fn drop_blocks(_block_def: &mut VecDeque<BlockDef>, _idx: usize) {
    // TODO: this. there is only pathological cases it seems.
    // loop {
    //     if let Some(block) = block_def.get(0) {
    //         if block.range.end < idx {
    //             block_def.pop_front();
    //         } else {
    //             break;
    //         }
    //     } else {
    //         break;
    //     }
    // }
}

fn page_break_blocks(
    page: &mut Page,
    block_def: &mut VecDeque<BlockDef>,
    idx: usize,
    blocks_out: &mut Vec<BlockOut>,
) {
    // close and push containers
    // rev() ensures closing from outermost to innermost container.
    for block in block_def.iter_mut().rev() {
        if idx > block.range.start && idx < block.range.end {
            end_block(page, block);
            blocks_out.push(block.as_out());

            // restart on next page
            block.range.start = idx;
        }
        // if block.range.start > idx {
        //     break;
        // }
    }
}

// do a page-break
fn page_break<const ENDLESS: bool>(page: &mut Page) {
    // advance
    page.page_no += 1;

    let column = page.page_no % page.def.columns;
    let mirror = (page.page_no / page.def.columns) % 2 == 1;

    if !ENDLESS {
        page.page_start = (page.page_no / page.def.columns).saturating_mul(page.def.height);
        page.page_end = page
            .page_start
            .saturating_add(page.def.height.saturating_sub(page.def.bottom));
    }

    page.x_pos = XPositions::new(page, column, mirror);
    page.y = page.page_start.saturating_add(page.def.top);

    page.effective_line_spacing = 0;
    page.top_padding = 0;
    page.bottom_padding = 0;
    page.bottom_padding_break = 0;
}

// add next widget
fn next_widget<W>(
    page: &mut Page,
    block_def: &mut VecDeque<BlockDef>,
    widget: &WidgetDef<W>,
    idx: usize,
    must_fit: bool,
) -> (Rect, Rect)
where
    W: Eq + Hash + Clone + Debug,
{
    // line spacing
    page.y = page.y.saturating_add(page.effective_line_spacing);

    page.effective_line_spacing = page.def.line_spacing;
    page.top_padding = 0;
    page.bottom_padding = 0;
    page.bottom_padding_break = 0;

    // start container
    for block in block_def.iter_mut() {
        if block.range.start == idx {
            start_block(page, block);
        }
        if block.range.start <= idx {
            widget_padding(page, idx, block);
        }
        // if block.range.start > idx {
        //     break;
        // }
    }

    // get areas + advance
    let (label_area, widget_area, advance) = areas_and_advance(page, widget, must_fit);

    page.y = page.y.saturating_add(advance);

    (label_area, widget_area)
}

// open the given container
fn start_block(page: &mut Page, block: &mut BlockDef) {
    // adjust block
    block.area.x = page.x_pos.container_left;
    block.area.width = page
        .x_pos
        .container_right
        .saturating_sub(page.x_pos.container_left);
    block.area.y = page.y;

    // advance page
    page.y = page.y.saturating_add(block.padding.top);
    page.top_padding += block.padding.top;
    page.x_pos.container_left = page.x_pos.container_left.saturating_add(block.padding.left);
    page.x_pos.container_right = page
        .x_pos
        .container_right
        .saturating_sub(block.padding.right);
}

fn widget_padding(page: &mut Page, idx: usize, block: &mut BlockDef) {
    if block.range.end > idx + 1 {
        page.bottom_padding_break += block.padding.bottom;
    } else if block.range.end == idx + 1 {
        page.bottom_padding += block.padding.bottom;
    }
}

fn next_blocks(
    page: &mut Page,
    block_def: &mut VecDeque<BlockDef>,
    idx: usize,
    blocks_out: &mut Vec<BlockOut>,
) {
    // close and push containers
    // rev() ensures closing from outermost to innermost container.
    for block in block_def.iter_mut().rev() {
        if idx + 1 == block.range.end {
            end_block(page, block);
            blocks_out.push(block.as_out());
        }
        // if block.range.start > idx {
        //     break;
        // }
    }
}

fn push_blocks<W: Eq + Hash + Clone>(
    blocks_out: &mut Vec<BlockOut>,
    gen_layout: &mut GenericLayout<W>,
) {
    while let Some(cc) = blocks_out.pop() {
        gen_layout.add_block(cc.area, cc.block);
    }
}

// close the given container
fn end_block(page: &mut Page, block: &mut BlockDef) {
    // advance page
    page.y = page.y.saturating_add(block.padding.bottom);
    page.x_pos.container_left = page.x_pos.container_left.saturating_sub(block.padding.left);
    page.x_pos.container_right = page
        .x_pos
        .container_right
        .saturating_add(block.padding.right);

    // adjust block
    block.area.height = page.y.saturating_sub(block.area.y);
}

// calculate widget and label area.
// advance the page.y
fn areas_and_advance<W: Debug + Clone>(
    page: &Page,
    widget: &WidgetDef<W>,
    must_fit: bool,
) -> (Rect, Rect, u16) {
    // [label]
    // [widget]
    // vs
    // [label] [widget]
    let stacked = matches!(
        widget.widget,
        FormWidget::Wide(_, _) | FormWidget::WideStretchX(_, _) | FormWidget::WideStretchXY(_, _)
    );

    let mut label_height = match &widget.label {
        FormLabel::None => 0,
        FormLabel::Str(_) | FormLabel::String(_) => unreachable!(),
        FormLabel::Width(_) => 1,
        FormLabel::Size(_, h) => *h,
    };

    let mut widget_height = match &widget.widget {
        FormWidget::None => 0,
        FormWidget::Width(_) => 1,
        FormWidget::Size(_, h) => *h,
        FormWidget::StretchY(_, h) => *h,
        FormWidget::Wide(_, h) => *h,
        FormWidget::StretchX(_, h) => *h,
        FormWidget::WideStretchX(_, h) => *h,
        FormWidget::StretchXY(_, h) => *h,
        FormWidget::WideStretchXY(_, h) => *h,
    };

    let stretch_width = page
        .x_pos
        .widget_right
        .saturating_sub(page.x_pos.widget_left);
    let total_stretch_width = page
        .x_pos
        .widget_right
        .saturating_sub(page.x_pos.label_left);

    let max_height = if !must_fit {
        page.def
            .height
            .saturating_sub(page.def.top)
            .saturating_sub(page.def.bottom)
            .saturating_sub(page.top_padding)
            .saturating_sub(page.bottom_padding)
    } else {
        page.def
            .height
            .saturating_sub(page.y - page.page_start)
            .saturating_sub(page.def.bottom)
            .saturating_sub(page.bottom_padding_break)
    };

    if stacked {
        if label_height + widget_height > max_height {
            label_height = max(1, max_height.saturating_sub(widget_height));
        }
        if label_height + widget_height > max_height {
            widget_height = max(1, max_height.saturating_sub(label_height));
        }
        if label_height + widget_height > max_height {
            label_height = 0;
        }
        if label_height + widget_height > max_height {
            widget_height = max_height;
        }

        let mut label_area = match &widget.label {
            FormLabel::None => Rect::new(
                page.x_pos.label_left, //
                page.y,
                0,
                0,
            ),
            FormLabel::Str(_) | FormLabel::String(_) => unreachable!(),
            FormLabel::Width(_) => Rect::new(
                page.x_pos.label_left,
                page.y,
                page.x_pos.label_width,
                label_height,
            ),
            FormLabel::Size(_, _) => Rect::new(
                page.x_pos.label_left,
                page.y,
                page.x_pos.label_width,
                label_height,
            ),
        };
        match &widget.widget {
            FormWidget::Wide(_, _) => label_area.width = page.x_pos.total_width,
            FormWidget::WideStretchX(_, _) => label_area.width = total_stretch_width,
            FormWidget::WideStretchXY(_, _) => label_area.width = total_stretch_width,
            _ => {}
        }

        let widget_area = match &widget.widget {
            FormWidget::None => unreachable!(),
            FormWidget::Width(_) => unreachable!(),
            FormWidget::Size(_, _) => unreachable!(),
            FormWidget::StretchY(_, _) => unreachable!(),
            FormWidget::Wide(_, _) => Rect::new(
                page.x_pos.label_left,
                page.y + label_height,
                page.x_pos.total_width,
                widget_height,
            ),
            FormWidget::StretchX(_, _) => unreachable!(),
            FormWidget::WideStretchX(_, _) => Rect::new(
                page.x_pos.label_left,
                page.y + label_height,
                total_stretch_width,
                widget_height,
            ),
            FormWidget::StretchXY(_, _) => unreachable!(),
            FormWidget::WideStretchXY(_, _) => Rect::new(
                page.x_pos.label_left,
                page.y + label_height,
                total_stretch_width,
                widget_height,
            ),
        };

        (
            label_area,
            widget_area,
            label_area.height + widget_area.height,
        )
    } else {
        label_height = min(label_height, max_height);
        widget_height = min(widget_height, max_height);
        let height = max(label_height, widget_height);

        let label_area = match &widget.label {
            FormLabel::None => Rect::new(
                page.x_pos.label_left, //
                page.y,
                0,
                0,
            ),
            FormLabel::Str(_) | FormLabel::String(_) => unreachable!(),
            FormLabel::Width(_) => Rect::new(
                page.x_pos.label_left,
                page.y,
                page.x_pos.label_width,
                height,
            ),
            FormLabel::Size(_, _) => Rect::new(
                page.x_pos.label_left,
                page.y,
                page.x_pos.label_width,
                height,
            ),
        };

        let widget_area = match &widget.widget {
            FormWidget::None => Rect::default(),
            FormWidget::Width(w) => Rect::new(
                page.x_pos.widget_left,
                page.y,
                min(*w, page.x_pos.widget_width),
                height,
            ),
            FormWidget::Size(w, _) => Rect::new(
                page.x_pos.widget_left,
                page.y,
                min(*w, page.x_pos.widget_width),
                height,
            ),
            FormWidget::StretchY(w, _) => Rect::new(
                page.x_pos.widget_left,
                page.y,
                min(*w, page.x_pos.widget_width),
                height,
            ),
            FormWidget::Wide(_, _) => unreachable!(),
            FormWidget::StretchX(_, _) => Rect::new(
                page.x_pos.widget_left, //
                page.y,
                stretch_width,
                height,
            ),
            FormWidget::WideStretchX(_, _) => unreachable!(),
            FormWidget::StretchXY(_, _) => Rect::new(
                page.x_pos.widget_left, //
                page.y,
                stretch_width,
                height,
            ),
            FormWidget::WideStretchXY(_, _) => unreachable!(),
        };

        (
            label_area,
            widget_area,
            max(label_area.height, widget_area.height),
        )
    }
}

// some stretching
// stretch_y contains the recorded widget indexes that need adjustment.
fn adjust_y_stretch<W: Eq + Hash + Clone>(
    page: &Page,
    stretch_y: &mut Vec<usize>,
    gen_layout: &mut GenericLayout<W>,
) {
    let mut remainder = page.page_end.saturating_sub(page.y);
    if remainder == 0 {
        stretch_y.clear();
        return;
    }

    let mut n = stretch_y.len() as u16;
    for y_idx in stretch_y.drain(..) {
        // calculate stretch as a new fraction every time.
        // this makes a better distribution.
        let stretch = remainder / n;
        remainder -= stretch;
        n -= 1;

        // stretch
        let mut area = gen_layout.widget(y_idx);
        let test_y = area.bottom();
        let test_x = page.x_pos.container_left;

        area.height += stretch;
        gen_layout.set_widget(y_idx, area);

        // shift everything after
        for idx in y_idx + 1..gen_layout.widget_len() {
            let mut area = gen_layout.widget(idx);
            if area.y >= test_y {
                area.y += stretch;
            }
            gen_layout.set_widget(idx, area);

            let mut area = gen_layout.label(idx);
            if area.y >= test_y {
                area.y += stretch;
            }
            gen_layout.set_label(idx, area);
        }

        // containers may be shifted or stretched.
        for idx in 0..gen_layout.block_len() {
            let mut area = gen_layout.block_area(idx);
            if area.x >= test_x && area.y >= test_y {
                area.y += stretch;
            }
            // may stretch the container
            if area.x >= test_x && area.y <= test_y && area.bottom() > test_y {
                area.height += stretch;
            }
            gen_layout.set_block_area(idx, area);
        }
    }
}
