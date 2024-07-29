use rat_widget::text::textinput_core::TextInputCore;

#[test]
fn test_byte_at() {
    let mut core = TextInputCore::new();
    core.set_value("jklö asdf");

    assert_eq!(core.byte_at(0), Some((0, 1)));
    assert_eq!(core.byte_at(1), Some((1, 2)));
    assert_eq!(core.byte_at(2), Some((2, 3)));
    assert_eq!(core.byte_at(3), Some((3, 5)));
    assert_eq!(core.byte_at(4), Some((5, 6)));
    assert_eq!(core.byte_at(5), Some((6, 7)));
    assert_eq!(core.byte_at(6), Some((7, 8)));
    assert_eq!(core.byte_at(7), Some((8, 9)));
    assert_eq!(core.byte_at(8), Some((9, 10)));
    assert_eq!(core.byte_at(9), Some((10, 10)));
    assert_eq!(core.byte_at(10), None);
}

#[test]
fn test_byte_pos() {
    let mut core = TextInputCore::new();
    core.set_value("jklö asdf");

    assert_eq!(core.byte_pos(0), Some(0));
    assert_eq!(core.byte_pos(1), Some(1));
    assert_eq!(core.byte_pos(2), Some(2));
    assert_eq!(core.byte_pos(3), Some(3));
    assert_eq!(core.byte_pos(4), Some(4));
    assert_eq!(core.byte_pos(5), Some(4));
    assert_eq!(core.byte_pos(6), Some(5));
    assert_eq!(core.byte_pos(7), Some(6));
    assert_eq!(core.byte_pos(8), Some(7));
    assert_eq!(core.byte_pos(9), Some(8));
    assert_eq!(core.byte_pos(10), None);
    assert_eq!(core.byte_pos(11), None);
}

#[test]
fn test_char_at() {
    let mut core = TextInputCore::new();
    core.set_value("jklö 👩🏾‍🏫asdf");

    assert_eq!(core.char_at(0), Some(0));
    assert_eq!(core.char_at(1), Some(1));
    assert_eq!(core.char_at(2), Some(2));
    assert_eq!(core.char_at(3), Some(3));
    assert_eq!(core.char_at(4), Some(4));
    assert_eq!(core.char_at(5), Some(5));
    assert_eq!(core.char_at(6), Some(9));
    assert_eq!(core.char_at(7), Some(10));
    assert_eq!(core.char_at(8), Some(11));
    assert_eq!(core.char_at(9), Some(12));
    assert_eq!(core.char_at(10), Some(13));
    assert_eq!(core.char_at(11), None);
    assert_eq!(core.char_at(12), None);
}

#[test]
fn test_char_pos() {
    let mut core = TextInputCore::new();
    core.set_value("jklö 👩🏾‍🏫asdf");

    assert_eq!(core.char_pos(0), Some(0));
    assert_eq!(core.char_pos(1), Some(1));
    assert_eq!(core.char_pos(2), Some(2));
    assert_eq!(core.char_pos(3), Some(3));
    assert_eq!(core.char_pos(4), Some(4));
    assert_eq!(core.char_pos(5), Some(5));
    assert_eq!(core.char_pos(6), Some(6));
    assert_eq!(core.char_pos(7), Some(6));
    assert_eq!(core.char_pos(8), Some(6));
    assert_eq!(core.char_pos(9), Some(6));
    assert_eq!(core.char_pos(10), Some(7));
    assert_eq!(core.char_pos(11), Some(8));
    assert_eq!(core.char_pos(12), Some(9));
    assert_eq!(core.char_pos(13), Some(10));
    assert_eq!(core.char_pos(14), None);
    assert_eq!(core.char_pos(15), None);
}
