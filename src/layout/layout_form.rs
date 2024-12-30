use crate::layout::generic_layout::GenericLayout;
use crate::util::block_padding;
use ratatui::layout::{Flex, Rect, Size};
use ratatui::widgets::{Block, Padding};
use std::borrow::Cow;
use std::cmp::{max, min};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Range;

/// Label constraints.
///
/// Any given widths and heights will be reduced if there is not enough space.
#[derive(Debug, Default)]
pub enum FormLabel {
    /// No label, just the widget.
    #[default]
    None,
    /// Not a label, just a measure for column-width.
    ///
    /// This will not create an output area but influences the label-width
    /// in the resulting layout.
    ///
    ///  __unit: cols__
    Measure(u16),
    /// Label by example.
    /// Line breaks in the text don't work.
    ///
    /// Will create a label area with the max width of all labels and a height of 1.
    /// The area will be top aligned with the widget.
    Str(&'static str),
    /// Label by example.
    /// Line breaks in the text don't work.
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

/// Widget constraints.
///
/// Any given widths and heights will be reduced if there is not enough space.
#[derive(Debug, Default)]
pub enum FormWidget {
    /// No widget, just a label.
    #[default]
    None,
    /// Not a widget, just a measure for widget width.
    ///
    /// This also discards any label added alongside, regardless of
    /// label type, but it still uses any label-width as a measure.
    ///
    /// This will not create an output area but influences the
    /// width for *Stretch* widgets and the overall layout.
    ///
    /// __unit: cols__
    Measure(u16),
    /// Widget aligned with the label.
    ///
    /// Will create an area with the given width and height 1.
    /// The area will be top aligned with the label.
    ///
    /// __unit: cols__
    Width(u16),
    /// Widget aligned with the label.
    ///
    /// Will create an area with the given width and height.
    /// The area will be top aligned with the label.
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
    /// Will create an area with the given width and height.
    /// The area will be top aligned with the label.
    ///
    /// __unit: cols,rows__
    StretchY(u16, u16),
    /// Fill the total width of labels+widget.
    /// Any label that is not FormLabel::None will be placed above
    /// the widget.
    ///
    /// Will create an area with the full width of labels + widgets
    /// and the given height.
    ///
    /// __unit: rows__
    Wide(u16),

    /// Stretch the widget to the maximum extent horizontally.
    ///
    /// Will create an area with the full width of the given area,
    /// still respecting labels, borders and blocks.
    ///
    /// __unit: rows__
    StretchX(u16),

    /// Stretch the widget to the maximum extend horizontally,
    /// including the label. (rows).
    ///
    /// Will create an area with the full width of the given area,
    /// still respecting borders and blocks.
    ///
    /// __unit: rows__
    WideStretchX(u16),

    /// Stretch the widget to the maximum extent horizontally and vertically.
    ///
    /// The widget will start with the given number of rows.
    /// If there is remaining vertical space left after a page-break this
    /// widget will get it. If there are more than one such widget
    /// the remainder will be evenly distributed.
    ///
    /// __unit: rows__
    StretchXY(u16),

    /// Stretch the widget to the maximum extent horizontally and vertically,
    /// including the label.
    ///
    /// The widget will start with the given number of rows.
    /// If there is remaining vertical space left after a page-break this
    /// widget will get it. If there are more than one such widget
    /// the remainder will be evenly distributed.
    ///
    /// __unit: rows__
    WideStretchXY(u16),
}

/// Create a layout with a single column of label+widget.
///
/// There are a number of possible constraints that influence
/// the exact layout: [FormLabel] and [FormWidget].
///
/// * This layout can page break the form, if there is not enough
///   space on one page. This can be used with [SinglePager](crate::pager::SinglePager)
///   and friends.
///
/// * Or it can generate an endless layout that will be used
///   with scrolling logic like [Clipper](crate::clipper::Clipper).
///
/// * There is currently no functionality to shrink-fit the layout
///   to a given page size.
///
/// The widgets can be grouped together and a [Block] can be set
/// to highlight this grouping. Groups can cascade. Groups will
/// be correctly broken by the page break logic. There is no
/// special handling for orphans and widows.
///
/// Other features:
/// * Spacing/Line spacing.
/// * Supports Flex.
/// * Manual page breaks.
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
    /// Mirror the borders between even/odd pages.
    mirror: bool,
    /// Flex
    flex: Flex,
    /// Areas
    widgets: Vec<WidgetDef<W>>,
    /// Containers/Blocks
    blocks: Vec<BlockDef>,
    /// Page breaks.
    page_breaks: Vec<usize>,

    /// maximum padding due to containers.
    max_left_padding: u16,
    max_right_padding: u16,

    /// container padding, accumulated.
    /// current active top-padding. valid for 1 widget.
    c_top: u16,
    /// current active bottom-padding.
    /// valid for every contained widget to calculate a page-break.
    c_bottom: u16,
    /// current left indent.
    c_left: u16,
    /// current right indent.
    c_right: u16,
}

#[derive(Debug)]
struct WidgetDef<W>
where
    W: Debug + Clone,
{
    // widget id
    id: W,
    // label constraint
    label: FormLabel,
    // widget constraint
    widget: FormWidget,
    // effective top border due to container padding.
    top_border: u16,
    // effective bottom border due to container padding.
    bottom_border: u16,
    // optional bottom border. all containers that
    // do not end exactly at this widget contribute.
    opt_bottom_border: u16,
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

// widths deduced from constraints.
#[derive(Debug, Clone, Copy)]
struct Widths {
    // max label width
    label: u16,
    // max widget width
    widget: u16,
    // actual spacing between label and widget
    spacing: u16,
}

// effective positions for layout construction.
#[derive(Debug, Default, Clone, Copy)]
struct Positions {
    // label position
    label_x: u16,
    // label width, max.
    label_width: u16,
    // widget position
    widget_x: u16,
    // widget width, max.
    widget_width: u16,
    // left position for container blocks.
    container_left: u16,
    // right position for container blocks.
    container_right: u16,
    // total width label+spacing+widget
    total_width: u16,
}

// Current page data
#[derive(Debug, Default, Clone, Copy)]
struct Page {
    // page width
    #[allow(dead_code)]
    width: u16,
    // page height
    height: u16,
    // top border
    top: u16,
    // bottom border
    bottom: u16,
    // maximum widget + label height
    max_height: u16,

    // page number
    page_no: u16,
    // page start y
    y_page: u16,
    // current y
    y: u16,

    // current line spacing
    line_spacing: u16,
    // container left pos
    container_left: u16,
    // container right pos
    container_right: u16,
}

impl BlockDef {
    fn as_out(&self) -> BlockOut {
        BlockOut {
            block: self.block.clone(),
            area: self.area,
        }
    }
}

impl FormLabel {
    fn label_txt(self) -> Option<Cow<'static, str>> {
        match self {
            FormLabel::None => None,
            FormLabel::Measure(_) => None,
            FormLabel::Str(s) => Some(Cow::Borrowed(s)),
            FormLabel::String(s) => Some(Cow::Owned(s)),
            FormLabel::Width(_) => None,
            FormLabel::Size(_, _) => None,
        }
    }
}

impl FormWidget {
    fn is_stretch_y(&self) -> bool {
        match self {
            FormWidget::None => false,
            FormWidget::Measure(_) => false,
            FormWidget::Width(_) => false,
            FormWidget::Size(_, _) => false,
            FormWidget::Wide(_) => false,
            FormWidget::StretchX(_) => false,
            FormWidget::WideStretchX(_) => false,
            FormWidget::StretchXY(_) => true,
            FormWidget::WideStretchXY(_) => true,
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
            mirror: Default::default(),
            flex: Default::default(),
            widgets: Default::default(),
            page_breaks: Default::default(),
            blocks: Default::default(),
            max_left_padding: Default::default(),
            max_right_padding: Default::default(),
            c_top: Default::default(),
            c_bottom: Default::default(),
            c_left: Default::default(),
            c_right: Default::default(),
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

    /// Mirror the border given to layout between even and odd pages.
    /// The layout starts with page 0 which is even.
    #[inline]
    pub fn mirror_odd_border(mut self) -> Self {
        self.mirror = true;
        self
    }

    /// Flex.
    #[inline]
    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
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
        self.blocks.push(BlockDef {
            id: tag,
            block,
            padding,
            constructing: true,
            range: max_idx..max_idx,
            area: Rect::default(),
        });

        self.c_top += padding.top;
        self.c_bottom += padding.bottom;
        self.c_left += padding.left;
        self.c_right += padding.right;

        self.max_left_padding = max(self.max_left_padding, self.c_left);
        self.max_right_padding = max(self.max_right_padding, self.c_right);

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
                if self.c_top > 0 {
                    self.c_top -= cc.padding.top;
                }
                self.c_bottom -= cc.padding.bottom;
                self.c_left -= cc.padding.left;
                self.c_right -= cc.padding.right;

                if let Some(last) = self.widgets.last_mut() {
                    last.opt_bottom_border -= cc.padding.bottom;
                }

                return;
            }
            if cc.constructing {
                panic!("Unclosed container {:?}", cc.id);
            }
        }

