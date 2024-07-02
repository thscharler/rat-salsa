use rat_widget::textarea::graphemes::RopeGraphemesIdx;
use ropey::{Rope, RopeSlice};

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
