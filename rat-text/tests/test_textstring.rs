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

// TODO

#[test]
fn test_byte_range_at() {
    let s = TextString::new_text("");
    assert_eq!(s.byte_range_at((0, 0).into()), Ok(0..0));
    assert!(matches!(s.byte_range_at((1, 0).into()), Err(_)));
    assert_eq!(s.byte_range_at((0, 1).into()), Ok(0..0));
    assert!(matches!(s.byte_range_at((0, 2).into()), Err(_)));

    let s = TextString::new_text("abcd");
    assert_eq!(s.byte_range_at((0, 0).into()), Ok(0..1));
    assert_eq!(s.byte_range_at((1, 0).into()), Ok(1..2));
    assert_eq!(s.byte_range_at((2, 0).into()), Ok(2..3));
    assert_eq!(s.byte_range_at((3, 0).into()), Ok(3..4));
    assert_eq!(s.byte_range_at((4, 0).into()), Ok(4..4));
    assert_eq!(s.byte_range_at((0, 1).into()), Ok(4..4));
    assert!(matches!(s.byte_range_at((0, 2).into()), Err(_)));
    let s = TextString::new_text("abcd\n");
    assert_eq!(s.byte_range_at((0, 0).into()), Ok(0..1));
    assert_eq!(s.byte_range_at((1, 0).into()), Ok(1..2));
    assert_eq!(s.byte_range_at((2, 0).into()), Ok(2..3));
    assert_eq!(s.byte_range_at((3, 0).into()), Ok(3..4));
    assert_eq!(s.byte_range_at((4, 0).into()), Ok(4..5));
    assert_eq!(s.byte_range_at((5, 0).into()), Ok(5..5));
    assert_eq!(s.byte_range_at((0, 1).into()), Ok(5..5));
    assert!(matches!(s.byte_range_at((0, 2).into()), Err(_)));
}

#[test]
fn test_byte_range() {
    let s = TextString::new_text("");
    assert_eq!(s.byte_range(TextRange::new((0, 0), (0, 0))), Ok(0..0));
    assert!(matches!(
        s.byte_range(TextRange::new((1, 0), (1, 0))),
        Err(_)
    ));
    assert!(matches!(
        s.byte_range(TextRange::new((0, 0), (1, 0))),
        Err(_)
    ));
    assert_eq!(s.byte_range(TextRange::new((0, 1), (0, 1))), Ok(0..0));
    assert!(matches!(
        s.byte_range(TextRange::new((0, 1), (0, 2))),
        Err(_)
    ));

    let s = TextString::new_text("abcd");
    assert_eq!(s.byte_range(TextRange::new((0, 0), (1, 0))), Ok(0..1));
    assert_eq!(s.byte_range(TextRange::new((1, 0), (3, 0))), Ok(1..3));
    assert_eq!(s.byte_range(TextRange::new((1, 0), (4, 0))), Ok(1..4));
    assert!(matches!(
        s.byte_range(TextRange::new((1, 0), (5, 0))),
        Err(_)
    ));
    assert_eq!(s.byte_range(TextRange::new((1, 0), (0, 1))), Ok(1..4));
    assert!(matches!(
        s.byte_range(TextRange::new((0, 1), (0, 2))),
        Err(_)
    ));
    let s = TextString::new_text("abcd\n");
    assert_eq!(s.byte_range(TextRange::new((0, 0), (1, 0))), Ok(0..1));
    assert_eq!(s.byte_range(TextRange::new((1, 0), (3, 0))), Ok(1..3));
    assert_eq!(s.byte_range(TextRange::new((1, 0), (4, 0))), Ok(1..4));
    assert_eq!(s.byte_range(TextRange::new((1, 0), (5, 0))), Ok(1..5));
    assert!(matches!(
        s.byte_range(TextRange::new((1, 0), (6, 0))),
        Err(_)
    ));
    assert_eq!(s.byte_range(TextRange::new((1, 0), (0, 1))), Ok(1..5));
    assert!(matches!(
        s.byte_range(TextRange::new((0, 1), (0, 2))),
        Err(_)
    ));
}

