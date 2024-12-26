use crate::layout::generic_layout::GenericLayout;
use crate::util::block_padding;
use ratatui::layout::{Flex, Rect, Size};
use ratatui::widgets::{Block, Padding};
use std::borrow::Cow;
use std::cmp::{max, min};
use std::fmt::Debug;
use std::ops::Range;

/// Label constraints.
///
/// Any given widths and heights will be reduced if there is not enough space.
#[derive(Debug, Default, Clone)]
pub enum FormLabel {
    /// No label, just the widget.
    #[default]
    None,
    /// Not a label, just a measure for column-width.
    ///
    /// This will not create an output area.
    Measure(u16),
    /// Label by example.
    /// Line breaks in the text don't work.
    ///
    /// Will create a label area with the max width of all labels and a height of 1.
    /// The area will be top aligned with the widget.
    Str(Cow<'static, str>),
    /// Label by width. (cols).
    ///
    /// Will create a label area with the max width of all labels and a height of 1.
    /// The area will be top aligned with the widget.
    Width(u16),
    /// Label by height+width. (cols, rows).
    ///
    /// Will create a label area with the max width of all labels and a height of rows.
    /// The area will be top aligned with the widget.
    Size(u16, u16),
}

/// Widget constraints.
///
/// Any given widths and heights will be reduced if there is not enough space.
#[derive(Debug, Default, Clone)]
pub enum FormWidget {
    /// No widget, just a label.
    #[default]
    None,
    /// Not a widget, just a measure for widget width.
    ///
    /// This also discards any label added alongside, regardless of
    /// label type, but it still uses the label-width as a measure.
    ///
    /// This will not create an output area.
    Measure(u16),

    /// Widget aligned with the label. (cols)
    ///
    /// Will create an area with the given width and height 1.
    /// The area will be top aligned with the label.
    Width(u16),
    /// Widget aligned with the label. (cols, rows)
    ///
    /// Will create an area with the given width and height.
    /// The area will be top aligned with the label.
    Size(u16, u16),

    /// Fill the total width of labels+widget. (rows).
    /// Any label that is not FormLabel::None will be placed above
    /// the widget.
    ///
    /// Will create an area with the full width of labels + widgets
    /// and the given height.
    FullWidth(u16),

    /// Stretch the widget to the maximum extent horizontally. (rows).
    ///
    /// Will create an area with the full width of the given area,
    /// still respecting labels, borders and blocks.
    Stretch(u16),

    /// Stretch the widget to the maximum extend horizontally,
    /// including the label. (rows).
    ///
    /// Will create an area with the full width of the given area,
    /// still respecting borders and blocks.
    FullStretch(u16),
    // todo: fill height
}

/// Create a layout with a single column of label+widget.
///
/// There are a number of possible constraints that influence
/// the exact layout: [FormLabel] and [FormWidget].
///
/// * This layout can page break the form, if there is not enough
/// space on one page. This can be used with [SinglePager] and friends.
///
/// * Or it can generate an endless layout that will be used
/// with scrolling logic like [Clipper].
///
/// * There is currently no functionality to shrink-fit the layout
/// to a given page size.
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
#[derive(Debug, Clone)]
pub struct LayoutForm<W, C = ()>
where
    W: Eq + Clone + Debug,
    C: Eq + Clone + Debug,
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
    areas: Vec<WidgetDef<W>>,
    /// Page breaks.
    page_breaks: Vec<usize>,

