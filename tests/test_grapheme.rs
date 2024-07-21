use log::debug;
use rat_widget::text::graphemes::{is_word_boundary, RopeGraphemesIdx};
use ropey::{Rope, RopeSlice};
use unicode_segmentation::UnicodeSegmentation;

#[test]
fn test_iter() {
    let r = Rope::from_str("asdµ\njklö");
    let mut it = RopeGraphemesIdx::new(r.slice(..));
    assert_eq!(it.next(), Some(((0, 1), RopeSlice::from("a"))));
    assert_eq!(it.next(), Some(((1, 2), RopeSlice::from("s"))));
    assert_eq!(it.next(), Some(((2, 3), RopeSlice::from("d"))));
    assert_eq!(it.next(), Some(((3, 5), RopeSlice::from("µ"))));
    assert_eq!(it.next(), Some(((5, 6), RopeSlice::from("\n"))));
    assert_eq!(it.next(), Some(((6, 7), RopeSlice::from("j"))));
    assert_eq!(it.next(), Some(((7, 8), RopeSlice::from("k"))));
    assert_eq!(it.next(), Some(((8, 9), RopeSlice::from("l"))));
    assert_eq!(it.next(), Some(((9, 11), RopeSlice::from("ö"))));
}

#[test]
fn test_word_boundary() {
    //            012345678901234567890
    let s = "asdf jklö qwer uiop ";

    for i in 0..30 {
        println!("{}: {}", i, is_word_boundary(s, i));
    }
    assert_eq!(is_word_boundary(s, 0), true);
    assert_eq!(is_word_boundary(s, 1), false);
    assert_eq!(is_word_boundary(s, 2), false);
    assert_eq!(is_word_boundary(s, 3), false);
    assert_eq!(is_word_boundary(s, 4), true);
    assert_eq!(is_word_boundary(s, 5), true);
    assert_eq!(is_word_boundary(s, 6), false);
    assert_eq!(is_word_boundary(s, 7), false);
    assert_eq!(is_word_boundary(s, 8), false);
    assert_eq!(is_word_boundary(s, 9), true);
    assert_eq!(is_word_boundary(s, 10), true);
    assert_eq!(is_word_boundary(s, 11), false);
    assert_eq!(is_word_boundary(s, 12), false);
    assert_eq!(is_word_boundary(s, 13), false);
    assert_eq!(is_word_boundary(s, 14), true);
    assert_eq!(is_word_boundary(s, 15), true);
    assert_eq!(is_word_boundary(s, 16), false);
    assert_eq!(is_word_boundary(s, 17), false);
    assert_eq!(is_word_boundary(s, 18), false);
    assert_eq!(is_word_boundary(s, 19), true);
    assert_eq!(is_word_boundary(s, 20), true);
    assert_eq!(is_word_boundary(s, 21), true);
}
