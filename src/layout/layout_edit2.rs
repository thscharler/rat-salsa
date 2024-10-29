//!
//! Calculate the layout for an edit-mask with lots of label/widget pairs.
//!

use crate::layout::LabelWidget::{Lbl, Wdg};
use crate::layout::StructuredLayout;
use ratatui::layout::Rect;
use std::cmp::{max, min};
use std::ops::{Index, IndexMut};

/// [layout_edit] returns pairs of areas as `[label, widget]`.
///
/// This type can be used to index into the array.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelWidget {
    Lbl,
    Wdg,
}

impl LabelWidget {
    pub fn size() -> usize {
        2
    }
}

impl Index<LabelWidget> for [Rect] {
    type Output = Rect;

    fn index(&self, index: LabelWidget) -> &Self::Output {
        match index {
            Lbl => &self[0],
            Wdg => &self[1],
        }
    }
}

impl IndexMut<LabelWidget> for [Rect] {
    fn index_mut(&mut self, index: LabelWidget) -> &mut Self::Output {
        match index {
            Lbl => &mut self[0],
            Wdg => &mut self[1],
        }
    }
}

/// Constraint data for [layout_edit]
#[allow(variant_size_differences)]
#[derive(Debug)]
pub enum EditConstraint<'a> {
    /// Label by sample
    Label(&'a str),
    /// Label by width. (cols)
    LabelWidth(u16),
    /// Label by height+width. ( cols, rows).
    LabelRows(u16, u16),
    /// Label occupying the full row.
    TitleLabel,
    /// Label occupying the full row, but rendering only part of it. (cols)
    TitleLabelWidth(u16),
    /// Label occupying multiple full rows. (rows)
    TitleLabelRows(u16),
    /// Widget aligned with the label. (cols)
    Widget(u16),
    /// Widget aligned with the label. (cols, rows)
    WidgetRows(u16, u16),
    /// Empty line. Only increase the line counter.
    Empty,
    /// Empty lines. (rows). Only increase the line counter.
    EmptyRows(u16),
    /// Widget aligned with the left margin. (cols)
    LineWidget(u16),
    /// Widget aligned with the left margin. (cols, rows)
    LineWidgetRows(u16, u16),
    /// Add a page break marker.
    Break,
}

