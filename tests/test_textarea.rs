use rat_widget::textarea::core::InputCore;

#[test]
fn test_byte_at() {
    let mut core = InputCore::new();
    core.set_value("asdf\njklö\nqwert");

    assert_eq!(core.byte_at((0, 0)), Some((0, 1)));
    assert_eq!(core.byte_at((1, 0)), Some((1, 2)));
    assert_eq!(core.byte_at((2, 0)), Some((2, 3)));
    assert_eq!(core.byte_at((3, 0)), Some((3, 4)));
    assert_eq!(core.byte_at((4, 0)), Some((4, 5)));
    assert_eq!(core.byte_at((5, 0)), None);
    assert_eq!(core.byte_at((6, 0)), None);
    assert_eq!(core.byte_at((0, 1)), Some((5, 6)));
    assert_eq!(core.byte_at((1, 1)), Some((6, 7)));
    assert_eq!(core.byte_at((2, 1)), Some((7, 8)));
    assert_eq!(core.byte_at((3, 1)), Some((8, 10)));
    assert_eq!(core.byte_at((4, 1)), Some((10, 11)));
    assert_eq!(core.byte_at((5, 1)), None);
    assert_eq!(core.byte_at((6, 1)), None);
    assert_eq!(core.byte_at((0, 2)), Some((11, 12)));
    assert_eq!(core.byte_at((1, 2)), Some((12, 13)));
    assert_eq!(core.byte_at((2, 2)), Some((13, 14)));
    assert_eq!(core.byte_at((3, 2)), Some((14, 15)));
    assert_eq!(core.byte_at((4, 2)), Some((15, 16)));
    assert_eq!(core.byte_at((5, 2)), Some((16, 16)));
    assert_eq!(core.byte_at((0, 3)), Some((16, 16)));
    assert_eq!(core.byte_at((1, 3)), None);

    let mut core = InputCore::new();
    core.set_value("asdf");

    assert_eq!(core.byte_at((0, 0)), Some((0, 1)));
    assert_eq!(core.byte_at((1, 0)), Some((1, 2)));
    assert_eq!(core.byte_at((2, 0)), Some((2, 3)));
    assert_eq!(core.byte_at((3, 0)), Some((3, 4)));
    assert_eq!(core.byte_at((4, 0)), Some((4, 4)));

    let mut core = InputCore::new();
    core.set_value("asdf\n");

    assert_eq!(core.byte_at((0, 0)), Some((0, 1)));
    assert_eq!(core.byte_at((1, 0)), Some((1, 2)));
    assert_eq!(core.byte_at((2, 0)), Some((2, 3)));
    assert_eq!(core.byte_at((3, 0)), Some((3, 4)));
    assert_eq!(core.byte_at((4, 0)), Some((4, 5)));
    // two valid last positions:
    assert_eq!(core.byte_at((5, 0)), Some((5, 5)));
    assert_eq!(core.byte_at((0, 1)), Some((5, 5)));

    let mut core = InputCore::new();
    core.set_value("");

    assert_eq!(core.byte_at((0, 0)), Some((0, 0)));
}