        panic!("No open container.");
    }

    fn validate_containers(&self) {
        for cc in self.blocks.iter() {
            if cc.constructing {
                panic!("Unclosed container {:?}", cc.id);
            }
        }
    }

    /// Add label + widget constraint.
    /// Key must be a unique identifier.
    pub fn widget(&mut self, key: W, label: FormLabel, widget: FormWidget) {
        self.widgets.push(WidgetDef {
            id: key,
            label,
            widget,
            top_border: self.c_top,
            bottom_border: self.c_bottom,
            opt_bottom_border: self.c_bottom,
        });

        // top padding is only used once.
        // bottom padding may apply for every contained widget
        // in case of page-break.
        self.c_top = 0;
    }

    /// Add a manual page break after the last widget.
    ///
    /// This will panic if the widget list is empty.
    pub fn page_break(&mut self) {
        self.page_breaks.push(self.widgets.len() - 1);
    }

    // find maximum width for label, widget and spacing.
    fn find_max(&self, width: u16, border: Padding) -> Widths {
        let mut label_width = 0;
        let mut widget_width = 0;
        let mut spacing = self.spacing;

        // find max
        for widget in self.widgets.iter() {
            match &widget.label {
                FormLabel::None => {}
                FormLabel::Str(s) => label_width = label_width.max(s.len() as u16),
                FormLabel::String(s) => label_width = label_width.max(s.len() as u16),
                FormLabel::Width(w) => label_width = label_width.max(*w),
                FormLabel::Size(w, _) => label_width = label_width.max(*w),
                FormLabel::Measure(w) => label_width = label_width.max(*w),
            }
            match &widget.widget {
                FormWidget::None => {}
                FormWidget::Width(w) => widget_width = widget_width.max(*w),
                FormWidget::Size(w, _) => widget_width = widget_width.max(*w),
                FormWidget::StretchY(w, _) => widget_width = widget_width.max(*w),
                FormWidget::Wide(_) => {}
                FormWidget::Measure(w) => widget_width = widget_width.max(*w),
                FormWidget::StretchX(_) => {}
                FormWidget::WideStretchX(_) => {}
                FormWidget::StretchXY(_) => {}
                FormWidget::WideStretchXY(_) => {}
            }
        }

        // cut excess
        let width = width.saturating_sub(
            border.left + self.max_left_padding + self.max_right_padding + border.right,
        );
        if label_width + self.spacing + widget_width > width {
            let mut reduce = label_width + self.spacing + widget_width - width;

            if self.spacing > reduce {
                spacing -= reduce;
                reduce = 0;
            } else {
                reduce -= self.spacing;
                spacing = 0;
            }
            if label_width > 5 {
                if label_width - 5 > reduce {
                    label_width -= reduce;
                    reduce = 0;
                } else {
                    reduce -= label_width - 5;
                    label_width = 5;
                }
            }
            if widget_width > 5 {
                if widget_width - 5 > reduce {
                    widget_width -= reduce;
                    reduce = 0;
                } else {
                    reduce -= widget_width - 5;
                    widget_width = 5;
                }
            }
            if label_width > reduce {
                label_width -= reduce;
                reduce = 0;
            } else {
                reduce -= label_width;
                label_width = 0;
            }
            if widget_width > reduce {
                widget_width -= reduce;
                // reduce = 0;
            } else {
                // reduce -= max_widget;
                widget_width = 0;
            }
        }

        Widths {
            label: label_width,
            widget: widget_width,
            spacing,
        }
    }

