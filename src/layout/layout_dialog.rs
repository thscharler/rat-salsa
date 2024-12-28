use crate::layout::GenericLayout;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::Padding;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DialogItem {
    /// Area inside the border.
    Inner,
    /// Area where dialog content can be rendered.
    Content,
    /// Button area
    Buttons,
    /// Area for the nth button.
    Button(usize),
}

/// Calculates a layout for a dialog with buttons.
pub fn layout_dialog<const N: usize>(
    area: Rect,
    padding: Padding,
    buttons: [Constraint; N],
    button_spacing: u16,
    button_flex: Flex,
) -> GenericLayout<DialogItem> {
    let inner = Rect::new(
        area.x + padding.left,
        area.y + padding.top,
        area.width.saturating_sub(padding.left + padding.right),
        area.height.saturating_sub(padding.top + padding.bottom),
    );

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
    gen_layout.set_page_size(area.as_size());
    gen_layout.set_page_count(1);

    gen_layout.add(DialogItem::Inner, inner, None, Rect::default());
    gen_layout.add(DialogItem::Content, l_content[0], None, Rect::default());
    gen_layout.add(DialogItem::Buttons, l_content[2], None, Rect::default());
    for (n, area) in l_buttons.iter().enumerate() {
        gen_layout.add(DialogItem::Button(n), *area, None, Rect::default());
    }

    gen_layout
}
