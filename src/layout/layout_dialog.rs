use crate::layout::GenericLayout;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::BlockExt;
use ratatui::widgets::Block;

#[derive(Debug, PartialEq, Eq)]
pub enum DialogItem {
    /// Area inside the border.
    Inner,
    /// Area where dialog content can be rendered.
    Content,
    /// Area for the nth button.
    Button(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DialogAreas {
    /// Dialog area
    Dialog,
    /// Button area
    Buttons,
}

/// Calculates a layout for a dialog with buttons.
pub fn layout_dialog<const N: usize>(
    area: Rect,
    block: Option<Block<'static>>,
    buttons: [Constraint; N],
    button_spacing: u16,
    button_flex: Flex,
) -> GenericLayout<DialogItem, DialogAreas> {
    let inner = block.inner_if_some(area);

    let l_content = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);

    let l_buttons = Layout::horizontal(buttons)
        .spacing(button_spacing)
        .flex(button_flex)
        .areas::<N>(l_content[2]);

    let mut gen_layout = GenericLayout::new();
    gen_layout.set_area(area);

    gen_layout.add(DialogItem::Inner, inner, None, Rect::default());
    gen_layout.add(DialogItem::Content, l_content[0], None, Rect::default());
    for (n, area) in l_buttons.iter().enumerate() {
        gen_layout.add(DialogItem::Button(n), *area, None, Rect::default());
    }
    gen_layout.add_container(DialogAreas::Dialog, area, block);
    gen_layout.add_container(DialogAreas::Buttons, l_content[2], None);

    gen_layout
}