    // Find horizontal positions for label and widget.
    fn find_pos(&self, layout_width: u16, border: Padding, width: Widths) -> Positions {
        let label_x;
        let widget_x;
        let container_left;
        let container_right;
        let total_width;

        match self.flex {
            Flex::Legacy => {
                label_x = border.left + self.max_left_padding;
                widget_x = label_x + width.label + width.spacing;

                container_left = label_x.saturating_sub(self.max_left_padding);
                container_right = layout_width.saturating_sub(border.right);

                total_width = width.label + width.spacing + width.widget;
            }
            Flex::Start => {
                label_x = border.left + self.max_left_padding;
                widget_x = label_x + width.label + width.spacing;

                container_left = label_x.saturating_sub(self.max_left_padding);
                container_right = widget_x + width.widget + self.max_right_padding;

                total_width = width.label + width.spacing + width.widget;
            }
            Flex::Center => {
                let rest = layout_width.saturating_sub(
                    border.left
                        + self.max_left_padding
                        + width.label
                        + width.spacing
                        + width.widget
                        + self.max_right_padding
                        + border.right,
                );
                label_x = border.left + self.max_left_padding + rest / 2;
                widget_x = label_x + width.label + width.spacing;

                container_left = label_x.saturating_sub(self.max_left_padding);
                container_right = widget_x + width.widget + self.max_right_padding;

                total_width = width.label + width.spacing + width.widget;
            }
            Flex::End => {
                widget_x = layout_width
                    .saturating_sub(border.right + self.max_right_padding + width.widget);
                label_x = widget_x.saturating_sub(width.spacing + width.label);

                container_left = label_x.saturating_sub(self.max_left_padding);
                container_right = layout_width.saturating_sub(border.right);

                total_width = width.label + width.spacing + width.widget;
            }
            Flex::SpaceAround => {
                let rest = layout_width.saturating_sub(
                    border.left
                        + self.max_left_padding
                        + width.label
                        + width.widget
                        + self.max_right_padding
                        + border.right,
                );
                let spacing = rest / 3;

                label_x = border.left + self.max_left_padding + spacing;
                widget_x = label_x + width.label + spacing;

                container_left = border.left;
                container_right = layout_width.saturating_sub(border.right);

                total_width = width.label + spacing + width.widget;
            }
            Flex::SpaceBetween => {
                label_x = border.left + self.max_left_padding;
                widget_x = layout_width
                    .saturating_sub(border.right + self.max_right_padding + width.widget);

                container_left = label_x.saturating_sub(self.max_left_padding);
                container_right = layout_width.saturating_sub(border.right);

                total_width = layout_width.saturating_sub(
                    border.left + self.max_left_padding + border.right + self.max_right_padding,
                );
            }
        }

        Positions {
            container_left,
            label_x,
            label_width: width.label,
            widget_x,
            widget_width: width.widget,
            container_right,
            total_width,
        }
    }

