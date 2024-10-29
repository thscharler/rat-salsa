//!
//! Simple layout for dialogs.
//!
//! Calculates the content-area and the placement of buttons.
//!

use crate::layout::StructuredLayout;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::Block;
use std::ops::{Index, IndexMut};

/// Index type for the items returned by [layout_dialog].
///
/// __See__
/// [layout_dialog] returns a StructuredLayout that can be indexed
/// with this.
#[derive(Debug)]
pub enum DialogItem {
    /// Area inside the border.
    Inner,
    /// Area where dialog content can be rendered.
    Content,
    /// All buttons area.
    Buttons,
    /// Area for the nth button.
    Button(usize),
}

impl Index<DialogItem> for StructuredLayout {
    type Output = Rect;

    fn index(&self, index: DialogItem) -> &Self::Output {
        match index {
            DialogItem::Inner => &self.as_slice()[0],
            DialogItem::Content => &self.as_slice()[1],
            DialogItem::Buttons => &self.as_slice()[2],
            DialogItem::Button(n) => &self.as_slice()[n + 3],
        }
    }
}

impl IndexMut<DialogItem> for StructuredLayout {
    fn index_mut(&mut self, index: DialogItem) -> &mut Self::Output {
        match index {
            DialogItem::Inner => &mut self.as_mut_slice()[0],
            DialogItem::Content => &mut self.as_mut_slice()[1],
            DialogItem::Buttons => &mut self.as_mut_slice()[2],
            DialogItem::Button(n) => &mut self.as_mut_slice()[n + 3],
        }
    }
}

/// Calculates a layout for a dialog with buttons.
///
/// Access the items via the index operator, using DialogItem as index.
///
/// ```
/// # use rat_widget::layout::{DialogItem, StructuredLayout};
/// # let l = StructuredLayout::default();
///
/// l[DialogItem::Content];
/// l[DialogItem::Button(0)];
///
/// ```
///
pub fn layout_dialog<const N: usize>(
    area: Rect,
    block: Option<&Block<'_>>,
    buttons: [Constraint; N],
    button_spacing: u16,
    button_flex: Flex,
) -> StructuredLayout {
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
        .areas::<N>(l_content[2]);

    let mut ll = StructuredLayout::new(1);
    ll.set_area(area);
    ll.add(&[inner]);
    ll.add(&[l_content[0]]);
    ll.add(&[l_content[2]]);
    for a in l_buttons {
        ll.add(&[a]);
    }
    ll
}

#[cfg(test)]
#[test]
fn test_dialog() {
    let ll = layout_dialog(
        Rect::new(0, 0, 40, 20),
        None,
        [Constraint::Length(4), Constraint::Length(4)],
        1,
        Flex::End,
    );

    use DialogItem::*;

    assert_eq!(ll[Inner], Rect::new(0, 0, 40, 20));
    assert_eq!(ll[Content], Rect::new(0, 0, 40, 18));
    assert_eq!(ll[Buttons], Rect::new(0, 19, 40, 1));
    assert_eq!(ll[Button(0)], Rect::new(31, 19, 4, 1));
    assert_eq!(ll[Button(1)], Rect::new(36, 19, 4, 1));
}
