use rat_text::core::{TextRope, TextStore};
use rat_text::Cursor;
use rat_text::{TextError, TextPosition, TextRange};

#[test]
fn test_string1() {
    // positions
    let s = TextRope::new_text("asdfg");

    assert_eq!(s.line_width(0).unwrap(), 5);

    assert_eq!(s.byte_range_at(TextPosition::new(0, 0)).unwrap(), 0..1);
    assert_eq!(s.byte_range_at(TextPosition::new(1, 0)).unwrap(), 1..2);
    assert_eq!(s.byte_range_at(TextPosition::new(2, 0)).unwrap(), 2..3);
    assert_eq!(s.byte_range_at(TextPosition::new(3, 0)).unwrap(), 3..4);
    assert_eq!(s.byte_range_at(TextPosition::new(4, 0)).unwrap(), 4..5);
    assert_eq!(s.byte_range_at(TextPosition::new(5, 0)).unwrap(), 5..5);
    assert_eq!(
        s.byte_range_at(TextPosition::new(6, 0)),
        Err(TextError::ColumnIndexOutOfBounds(6, 5))
    );

    assert_eq!(s.byte_range(TextRange::new((1, 0), (4, 0))).unwrap(), 1..4);

    assert_eq!(s.byte_to_pos(0).unwrap(), TextPosition::new(0, 0));
    assert_eq!(s.byte_to_pos(1).unwrap(), TextPosition::new(1, 0));
    assert_eq!(s.byte_to_pos(2).unwrap(), TextPosition::new(2, 0));
    assert_eq!(s.byte_to_pos(3).unwrap(), TextPosition::new(3, 0));
    assert_eq!(s.byte_to_pos(4).unwrap(), TextPosition::new(4, 0));
    assert_eq!(s.byte_to_pos(5).unwrap(), TextPosition::new(5, 0));
    assert_eq!(s.byte_to_pos(6), Err(TextError::ByteIndexOutOfBounds(6, 5)));

    assert_eq!(
        s.bytes_to_range(1..4).unwrap(),
        TextRange::new((1, 0), (4, 0))
    );
}