    /// Calculate a layout without page-breaks using the given layout-width and padding.
    #[inline(always)]
    pub fn endless(self, width: u16, border: Padding) -> GenericLayout<W> {
        self._layout::<true>(Size::new(width, u16::MAX), border)
    }

    /// Calculate the layout for the given page size and padding.
    #[inline(always)]
    pub fn paged(self, page: Size, border: Padding) -> GenericLayout<W> {
        self._layout::<false>(page, border)
    }

    /// Calculate the layout for the given page size and padding.
    fn _layout<const ENDLESS: bool>(mut self, page: Size, border: Padding) -> GenericLayout<W> {
        self.validate_containers();

        let width = self.find_max(page.width, border);
        let pos_even = self.find_pos(page.width, border, width);
        let pos_odd = if self.mirror {
            self.find_pos(
                page.width,
                Padding::new(border.right, border.left, border.top, border.bottom),
                width,
            )
        } else {
            pos_even
        };

        let mut gen_layout =
            GenericLayout::with_capacity(self.widgets.len(), self.blocks.len() * 2);
        gen_layout.set_page_size(page);

        let mut tmp = Vec::new();

        let mut pos = &pos_even;
        let mut page_bak;
        let mut page = Page {
            width: page.width,
            height: page.height,
            top: border.top,
            bottom: border.bottom,
            max_height: page.height.saturating_sub(border.top + border.bottom),

            page_no: 0,
            y_page: 0,
            y: border.top,

            line_spacing: 0,
            container_left: pos.container_left,
            container_right: pos.container_right,
        };
        // indexes into gen_layout.
        let mut stretch_y = Vec::new();

        for (idx, widget) in self.widgets.into_iter().enumerate() {
            if matches!(widget.widget, FormWidget::Measure(_)) {
                continue;
            }

            // safe point
            page_bak = page;

            // line spacing
            page.next_widget();
            // start container
            for cc in self.blocks.iter_mut() {
                if cc.range.start == idx {
                    page.start_container(cc);
                }
            }
            // get areas + advance
            let (mut label_area, mut widget_area) = page.widget_area(&widget, pos);
            // end and push containers
            for cc in self.blocks.iter_mut().rev() {
                if idx + 1 == cc.range.end {
                    page.end_container(cc);
                    tmp.push(cc.as_out());
                }
            }

            let break_overflow = if ENDLESS {
                false
            } else {
                page.y.saturating_add(widget.opt_bottom_border)
                    >= page
                        .y_page
                        .saturating_add(page.height.saturating_sub(page.bottom))
            };
            let break_manual = if ENDLESS {
                false
            } else {
                self.page_breaks.contains(&idx)
            };

            // page overflow induces page-break
            if break_overflow {
                // reset safe-point
                page = page_bak;
                // any container areas are invalid
                tmp.clear();

                // close and push containers
                // rev() ensures closing from innermost to outermost container.
                for cc in self.blocks.iter_mut().rev() {
                    if idx > cc.range.start && idx < cc.range.end {
                        page.end_container(cc);
                        tmp.push(cc.as_out());
                        // restart on next page
                        cc.range.start = idx;
                    }
                }
                // pop reverts the ordering for render
                while let Some(cc) = tmp.pop() {
                    gen_layout.add_block(cc.area, cc.block);
                }

                // modify layout to add y-stretch
                Self::y_stretch(&page, &mut stretch_y, &mut gen_layout);

                // advance
                pos = page.next_page(&pos_even, &pos_odd);

                // redo current widget

                // line spacing
                page.next_widget();
                // start container
                for cc in self.blocks.iter_mut() {
                    if idx == cc.range.start {
                        page.start_container(cc);
                    }
                }
                // get areas + advance
                (label_area, widget_area) = page.widget_area(&widget, pos);
                // end and push containers
                // rev() ensures closing from innermost to outermost container.
                for cc in self.blocks.iter_mut().rev() {
                    if idx + 1 == cc.range.end {
                        page.end_container(cc);
                        tmp.push(cc.as_out());
                    }
                }

                page.line_spacing = self.line_spacing;
            }

            // remember stretch widget.
            if widget.widget.is_stretch_y() && !ENDLESS {
                stretch_y.push(gen_layout.widget_len());
            }
            // add label + widget
            gen_layout.add(
                widget.id.clone(),
                widget_area,
                widget.label.label_txt(),
                label_area,
            );
            // pop reverts the ordering for render
            while let Some(cc) = tmp.pop() {
                gen_layout.add_block(cc.area, cc.block);
            }

            if break_manual {
                // page-break after widget

                // close and push containers
                // rev() ensures closing from innermost to outermost container.
                for cc in self.blocks.iter_mut().rev() {
                    if idx + 1 > cc.range.start && idx + 1 < cc.range.end {
                        page.end_container(cc);
                        tmp.push(cc.as_out());
                        // restart on next page
                        cc.range.start = idx + 1;
                    }
                }
                // pop reverts the ordering for render
                while let Some(cc) = tmp.pop() {
                    gen_layout.add_block(cc.area, cc.block);
                }

                // modify layout to add y-stretch
                Self::y_stretch(&page, &mut stretch_y, &mut gen_layout);

                // advance
                pos = page.next_page(&pos_even, &pos_odd);
            }

            if !break_overflow && !break_manual {
                // reset line spacing
                page.line_spacing = self.line_spacing;
            }
        }

        gen_layout.set_page_count((page.page_no + 1) as usize);

        gen_layout
    }