/// Layout for an edit mask with lots of label+widget pairs.
///
/// This neatly aligns labels and widgets in one column.
/// Use the edit constraints to define the layout.
///
/// This returns a [StructuredLayout] with pairs of [label area, widget area]
/// for each index.
#[allow(clippy::comparison_chain)]
pub fn layout_edit(area: Rect, constraints: &[EditConstraint<'_>]) -> StructuredLayout {
    let mut max_label = 0;
    let mut max_widget = 0;
    let mut space = 1;

    // find max
    for l in constraints.iter() {
        match l {
            EditConstraint::Label(s) => max_label = max(max_label, s.len() as u16),
            EditConstraint::LabelWidth(w) => max_label = max(max_label, *w),
            EditConstraint::LabelRows(w, _) => max_label = max(max_label, *w),
            EditConstraint::TitleLabel => {}
            EditConstraint::TitleLabelWidth(_) => {}
            EditConstraint::TitleLabelRows(_) => {}
            EditConstraint::Widget(w) => max_widget = max(max_widget, *w),
            EditConstraint::WidgetRows(w, _) => max_widget = max(max_widget, *w),
            EditConstraint::LineWidget(_) => {}
            EditConstraint::LineWidgetRows(_, _) => {}
            EditConstraint::Empty => {}
            EditConstraint::EmptyRows(_) => {}
            EditConstraint::Break => {}
        }
    }

    let mut ll = StructuredLayout::new(2);

    // area.width is a constraint too
    if max_label + space + max_widget < area.width {
        space = area.width - max_label - max_widget;
    } else if max_label + space + max_widget > area.width {
        let mut reduce = max_label + space + max_widget - area.width;

        if space > reduce {
            space -= reduce;
            reduce = 0;
        } else {
            reduce -= space;
            space = 0;
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

    let mut x = area.x;
    let mut y = area.y;
    let total = max_label + space + max_widget;
    let mut rest_height = if area.height > 0 { area.height - 1 } else { 0 }; //todo: verify the '-1' somehow??
    let mut height = min(1, rest_height);

    let mut cur: [Rect; 2] = Default::default();

    for l in constraints.iter() {
        // break before
        match l {
            EditConstraint::LineWidget(_) | EditConstraint::LineWidgetRows(_, _) => {
                if x != area.x {
                    // result.add(cur);
                    // cur = Default::default();

                    x = area.x;
                    y += height;
                    rest_height -= height;
                    height = min(1, rest_height);
                }
            }
            EditConstraint::TitleLabel
            | EditConstraint::TitleLabelWidth(_)
            | EditConstraint::TitleLabelRows(_) => {
                if x != area.x {
                    ll.add(&cur);
                    cur = Default::default();

                    x = area.x;
                    y += height;
                    rest_height -= height;
                    height = min(1, rest_height);
                }
            }
            EditConstraint::Label(_)
            | EditConstraint::LabelWidth(_)
            | EditConstraint::LabelRows(_, _)
            | EditConstraint::Widget(_)
            | EditConstraint::WidgetRows(_, _)
            | EditConstraint::Empty
            | EditConstraint::EmptyRows(_) => {}
            EditConstraint::Break => {}
        }

        // self
        match l {
            EditConstraint::Label(s) => {
                cur[Lbl] = Rect::new(x, y, min(s.len() as u16, max_label), min(1, rest_height))
            }
            EditConstraint::LabelWidth(w) => {
                cur[Lbl] = Rect::new(x, y, min(*w, max_label), min(1, rest_height))
            }
            EditConstraint::LabelRows(w, h) => {
                cur[Lbl] = Rect::new(x, y, min(*w, max_label), min(1, *h))
            }
            EditConstraint::TitleLabel => {
                cur[Lbl] = Rect::new(x, y, total, min(1, rest_height));
                ll.add(&cur);
                cur = Default::default();
            }
            EditConstraint::TitleLabelWidth(w) => {
                cur[Lbl] = Rect::new(x, y, min(*w, max_label), min(1, rest_height));
                ll.add(&cur);
                cur = Default::default();
            }
            EditConstraint::TitleLabelRows(h) => {
                cur[Lbl] = Rect::new(x, y, total, min(*h, rest_height));
                ll.add(&cur);
                cur = Default::default();
            }
            EditConstraint::Widget(w) => {
                cur[Wdg] = Rect::new(x, y, min(*w, max_widget), min(1, rest_height));
                ll.add(&cur);
                cur = Default::default();
            }
            EditConstraint::WidgetRows(w, h) => {
                cur[Wdg] = Rect::new(x, y, min(*w, max_widget), min(*h, rest_height));
                ll.add(&cur);
                cur = Default::default();
            }
            EditConstraint::LineWidget(w) => {
                cur[Wdg] = Rect::new(x, y, min(*w, max_widget), min(1, rest_height));
                ll.add(&cur);
                cur = Default::default();
            }
            EditConstraint::LineWidgetRows(w, h) => {
                cur[Wdg] = Rect::new(x, y, min(*w, max_widget), min(*h, rest_height));
                ll.add(&cur);
                cur = Default::default();
            }
            EditConstraint::Empty => {}
            EditConstraint::EmptyRows(_) => {}
            EditConstraint::Break => ll.break_before_row(y),
        }

        // row-height
        match l {
            EditConstraint::Label(_)
            | EditConstraint::LabelWidth(_)
            | EditConstraint::TitleLabel
            | EditConstraint::TitleLabelWidth(_)
            | EditConstraint::Widget(_)
            | EditConstraint::Empty
            | EditConstraint::LineWidget(_) => {
                height = min(max(height, 1), rest_height);
            }
            EditConstraint::LabelRows(_, h)
            | EditConstraint::TitleLabelRows(h)
            | EditConstraint::WidgetRows(_, h)
            | EditConstraint::EmptyRows(h)
            | EditConstraint::LineWidgetRows(_, h) => {
                height = min(max(height, *h), rest_height);
            }
            EditConstraint::Break => {}
        }

        // break after
        match l {
            EditConstraint::Label(_)
            | EditConstraint::LabelWidth(_)
            | EditConstraint::LabelRows(_, _) => {
                x += max_label + space;
            }
            EditConstraint::TitleLabel
            | EditConstraint::TitleLabelWidth(_)
            | EditConstraint::TitleLabelRows(_) => {
                x = area.x;
                y += height;
                rest_height -= height;
                height = min(1, rest_height);
            }
            EditConstraint::Widget(_)
            | EditConstraint::WidgetRows(_, _)
            | EditConstraint::Empty
            | EditConstraint::EmptyRows(_)
            | EditConstraint::LineWidget(_)
            | EditConstraint::LineWidgetRows(_, _) => {
                x = area.x;
                y += height;
                rest_height -= height;
                height = min(1, rest_height);
            }
            EditConstraint::Break => {}
        };
    }

    ll
}