#[test]
fn test_string1_1() {
    // positions
    let s = TextRope::new_text("asdfg\nhjklö\r\n");

    assert_eq!(s.line_width(0).unwrap(), 5);
    assert_eq!(s.line_width(1).unwrap(), 5);
    assert_eq!(s.line_width(2).unwrap(), 0);

    assert_eq!(s.rope().len_bytes(), 14);
    assert_eq!(s.rope().len_lines(), 3);

    assert_eq!(s.byte_range_at(TextPosition::new(0, 0)).unwrap(), 0..1);
    assert_eq!(s.byte_range_at(TextPosition::new(5, 0)).unwrap(), 5..6);
    assert_eq!(s.byte_range_at(TextPosition::new(6, 0)).unwrap(), 6..6);
    assert_eq!(
        s.byte_range_at(TextPosition::new(7, 0)),
        Err(TextError::ColumnIndexOutOfBounds(7, 6))
    );
    assert_eq!(s.byte_range_at(TextPosition::new(0, 1)).unwrap(), 6..7);
    assert_eq!(s.byte_range_at(TextPosition::new(1, 1)).unwrap(), 7..8);
    assert_eq!(s.byte_range_at(TextPosition::new(2, 1)).unwrap(), 8..9);
    assert_eq!(s.byte_range_at(TextPosition::new(3, 1)).unwrap(), 9..10);
    assert_eq!(s.byte_range_at(TextPosition::new(4, 1)).unwrap(), 10..12);
    assert_eq!(s.byte_range_at(TextPosition::new(5, 1)).unwrap(), 12..14);
    assert_eq!(s.byte_range_at(TextPosition::new(6, 1)).unwrap(), 14..14);
    assert_eq!(
        s.byte_range_at(TextPosition::new(7, 1)),
        Err(TextError::ColumnIndexOutOfBounds(7, 6))
    );
    assert_eq!(s.byte_range_at(TextPosition::new(0, 2)).unwrap(), 14..14);

    assert_eq!(s.byte_range(TextRange::new((0, 1), (0, 2))).unwrap(), 6..14);
    assert_eq!(s.byte_range(TextRange::new((1, 0), (1, 1))).unwrap(), 1..7);

    assert_eq!(s.byte_to_pos(0).unwrap(), TextPosition::new(0, 0));
    assert_eq!(s.byte_to_pos(1).unwrap(), TextPosition::new(1, 0));
    assert_eq!(s.byte_to_pos(2).unwrap(), TextPosition::new(2, 0));
    assert_eq!(s.byte_to_pos(3).unwrap(), TextPosition::new(3, 0));
    assert_eq!(s.byte_to_pos(4).unwrap(), TextPosition::new(4, 0));
    assert_eq!(s.byte_to_pos(5).unwrap(), TextPosition::new(5, 0));
    assert_eq!(s.byte_to_pos(6).unwrap(), TextPosition::new(0, 1));
    assert_eq!(s.byte_to_pos(7).unwrap(), TextPosition::new(1, 1));
    assert_eq!(s.byte_to_pos(8).unwrap(), TextPosition::new(2, 1));
    assert_eq!(s.byte_to_pos(9).unwrap(), TextPosition::new(3, 1));
    assert_eq!(s.byte_to_pos(10).unwrap(), TextPosition::new(4, 1)); // ö
    assert_eq!(s.byte_to_pos(11).unwrap(), TextPosition::new(4, 1)); // ö
    assert_eq!(s.byte_to_pos(12).unwrap(), TextPosition::new(5, 1)); // \r
    assert_eq!(s.byte_to_pos(13).unwrap(), TextPosition::new(5, 1)); // \n
    assert_eq!(s.byte_to_pos(14).unwrap(), TextPosition::new(0, 2));
    assert_eq!(
        s.byte_to_pos(15),
        Err(TextError::ByteIndexOutOfBounds(15, 14))
    );

    assert_eq!(
        s.bytes_to_range(1..4).unwrap(),
        TextRange::new((1, 0), (4, 0))
    );
}

#[test]
fn test_string1_2() {
    // positions
    let s = TextRope::new_text("aöa");

    assert_eq!(s.byte_range_at(TextPosition::new(0, 0)).unwrap(), 0..1);
    assert_eq!(s.byte_range_at(TextPosition::new(1, 0)).unwrap(), 1..3);
    assert_eq!(s.byte_range_at(TextPosition::new(2, 0)).unwrap(), 3..4);
    assert_eq!(s.byte_range_at(TextPosition::new(3, 0)).unwrap(), 4..4);

    assert_eq!(s.byte_to_pos(0).unwrap(), TextPosition::new(0, 0));
    assert_eq!(s.byte_to_pos(1).unwrap(), TextPosition::new(1, 0));
    assert_eq!(s.byte_to_pos(2).unwrap(), TextPosition::new(1, 0));
    assert_eq!(s.byte_to_pos(3).unwrap(), TextPosition::new(2, 0));
    assert_eq!(s.byte_to_pos(4).unwrap(), TextPosition::new(3, 0));
}

#[test]
fn test_string2() {
    // different status
    let s = TextRope::new_text("asöfg");

    assert_eq!(s.str_slice(TextRange::new((1, 0), (3, 0))).unwrap(), "sö");

    assert_eq!(s.line_at(0).unwrap(), "asöfg");

    assert_eq!(s.line_width(0).unwrap(), 5);
    assert_eq!(s.len_lines(), 1);

    let mut lines = s.lines_at(0).unwrap();
    assert_eq!(lines.next().unwrap(), "asöfg");
    assert_eq!(lines.next(), None);
}