    // some stretching
    fn y_stretch(page: &Page, stretch_y: &mut Vec<usize>, gen_layout: &mut GenericLayout<W>) {
        let bottom_y = page
            .y_page
            .saturating_add(page.height.saturating_sub(page.bottom));

        let mut remainder = bottom_y.saturating_sub(page.y);
        if remainder == 0 {
            return;
        }
        let mut n = stretch_y.len() as u16;

        for y_idx in stretch_y.drain(..) {
            // calculate stretch as a new fraction.
            // this makes a better distribution.
            let stretch = remainder / n;
            remainder -= stretch;
            n -= 1;

            // stretch
            let mut area = gen_layout.widget(y_idx);
            let test_y = area.bottom();
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
                if area.y >= test_y {
                    area.y += stretch;
                }
                // may stretch the container
                if area.y <= test_y && area.bottom() > test_y {
                    area.height += stretch;
                }
                gen_layout.set_block_area(idx, area);
            }
        }
    }
}

impl Page {
    fn widget_area<W: Debug + Clone>(
        &mut self,
        widget: &WidgetDef<W>,
        pos: &Positions,
    ) -> (Rect, Rect) {
        let stacked = matches!(
            widget.widget,
            FormWidget::Wide(_) | FormWidget::WideStretchX(_) | FormWidget::WideStretchXY(_)
        );

        let mut label_height = match &widget.label {
            FormLabel::None => 0,
            FormLabel::Measure(_) => 0,
            FormLabel::Str(_) | FormLabel::String(_) => 1,
            FormLabel::Width(_) => 1,
            FormLabel::Size(_, h) => *h,
        };

        let mut widget_height = match &widget.widget {
            FormWidget::None => 0,
            FormWidget::Measure(_) => {
                unreachable!()
            }
            FormWidget::Width(_) => 1,
            FormWidget::Size(_, h) => *h,
            FormWidget::StretchY(_, h) => *h,
            FormWidget::Wide(h) => *h,
            FormWidget::StretchX(h) => *h,
            FormWidget::WideStretchX(h) => *h,
            FormWidget::StretchXY(h) => *h,
            FormWidget::WideStretchXY(h) => *h,
        };

        let stretch_width = self.container_right.saturating_sub(pos.widget_x + 1);
        let total_stretch_width = self.container_right.saturating_sub(pos.label_x + 1);

        if stacked {
            let max_height = self
                .max_height
                .saturating_sub(widget.top_border + widget.bottom_border);
            if label_height + widget_height > max_height {
                label_height = min(1, max_height.saturating_sub(widget_height));
            }
            if label_height + widget_height > max_height {
                widget_height = min(1, max_height.saturating_sub(label_height));
            }
            if label_height + widget_height > max_height {
                label_height = 0;
            }
            if label_height + widget_height > max_height {
                widget_height = max_height;
            }

            let mut label_area = match &widget.label {
                FormLabel::None => Rect::default(),
                FormLabel::Measure(_) => Rect::default(),
                FormLabel::Str(_) | FormLabel::String(_) => {
                    Rect::new(pos.label_x, self.y, pos.label_width, label_height)
                }
                FormLabel::Width(_) => {
                    Rect::new(pos.label_x, self.y, pos.label_width, label_height)
                }
                FormLabel::Size(_, _) => {
                    Rect::new(pos.label_x, self.y, pos.label_width, label_height)
                }
            };
            match &widget.widget {
                FormWidget::Wide(_) => label_area.width = pos.total_width,
                FormWidget::WideStretchX(_) => label_area.width = total_stretch_width,
                FormWidget::WideStretchXY(_) => label_area.width = total_stretch_width,
                _ => {}
            }

            self.y = self.y.saturating_add(label_area.height);

            let widget_area = match &widget.widget {
                FormWidget::None => Rect::default(),
                FormWidget::Measure(_) => {
                    unreachable!()
                }
                FormWidget::Width(w) => Rect::new(
                    pos.widget_x,
                    self.y,
                    min(*w, pos.widget_width),
                    widget_height,
                ),
                FormWidget::Size(w, _) => Rect::new(
                    pos.widget_x,
                    self.y,
                    min(*w, pos.widget_width),
                    widget_height,
                ),
                FormWidget::StretchY(w, _) => Rect::new(
                    pos.widget_x,
                    self.y,
                    min(*w, pos.widget_width),
                    widget_height,
                ),
                FormWidget::Wide(_) => {
                    Rect::new(pos.label_x, self.y, pos.total_width, widget_height)
                }
                FormWidget::StretchX(_) => {
                    Rect::new(pos.widget_x, self.y, stretch_width, widget_height)
                }
                FormWidget::WideStretchX(_) => {
                    Rect::new(pos.label_x, self.y, total_stretch_width, widget_height)
                }
                FormWidget::StretchXY(_) => {
                    Rect::new(pos.widget_x, self.y, stretch_width, widget_height)
                }
                FormWidget::WideStretchXY(_) => {
                    Rect::new(pos.label_x, self.y, total_stretch_width, widget_height)
                }
            };

            self.y = self.y.saturating_add(widget_area.height);

            (label_area, widget_area)
        } else {
            let max_height = self
                .max_height
                .saturating_sub(widget.top_border + widget.bottom_border);
            label_height = min(label_height, max_height);
            widget_height = min(widget_height, max_height);

            let label_area = match &widget.label {
                FormLabel::None => Rect::default(),
                FormLabel::Measure(_) => Rect::default(),
                FormLabel::Str(_) | FormLabel::String(_) => {
                    Rect::new(pos.label_x, self.y, pos.label_width, label_height)
                }
                FormLabel::Width(_) => {
                    Rect::new(pos.label_x, self.y, pos.label_width, label_height)
                }
                FormLabel::Size(_, _) => {
                    Rect::new(pos.label_x, self.y, pos.label_width, label_height)
                }
            };

            let widget_area = match &widget.widget {
                FormWidget::None => Rect::default(),
                FormWidget::Measure(_) => {
                    unreachable!()
                }
                FormWidget::Width(w) => Rect::new(
                    pos.widget_x,
                    self.y,
                    min(*w, pos.widget_width),
                    widget_height,
                ),
                FormWidget::Size(w, _) => Rect::new(
                    pos.widget_x,
                    self.y,
                    min(*w, pos.widget_width),
                    widget_height,
                ),
                FormWidget::StretchY(w, _) => Rect::new(
                    pos.widget_x,
                    self.y,
                    min(*w, pos.widget_width),
                    widget_height,
                ),
                FormWidget::Wide(_) => {
                    unreachable!()
                }
                FormWidget::StretchX(_) => {
                    Rect::new(pos.widget_x, self.y, stretch_width, widget_height)
                }
                FormWidget::WideStretchX(_) => {
                    unreachable!()
                }
                FormWidget::StretchXY(_) => {
                    Rect::new(pos.widget_x, self.y, stretch_width, widget_height)
                }
                FormWidget::WideStretchXY(_) => {
                    unreachable!()
                }
            };

            self.y = self
                .y
                .saturating_add(max(label_area.height, widget_area.height));

            (label_area, widget_area)
        }
    }

