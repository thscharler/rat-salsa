use rat_widget::layout::{FormLabel, FormWidget, LayoutForm};
use ratatui::layout::{Rect, Size};
use ratatui::widgets::{Block, Padding};

#[test]
fn test_break() {
    let mut layout = LayoutForm::<i32>::new();

    layout.widget(1, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(2, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(3, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(4, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(5, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(6, FormLabel::Width(5), FormWidget::Width(15));

    let g = layout.layout(Size::new(10, 5), Padding::default());

    assert_eq!(g.page_of(&6), Some(1));
}

#[test]
fn test_break2() {
    let mut layout = LayoutForm::<i32>::new();

    layout.widget(1, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(2, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(3, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(4, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(5, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(6, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(7, FormLabel::Width(5), FormWidget::Width(15));

    let g = layout.layout(Size::new(10, 5), Padding::new(0, 0, 1, 1));

    assert_eq!(g.page_of(&4), Some(1));
    assert_eq!(g.page_of(&7), Some(2));
}

#[test]
fn test_break3() {
    let mut layout = LayoutForm::<i32>::new();

    layout.widget(1, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(2, FormLabel::Size(5, 3), FormWidget::Width(15));
    layout.widget(3, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(4, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(5, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(6, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(7, FormLabel::Width(5), FormWidget::Width(15));

    let g = layout.layout(Size::new(10, 5), Padding::new(0, 0, 1, 1));

    assert_eq!(g.page_of(&1), Some(0));
    assert_eq!(g.page_of(&2), Some(1));
    assert_eq!(g.page_of(&3), Some(2));
    assert_eq!(g.page_of(&4), Some(2));
    assert_eq!(g.page_of(&5), Some(2));
    assert_eq!(g.page_of(&6), Some(3));
    assert_eq!(g.page_of(&7), Some(3));
}

#[test]
fn test_break4() {
    let mut layout = LayoutForm::<i32>::new();

    layout.start((), Some(Block::bordered()));
    layout.widget(1, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(2, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(3, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(4, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(5, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(6, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(7, FormLabel::Width(5), FormWidget::Width(15));
    layout.end(());

    let g = layout.layout(Size::new(10, 5), Padding::new(0, 0, 1, 1));
    assert_eq!(g.page_of(&7), Some(6));
    assert_eq!(g.container_areas[6], Rect::new(0, 31, 10, 3));
}

#[test]
fn test_break5() {
    let mut layout = LayoutForm::<i32>::new();

    layout.start((), Some(Block::bordered()));
    layout.start((), Some(Block::bordered()));
    layout.widget(1, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(2, FormLabel::Width(5), FormWidget::Width(15));
    layout.end(());
    layout.widget(3, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(4, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(5, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(6, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(7, FormLabel::Width(5), FormWidget::Width(15));
    layout.end(());

    let g = layout.layout(Size::new(10, 14), Padding::new(0, 0, 1, 1));
    dbg!(&g);
    assert_eq!(g.page_of(&7), Some(6));
    assert_eq!(g.container_areas[6], Rect::new(0, 31, 10, 3));
}