#[test]
fn test_str_slice() {
    let s = TextString::new_text("abcd");
    assert_eq!(s.str_slice(((0, 0)..(1, 0)).into()).expect("valid"), "a");
    assert_eq!(s.str_slice(((0, 0)..(4, 0)).into()).expect("valid"), "abcd");
    assert_eq!(s.str_slice(((0, 0)..(0, 1)).into()).expect("valid"), "abcd");
    assert_eq!(s.str_slice(((0, 1)..(0, 1)).into()).expect("valid"), "");

    let s = TextString::new_text("abcd\r\n");
    assert_eq!(s.str_slice(((0, 0)..(1, 0)).into()).expect("valid"), "a");
    assert_eq!(s.str_slice(((0, 0)..(4, 0)).into()).expect("valid"), "abcd");
    assert_eq!(
        s.str_slice(((0, 0)..(5, 0)).into()).expect("valid"),
        "abcd\r\n"
    );
    assert!(matches!(s.str_slice(((0, 0)..(6, 0)).into()), Err(_)));
    assert_eq!(
        s.str_slice(((0, 0)..(0, 1)).into()).expect("valid"),
        "abcd\r\n"
    );
    assert_eq!(s.str_slice(((0, 1)..(0, 1)).into()).expect("valid"), "");
    assert!(matches!(s.str_slice(((0, 0)..(0, 2)).into()), Err(_)));
    assert!(matches!(s.str_slice(((0, 0)..(0, 3)).into()), Err(_)));
}

#[test]
fn test_line_at() {
    let s = TextString::new_text("");
    assert_eq!(s.line_at(0).expect("valid"), "");
    assert!(matches!(s.line_at(1), Ok(_)));
    assert!(matches!(s.line_at(2), Err(_)));

    let s = TextString::new_text("abcd");
    assert_eq!(s.line_at(0).expect("valid"), "abcd");
    assert_eq!(s.line_at(1).expect("valid"), "");
    assert!(matches!(s.line_at(2), Err(_)));
    let s = TextString::new_text("abcd\n");
    assert_eq!(s.line_at(0).expect("valid"), "abcd\n");
    assert_eq!(s.line_at(1).expect("valid"), "");
    assert!(matches!(s.line_at(2), Err(_)));
    assert!(matches!(s.line_at(3), Err(_)));
    let s = TextString::new_text("abcd\r");
    assert_eq!(s.line_at(0).expect("valid"), "abcd\r");
    assert_eq!(s.line_at(1).expect("valid"), "");
    assert!(matches!(s.line_at(2), Err(_)));
    assert!(matches!(s.line_at(3), Err(_)));
    let s = TextString::new_text("abcd\r\n");
    assert_eq!(s.line_at(0).expect("valid"), "abcd\r\n");
    assert_eq!(s.line_at(1).expect("valid"), "");
    assert!(matches!(s.line_at(2), Err(_)));
    assert!(matches!(s.line_at(3), Err(_)));
}

#[test]
fn test_lines_at() {
    let s = TextString::new_text("");
    assert!(matches!(s.lines_at(0), Ok(_)));
    assert!(matches!(s.lines_at(1), Ok(_)));
    assert!(matches!(s.lines_at(2), Err(_)));

    let s = TextString::new_text("abcd");
    assert!(matches!(s.lines_at(0), Ok(_)));
    assert!(matches!(s.lines_at(1), Ok(_)));
    assert!(matches!(s.lines_at(2), Err(_)));
    let s = TextString::new_text("abcd\n");
    assert!(matches!(s.lines_at(0), Ok(_)));
    assert!(matches!(s.line_at(1), Ok(_)));
    assert!(matches!(s.line_at(2), Err(_)));
    assert!(matches!(s.line_at(3), Err(_)));
    let s = TextString::new_text("abcd\r");
    assert!(matches!(s.lines_at(0), Ok(_)));
    assert!(matches!(s.line_at(1), Ok(_)));
    assert!(matches!(s.line_at(2), Err(_)));
    assert!(matches!(s.line_at(3), Err(_)));
    let s = TextString::new_text("abcd\r\n");
    assert!(matches!(s.lines_at(0), Ok(_)));
    assert!(matches!(s.line_at(1), Ok(_)));
    assert!(matches!(s.line_at(2), Err(_)));
    assert!(matches!(s.line_at(3), Err(_)));

    let s = TextString::new_text("1234\nabcd");
    assert!(matches!(s.lines_at(0), Ok(_)));
    assert!(matches!(s.line_at(1), Ok(_)));
    assert!(matches!(s.line_at(2), Err(_)));
    assert!(matches!(s.line_at(3), Err(_)));
    let s = TextString::new_text("1234\nabcd\n");
    assert!(matches!(s.lines_at(0), Ok(_)));
    assert!(matches!(s.line_at(1), Ok(_)));
    assert!(matches!(s.line_at(2), Err(_)));
    assert!(matches!(s.line_at(3), Err(_)));
}