#[test]
fn test_string3() {
    // grapheme
    let s = TextRope::new_text("asöfg");

    let range = s.byte_range(TextRange::new((1, 0), (4, 0))).expect("valid");
    let pos = s.byte_range_at(TextPosition::new(1, 0)).expect("valid");

    let mut g = s.graphemes_byte(range, pos.start).unwrap();
    let gg = g.next().unwrap();
    assert_eq!(gg.text_bytes(), 1..2);
    assert_eq!(gg.grapheme(), "s");
    let gg = g.next().unwrap();
    assert_eq!(gg.text_bytes(), 2..4);
    assert_eq!(gg.grapheme(), "ö");
    let gg = g.next().unwrap();
    assert_eq!(gg.text_bytes(), 4..5);
    assert_eq!(gg.grapheme(), "f");
    assert_eq!(g.next(), None);
}

#[test]
fn test_string3_1() {
    // grapheme
    let s = TextRope::new_text("asöfg");

    let mut g = s.line_graphemes(0).unwrap();
    let gg = g.next().unwrap();
    assert_eq!(gg.text_bytes(), 0..1);
    assert_eq!(gg.grapheme(), "a");
    let gg = g.next().unwrap();
    assert_eq!(gg.text_bytes(), 1..2);
    assert_eq!(gg.grapheme(), "s");
    let gg = g.next().unwrap();
    assert_eq!(gg.text_bytes(), 2..4);
    assert_eq!(gg.grapheme(), "ö");
    let gg = g.next().unwrap();
    assert_eq!(gg.text_bytes(), 4..5);
    assert_eq!(gg.grapheme(), "f");
    let gg = g.next().unwrap();
    assert_eq!(gg.text_bytes(), 5..6);
    assert_eq!(gg.grapheme(), "g");
    assert_eq!(g.next(), None);
}

#[test]
fn test_string3_2() {
    // grapheme iterator at the boundaries.
    let s = TextRope::new_text("asöfg");

    let range = s.byte_range(TextRange::new((1, 0), (4, 0))).expect("valid");
    let pos = s.byte_range_at(TextPosition::new(1, 0)).expect("valid");
    let mut g = s.graphemes_byte(range, pos.start).unwrap();
    assert_eq!(g.prev(), None);

    let range = s.byte_range(TextRange::new((1, 0), (4, 0))).expect("valid");
    let pos = s.byte_range_at(TextPosition::new(4, 0)).expect("valid");
    let mut g = s.graphemes_byte(range, pos.start).unwrap();
    assert_eq!(g.prev().unwrap(), "f");

    let range = s.byte_range(TextRange::new((1, 0), (4, 0))).expect("valid");
    let pos = s.byte_range_at(TextPosition::new(1, 0)).expect("valid");
    let mut g = s.graphemes_byte(range, pos.start).unwrap();
    assert_eq!(g.next().unwrap(), "s");

    let range = s.byte_range(TextRange::new((1, 0), (4, 0))).expect("valid");
    let pos = s.byte_range_at(TextPosition::new(4, 0)).expect("valid");
    let mut g = s.graphemes_byte(range, pos.start).unwrap();
    assert_eq!(g.next(), None);
}

#[test]
fn test_string4() {
    let mut s = TextRope::new_text("asöfg");

    assert_eq!(
        (TextRange::new((0, 0), (1, 0)), 0..1),
        s.insert_char(TextPosition::new(0, 0), 'X').unwrap()
    );
    assert_eq!(
        (TextRange::new((3, 0), (4, 0)), 3..4),
        s.insert_char(TextPosition::new(3, 0), 'X').unwrap()
    );
    assert_eq!(
        (TextRange::new((7, 0), (8, 0)), 8..9),
        s.insert_char(TextPosition::new(7, 0), 'X').unwrap()
    );
    assert_eq!(s.string(), "XasXöfgX");
}

