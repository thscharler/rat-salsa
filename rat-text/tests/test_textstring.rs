use rat_text::core::{TextStore, TextString};
use rat_text::{TextError, TextPosition, TextRange};

#[test]
fn test_string1() {
    // positions
    let s = TextString::new_text("asdfg");

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
fn test_string1_2() {
    // positions
    let s = TextString::new_text("aöa");

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
    let s = TextString::new_text("asöfg");

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
    let s = TextString::new_text("asöfg");

    let r = s.byte_range(TextRange::new((1, 0), (4, 0))).expect("valid");
    let p = s.byte_range_at(TextPosition::new(1, 0)).expect("valid");

    let mut g = s.graphemes_byte(r, p.start).unwrap();
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
fn test_string4() {
    let mut s = TextString::new_text("asöfg");

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
    let mut s = TextString::new_text("asöfg");

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
    let mut s = TextString::new_text("asöfg");
    assert_eq!(
        ("s".to_string(), (TextRange::new((1, 0), (2, 0)), 1..2)),
        s.remove(TextRange::new((1, 0), (2, 0))).unwrap()
    );
    assert_eq!(s.string(), "aöfg");

    let mut s = TextString::new_text("asöfg");
    assert_eq!(
        ("asöfg".to_string(), (TextRange::new((0, 0), (5, 0)), 0..6)),
        s.remove(TextRange::new((0, 0), (5, 0))).unwrap()
    );
    assert_eq!(s.string(), "");
}