#[test]
fn test_line_grapheme_0() {
    let s = TextString::new_text("abcd");
    assert!(matches!(s.line_graphemes(0), Ok(_)));
    assert!(matches!(s.line_graphemes(1), Ok(_)));
    assert!(matches!(s.line_graphemes(2), Err(_)));
    let s = TextString::new_text("abcd\n");
    assert!(matches!(s.line_graphemes(0), Ok(_)));
    assert!(matches!(s.line_graphemes(1), Ok(_)));
    assert!(matches!(s.line_graphemes(2), Err(_)));
    let s = TextString::new_text("abcd\r");
    assert!(matches!(s.line_graphemes(0), Ok(_)));
    assert!(matches!(s.line_graphemes(1), Ok(_)));
    assert!(matches!(s.line_graphemes(2), Err(_)));
    let s = TextString::new_text("abcd\r\n");
    assert!(matches!(s.line_graphemes(0), Ok(_)));
    assert!(matches!(s.line_graphemes(1), Ok(_)));
    assert!(matches!(s.line_graphemes(2), Err(_)));

    let s = TextString::new_text("abcd\na");
    assert!(matches!(s.line_graphemes(1), Ok(_)));
    assert!(matches!(s.line_graphemes(2), Err(_)));
    assert!(matches!(s.line_graphemes(3), Err(_)));
    let s = TextString::new_text("abcd\ra");
    assert!(matches!(s.line_graphemes(1), Ok(_)));
    assert!(matches!(s.line_graphemes(2), Err(_)));
    assert!(matches!(s.line_graphemes(3), Err(_)));
    let s = TextString::new_text("abcd\r\na");
    assert!(matches!(s.line_graphemes(1), Ok(_)));
    assert!(matches!(s.line_graphemes(2), Err(_)));
    assert!(matches!(s.line_graphemes(3), Err(_)));

    let s = TextString::new_text("abcd\na");
    assert!(matches!(s.line_graphemes(1), Ok(_)));
    assert!(matches!(s.line_graphemes(2), Err(_)));
    assert!(matches!(s.line_graphemes(3), Err(_)));
    let s = TextString::new_text("abcd\na\n");
    assert!(matches!(s.line_graphemes(1), Ok(_)));
    assert!(matches!(s.line_graphemes(2), Err(_)));
    assert!(matches!(s.line_graphemes(3), Err(_)));
}

#[test]
fn test_line_width() {
    let s = TextString::new_text("");
    assert_eq!(s.line_width(0), Ok(0));
    assert!(matches!(s.line_width(1), Ok(0)));
    assert!(matches!(s.line_width(2), Err(_)));

    let s = TextString::new_text("abcd");
    assert_eq!(s.line_width(0), Ok(4));
    assert_eq!(s.line_width(1), Ok(0));
    assert!(matches!(s.line_width(2), Err(_)));
    let s = TextString::new_text("abcd\n");
    assert_eq!(s.line_width(0), Ok(5));
    assert_eq!(s.line_width(1), Ok(0));
    assert!(matches!(s.line_width(2), Err(_)));
    let s = TextString::new_text("abcd\r");
    assert_eq!(s.line_width(0), Ok(5));
    assert_eq!(s.line_width(1), Ok(0));
    assert!(matches!(s.line_width(2), Err(_)));
    let s = TextString::new_text("abcd\r\n");
    assert_eq!(s.line_width(0), Ok(5));
    assert_eq!(s.line_width(1), Ok(0));
    assert!(matches!(s.line_width(2), Err(_)));

    let s = TextString::new_text("abcd\nefghi");
    assert_eq!(s.line_width(0), Ok(10));
    assert_eq!(s.line_width(1), Ok(0));
    assert!(matches!(s.line_width(2), Err(_)));
    assert!(matches!(s.line_width(3), Err(_)));
    let s = TextString::new_text("abcd\nefghi\n");
    assert_eq!(s.line_width(0), Ok(11));
    assert_eq!(s.line_width(1), Ok(0));
    assert!(matches!(s.line_width(2), Err(_)));
    assert!(matches!(s.line_width(3), Err(_)));
}

