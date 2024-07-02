//!
//! Simple layout for dialogs.
//!
//! Calculates the content-area and the placement of buttons.
//!
use ratatui::layout::{Constraint, Direction, Flex, Layout, Margin, Rect};

/// Layout produced by [layout_dialog].
#[derive(Debug)]
pub struct LayoutDialog<const N: usize> {
    /// Complete area covered by the dialog box.
    pub dialog: Rect,
    /// Area that is not used by borders and buttons.
    pub area: Rect,
    /// Complete button area.
    pub button_area: Rect,
    /// Areas for each button.
    pub buttons: [Rect; N],
}

impl<const N: usize> LayoutDialog<N> {
    /// Area for the buttons.
    pub fn button(&self, n: usize) -> Rect {
        self.buttons[n]
    }
}

/// Calculates a layout for a dialog with buttons.
///
/// width and height give the size of the target area.
/// The target area is centered.
pub fn layout_dialog<const N: usize>(
    area: Rect,
    width: Constraint,
    height: Constraint,
    insets: Margin,
    buttons: [Constraint; N],
    button_spacing: u16,
    button_flex: Flex,
) -> LayoutDialog<N> {
    let l_vertical = Layout::new(Direction::Vertical, [width])
        .flex(Flex::Center)
        .split(area);
    let l_dialog = Layout::new(Direction::Horizontal, [height])
        .flex(Flex::Center)
        .split(l_vertical[0])[0];

    let l_inner = l_dialog.inner(insets);

    let l_content = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(l_inner);

    let l_buttons = Layout::horizontal(buttons)
        .spacing(button_spacing)
        .flex(button_flex)
        .areas(l_content[2]);

    LayoutDialog {
        dialog: l_dialog,
        area: l_content[0],
        button_area: l_content[2],
        buttons: l_buttons,
    }
}
