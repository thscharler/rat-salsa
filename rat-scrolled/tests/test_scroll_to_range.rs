use rat_scrolled::ScrollState;

#[test]
fn test_1() {
    let mut s = ScrollState::new();

    s.page_len = 10;

    s.scroll_to_range(0..10);
    assert_eq!(s.offset, 0);
    s.scroll_to_range(1..10);
    assert_eq!(s.offset, 0);
    s.scroll_to_range(2..10);
    assert_eq!(s.offset, 0);

    s.scroll_to_range(0..20);
    assert_eq!(s.offset, 0);

    s.scroll_to_range(9..9);
    assert_eq!(s.offset, 0);
    s.scroll_to_range(9..10);
    assert_eq!(s.offset, 0);
    s.scroll_to_range(9..11);
    assert_eq!(s.offset, 1);

    s.scroll_to_range(10..10);
    assert_eq!(s.offset, 1);
    s.scroll_to_range(10..11);
    assert_eq!(s.offset, 1);
    s.scroll_to_range(10..12);
    assert_eq!(s.offset, 2);
}

#[test]
fn test_2() {
    let mut s = ScrollState::new();

    s.page_len = 0;

    s.scroll_to_range(0..10);
    assert_eq!(s.offset, 0);
    s.scroll_to_range(1..10);
    assert_eq!(s.offset, 1);
    s.scroll_to_range(2..10);
    assert_eq!(s.offset, 2);

    s.scroll_to_range(0..20);
    assert_eq!(s.offset, 0);

    s.scroll_to_range(9..9);
    assert_eq!(s.offset, 9);
    s.scroll_to_range(9..10);
    assert_eq!(s.offset, 9);
    s.scroll_to_range(9..11);
    assert_eq!(s.offset, 9);

    s.scroll_to_range(10..10);
    assert_eq!(s.offset, 10);
    s.scroll_to_range(10..11);
    assert_eq!(s.offset, 10);
    s.scroll_to_range(10..12);
    assert_eq!(s.offset, 10);
}