#[test]
fn test_string5() {
    let mut s = TextRope::new_text("asöfg");

    assert_eq!(
        (TextRange::new((0, 0), (2, 0)), 0..2),
        s.insert_str(TextPosition::new(0, 0), "XX").unwrap()
    );
    assert_eq!(
        (TextRange::new((3, 0), (5, 0)), 3..5),
        s.insert_str(TextPosition::new(3, 0), "XX").unwrap()
    );
    assert_eq!(
        (TextRange::new((9, 0), (11, 0)), 10..12),
        s.insert_str(TextPosition::new(9, 0), "XX").unwrap()
    );
    assert_eq!(s.string(), "XXaXXsöfgXX");
}

#[test]
fn test_string6() {
    let mut s = TextRope::new_text("asöfg");
    assert_eq!(
        ("s".to_string(), (TextRange::new((1, 0), (2, 0)), 1..2)),
        s.remove(TextRange::new((1, 0), (2, 0))).unwrap()
    );
    assert_eq!(s.string(), "aöfg");

    let mut s = TextRope::new_text("asöfg");
    assert_eq!(
        ("asöfg".to_string(), (TextRange::new((0, 0), (5, 0)), 0..6)),
        s.remove(TextRange::new((0, 0), (5, 0))).unwrap()
    );
    assert_eq!(s.string(), "");
}

#[test]
fn test_cr() {
    let mut s = TextRope::new_text("asdf");

    assert_eq!(
        s.insert_char(TextPosition::new(2, 0), '\r'),
        Ok((TextRange::new((2, 0), (0, 1)), 2..3))
    );
    assert_eq!(
        s.insert_char(TextPosition::new(3, 0), '\n'),
        Ok((TextRange::new((3, 0), (3, 0)), 3..4))
    );

    let mut s = TextRope::new_text("asdf");

    assert_eq!(
        s.insert_char(TextPosition::new(2, 0), '\n'),
        Ok((TextRange::new((2, 0), (0, 1)), 2..3))
    );
    assert_eq!(
        s.insert_char(TextPosition::new(2, 0), '\r'),
        Ok((TextRange::new((2, 0), (2, 0)), 2..3))
    );

    let mut s = TextRope::new_text("X🙍♀X");
    assert_eq!(
        s.insert_char(TextPosition::new(2, 0), '‍'),
        Ok((TextRange::new((2, 0), (2, 0)), 5..8))
    );
    let mut s = TextRope::new_text("X🙍♀X");
    assert_eq!(
        s.insert_char(TextPosition::new(2, 0), '🏿'),
        Ok((TextRange::new((2, 0), (2, 0)), 5..9))
    );
    let mut s = TextRope::new_text("X🙍♀X");
    assert_eq!(
        s.insert_char(TextPosition::new(2, 0), 'A'),
        Ok((TextRange::new((2, 0), (3, 0)), 5..6))
    );

    let mut s = TextRope::new_text("asdf");
    let mut pos = TextPosition::new(2, 0);
    for c in "\r\n".chars() {
        let (r, _) = s.insert_char(pos, c).expect("fine");
        pos = r.end;
    }
}

#[test]
fn test_cr2() {
    let mut s = TextRope::new_text("asdf");

    assert_eq!(
        s.insert_str(TextPosition::new(2, 0), "\r\n"),
        Ok((TextRange::new((2, 0), (0, 1)), 2..4))
    );
}

#[test]
#[should_panic]
fn test_str_slice_panic_1() {
    let s = TextRope::new_text("1234\r\n");
    s.str_slice(TextRange::new((0, 0), (6, 0))).expect("valid");
}

#[test]
#[should_panic]
fn test_str_slice_panic_2() {
    let s = TextRope::new_text("1234\r\n");
    s.str_slice(TextRange::new((0, 0), (0, 3))).expect("valid");
}