#[test]
fn test_final_newline() {
    let s = TextString::new_text("");
    assert!(s.has_final_newline());

    let s = TextString::new_text("abcd");
    assert!(s.has_final_newline());
    let s = TextString::new_text("abcd\n");
    assert!(s.has_final_newline());
    let s = TextString::new_text("abcd\r");
    assert!(s.has_final_newline());
    let s = TextString::new_text("abcd\r\n");
    assert!(s.has_final_newline());
}

// TODO: ---

#[test]
fn test_insert_char_0() {
    let s = TextString::new_text("");
    let (r, b) = s.clone().insert_char((0, 0).into(), 'x').expect("valid");
    assert_eq!(r, ((0, 0)..(1, 0)).into());
    assert_eq!(b, 0..1);

    let s = TextString::new_text("1234");
    let (r, b) = s.clone().insert_char((0, 0).into(), 'x').expect("valid");
    assert_eq!(r, ((0, 0)..(1, 0)).into());
    assert_eq!(b, 0..1);
    let (r, b) = s.clone().insert_char((4, 0).into(), 'x').expect("valid");
    assert_eq!(r, ((4, 0)..(5, 0)).into());
    assert_eq!(b, 4..5);
    assert_eq!(s.len_lines(), 1);
    let (r, b) = s.clone().insert_char((0, 1).into(), 'x').expect("valid");
    assert_eq!(r, ((4, 0)..(5, 0)).into());
    assert_eq!(b, 4..5);
    assert!(matches!(s.clone().insert_char((0, 2).into(), 'x'), Err(_)));

    let s = TextString::new_text("1234\n");
    let (r, b) = s.clone().insert_char((0, 1).into(), 'x').expect("valid");
    assert_eq!(r, ((5, 0)..(6, 0)).into());
    assert_eq!(b, 5..6);
}

#[test]
fn test_insert_char_1() {
    // multi byte
    let s = TextString::new_text("1234");
    let (r, b) = s.clone().insert_char((0, 0).into(), 'ß').expect("valid");
    assert_eq!(r, ((0, 0)..(1, 0)).into());
    assert_eq!(b, 0..2);
    let (r, b) = s.clone().insert_char((4, 0).into(), 'ß').expect("valid");
    assert_eq!(r, ((4, 0)..(5, 0)).into());
    assert_eq!(b, 4..6);
    let (r, b) = s.clone().insert_char((0, 1).into(), 'ß').expect("valid");
    assert_eq!(r, ((4, 0)..(5, 0)).into());
    assert_eq!(b, 4..6);
    assert!(matches!(s.clone().insert_char((0, 2).into(), 'ß'), Err(_)));
}

#[test]
fn test_insert_char_2() {
    // lf
    let s = TextString::new_text("1234");
    let (r, b) = s.clone().insert_char((0, 0).into(), '\n').expect("valid");
    assert_eq!(r, ((0, 0)..(1, 0)).into());
    assert_eq!(b, 0..1);
    let (r, b) = s.clone().insert_char((4, 0).into(), '\n').expect("valid");
    assert_eq!(r, ((4, 0)..(5, 0)).into());
    assert_eq!(b, 4..5);
    let (r, b) = s.clone().insert_char((0, 1).into(), '\n').expect("valid");
    assert_eq!(r, ((4, 0)..(5, 0)).into());
    assert_eq!(b, 4..5);

    let s = TextString::new_text("1234\rabcd");
    let (r, b) = s.clone().insert_char((0, 0).into(), '\n').expect("valid");
    assert_eq!(r, ((0, 0)..(1, 0)).into());
    assert_eq!(b, 0..1);
    let (r, b) = s.clone().insert_char((4, 0).into(), '\n').expect("valid");
    assert_eq!(s.len_lines(), 1);
    assert_eq!(r, ((4, 0)..(5, 0)).into());
    assert_eq!(b, 4..5);
    let (r, b) = s.clone().insert_char((5, 0).into(), '\n').expect("valid");
    assert_eq!(s.len_lines(), 1);
    assert_eq!(r, ((5, 0)..(5, 0)).into());
    assert_eq!(b, 5..6);
    let (r, b) = s.clone().insert_char((0, 1).into(), '\n').expect("valid");
    assert_eq!(r, ((9, 0)..(10, 0)).into());
    assert_eq!(b, 9..10);
    assert!(matches!(s.clone().insert_char((0, 2).into(), '\n'), Err(_)));
    assert!(matches!(s.clone().insert_char((0, 3).into(), '\n'), Err(_)));
    assert!(matches!(s.clone().insert_char((0, 4).into(), '\n'), Err(_)));
}

