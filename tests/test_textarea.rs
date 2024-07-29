use rat_widget::text::textarea_core::{TextAreaCore, TextRange};

#[test]
fn test_byte_at() {
    let mut core = TextAreaCore::new();
    core.set_value("asdf\njklö\nqwert");

    assert_eq!(core.byte_at((0, 0).into()), Some(0..1));
    assert_eq!(core.byte_at((1, 0).into()), Some(1..2));
    assert_eq!(core.byte_at((2, 0).into()), Some(2..3));
    assert_eq!(core.byte_at((3, 0).into()), Some(3..4));
    assert_eq!(core.byte_at((4, 0).into()), Some(4..5));
    assert_eq!(core.byte_at((5, 0).into()), None);
    assert_eq!(core.byte_at((6, 0).into()), None);
    assert_eq!(core.byte_at((0, 1).into()), Some(5..6));
    assert_eq!(core.byte_at((1, 1).into()), Some(6..7));
    assert_eq!(core.byte_at((2, 1).into()), Some(7..8));
    assert_eq!(core.byte_at((3, 1).into()), Some(8..10));
    assert_eq!(core.byte_at((4, 1).into()), Some(10..11));
    assert_eq!(core.byte_at((5, 1).into()), None);
    assert_eq!(core.byte_at((6, 1).into()), None);
    assert_eq!(core.byte_at((0, 2).into()), Some(11..12));
    assert_eq!(core.byte_at((1, 2).into()), Some(12..13));
    assert_eq!(core.byte_at((2, 2).into()), Some(13..14));
    assert_eq!(core.byte_at((3, 2).into()), Some(14..15));
    assert_eq!(core.byte_at((4, 2).into()), Some(15..16));
    assert_eq!(core.byte_at((5, 2).into()), Some(16..16));
    assert_eq!(core.byte_at((0, 3).into()), Some(16..16));
    assert_eq!(core.byte_at((1, 3).into()), None);

    let mut core = TextAreaCore::new();
    core.set_value("asdf");

    assert_eq!(core.byte_at((0, 0).into()), Some(0..1));
    assert_eq!(core.byte_at((1, 0).into()), Some(1..2));
    assert_eq!(core.byte_at((2, 0).into()), Some(2..3));
    assert_eq!(core.byte_at((3, 0).into()), Some(3..4));
    assert_eq!(core.byte_at((4, 0).into()), Some(4..4));

    let mut core = TextAreaCore::new();
    core.set_value("asdf\n");

    assert_eq!(core.byte_at((0, 0).into()), Some(0..1));
    assert_eq!(core.byte_at((1, 0).into()), Some(1..2));
    assert_eq!(core.byte_at((2, 0).into()), Some(2..3));
    assert_eq!(core.byte_at((3, 0).into()), Some(3..4));
    assert_eq!(core.byte_at((4, 0).into()), Some(4..5));
    // two valid last positions:
    assert_eq!(core.byte_at((5, 0).into()), Some(5..5));
    assert_eq!(core.byte_at((0, 1).into()), Some(5..5));

    let mut core = TextAreaCore::new();
    core.set_value("");

    assert_eq!(core.byte_at((0, 0).into()), Some(0..0));
}

