//!
//! Calculate the layout for an edit-mask with lots of label/widget pairs.
//!
//! This is the progenitor of [LayoutForm].
//!
use crate::layout::GenericLayout;
use ratatui_core::layout::{Flex, Rect, Size};
use std::borrow::Cow;
use std::cmp::{max, min};

/// Constraint data for [layout_edit]
#[allow(variant_size_differences)]
#[derive(Debug)]
pub enum EditConstraint {
    /// Label by sample.
    Label(Cow<'static, str>),
    /// Label by width.
    /// __unit: cols__
    LabelWidth(u16),
    /// Label by height+width.
    /// __unit: cols, rows__
    LabelRows(u16, u16),
    /// Label occupying the full row.
    /// This is added with its own index.
    TitleLabel(Cow<'static, str>),
    /// Label occupying the full row, but rendering only part of it.
    /// This is added with its own index.
    /// __unit: cols__
    TitleLabelWidth(u16),
    /// Label occupying multiple full rows.
    /// This is added with its own index.
    /// __unit: rows__
    TitleLabelRows(u16),
    /// Widget aligned with the label.
    /// __unit: cols__
    Widget(u16),
    /// Widget aligned with the label.
    /// __unit: cols, rows__
    WidgetRows(u16, u16),
    /// Empty space.
    /// This is not a widget, just some spacing.
    Empty,
    /// Empty space.
    /// This is not a widget, just some spacing.
    /// __unit: rows__
    EmptyRows(u16),
    /// Widget aligned with the left margin.
    /// __unit: cols__
    LineWidget(u16),
    /// Widget aligned with the left margin.
    /// __unit: cols, rows__
    LineWidgetRows(u16, u16),
}

/// Layout for an edit mask with lots of label+widget pairs.
///
/// This neatly aligns labels and widgets in one column.
/// Use the edit constraints to define the layout.
///
/// This returns a [GenericLayout] with indexed widgets.
///
/// If the space runs out during layout, everything
/// gets stuffed in the last row, regardless.
///
/// For more features see [LayoutForm](crate::layout::LayoutForm).
///
#[allow(clippy::comparison_chain)]
pub fn layout_edit(
    area: Size,
    constraints: &[EditConstraint],
    mut spacing: u16,
    flex: Flex,
) -> GenericLayout<usize> {
    let mut max_label = 0;
    let mut max_widget = 0;

    // find max
    for l in constraints.iter() {
        match l {
            EditConstraint::Label(s) => max_label = max(max_label, s.len() as u16),
            EditConstraint::LabelWidth(w) => max_label = max(max_label, *w),
            EditConstraint::LabelRows(w, _) => max_label = max(max_label, *w),
            EditConstraint::TitleLabel(_) => {}
            EditConstraint::TitleLabelWidth(_) => {}
            EditConstraint::TitleLabelRows(_) => {}
            EditConstraint::Widget(w) => max_widget = max(max_widget, *w),
            EditConstraint::WidgetRows(w, _) => max_widget = max(max_widget, *w),
            EditConstraint::LineWidget(_) => {}
            EditConstraint::LineWidgetRows(_, _) => {}
            EditConstraint::Empty => {}
            EditConstraint::EmptyRows(_) => {}
        }
    }

    let mut gen_layout = GenericLayout::new();

    // cut excess
    if max_label + spacing + max_widget > area.width {
        let mut reduce = max_label + spacing + max_widget - area.width;

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

    let label_x;
    let widget_x;

    match flex {
        Flex::Legacy => {
            label_x = 0;
            widget_x = label_x + spacing + max_label;
        }
        Flex::Start => {
            label_x = 0;
            widget_x = label_x + spacing + max_label;
        }
        Flex::End => {
            widget_x = area.width - max_widget;
            label_x = widget_x - spacing - max_label;
        }
        Flex::Center => {
            let rest = area.width - max_label - max_widget - spacing;
            label_x = rest / 2;
            widget_x = label_x + spacing + max_label;
        }
        Flex::SpaceAround | Flex::SpaceEvenly => {
            let rest = area.width - max_label - max_widget - spacing;
            label_x = rest / 2;
            widget_x = label_x + spacing + max_label;
        }
        Flex::SpaceBetween => {
            let rest = area.width - max_label - max_widget;
            label_x = rest / 3;
            widget_x = label_x + rest / 3 + max_label;
        }
    }
    let total = max_label + spacing + max_widget;

    let mut n = 0;
    let mut y = 0;

    let mut label_area = Rect::default();
    let mut label_text = None;

    let mut rest_height = if area.height > 0 { area.height - 1 } else { 0 };

    for l in constraints.iter() {
        let height;

        // self
        match l {
            EditConstraint::Label(s) => {
                height = 0;
                label_area = Rect::new(label_x, y, max_label, min(1, rest_height));
                label_text = Some(s.clone());
            }
            EditConstraint::LabelWidth(_) => {
                height = 0;
                label_area = Rect::new(label_x, y, max_label, min(1, rest_height));
                label_text = None;
            }
            EditConstraint::LabelRows(_, h) => {
                height = 0;
                label_area = Rect::new(label_x, y, max_label, min(1, min(*h, rest_height)));
                label_text = None;
            }
            EditConstraint::TitleLabel(s) => {
                height = min(1, rest_height);

                label_area = Rect::new(label_x, y, total, height);
                gen_layout.add(n, Rect::default(), Some(s.clone()), label_area);

                n += 1;
                label_area = Rect::default();
                label_text = None;
            }
            EditConstraint::TitleLabelWidth(w) => {
                height = min(1, rest_height);

                label_area = Rect::new(label_x, y, min(*w, max_label), height);
                gen_layout.add(n, Rect::default(), None, label_area);

                n += 1;
                label_area = Rect::default();
                label_text = None;
            }
            EditConstraint::TitleLabelRows(h) => {
                height = min(*h, rest_height);

                label_area = Rect::new(label_x, y, total, height);
                gen_layout.add(n, Rect::default(), None, label_area);

                n += 1;
                label_area = Rect::default();
                label_text = None;
            }
            EditConstraint::Widget(w) => {
                let w_height = min(1, rest_height);
                height = max(label_area.height, w_height);

                let area = Rect::new(widget_x, y, min(*w, max_widget), w_height);
                gen_layout.add(n, area, label_text.take(), label_area);

                n += 1;
                label_area = Rect::default();
            }
            EditConstraint::WidgetRows(w, h) => {
                let w_height = min(*h, rest_height);
                height = max(label_area.height, w_height);

                let area = Rect::new(widget_x, y, min(*w, max_widget), w_height);
                gen_layout.add(n, area, label_text.take(), label_area);

                n += 1;
                label_area = Rect::default();
            }
            EditConstraint::LineWidget(w) => {
                height = min(1, rest_height);

                let area = Rect::new(label_x, y, min(*w, total), height);
                gen_layout.add(n, area, label_text.take(), label_area);

                n += 1;
                label_area = Rect::default();
            }
            EditConstraint::LineWidgetRows(w, h) => {
                height = min(*h, rest_height);

                let area = Rect::new(label_x, y, min(*w, total), height);
                gen_layout.add(n, area, label_text.take(), label_area);

                n += 1;
                label_area = Rect::default();
            }
            EditConstraint::Empty => {
                height = min(1, rest_height);

                label_text = None;
                label_area = Rect::default();
            }
            EditConstraint::EmptyRows(h) => {
                height = min(*h, rest_height);

                label_text = None;
                label_area = Rect::default();
            }
        }

        y += height;
        rest_height -= height;
    }

    gen_layout
}