    // advance to next page
    #[inline(always)]
    fn next_page<'a>(&mut self, pos_even: &'a Positions, pos_odd: &'a Positions) -> &'a Positions {
        self.page_no += 1;
        self.y_page = self.page_no.saturating_mul(self.height);
        self.y = self.y_page.saturating_add(self.top);
        self.line_spacing = 0;

        if self.page_no % 2 == 0 {
            pos_even
        } else {
            pos_odd
        }
    }

    // advance to next widget
    #[inline(always)]
    fn next_widget(&mut self) {
        self.y = self.y.saturating_add(self.line_spacing);
    }

    // close the given container
    #[inline(always)]
    fn end_container(&mut self, cc: &mut BlockDef) {
        self.y = self.y.saturating_add(cc.padding.bottom);
        self.container_left = self.container_left.saturating_sub(cc.padding.left);
        self.container_right = self.container_right.saturating_add(cc.padding.right);

        cc.area.height = self.y.saturating_sub(cc.area.y);
    }

    // open the given container
    #[inline(always)]
    fn start_container(&mut self, cc: &mut BlockDef) {
        cc.area.x = self.container_left;
        cc.area.width = self.container_right.saturating_sub(self.container_left);
        cc.area.y = self.y;

        self.y = self.y.saturating_add(cc.padding.top);
        self.container_left = self.container_left.saturating_add(cc.padding.left);
        self.container_right = self.container_right.saturating_sub(cc.padding.right);
    }
}