#[test]
fn test_char_at() {
    let mut core = TextAreaCore::new();
    core.set_value("asdf\njklö\nqwert");

    assert_eq!(core.char_at((0, 0).into()), Some(0));
    assert_eq!(core.char_at((1, 0).into()), Some(1));
    assert_eq!(core.char_at((2, 0).into()), Some(2));
    assert_eq!(core.char_at((3, 0).into()), Some(3));
    assert_eq!(core.char_at((4, 0).into()), Some(4));
    assert_eq!(core.char_at((5, 0).into()), None);
    assert_eq!(core.char_at((0, 1).into()), Some(5));
    assert_eq!(core.char_at((1, 1).into()), Some(6));
    assert_eq!(core.char_at((2, 1).into()), Some(7));
    assert_eq!(core.char_at((3, 1).into()), Some(8));
    assert_eq!(core.char_at((4, 1).into()), Some(9));
    assert_eq!(core.char_at((5, 1).into()), None);
    assert_eq!(core.char_at((0, 2).into()), Some(10));
    assert_eq!(core.char_at((1, 2).into()), Some(11));
    assert_eq!(core.char_at((2, 2).into()), Some(12));
    assert_eq!(core.char_at((3, 2).into()), Some(13));
    assert_eq!(core.char_at((4, 2).into()), Some(14));
    assert_eq!(core.char_at((5, 2).into()), Some(15));
    assert_eq!(core.char_at((6, 2).into()), None);
    assert_eq!(core.char_at((0, 3).into()), Some(15));

    let mut core = TextAreaCore::new();
    core.set_value("asdf");

    assert_eq!(core.char_at((0, 0).into()), Some(0));
    assert_eq!(core.char_at((1, 0).into()), Some(1));
    assert_eq!(core.char_at((2, 0).into()), Some(2));
    assert_eq!(core.char_at((3, 0).into()), Some(3));
    assert_eq!(core.char_at((4, 0).into()), Some(4));

    let mut core = TextAreaCore::new();
    core.set_value("asdf\n");

    assert_eq!(core.char_at((0, 0).into()), Some(0));
    assert_eq!(core.char_at((1, 0).into()), Some(1));
    assert_eq!(core.char_at((2, 0).into()), Some(2));
    assert_eq!(core.char_at((3, 0).into()), Some(3));
    assert_eq!(core.char_at((4, 0).into()), Some(4));
    // two valid last positions:
    assert_eq!(core.char_at((5, 0).into()), Some(5));
    assert_eq!(core.char_at((0, 1).into()), Some(5));

    let mut core = TextAreaCore::new();
    core.set_value("");

    assert_eq!(core.char_at((0, 0).into()), Some(0));
}

#[test]
fn test_byte_pos() {
    let mut core = TextAreaCore::new();
    core.set_value("asdf\njklö\nqwert");

    assert_eq!(core.byte_pos(0), Some((0, 0).into()));
    assert_eq!(core.byte_pos(1), Some((1, 0).into()));
    assert_eq!(core.byte_pos(2), Some((2, 0).into()));
    assert_eq!(core.byte_pos(3), Some((3, 0).into()));
    assert_eq!(core.byte_pos(4), Some((4, 0).into()));
    assert_eq!(core.byte_pos(5), Some((0, 1).into()));
    assert_eq!(core.byte_pos(6), Some((1, 1).into()));
    assert_eq!(core.byte_pos(7), Some((2, 1).into()));
    assert_eq!(core.byte_pos(8), Some((3, 1).into()));
    assert_eq!(core.byte_pos(9), Some((4, 1).into()));
    assert_eq!(core.byte_pos(10), Some((4, 1).into()));
    assert_eq!(core.byte_pos(11), Some((0, 2).into()));
    assert_eq!(core.byte_pos(12), Some((1, 2).into()));
    assert_eq!(core.byte_pos(13), Some((2, 2).into()));
    assert_eq!(core.byte_pos(14), Some((3, 2).into()));
    assert_eq!(core.byte_pos(15), Some((4, 2).into()));
    assert_eq!(core.byte_pos(16), Some((5, 2).into()));

    let mut core = TextAreaCore::new();
    core.set_value("asdf");

    assert_eq!(core.byte_pos(0), Some((0, 0).into()));
    assert_eq!(core.byte_pos(1), Some((1, 0).into()));
    assert_eq!(core.byte_pos(2), Some((2, 0).into()));
    assert_eq!(core.byte_pos(3), Some((3, 0).into()));
    assert_eq!(core.byte_pos(4), Some((4, 0).into()));

    let mut core = TextAreaCore::new();
    core.set_value("asdf\n");

    assert_eq!(core.byte_pos(0), Some((0, 0).into()));
    assert_eq!(core.byte_pos(1), Some((1, 0).into()));
    assert_eq!(core.byte_pos(2), Some((2, 0).into()));
    assert_eq!(core.byte_pos(3), Some((3, 0).into()));
    assert_eq!(core.byte_pos(4), Some((4, 0).into()));
    assert_eq!(core.byte_pos(5), Some((0, 1).into()));

    let mut core = TextAreaCore::new();
    core.set_value("");

    assert_eq!(core.byte_pos(0), Some((0, 0).into()));
}