#[test]
fn test_insert_char_3() {
    let s = TextString::new_text("1234");
    let (r, b) = s.clone().insert_char((0, 0).into(), '\r').expect("valid");
    assert_eq!(r, ((0, 0)..(1, 0)).into());
    assert_eq!(b, 0..1);
    let (r, b) = s.clone().insert_char((4, 0).into(), '\r').expect("valid");
    assert_eq!(r, ((4, 0)..(5, 0)).into());
    assert_eq!(b, 4..5);
    let (r, b) = s.clone().insert_char((0, 1).into(), '\r').expect("valid");
    assert_eq!(r, ((4, 0)..(5, 0)).into());
    assert_eq!(b, 4..5);
}

#[test]
fn test_insert_str_2() {
    // lf
    let s = TextString::new_text("1234");
    let (r, b) = s.clone().insert_str((0, 0).into(), "\n").expect("valid");
    assert_eq!(r, ((0, 0)..(1, 0)).into());
    assert_eq!(b, 0..1);
    let (r, b) = s.clone().insert_str((4, 0).into(), "\n").expect("valid");
    assert_eq!(r, ((4, 0)..(5, 0)).into());
    assert_eq!(b, 4..5);
    let (r, b) = s.clone().insert_str((0, 1).into(), "\n").expect("valid");
    assert_eq!(r, ((4, 0)..(5, 0)).into());
    assert_eq!(b, 4..5);
    let (r, b) = s.clone().insert_str((0, 0).into(), "\r").expect("valid");
    assert_eq!(r, ((0, 0)..(1, 0)).into());
    assert_eq!(b, 0..1);
    let (r, b) = s.clone().insert_str((4, 0).into(), "\r").expect("valid");
    assert_eq!(r, ((4, 0)..(5, 0)).into());
    assert_eq!(b, 4..5);
    let (r, b) = s.clone().insert_str((0, 1).into(), "\r").expect("valid");
    assert_eq!(r, ((4, 0)..(5, 0)).into());
    assert_eq!(b, 4..5);

    let s = TextString::new_text("1234\r");

    let (r, b) = s.clone().insert_str((0, 0).into(), "\n").expect("valid");
    assert_eq!(r, ((0, 0)..(1, 0)).into());
    assert_eq!(b, 0..1);
    let (r, b) = s.clone().insert_str((4, 0).into(), "\n").expect("valid");
    assert_eq!(r, ((4, 0)..(5, 0)).into());
    assert_eq!(b, 4..5);
    let (r, b) = s.clone().insert_str((5, 0).into(), "\n").expect("valid");
    assert_eq!(r, ((5, 0)..(5, 0)).into());
    assert_eq!(b, 5..6);
    let (r, b) = s.clone().insert_str((0, 1).into(), "\n").expect("valid");
    assert_eq!(r, ((5, 0)..(5, 0)).into());
    assert_eq!(b, 5..6);
    assert!(matches!(s.clone().insert_str((0, 2).into(), "\n"), Err(_)));
    assert!(matches!(s.clone().insert_str((0, 3).into(), "\n"), Err(_)));
}

#[test]
fn test_remove_1() {
    let s = TextString::new_text("1234");
    assert_eq!(
        s.clone().remove(((0, 0)..(0, 0)).into()).expect("fine"),
        (String::from(""), (((0, 0)..(0, 0)).into(), 0..0))
    );
    assert_eq!(
        s.clone().remove(((3, 0)..(4, 0)).into()).expect("fine"),
        (String::from("4"), (((3, 0)..(4, 0)).into(), 3..4))
    );
    assert_eq!(
        s.clone().remove(((4, 0)..(0, 1)).into()).expect("fine"),
        (String::from(""), (((4, 0)..(4, 0)).into(), 4..4))
    );
    assert_eq!(
        s.clone().remove(((0, 1)..(0, 1)).into()).expect("fine"),
        (String::from(""), (((4, 0)..(4, 0)).into(), 4..4))
    );
}