#[test]
fn test_str_slice() {
    let s = TextRope::new_text("1234");
    assert_eq!(
        s.str_slice(TextRange::new((0, 0), (1, 0))).expect("valid"),
        "1"
    );
    assert_eq!(
        s.str_slice(TextRange::new((0, 0), (4, 0))).expect("valid"),
        "1234"
    );
    assert_eq!(
        s.str_slice(TextRange::new((0, 0), (0, 1))).expect("valid"),
        "1234"
    );
    assert_eq!(
        s.str_slice(TextRange::new((0, 1), (0, 1))).expect("valid"),
        ""
    );

    let s = TextRope::new_text("1234\r\n");
    assert_eq!(
        s.str_slice(TextRange::new((0, 0), (1, 0))).expect("valid"),
        "1"
    );
    assert_eq!(
        s.str_slice(TextRange::new((0, 0), (4, 0))).expect("valid"),
        "1234"
    );
    assert_eq!(
        s.str_slice(TextRange::new((0, 0), (5, 0))).expect("valid"),
        "1234\r\n"
    );
    assert_eq!(
        s.str_slice(TextRange::new((0, 0), (0, 1))).expect("valid"),
        "1234\r\n"
    );
    assert_eq!(
        s.str_slice(TextRange::new((0, 1), (0, 1))).expect("valid"),
        ""
    );
    assert_eq!(
        s.str_slice(TextRange::new((0, 1), (0, 2))).expect("valid"),
        ""
    );
}

#[test]
#[should_panic]
fn test_line_at_panic_1() {
    let s = TextRope::new_text("1234");
    assert_eq!(s.line_at(2).expect("valid"), "1234");
}

#[test]
#[should_panic]
fn test_line_at_panic_2() {
    let s = TextRope::new_text("1234\r\n");
    assert_eq!(s.line_at(3).expect("valid"), "1234");
}

#[test]
fn test_line_at() {
    let s = TextRope::new_text("1234");
    assert_eq!(s.line_at(0).expect("valid"), "1234");
    assert_eq!(s.line_at(1).expect("valid"), "");

    let s = TextRope::new_text("1234\r\n");
    assert_eq!(s.line_at(0).expect("valid"), "1234\r\n");
    assert_eq!(s.line_at(1).expect("valid"), "");
    assert_eq!(s.line_at(2).expect("valid"), "");
}

#[test]
fn test_insert_char_0() {
    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_char(TextPosition::new(0, 0), 'x').expect("valid");
    assert_eq!(r, TextRange::new((0, 0), (1, 0)));
    assert_eq!(b, 0..1);

    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_char(TextPosition::new(4, 0), 'x').expect("valid");
    assert_eq!(r, TextRange::new((4, 0), (5, 0)));
    assert_eq!(b, 4..5);

    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_char(TextPosition::new(0, 1), 'x').expect("valid");
    assert_eq!(r, TextRange::new((4, 0), (5, 0)));
    assert_eq!(b, 4..5);

    let mut s = TextRope::new_text("1234");
    assert!(matches!(
        s.insert_char(TextPosition::new(0, 2), 'x'),
        Err(_)
    ));
}

#[test]
fn test_insert_char_1() {
    // multi byte
    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_char(TextPosition::new(0, 0), 'ß').expect("valid");
    assert_eq!(r, TextRange::new((0, 0), (1, 0)));
    assert_eq!(b, 0..2);

    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_char(TextPosition::new(4, 0), 'ß').expect("valid");
    assert_eq!(r, TextRange::new((4, 0), (5, 0)));
    assert_eq!(b, 4..6);

    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_char(TextPosition::new(0, 1), 'ß').expect("valid");
    assert_eq!(r, TextRange::new((4, 0), (5, 0)));
    assert_eq!(b, 4..6);

    let mut s = TextRope::new_text("1234");
    assert!(matches!(
        s.insert_char(TextPosition::new(0, 2), 'ß'),
        Err(_)
    ));
}

