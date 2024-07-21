//!
//! Simple layout for dialogs.
//!
//! Calculates the content-area and the placement of buttons.
//!
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::Block;

/// Layout produced by [layout_dialog].
#[derive(Debug)]
pub struct LayoutDialog<const N: usize> {
    /// Complete area covered by the dialog box.
    pub area: Rect,
    /// Area inside the border.
    pub inner: Rect,
    /// Area for dialog content, sans borders and buttons.
    pub content: Rect,
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
pub fn layout_dialog<const N: usize>(
    area: Rect,
    block: Option<&Block<'_>>,
    buttons: [Constraint; N],
    button_spacing: u16,
    button_flex: Flex,
) -> LayoutDialog<N> {
    let inner = if let Some(block) = block {
        block.inner(area)
    } else {
        area
    };
    let l_content = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);

    let l_buttons = Layout::horizontal(buttons)
        .spacing(button_spacing)
        .flex(button_flex)
        .areas(l_content[2]);

    LayoutDialog {
        area,
        inner,
        content: l_content[0],
        button_area: l_content[2],
        buttons: l_buttons,
    }
}
