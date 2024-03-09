use ratatui::layout::{Constraint, Layout, Margin, Rect};
use std::cmp::max;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ratio {
    pub num: u16,
    pub den: u16,
}

impl Ratio {
    pub fn new(num: u16, den: u16) -> Self {
        Self { num, den }
    }
}

#[macro_export]
macro_rules! ratio {
    ($n:literal / $d:literal) => {
        crate::layout::Ratio { num: $n, den: $d }
    };
}
#[allow(unused_imports)]
pub use ratio;

#[derive(Debug)]
pub enum EditConstraint<'a> {
    Label(&'a str),
    Widget(u16),
}

#[derive(Debug, Default)]
pub struct LayoutEdit {
    pub label: Vec<Rect>,
    pub widget: Vec<Rect>,
}

/// Simple layout with one column of input widgets. But it aligns the labels.
pub fn layout_edit<const N: usize>(area: Rect, constraints: [EditConstraint<'_>; N]) -> LayoutEdit {
    let mut max_width = 0;
    for l in constraints.iter() {
        match l {
            EditConstraint::Label(s) => {
                max_width = max(max_width, s.len() as u16);
            }
            EditConstraint::Widget(_) => {}
        }
    }

    let mut result = LayoutEdit::default();

    let mut x = area.x;
    let mut y = area.y;
    for l in constraints.iter() {
        match l {
            EditConstraint::Label(_) => {
                result.label.push(Rect::new(x, y, max_width, 1));
                x += max_width + 1;
            }
            EditConstraint::Widget(w) => {
                result.widget.push(Rect::new(x, y, *w, 1));
                x = area.x;
                y += 1;
            }
        };
    }

    result.into()
}

#[derive(Debug)]
pub struct LayoutDialog<const N: usize> {
    pub dialog: Rect,
    pub area: Rect,
    pub buttons: [Rect; N],
}

pub fn layout_dialog<const N: usize>(
    area: Rect,
    h_ratio: Ratio,
    v_ratio: Ratio,
    insets: Margin,
    buttons: [u16; N],
) -> LayoutDialog<N> {
    assert!(h_ratio.num <= h_ratio.den);
    assert!(v_ratio.num <= v_ratio.den);

    let dlg_width = area.width * h_ratio.num / h_ratio.den;
    let dlg_height = area.height * v_ratio.num / v_ratio.den;
    let dlg_space_x = (area.width - dlg_width) / 2;
    let dlg_space_y = (area.height - dlg_height) / 2;

    let dlg = Rect::new(
        area.x + dlg_space_x,
        area.y + dlg_space_y,
        dlg_width,
        dlg_height,
    );

    let inner = dlg.inner(&insets);

    let l0 = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(insets.vertical),
        Constraint::Length(1),
    ])
    .split(inner);

    let mut bb = Vec::new();
    bb.push(Constraint::Fill(1));
    for w in buttons.iter() {
        bb.push(Constraint::Length(*w));
    }

    let l1 = Layout::horizontal(bb).spacing(1).split(l0[2]);
    let mut buttons = [Rect::default(); N];
    for i in 0..N {
        buttons[i] = l1[i + 1];
    }

    LayoutDialog {
        dialog: dlg,
        area: l0[0],
        buttons,
    }
}