#[test]
fn test_insert_char_2() {
    // lf
    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_char(TextPosition::new(0, 0), '\n').expect("valid");
    assert_eq!(r, TextRange::new((0, 0), (0, 1)));
    assert_eq!(b, 0..1);

    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_char(TextPosition::new(4, 0), '\n').expect("valid");
    assert_eq!(r, TextRange::new((4, 0), (0, 1)));
    assert_eq!(b, 4..5);

    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_char(TextPosition::new(0, 1), '\n').expect("valid");
    assert_eq!(r, TextRange::new((4, 0), (0, 1)));
    assert_eq!(b, 4..5);

    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_char(TextPosition::new(0, 0), '\r').expect("valid");
    assert_eq!(r, TextRange::new((0, 0), (0, 1)));
    assert_eq!(b, 0..1);

    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_char(TextPosition::new(4, 0), '\r').expect("valid");
    assert_eq!(r, TextRange::new((4, 0), (0, 1)));
    assert_eq!(b, 4..5);

    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_char(TextPosition::new(0, 1), '\r').expect("valid");
    assert_eq!(r, TextRange::new((4, 0), (0, 1)));
    assert_eq!(b, 4..5);

    let mut s = TextRope::new_text("1234\r");
    let (r, b) = s.insert_char(TextPosition::new(0, 0), '\n').expect("valid");
    assert_eq!(r, TextRange::new((0, 0), (0, 1)));
    assert_eq!(b, 0..1);

    let mut s = TextRope::new_text("1234\r");
    let (r, b) = s.insert_char(TextPosition::new(4, 0), '\n').expect("valid");
    assert_eq!(r, TextRange::new((4, 0), (0, 1)));
    assert_eq!(b, 4..5);

    let mut s = TextRope::new_text("1234\r");
    let (r, b) = s.insert_char(TextPosition::new(5, 0), '\n').expect("valid");
    assert_eq!(r, TextRange::new((0, 1), (0, 1)));
    assert_eq!(b, 5..6);

    let mut s = TextRope::new_text("1234\r");
    let (r, b) = s.insert_char(TextPosition::new(0, 1), '\n').expect("valid");
    assert_eq!(r, TextRange::new((0, 1), (0, 1)));
    assert_eq!(b, 5..6);

    let mut s = TextRope::new_text("1234\r");
    let (r, b) = s.insert_char(TextPosition::new(0, 2), '\n').expect("valid");
    assert_eq!(r, TextRange::new((0, 1), (0, 1)));
    assert_eq!(b, 5..6);

    let mut s = TextRope::new_text("1234\r");
    assert!(matches!(
        s.insert_char(TextPosition::new(0, 3), '\n'),
        Err(_)
    ));
}

#[test]
fn test_insert_str_2() {
    // lf
    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_str(TextPosition::new(0, 0), "\n").expect("valid");
    assert_eq!(r, TextRange::new((0, 0), (0, 1)));
    assert_eq!(b, 0..1);

    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_str(TextPosition::new(4, 0), "\n").expect("valid");
    assert_eq!(r, TextRange::new((4, 0), (0, 1)));
    assert_eq!(b, 4..5);

    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_str(TextPosition::new(0, 1), "\n").expect("valid");
    assert_eq!(r, TextRange::new((4, 0), (0, 1)));
    assert_eq!(b, 4..5);

    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_str(TextPosition::new(0, 0), "\r").expect("valid");
    assert_eq!(r, TextRange::new((0, 0), (0, 1)));
    assert_eq!(b, 0..1);

    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_str(TextPosition::new(4, 0), "\r").expect("valid");
    assert_eq!(r, TextRange::new((4, 0), (0, 1)));
    assert_eq!(b, 4..5);

    let mut s = TextRope::new_text("1234");
    let (r, b) = s.insert_str(TextPosition::new(0, 1), "\r").expect("valid");
    assert_eq!(r, TextRange::new((4, 0), (0, 1)));
    assert_eq!(b, 4..5);

    let mut s = TextRope::new_text("1234\r");
    let (r, b) = s.insert_str(TextPosition::new(0, 0), "\n").expect("valid");
    assert_eq!(r, TextRange::new((0, 0), (0, 1)));
    assert_eq!(b, 0..1);

    let mut s = TextRope::new_text("1234\r");
    let (r, b) = s.insert_str(TextPosition::new(4, 0), "\n").expect("valid");
    assert_eq!(r, TextRange::new((4, 0), (0, 1)));
    assert_eq!(b, 4..5);

    let mut s = TextRope::new_text("1234\r");
    let (r, b) = s.insert_str(TextPosition::new(5, 0), "\n").expect("valid");
    assert_eq!(r, TextRange::new((5, 0), (0, 1)));
    assert_eq!(b, 5..6);

    let mut s = TextRope::new_text("1234\r");
    let (r, b) = s.insert_str(TextPosition::new(0, 1), "\n").expect("valid");
    assert_eq!(r, TextRange::new((0, 1), (0, 2)));
    assert_eq!(b, 5..6);

    let mut s = TextRope::new_text("1234\r");
    let (r, b) = s.insert_str(TextPosition::new(0, 2), "\n").expect("valid");
    assert_eq!(r, TextRange::new((0, 1), (0, 2)));
    assert_eq!(b, 5..6);

    let mut s = TextRope::new_text("1234\r");
    assert!(matches!(
        s.insert_str(TextPosition::new(0, 3), "\n"),
        Err(_)
    ));
}