    /// Containers/Blocks
    containers: Vec<ContainerDef<C>>,
    container_areas: Vec<ContainerArea>,

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

#[derive(Debug, Clone)]
struct WidgetDef<W>
where
    W: Debug + Clone,
{
    id: W,
    label: FormLabel,
    widget: FormWidget,
    top_border: u16,
    bottom_border: u16,
}

#[derive(Debug, Clone)]
struct ContainerDef<C>
where
    C: Debug + Clone,
{
    id: C,
    block: Option<Block<'static>>,
    padding: Padding,
    open: bool,
}

#[derive(Debug, Default, Clone)]
struct ContainerArea {
    range: Range<usize>,
    area: Rect,
}

#[derive(Debug, Default, Clone)]
struct Widths {
    label: u16,
    widget: u16,
    spacing: u16,
    stretch: bool,
}

#[derive(Debug, Default, Clone)]
struct Positions {
    label_x: u16,
    label_width: u16,
    widget_x: u16,
    widget_width: u16,
    container_left: u16,
    container_right: u16,
    total_width: u16,
    stretch_width: u16,
    total_stretch_width: u16,
}

impl<W, C> LayoutForm<W, C>
where
    W: Eq + Clone + Debug,
    C: Eq + Clone + Debug,
{
    pub fn new() -> Self {
        Self {
            spacing: Default::default(),
            line_spacing: Default::default(),
            mirror: Default::default(),
            flex: Default::default(),
            areas: Default::default(),
            page_breaks: Default::default(),
            containers: Default::default(),
            container_areas: Default::default(),
            max_left_padding: Default::default(),
            max_right_padding: Default::default(),
            c_top: Default::default(),
            c_bottom: Default::default(),
            c_left: Default::default(),
            c_right: Default::default(),
        }
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

    /// Start a container/block.
    ///
    /// This will create a block that covers all widgets added
    /// before calling `end()`.
    ///
    /// The container identifier need not be unique. It
    /// can be ignored completely by using `()`.
    pub fn start(&mut self, container: C, block: Option<Block<'static>>) {
        let max_idx = self.areas.len();
        let padding = block_padding(&block);
        self.containers.push(ContainerDef {
            id: container,
            block,
            padding,
            open: true,
        });
        self.container_areas.push(ContainerArea {
            range: max_idx..max_idx,
            area: Rect::default(),
        });

        self.c_top += padding.top;
        self.c_bottom += padding.bottom;
        self.c_left += padding.left;
        self.c_right += padding.right;

        self.max_left_padding = max(self.max_left_padding, self.c_left);
        self.max_right_padding = max(self.max_right_padding, self.c_right);
    }

    /// End a container.
    ///
    /// This will close the last container with the given
    /// container id that has not been closed already.
    ///
    /// Containers must be ended in the reverse start order, otherwise
    /// this function will panic.
    /// It will also panic if there is no open container for
    /// the given container id.
    ///
    /// This works fine with `()` too.
    ///
    pub fn end(&mut self, container: C) {
        let max = self.areas.len();
        for (c_idx, cc) in self.containers.iter_mut().enumerate().rev() {
            if cc.id == container && cc.open {
                self.container_areas[c_idx].range.end = max;
                cc.open = false;

                // might have been used by a widget.
                if self.c_top > 0 {
                    self.c_top -= cc.padding.top;
                }
                self.c_bottom -= cc.padding.bottom;
                self.c_left -= cc.padding.left;
                self.c_right -= cc.padding.right;

                return;
            }
            if cc.open {
                panic!("Unclosed container {:?}", cc.id);
            }
        }

        panic!("No open container.");
    }

    fn validate_containers(&self) {
        for cc in self.containers.iter() {
            if cc.open {
                panic!("Unclosed container {:?}", cc.id);
            }
        }
    }

    /// Add label + widget constraint.
    /// Key must be a unique identifier.
    pub fn widget(&mut self, key: W, label: FormLabel, widget: FormWidget) {
        self.areas.push(WidgetDef {
            id: key,
            label,
            widget,
            top_border: self.c_top,
            bottom_border: self.c_bottom,
        });

        // top padding is only used once.
        // bottom padding may apply for every contained widget
        // in case of page-break.
        self.c_top = 0;
    }

    /// Add a manual page break.
    /// This will panic if the widget list is empty.
    pub fn page_break(&mut self) {
        self.page_breaks.push(self.areas.len() - 1);
    }

    // find maximum width for label, widget and spacing.
    fn find_max(&self, width: u16, border: Padding) -> Widths {
        let mut label_width = 0;
        let mut widget_width = 0;
        let mut spacing = self.spacing;
        let mut stretch = false;

        // find max
        for widget in self.areas.iter() {
            match &widget.label {
                FormLabel::None => {}
                FormLabel::Str(s) => label_width = label_width.max(s.len() as u16),
                FormLabel::Width(w) => label_width = label_width.max(*w),
                FormLabel::Size(w, _) => label_width = label_width.max(*w),
                FormLabel::Measure(w) => label_width = label_width.max(*w),
            }
            match &widget.widget {
                FormWidget::None => {}
                FormWidget::Width(w) => widget_width = widget_width.max(*w),
                FormWidget::Size(w, _) => widget_width = widget_width.max(*w),
                FormWidget::FullWidth(_) => {}
                FormWidget::Measure(w) => widget_width = widget_width.max(*w),
                FormWidget::Stretch(_) => stretch = true,
                FormWidget::FullStretch(_) => stretch = true,
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
            stretch,
        }
    }

    // Find horizontal positions for label and widget.
    fn find_pos(&self, layout_width: u16, border: Padding, width: Widths) -> Positions {
        let label_x;
        let widget_x;
        let container_left;
        let container_right;
        let total_width;
        let stretch_width;
        let total_stretch_width;

        let effective_flex = match self.flex {
            Flex::End => {
                // with stretch this is the same as start
                if width.stretch {
                    Flex::Start
                } else {
                    Flex::End
                }
            }
            Flex::Center => {
                // with stretch this is the same as start
                if width.stretch {
                    Flex::Start
                } else {
                    Flex::Center
                }
            }
            v => v,
        };

        match effective_flex {
            Flex::Legacy => {
                label_x = border.left + self.max_left_padding;
                widget_x = label_x + width.label + width.spacing;

                container_left = label_x.saturating_sub(self.max_left_padding);
                container_right = layout_width.saturating_sub(border.right);

                total_width = width.label + width.spacing + width.widget;
                stretch_width = container_right.saturating_sub(widget_x);
                total_stretch_width = container_right.saturating_sub(container_left);
            }
            Flex::Start => {
                label_x = border.left + self.max_left_padding;
                widget_x = label_x + width.label + width.spacing;

                container_left = label_x.saturating_sub(self.max_left_padding);
                if width.stretch {
                    container_right = layout_width.saturating_sub(border.right);
                } else {
                    container_right = widget_x + width.widget + self.max_right_padding;
                }

                total_width = width.label + width.spacing + width.widget;
                stretch_width = container_right.saturating_sub(widget_x);
                total_stretch_width = container_right.saturating_sub(container_left);
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
                stretch_width = width.widget;
                total_stretch_width = total_width;
            }
            Flex::End => {
                widget_x = layout_width
                    .saturating_sub(border.right + self.max_right_padding + width.widget);
                label_x = widget_x.saturating_sub(width.spacing + width.label);

                container_left = label_x.saturating_sub(self.max_left_padding);
                container_right = layout_width.saturating_sub(border.right);

                total_width = width.label + width.spacing + width.widget;
                stretch_width = width.widget;
                total_stretch_width = total_width;
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
                stretch_width = container_right.saturating_sub(widget_x);
                total_stretch_width = container_right.saturating_sub(container_left);
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
                stretch_width = container_right.saturating_sub(widget_x);
                total_stretch_width = container_right.saturating_sub(container_left);
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
            stretch_width,
            total_stretch_width,
        }
    }

    /// Calculate the layout for the given page size and padding.
    pub fn layout(mut self, page: Size, border: Padding) -> GenericLayout<W, C> {
        self.validate_containers();

        let width = self.find_max(page.width, border);
        let pos_even = self.find_pos(page.width, border, width.clone());
        let pos_odd = if self.mirror {
            self.find_pos(
                page.width,
                Padding::new(border.right, border.left, border.top, border.bottom),
                width,
            )
        } else {
            pos_even.clone()
        };

        let mut gen_layout =
            GenericLayout::with_capacity(self.areas.len(), self.containers.len() * 2);
        gen_layout.set_area(Rect::new(0, 0, page.width, page.height));
        gen_layout.set_page_size(page);

        let mut tmp = Vec::new();
        let mut page_no = 0u16;
        let mut page_y = page_no * page.height;
        let mut y = border.top;
        let mut line_spacing = 0;
        let mut pos = &pos_even;
        let mut container_left = pos.container_left;
        let mut container_right = pos.container_right;

        for (idx, widget) in self.areas.iter().enumerate() {
            if matches!(widget.widget, FormWidget::Measure(_)) {
                continue;
            }

            // page break
            let brk_label_height = match &widget.label {
                FormLabel::None => 0,
                FormLabel::Measure(_) => 0,
                FormLabel::Str(_) => 1,
                FormLabel::Width(_) => 1,
                FormLabel::Size(_, h) => *h,
            };
            let brk_widget_height = match &widget.widget {
                FormWidget::None => 0,
                FormWidget::Measure(_) => {
                    unreachable!()
                }
                FormWidget::Width(_) => 1,
                FormWidget::Size(_, h) => *h,
                FormWidget::FullWidth(h) => *h,

                FormWidget::Stretch(h) => *h,
                FormWidget::FullStretch(h) => *h,
            };

            let page_break_widget = y
                + line_spacing
                + widget.top_border
                + max(brk_label_height, brk_widget_height)
                + widget.bottom_border
                >= page_y + page.height.saturating_sub(border.bottom);
            let page_break_man = if idx > 0 {
                self.page_breaks.contains(&(idx - 1))
            } else {
                false
            };

            if page_break_widget || page_break_man {
                // close and push containers
                tmp.clear();
                for (c_idx, cc) in self.containers.iter().enumerate().rev() {
                    let c_area = &mut self.container_areas[c_idx];
                    if idx > c_area.range.start && idx < c_area.range.end {
                        y += cc.padding.bottom;
                        c_area.area.height = y - c_area.area.y;
                        tmp.push((cc.id.clone(), c_area.area.clone(), cc.block.clone()));
                        // restart on next page
                        c_area.range.start = idx;
                    }
                }
                while !tmp.is_empty() {
                    let c = tmp.pop().expect("value");
                    gen_layout.add_container(c.0, c.1, c.2);
                }

                // advance
                page_no += 1;
                page_y = page_no * page.height;
                y = page_y + border.top;
                pos = if page_no % 2 == 0 {
                    &pos_even
                } else {
                    &pos_odd
                };
                container_left = pos.container_left;
                container_right = pos.container_right;
            } else {
                // line spacing must not affect page-break of containers.
                y += line_spacing;
            }

            // start container
            for (c_idx, cc) in self.containers.iter().enumerate() {
                let c_area = &mut self.container_areas[c_idx];
                if c_area.range.start == idx {
                    c_area.area.x = container_left;
                    c_area.area.width = container_right - container_left;
                    c_area.area.y = y;

                    y += cc.padding.top;
                    container_left += cc.padding.left;
                    container_right -= cc.padding.right;
                }
            }

            // maximum allowable height.
            // might be exceeded by a pathological widget or label.
            let max_height = page
                .height
                .saturating_sub((y - page_y) + widget.bottom_border);

            // get area + advance label
            // labels fill the available space.
            let label_area = match &widget.label {
                FormLabel::None => Rect::default(),
                FormLabel::Measure(_) => Rect::default(),
                FormLabel::Str(_) | FormLabel::Width(_) => match &widget.widget {
                    FormWidget::FullWidth(_) => {
                        Rect::new(pos.label_x, y, pos.total_width, min(1, max_height))
                    }
                    FormWidget::FullStretch(_) => {
                        Rect::new(pos.label_x, y, pos.total_stretch_width, min(1, max_height))
                    }
                    _ => Rect::new(pos.label_x, y, pos.label_width, min(1, max_height)),
                },
                FormLabel::Size(_, h) => match &widget.widget {
                    FormWidget::FullWidth(_) => {
                        Rect::new(pos.label_x, y, pos.total_width, min(*h, max_height))
                    }
                    FormWidget::FullStretch(_) => {
                        Rect::new(pos.label_x, y, pos.total_stretch_width, min(*h, max_height))
                    }
                    _ => Rect::new(pos.label_x, y, pos.label_width, min(*h, max_height)),
                },
            };
            y += match &widget.widget {
                FormWidget::FullWidth(_) => label_area.height,
                FormWidget::FullStretch(_) => label_area.height,
                _ => 0,
            };

            // area + advance widget
            let widget_area = match &widget.widget {
                FormWidget::None => Rect::default(),
                FormWidget::Measure(_) => {
                    unreachable!()
                }
                FormWidget::Width(w) => Rect::new(
                    pos.widget_x,
                    y,
                    min(*w, pos.widget_width),
                    min(1, max_height),
                ),
                FormWidget::Size(w, h) => Rect::new(
                    pos.widget_x,
                    y,
                    min(*w, pos.widget_width),
                    min(*h, max_height),
                ),
                FormWidget::FullWidth(h) => {
                    Rect::new(pos.label_x, y, pos.total_width, min(*h, max_height))
                }
                FormWidget::Stretch(h) => {
                    Rect::new(pos.widget_x, y, pos.stretch_width, min(*h, max_height))
                }
                FormWidget::FullStretch(h) => {
                    Rect::new(pos.label_x, y, pos.total_stretch_width, min(*h, max_height))
                }
            };
            let label_text = match &widget.label {
                FormLabel::None => None,
                FormLabel::Str(s) => Some(s.clone()),
                FormLabel::Width(_) => None,
                FormLabel::Size(_, _) => None,
                FormLabel::Measure(_) => None,
            };
            gen_layout.add(widget.id.clone(), widget_area, label_text, label_area);
            y += widget_area.height;

            // end and push containers
            tmp.clear();
            for (c_idx, cc) in self.containers.iter().enumerate().rev() {
                let c_area = &mut self.container_areas[c_idx];
                if idx + 1 == c_area.range.end {
                    y += cc.padding.bottom;

                    c_area.area.height = y - c_area.area.y;

                    tmp.push((cc.id.clone(), c_area.area.clone(), cc.block.clone()));

                    container_left -= cc.padding.left;
                    container_right += cc.padding.right;
                }
            }
            while !tmp.is_empty() {
                let c = tmp.pop().expect("value");
                gen_layout.add_container(c.0, c.1, c.2);
            }

            line_spacing = self.line_spacing;
        }

        gen_layout.set_page_count((page_no + 1) as usize);

        gen_layout
    }
}