#[test]
fn test_insert_text() {
    let mut v = TextAreaCore::new();
    v.set_value("abcd");
    v.add_style(TextRange::new((2, 0), (3, 0)), 1);
    v.add_style(TextRange::new((1, 0), (3, 0)), 2);
    v.add_style(TextRange::new((1, 0), (2, 0)), 3);
    v.insert_str((2, 0).into(), "x");
    assert_eq!(v.value(), "abxcd");
    assert_eq!(
        v.styles().nth(0).expect("style").0,
        TextRange::new((1, 0), (2, 0))
    );
    assert_eq!(
        v.styles().nth(1).expect("style").0,
        TextRange::new((1, 0), (4, 0))
    );
    assert_eq!(
        v.styles().nth(2).expect("style").0,
        TextRange::new((3, 0), (4, 0))
    );

    let mut v = TextAreaCore::new();
    v.set_value("abcd");
    v.add_style(TextRange::new((2, 0), (3, 0)), 1);
    v.add_style(TextRange::new((1, 0), (3, 0)), 2);
    v.add_style(TextRange::new((1, 0), (2, 0)), 3);
    v.insert_str((2, 0).into(), "\n");
    assert_eq!(v.value(), "ab\ncd");
    assert_eq!(
        v.styles().nth(0).expect("style").0,
        TextRange::new((1, 0), (2, 0))
    );
    assert_eq!(
        v.styles().nth(1).expect("style").0,
        TextRange::new((1, 0), (1, 1))
    );
    assert_eq!(
        v.styles().nth(2).expect("style").0,
        TextRange::new((0, 1), (1, 1))
    );

    let mut v = TextAreaCore::new();
    v.set_value("abcd");
    v.add_style(TextRange::new((2, 0), (3, 0)), 1);
    v.add_style(TextRange::new((1, 0), (3, 0)), 2);
    v.add_style(TextRange::new((1, 0), (2, 0)), 3);
    v.insert_str((2, 0).into(), "\n\n");
    assert_eq!(v.value(), "ab\n\ncd");
    assert_eq!(
        v.styles().nth(0).expect("style").0,
        TextRange::new((1, 0), (2, 0))
    );
    assert_eq!(
        v.styles().nth(1).expect("style").0,
        TextRange::new((1, 0), (1, 2))
    );
    assert_eq!(
        v.styles().nth(2).expect("style").0,
        TextRange::new((0, 2), (1, 2))
    );

    let mut v = TextAreaCore::new();
    v.set_value("abcd");
    v.add_style(TextRange::new((2, 0), (3, 0)), 1);
    v.add_style(TextRange::new((1, 0), (3, 0)), 2);
    v.add_style(TextRange::new((1, 0), (2, 0)), 3);
    v.insert_str((2, 0).into(), "x\ny");
    assert_eq!(v.value(), "abx\nycd");
    assert_eq!(
        v.styles().nth(0).expect("style").0,
        TextRange::new((1, 0), (2, 0))
    );
    assert_eq!(
        v.styles().nth(1).expect("style").0,
        TextRange::new((1, 0), (2, 1))
    );
    assert_eq!(
        v.styles().nth(2).expect("style").0,
        TextRange::new((1, 1), (2, 1))
    );

    let mut v = TextAreaCore::new();
    v.set_value("abcd");
    v.add_style(TextRange::new((2, 0), (3, 0)), 1);
    v.add_style(TextRange::new((1, 0), (3, 0)), 2);
    v.add_style(TextRange::new((1, 0), (2, 0)), 3);
    v.insert_str((2, 0).into(), "xx\nyy\nzz");
    assert_eq!(v.value(), "abxx\nyy\nzzcd");
    assert_eq!(
        v.styles().nth(0).expect("style").0,
        TextRange::new((1, 0), (2, 0))
    );
    assert_eq!(
        v.styles().nth(1).expect("style").0,
        TextRange::new((1, 0), (3, 2))
    );
    assert_eq!(
        v.styles().nth(2).expect("style").0,
        TextRange::new((2, 2), (3, 2))
    );

    dbg!(v);
}