#[test]
fn test_char_at() {
    let mut core = InputCore::new();
    core.set_value("asdf\njklö\nqwert");

    assert_eq!(core.char_at((0, 0)), Some(0));
    assert_eq!(core.char_at((1, 0)), Some(1));
    assert_eq!(core.char_at((2, 0)), Some(2));
    assert_eq!(core.char_at((3, 0)), Some(3));
    assert_eq!(core.char_at((4, 0)), Some(4));
    assert_eq!(core.char_at((5, 0)), None);
    assert_eq!(core.char_at((0, 1)), Some(5));
    assert_eq!(core.char_at((1, 1)), Some(6));
    assert_eq!(core.char_at((2, 1)), Some(7));
    assert_eq!(core.char_at((3, 1)), Some(8));
    assert_eq!(core.char_at((4, 1)), Some(9));
    assert_eq!(core.char_at((5, 1)), None);
    assert_eq!(core.char_at((0, 2)), Some(10));
    assert_eq!(core.char_at((1, 2)), Some(11));
    assert_eq!(core.char_at((2, 2)), Some(12));
    assert_eq!(core.char_at((3, 2)), Some(13));
    assert_eq!(core.char_at((4, 2)), Some(14));
    assert_eq!(core.char_at((5, 2)), Some(15));
    assert_eq!(core.char_at((6, 2)), None);
    assert_eq!(core.char_at((0, 3)), Some(15));

    let mut core = InputCore::new();
    core.set_value("asdf");

    assert_eq!(core.char_at((0, 0)), Some(0));
    assert_eq!(core.char_at((1, 0)), Some(1));
    assert_eq!(core.char_at((2, 0)), Some(2));
    assert_eq!(core.char_at((3, 0)), Some(3));
    assert_eq!(core.char_at((4, 0)), Some(4));

    let mut core = InputCore::new();
    core.set_value("asdf\n");

    assert_eq!(core.char_at((0, 0)), Some(0));
    assert_eq!(core.char_at((1, 0)), Some(1));
    assert_eq!(core.char_at((2, 0)), Some(2));
    assert_eq!(core.char_at((3, 0)), Some(3));
    assert_eq!(core.char_at((4, 0)), Some(4));
    // two valid last positions:
    assert_eq!(core.char_at((5, 0)), Some(5));
    assert_eq!(core.char_at((0, 1)), Some(5));

    let mut core = InputCore::new();
    core.set_value("");

    assert_eq!(core.char_at((0, 0)), Some(0));
}

#[test]
fn test_byte_pos() {
    let mut core = InputCore::new();
    core.set_value("asdf\njklö\nqwert");

    assert_eq!(core.byte_pos(0), Some((0, 0)));
    assert_eq!(core.byte_pos(1), Some((1, 0)));
    assert_eq!(core.byte_pos(2), Some((2, 0)));
    assert_eq!(core.byte_pos(3), Some((3, 0)));
    assert_eq!(core.byte_pos(4), Some((4, 0)));
    assert_eq!(core.byte_pos(5), Some((0, 1)));
    assert_eq!(core.byte_pos(6), Some((1, 1)));
    assert_eq!(core.byte_pos(7), Some((2, 1)));
    assert_eq!(core.byte_pos(8), Some((3, 1)));
    assert_eq!(core.byte_pos(9), Some((4, 1)));
    assert_eq!(core.byte_pos(10), Some((4, 1)));
    assert_eq!(core.byte_pos(11), Some((0, 2)));
    assert_eq!(core.byte_pos(12), Some((1, 2)));
    assert_eq!(core.byte_pos(13), Some((2, 2)));
    assert_eq!(core.byte_pos(14), Some((3, 2)));
    assert_eq!(core.byte_pos(15), Some((4, 2)));
    assert_eq!(core.byte_pos(16), Some((5, 2)));

    let mut core = InputCore::new();
    core.set_value("asdf");

    assert_eq!(core.byte_pos(0), Some((0, 0)));
    assert_eq!(core.byte_pos(1), Some((1, 0)));
    assert_eq!(core.byte_pos(2), Some((2, 0)));
    assert_eq!(core.byte_pos(3), Some((3, 0)));
    assert_eq!(core.byte_pos(4), Some((4, 0)));

    let mut core = InputCore::new();
    core.set_value("asdf\n");

    assert_eq!(core.byte_pos(0), Some((0, 0)));
    assert_eq!(core.byte_pos(1), Some((1, 0)));
    assert_eq!(core.byte_pos(2), Some((2, 0)));
    assert_eq!(core.byte_pos(3), Some((3, 0)));
    assert_eq!(core.byte_pos(4), Some((4, 0)));
    assert_eq!(core.byte_pos(5), Some((0, 1)));

    let mut core = InputCore::new();
    core.set_value("");

    assert_eq!(core.byte_pos(0), Some((0, 0)));
}
