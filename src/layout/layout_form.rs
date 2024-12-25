use crate::layout::generic_layout::GenericLayout;
use crate::util::block_padding;
use ratatui::layout::{Flex, Rect, Size};
use ratatui::widgets::{Block, Padding};
use std::borrow::Cow;
use std::cmp::{max, min};
use std::fmt::Debug;
use std::ops::Range;

/// Label constraints.
#[derive(Debug, Clone)]
pub enum Label {
    /// No label. Widget fills the space.
    None,
    /// Label by sample
    Str(Cow<'static, str>),
    /// Label by width. (cols).
    Width(u16),
    /// Label by height+width. ( cols, rows).
    Size(u16, u16),
    /// Fill the total width of labels+widget. (rows).
    FullWidthStr(Cow<'static, str>),
    /// Fill the total width of labels+widget. (rows).
    FullWidth(u16),
}

/// Widget constraints.
#[derive(Debug, Clone)]
pub enum Widget {
    /// No widget, just a label.
    None,
    /// Widget aligned with the label. (cols)
    Width(u16),
    /// Widget aligned with the label. (cols, rows)
    Size(u16, u16),
    /// Fill the total width of labels+widget. (rows).
    FullWidth(u16),
    // todo: fill height
}

/// Layout a single column of label/widget.
#[derive(Debug, Clone)]
pub struct LayoutForm<W, C = ()> {
    /// Column spacing.
    spacing: u16,
    /// Line spacing.
    line_spacing: u16,
    /// Flex
    flex: Flex,
    /// Areas
    areas: Vec<(W, Label, Widget, (u16, u16))>,
    /// Page breaks.
    page_breaks: Vec<usize>,

    /// Containers/Blocks
    containers: Vec<(C, Option<Block<'static>>, bool, Range<usize>, Padding, Rect)>,

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

    tmp: Vec<(C, Rect, Option<Block<'static>>)>,
}

impl<W, C> LayoutForm<W, C>
where
    W: Eq + Clone + Debug + Default,
    C: Eq + Clone + Debug + Default,
{
    pub fn new() -> Self {
        Self {
            spacing: Default::default(),
            line_spacing: Default::default(),
            flex: Default::default(),
            areas: Default::default(),
            page_breaks: Default::default(),
            containers: Default::default(),
            max_left_padding: Default::default(),
            max_right_padding: Default::default(),
            c_top: Default::default(),
            c_bottom: Default::default(),
            c_left: Default::default(),
            c_right: Default::default(),
            tmp: Default::default(),
        }
    }

    #[inline]
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    #[inline]
    pub fn line_spacing(mut self, spacing: u16) -> Self {
        self.line_spacing = spacing;
        self
    }

    #[inline]
    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }

    /// Start a container.
    pub fn start(&mut self, container: C, block: Option<Block<'static>>) {
        let max_idx = self.areas.len();
        let padding = block_padding(&block);
        self.containers.push((
            container,
            block,
            true,
            max_idx..max_idx,
            padding,
            Rect::default(),
        ));

        self.c_top += padding.top;
        self.c_bottom += padding.bottom;
        self.c_left += padding.left;
        self.c_right += padding.right;

        self.max_left_padding = max(self.max_left_padding, self.c_left);
        self.max_right_padding = max(self.max_right_padding, self.c_right);
    }

    /// End a container.
    pub fn end(&mut self, container: C) {
        let max = self.areas.len();
        for (id, _, open, range, padding, _) in self.containers.iter_mut().rev() {
            if *id == container && *open {
                *range = range.start..max;
                *open = false;

                // might have been used by a widget.
                if self.c_top > 0 {
                    self.c_top -= padding.top;
                }
                self.c_bottom -= padding.bottom;
                self.c_left -= padding.left;
                self.c_right -= padding.right;

                return;
            }
            if *open {
                panic!("Unclosed container {:?}", *id);
            }
        }

        panic!("No open container.");
    }

    fn validate_containers(&self) {
        for (id, _, open, _, _, _) in self.containers.iter() {
            if *open {
                panic!("Unclosed container {:?}", *id);
            }
        }
    }

    /// Add label + widget.
    pub fn widget(&mut self, key: W, label: Label, widget: Widget) {
        self.areas
            .push((key, label, widget, (self.c_top, self.c_bottom)));

        // top padding is only used once.
        // bottom padding may apply for every contained widget
        // in case of page-break.
        self.c_top = 0;
    }

    /// Add page break.
    ///
    /// Doesn't work before adding at least one widget.
    pub fn page_break(&mut self) {
        self.page_breaks.push(self.areas.len().saturating_sub(1));
    }

    // find maximum width for label, widget and spacing.
    fn find_max(&self, width: u16, border: Padding) -> (u16, u16, u16) {
        let mut label_width = 0;
        let mut widget_width = 0;
        let mut spacing = self.spacing;

        // find max
        for (_, label, widget, _) in self.areas.iter() {
            match label {
                Label::None => {}
                Label::Str(s) => label_width = label_width.max(s.len() as u16),
                Label::Width(w) => label_width = label_width.max(*w),
                Label::Size(w, _) => label_width = label_width.max(*w),
                Label::FullWidthStr(s) => label_width = label_width.max(s.len() as u16),
                Label::FullWidth(_) => {}
            }
            match widget {
                Widget::None => {}
                Widget::Width(w) => widget_width = widget_width.max(*w),
                Widget::Size(w, _) => widget_width = widget_width.max(*w),
                Widget::FullWidth(_) => {}
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

        (label_width, widget_width, spacing)
    }

    // Find horizontal positions for label and widget.
    fn find_pos(
        &self,
        width: u16,
        label_width: u16,
        widget_width: u16,
        spacing: u16,
        border: Padding,
    ) -> (u16, u16, u16, u16, u16) {
        let label_x;
        let widget_x;
        let container_left;
        let container_right;
        let total_width;

        match self.flex {
            Flex::Legacy => {
                label_x = border.left + self.max_left_padding;
                widget_x = label_x + label_width + spacing;
                container_left = label_x.saturating_sub(self.max_left_padding);
                container_right = width.saturating_sub(border.right);
                total_width = label_width + spacing + widget_width;
            }
            Flex::Start => {
                label_x = border.left + self.max_left_padding;
                widget_x = label_x + label_width + spacing;
                container_left = label_x.saturating_sub(self.max_left_padding);
                container_right = widget_x + widget_width + self.max_right_padding;
                total_width = label_width + spacing + widget_width;
            }
            Flex::Center => {
                let rest = width.saturating_sub(
                    border.left
                        + self.max_left_padding
                        + label_width
                        + spacing
                        + widget_width
                        + self.max_right_padding
                        + border.right,
                );
                label_x = border.left + self.max_left_padding + rest / 2;
                widget_x = label_x + label_width + spacing;
                container_left = label_x.saturating_sub(self.max_left_padding);
                container_right = widget_x + widget_width + self.max_right_padding;
                total_width = label_width + spacing + widget_width;
            }
            Flex::End => {
                widget_x =
                    width.saturating_sub(border.right + self.max_right_padding + widget_width);
                label_x = widget_x.saturating_sub(spacing + label_width);
                container_left = label_x.saturating_sub(self.max_left_padding);
                container_right = width.saturating_sub(border.right);
                total_width = label_width + spacing + widget_width;
            }
            Flex::SpaceAround => {
                let rest = width.saturating_sub(
                    border.left
                        + self.max_left_padding
                        + label_width
                        + widget_width
                        + self.max_right_padding
                        + border.right,
                );
                let spacing = rest / 3;

                label_x = border.left + self.max_left_padding + spacing;
                widget_x = label_x + label_width + spacing;
                container_left = border.left;
                container_right = width.saturating_sub(border.right);
                total_width = label_width + spacing + widget_width;
            }
            Flex::SpaceBetween => {
                label_x = border.left + self.max_left_padding;
                widget_x =
                    width.saturating_sub(border.right + self.max_right_padding + widget_width);
                container_left = label_x.saturating_sub(self.max_left_padding);
                container_right = width.saturating_sub(border.right);
                total_width = width.saturating_sub(
                    border.left + self.max_left_padding + border.right + self.max_right_padding,
                );
            }
        }

        (
            container_left,
            label_x,
            widget_x,
            container_right,
            total_width,
        )
    }

    /// Create the layout.
    pub fn layout(mut self, page: Size, border: Padding) -> GenericLayout<W, C> {
        self.validate_containers();

        let (label_width, widget_width, spacing) = self.find_max(page.width, border);
        let (c_left, label_x, widget_x, c_right, total_width) =
            self.find_pos(page.width, label_width, widget_width, spacing, border);

        let mut gen_layout =
            GenericLayout::with_capacity(self.areas.len(), self.containers.len() * 2);
        gen_layout.set_area(Rect::new(0, 0, page.width, page.height));

        let mut page_no = 0u16;
        let mut page_y = page_no * page.height;
        let mut y = border.top;
        let mut line_spacing = 0;
        let mut container_left = c_left;
        let mut container_right = c_right;

        for (idx, (key, label, widget, (c_top, c_bottom))) in self.areas.into_iter().enumerate() {
            // page break
            let brk_label_height = match label {
                Label::None => 0,
                Label::Str(_) => 1,
                Label::Width(_) => 1,
                Label::Size(_, h) => h,
                Label::FullWidthStr(_) => 1,
                Label::FullWidth(h) => h,
            };
            let brk_widget_height = match widget {
                Widget::None => 0,
                Widget::Width(_) => 1,
                Widget::Size(_, h) => h,
                Widget::FullWidth(h) => h,
            };

            let page_break_widget =
                y + line_spacing + c_top + max(brk_label_height, brk_widget_height) + c_bottom
                    >= page_y + page.height.saturating_sub(border.bottom);
            let page_break_man = if idx > 0 {
                self.page_breaks.contains(&(idx - 1))
            } else {
                false
            };

            if page_break_widget || page_break_man {
                // close and push containers
                self.tmp.clear();
                for (key, block, _, range, padding, area) in self.containers.iter_mut().rev() {
                    if idx > range.start && idx < range.end {
                        y += padding.bottom;
                        area.height = y - area.y;
                        self.tmp.push((key.clone(), area.clone(), block.clone()));
                        // restart on next page
                        range.start = idx;
                    }
                }
                while !self.tmp.is_empty() {
                    let c = self.tmp.pop().expect("value");
                    gen_layout.add_container(c.0, c.1, c.2);
                }

                // advance
                page_no += 1;
                page_y = page_no * page.height;
                y = page_y + border.top;
                container_left = c_left;
                container_right = c_right;
            } else {
                // line spacing must not affect page-break of containers.
                y += line_spacing;
            }

            // start container
            for (_, _, _, range, padding, area) in self.containers.iter_mut() {
                if range.start == idx {
                    area.x = container_left;
                    area.width = container_right - container_left;
                    area.y = y;

                    y += padding.top;
                    container_left += padding.left;
                    container_right -= padding.right;
                }
            }

            // maximum allowable height.
            // might be exceeded by a pathological widget or label.
            let max_height = page.height.saturating_sub((y - page_y) + c_bottom);

            // get area + advance label
            // labels fill the available space.
            let label_size = match label {
                Label::None => (0, 0),
                Label::Str(_) => (label_width, min(1, max_height)),
                Label::Width(_) => (label_width, min(1, max_height)),
                Label::Size(_, h) => (label_width, min(h, max_height)),
                Label::FullWidthStr(_) => (label_width, min(1, max_height)),
                Label::FullWidth(h) => (total_width, min(h, max_height)),
            };
            let label_area = Rect::new(label_x, y, label_size.0, label_size.1);
            y += match label {
                Label::FullWidthStr(_) | Label::FullWidth(_) => label_size.1,
                _ => 0,
            };

            // area + advance widget
            let widget_size = match widget {
                Widget::None => (0, 0),
                Widget::Width(w) => (min(w, widget_width), min(1, max_height)),
                Widget::Size(w, h) => (min(w, widget_width), min(h, max_height)),
                Widget::FullWidth(h) => (total_width, min(h, max_height)),
            };
            let widget_area = match widget {
                Widget::None | Widget::Width(_) | Widget::Size(_, _) => {
                    Rect::new(widget_x, y, widget_size.0, widget_size.1)
                }
                Widget::FullWidth(_) => Rect::new(label_x, y, widget_size.0, widget_size.1),
            };
            y += match label {
                Label::FullWidthStr(_) | Label::FullWidth(_) => widget_size.1,
                _ => max(label_size.1, widget_size.1),
            };

            let label_text = match label {
                Label::None => None,
                Label::Str(s) => Some(s.clone()),
                Label::Width(_) => None,
                Label::Size(_, _) => None,
                Label::FullWidthStr(s) => Some(s.clone()),
                Label::FullWidth(_) => None,
            };

            gen_layout.add(key, widget_area, label_text, label_area);

            // end and push containers
            self.tmp.clear();
            for (key, block, _, range, padding, area) in self.containers.iter_mut().rev() {
                if idx + 1 == range.end {
                    y += padding.bottom;

                    area.height = y - area.y;

                    self.tmp.push((key.clone(), area.clone(), block.clone()));

                    container_left -= padding.left;
                    container_right += padding.right;
                }
            }
            while !self.tmp.is_empty() {
                let c = self.tmp.pop().expect("value");
                gen_layout.add_container(c.0, c.1, c.2);
            }

            line_spacing = self.line_spacing;
        }

        gen_layout.page_count = (page_no + 1) as usize;

        gen_layout
    }
}