#[test]
fn test_remove_1() {
    let mut s = TextRope::new_text("1234");
    assert_eq!(
        s.remove(TextRange::new((0, 0), (0, 0))).expect("fine"),
        (String::from(""), (TextRange::new((0, 0), (0, 0)), 0..0))
    );

    let mut s = TextRope::new_text("1234");
    assert_eq!(
        s.remove(TextRange::new((3, 0), (4, 0))).expect("fine"),
        (String::from("4"), (TextRange::new((3, 0), (4, 0)), 3..4))
    );

    let mut s = TextRope::new_text("1234");
    assert_eq!(
        s.remove(TextRange::new((4, 0), (0, 1))).expect("fine"),
        (String::from(""), (TextRange::new((4, 0), (4, 0)), 4..4))
    );

    let mut s = TextRope::new_text("1234");
    assert_eq!(
        s.remove(TextRange::new((0, 1), (0, 1))).expect("fine"),
        (String::from(""), (TextRange::new((4, 0), (4, 0)), 4..4))
    );
}

#[test]
fn test_remove_2() {
    let mut s = TextRope::new_text("1234\n");
    assert_eq!(
        s.remove(TextRange::new((0, 0), (0, 0))).expect("fine"),
        (String::from(""), (TextRange::new((0, 0), (0, 0)), 0..0))
    );

    let mut s = TextRope::new_text("1234\n");
    assert_eq!(
        s.remove(TextRange::new((3, 0), (4, 0))).expect("fine"),
        (String::from("4"), (TextRange::new((3, 0), (4, 0)), 3..4))
    );

    let mut s = TextRope::new_text("1234\n");
    assert_eq!(
        s.remove(TextRange::new((4, 0), (0, 1))).expect("fine"),
        (String::from("\n"), (TextRange::new((4, 0), (0, 1)), 4..5))
    );

    let mut s = TextRope::new_text("1234\n");
    assert_eq!(
        s.remove(TextRange::new((0, 1), (0, 1))).expect("fine"),
        (String::from(""), (TextRange::new((0, 1), (0, 1)), 5..5))
    );

    let mut s = TextRope::new_text("1234\n");
    assert_eq!(
        s.remove(TextRange::new((0, 2), (0, 2))).expect("fine"),
        (String::from(""), (TextRange::new((0, 1), (0, 1)), 5..5))
    );
}

