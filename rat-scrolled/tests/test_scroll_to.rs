use rat_scrolled::ScrollState;

#[test]
pub fn test_1() {
    let mut st = ScrollState::new();

    st.page_len = 10;
    st.max_offset = 1000;

    st.scroll_to_pos(0);
    assert_eq!(st.offset, 0);

    st.scroll_to_pos(1);
    assert_eq!(st.offset, 0);
    st.scroll_to_pos(5);
    assert_eq!(st.offset, 0);
    st.scroll_to_pos(9);
    assert_eq!(st.offset, 0);
    st.scroll_to_pos(10);
    assert_eq!(st.offset, 1);
    st.scroll_to_pos(11);
    assert_eq!(st.offset, 2);
}

#[test]
pub fn test_2() {
    let mut st = ScrollState::new();

    st.page_len = 0;
    st.max_offset = 1000;

    st.scroll_to_pos(0);
    assert_eq!(st.offset, 0);

    st.scroll_to_pos(1);
    assert_eq!(st.offset, 1);
    st.scroll_to_pos(5);
    assert_eq!(st.offset, 5);
    st.scroll_to_pos(9);
    assert_eq!(st.offset, 9);
    st.scroll_to_pos(10);
    assert_eq!(st.offset, 10);
    st.scroll_to_pos(11);
    assert_eq!(st.offset, 11);
}

#[test]
pub fn test_3() {
    let mut st = ScrollState::new();

    st.page_len = 1;
    st.max_offset = 1000;

    st.scroll_to_pos(0);
    assert_eq!(st.offset, 0);

    st.scroll_to_pos(1);
    assert_eq!(st.offset, 1);
    st.scroll_to_pos(5);
    assert_eq!(st.offset, 5);
    st.scroll_to_pos(9);
    assert_eq!(st.offset, 9);
    st.scroll_to_pos(10);
    assert_eq!(st.offset, 10);
    st.scroll_to_pos(11);
    assert_eq!(st.offset, 11);
}