#[test]
fn test_remove_2() {
    let s = TextString::new_text("1234\n");
    assert_eq!(
        s.clone().remove(((0, 0)..(0, 0)).into()).expect("fine"),
        (String::from(""), (((0, 0)..(0, 0)).into(), 0..0))
    );
    assert_eq!(
        s.clone().remove(((3, 0)..(4, 0)).into()).expect("fine"),
        (String::from("4"), (((3, 0)..(4, 0)).into(), 3..4))
    );
    assert_eq!(
        s.clone().remove(((4, 0)..(0, 1)).into()).expect("fine"),
        (String::from("\n"), (((4, 0)..(5, 0)).into(), 4..5))
    );
    assert_eq!(
        s.clone().remove(((0, 1)..(0, 1)).into()).expect("fine"),
        (String::from(""), (((5, 0)..(5, 0)).into(), 5..5))
    );
    assert!(matches!(s.clone().remove(((0, 2)..(0, 2)).into()), Err(_)));
}

#[test]
fn test_remove_3() {
    let s = TextString::new_text("1234\r");
    assert_eq!(
        s.clone().remove(((0, 0)..(0, 0)).into()).expect("fine"),
        (String::from(""), (((0, 0)..(0, 0)).into(), 0..0))
    );
    assert_eq!(
        s.clone().remove(((3, 0)..(4, 0)).into()).expect("fine"),
        (String::from("4"), (((3, 0)..(4, 0)).into(), 3..4))
    );
    assert_eq!(
        s.clone().remove(((4, 0)..(0, 1)).into()).expect("fine"),
        (String::from("\r"), (((4, 0)..(5, 0)).into(), 4..5))
    );
    assert_eq!(
        s.clone().remove(((0, 1)..(0, 1)).into()).expect("fine"),
        (String::from(""), (((5, 0)..(5, 0)).into(), 5..5))
    );
    assert!(matches!(s.clone().remove(((0, 2)..(0, 2)).into()), Err(_)));
}

#[test]
fn test_remove_4() {
    let mut s = TextString::new_text("1234\r\n");
    assert_eq!(
        s.remove(((0, 0)..(0, 0)).into()).expect("fine"),
        (String::from(""), (((0, 0)..(0, 0)).into(), 0..0))
    );

    let mut s = TextString::new_text("1234\r\n");
    assert_eq!(
        s.remove(((3, 0)..(4, 0)).into()).expect("fine"),
        (String::from("4"), (((3, 0)..(4, 0)).into(), 3..4))
    );

    let mut s = TextString::new_text("1234\r\n");
    assert_eq!(
        s.remove(((4, 0)..(0, 1)).into()).expect("fine"),
        (String::from("\r\n"), (((4, 0)..(5, 0)).into(), 4..6))
    );

    let mut s = TextString::new_text("1234\r\n");
    assert_eq!(
        s.remove(((5, 0)..(0, 1)).into()).expect("fine"),
        (String::from(""), (((5, 0)..(5, 0)).into(), 6..6))
    );

    let mut s = TextString::new_text("1234\r\n");
    assert_eq!(
        s.remove(((0, 1)..(0, 1)).into()).expect("fine"),
        (String::from(""), (((5, 0)..(5, 0)).into(), 6..6))
    );

    let mut s = TextString::new_text("1234\r\n");
    assert!(matches!(s.remove(((0, 2)..(0, 2)).into()), Err(_)));
}

#[test]
fn test_len_lines() {
    let s = TextString::new_text("");
    assert_eq!(s.len_lines(), 1);
    let s = TextString::new_text("\n");
    assert_eq!(s.len_lines(), 1);
    let s = TextString::new_text("\n\n");
    assert_eq!(s.len_lines(), 1);

    let s = TextString::new_text("abcde");
    assert_eq!(s.len_lines(), 1);
    let s = TextString::new_text("abcde\n");
    assert_eq!(s.len_lines(), 1);
    let s = TextString::new_text("abcde\r");
    assert_eq!(s.len_lines(), 1);
    let s = TextString::new_text("abcde\r\n");
    assert_eq!(s.len_lines(), 1);

    let s = TextString::new_text("abcde\nfghij");
    assert_eq!(s.len_lines(), 1);
    let s = TextString::new_text("abcde\nfghij\n");
    assert_eq!(s.len_lines(), 1);

    let s = TextString::new_text("abcde\nfghij\nklmno");
    assert_eq!(s.len_lines(), 1);
    let s = TextString::new_text("abcde\nfghij\nklmno\n");
    assert_eq!(s.len_lines(), 1);
}