#[test]
fn test_remove_3() {
    let mut s = TextRope::new_text("1234\r");
    assert_eq!(
        s.remove(TextRange::new((0, 0), (0, 0))).expect("fine"),
        (String::from(""), (TextRange::new((0, 0), (0, 0)), 0..0))
    );

    let mut s = TextRope::new_text("1234\r");
    assert_eq!(
        s.remove(TextRange::new((3, 0), (4, 0))).expect("fine"),
        (String::from("4"), (TextRange::new((3, 0), (4, 0)), 3..4))
    );

    let mut s = TextRope::new_text("1234\r");
    assert_eq!(
        s.remove(TextRange::new((4, 0), (0, 1))).expect("fine"),
        (String::from("\r"), (TextRange::new((4, 0), (0, 1)), 4..5))
    );

    let mut s = TextRope::new_text("1234\r");
    assert_eq!(
        s.remove(TextRange::new((0, 1), (0, 1))).expect("fine"),
        (String::from(""), (TextRange::new((0, 1), (0, 1)), 5..5))
    );

    let mut s = TextRope::new_text("1234\r");
    assert_eq!(
        s.remove(TextRange::new((0, 2), (0, 2))).expect("fine"),
        (String::from(""), (TextRange::new((0, 1), (0, 1)), 5..5))
    );
}

#[test]
fn test_remove_4() {
    let mut s = TextRope::new_text("1234\r\n");
    assert_eq!(
        s.remove(TextRange::new((0, 0), (0, 0))).expect("fine"),
        (String::from(""), (TextRange::new((0, 0), (0, 0)), 0..0))
    );

    let mut s = TextRope::new_text("1234\r\n");
    assert_eq!(
        s.remove(TextRange::new((3, 0), (4, 0))).expect("fine"),
        (String::from("4"), (TextRange::new((3, 0), (4, 0)), 3..4))
    );

    let mut s = TextRope::new_text("1234\r\n");
    assert_eq!(
        s.remove(TextRange::new((4, 0), (0, 1))).expect("fine"),
        (String::from("\r\n"), (TextRange::new((4, 0), (0, 1)), 4..6))
    );

    let mut s = TextRope::new_text("1234\r\n");
    assert_eq!(
        s.remove(TextRange::new((5, 0), (0, 1))).expect("fine"),
        (String::from(""), (TextRange::new((5, 0), (0, 1)), 6..6))
    );

    let mut s = TextRope::new_text("1234\r\n");
    assert_eq!(
        s.remove(TextRange::new((0, 1), (0, 1))).expect("fine"),
        (String::from(""), (TextRange::new((0, 1), (0, 1)), 6..6))
    );

    let mut s = TextRope::new_text("1234\r\n");
    assert_eq!(
        s.remove(TextRange::new((0, 2), (0, 2))).expect("fine"),
        (String::from(""), (TextRange::new((0, 1), (0, 1)), 6..6))
    );
}

#[test]
fn test_len_lines() {
    let s = TextRope::new_text("");
    assert_eq!(s.len_lines(), 1);

    let s = TextRope::new_text("abcde");
    assert_eq!(s.len_lines(), 1);

    let s = TextRope::new_text("\n");
    assert_eq!(s.len_lines(), 2);
    assert_eq!(
        s.byte_range_at(TextPosition::new(0, 2))
            .expect("valid")
            .start,
        1
    );

    let s = TextRope::new_text("abcde\n");
    assert_eq!(s.len_lines(), 2);
    assert_eq!(
        s.byte_range_at(TextPosition::new(0, 2))
            .expect("valid")
            .start,
        6
    );

    let s = TextRope::new_text("abcde\nfghij");
    assert_eq!(s.len_lines(), 2);
    assert_eq!(
        s.byte_range_at(TextPosition::new(0, 1))
            .expect("valid")
            .start,
        6
    );
    assert_eq!(
        s.byte_range_at(TextPosition::new(5, 1))
            .expect("valid")
            .start,
        11
    );
    assert_eq!(
        s.byte_range_at(TextPosition::new(0, 2))
            .expect("valid")
            .start,
        11
    );
}
