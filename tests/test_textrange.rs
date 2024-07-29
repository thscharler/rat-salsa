use rat_widget::text::textarea_core::{TextAreaCore, TextPosition, TextRange};
use std::cmp::Ordering;
#[allow(unused_imports)]
use std::hint::black_box;
#[allow(unused_imports)]
use std::time::Instant;

fn insert(v: &mut Vec<TextRange>, r: TextRange) {
    match v.binary_search(&r) {
        Ok(_) => {}
        Err(idx) => {
            v.insert(idx, r);
        }
    }
}

fn search(v: &Vec<TextRange>, p: (usize, usize)) -> Option<usize> {
    let p = TextPosition::from(p);
    match v.binary_search_by(|v| v.start.cmp(&p)) {
        Ok(i) => Some(i),
        Err(_) => None,
    }
}

fn find(v: &Vec<TextRange>, p: (usize, usize)) -> Option<usize> {
    let p = TextPosition::from(p);
    eprintln!("find {:?}", p);
    match v.binary_search_by(|v| v.start.cmp(&p)) {
        Ok(mut i) => loop {
            eprintln!("i={}", i);
            if i == 0 {
                eprintln!("break 0");
                return Some(i);
            }
            eprintln!("order {:?}", v[i - 1].start.cmp(&p));
            if !v[i - 1].contains_pos(p) {
                eprintln!("break !{:?}.contains({:?})", v[i - 1], p);
                return Some(i);
            }
            i -= 1;
        },
        Err(i) => {
            eprintln!("insert {}", i);
            None
        }
    }
}

#[test]
fn test_insert() {
    let mut v = Vec::new();

    insert(&mut v, TextRange::new((0, 0), (0, 0)));
    insert(&mut v, TextRange::new((0, 0), (0, 0)));
    assert_eq!(v.len(), 1);

    // overlapping
    v.clear();
    insert(&mut v, TextRange::new((1, 1), (2, 2)));
    insert(&mut v, TextRange::new((1, 1), (4, 2)));
    insert(&mut v, TextRange::new((1, 1), (3, 2)));
    assert_eq!(v[0], TextRange::new((1, 1), (2, 2)));
    assert_eq!(v[1], TextRange::new((1, 1), (3, 2)));
    assert_eq!(v[2], TextRange::new((1, 1), (4, 2)));
}

#[test]
fn test_partial_cmp() {
    let r = TextRange::new((1, 0), (10, 0));

    assert_eq!(r.start.cmp(&(0, 0).into()), Ordering::Greater);
    assert_eq!(r.start.cmp(&(1, 0).into()), Ordering::Equal);
    assert_eq!(r.start.cmp(&(2, 0).into()), Ordering::Equal);
    assert_eq!(r.start.cmp(&(5, 0).into()), Ordering::Equal);
    assert_eq!(r.start.cmp(&(9, 0).into()), Ordering::Equal);
    assert_eq!(r.start.cmp(&(10, 0).into()), Ordering::Less);
    assert_eq!(r.start.cmp(&(11, 0).into()), Ordering::Less);

    let r = TextRange::new((9, 0), (11, 0));
    assert_eq!(r.start.cmp(&(8, 0).into()), Ordering::Greater);
    assert_eq!(r.start.cmp(&(9, 0).into()), Ordering::Equal);
    assert_eq!(r.start.cmp(&(10, 0).into()), Ordering::Equal);
    assert_eq!(r.start.cmp(&(11, 0).into()), Ordering::Less);
    assert_eq!(r.start.cmp(&(11, 0).into()), Ordering::Less);

    let r = TextRange::new((10, 0), (20, 0));
    assert_eq!(r.start.cmp(&(9, 0).into()), Ordering::Greater);
    assert_eq!(r.start.cmp(&(10, 0).into()), Ordering::Equal);
    assert_eq!(r.start.cmp(&(11, 0).into()), Ordering::Equal);
    assert_eq!(r.start.cmp(&(19, 0).into()), Ordering::Equal);
    assert_eq!(r.start.cmp(&(20, 0).into()), Ordering::Less);
    assert_eq!(r.start.cmp(&(21, 0).into()), Ordering::Less);
}

#[test]
fn test_search() {
    let mut v = Vec::new();

    insert(&mut v, TextRange::new((1, 0), (10, 0)));
    insert(&mut v, TextRange::new((9, 0), (11, 0)));
    insert(&mut v, TextRange::new((10, 0), (20, 0)));

    assert_eq!(search(&v, (0, 0)), None);
    assert_eq!(search(&v, (1, 0)), Some(0));
    assert_eq!(search(&v, (2, 0)), Some(0));
    assert_eq!(search(&v, (3, 0)), Some(0));
    assert_eq!(search(&v, (4, 0)), Some(0));
    assert_eq!(search(&v, (5, 0)), Some(0));
    assert_eq!(search(&v, (6, 0)), Some(0));
    assert_eq!(search(&v, (7, 0)), Some(0));
    assert_eq!(search(&v, (8, 0)), Some(0));
    assert_eq!(search(&v, (9, 0)), Some(1));
    assert_eq!(search(&v, (10, 0)), Some(1));
    assert_eq!(search(&v, (11, 0)), Some(2));
    assert_eq!(search(&v, (12, 0)), Some(2));
    assert_eq!(search(&v, (13, 0)), Some(2));
    assert_eq!(search(&v, (14, 0)), Some(2));
    assert_eq!(search(&v, (15, 0)), Some(2));
    assert_eq!(search(&v, (16, 0)), Some(2));
    assert_eq!(search(&v, (17, 0)), Some(2));
    assert_eq!(search(&v, (18, 0)), Some(2));
    assert_eq!(search(&v, (19, 0)), Some(2));
    assert_eq!(search(&v, (20, 0)), None);
    assert_eq!(search(&v, (21, 0)), None);
}

#[test]
fn test_find() {
    let mut v = Vec::new();

    insert(&mut v, TextRange::new((1, 0), (10, 0)));
    assert_eq!(find(&v, (0, 0)), None);
    assert_eq!(find(&v, (1, 0)), Some(0));
    assert_eq!(find(&v, (5, 0)), Some(0));
    assert_eq!(find(&v, (9, 0)), Some(0));
    assert_eq!(find(&v, (10, 0)), None);
    assert_eq!(find(&v, (11, 0)), None);

    v.clear();

    insert(&mut v, TextRange::new((1, 0), (10, 0)));
    insert(&mut v, TextRange::new((9, 0), (11, 0)));
    insert(&mut v, TextRange::new((10, 0), (20, 0)));
    assert_eq!(find(&v, (0, 0)), None);
    assert_eq!(find(&v, (1, 0)), Some(0));
    assert_eq!(find(&v, (5, 0)), Some(0));
    assert_eq!(find(&v, (9, 0)), Some(0));
    assert_eq!(find(&v, (10, 0)), Some(1));
    assert_eq!(find(&v, (11, 0)), Some(2));
    assert_eq!(find(&v, (19, 0)), Some(2));
    assert_eq!(find(&v, (20, 0)), None);
    assert_eq!(find(&v, (21, 0)), None);
}

#[test]
fn test_textstyle() {
    let t0 = (TextRange::new((3, 4), (5, 4)), 0);
    let t1 = (TextRange::new((3, 4), (5, 4)), 1);

    assert_eq!(t0 < t1, true);
    assert_eq!(t0 == t1, false);
    assert_eq!(t0 > t1, false);

    assert_eq!(t1 < t1, false);
    assert_eq!(t1 == t1, true);
    assert_eq!(t1 > t1, false);

    assert_eq!(t1 > t0, true);
    assert_eq!(t1 == t0, false);
    assert_eq!(t1 < t0, false);
}

#[test]
fn test_stylemap() {
    let r0 = TextRange::new((30, 7), (42, 7));
    let r1 = TextRange::new((37, 7), (41, 7));
    let r2 = TextRange::new((31, 44), (44, 44));
    assert_eq!(r0.cmp(&r1), Ordering::Less);
    assert_eq!(r0.cmp(&r2), Ordering::Less);
    assert_eq!(r1.cmp(&r2), Ordering::Less);

    let mut txt = TextAreaCore::default();
    txt.add_style(TextRange::new((0, 0), (13, 0)), 0);
    txt.add_style(TextRange::new((0, 1), (13, 1)), 0);
    txt.add_style(TextRange::new((4, 3), (17, 3)), 0);
    txt.add_style(TextRange::new((31, 44), (44, 44)), 0);
    // overlapping styles
    txt.add_style(TextRange::new((30, 7), (42, 7)), 0);
    txt.add_style(TextRange::new((37, 7), (41, 7)), 1);

    let mut buf = Vec::new();
    txt.styles_at((37, 7).into(), &mut buf);
    assert_eq!(buf.len(), 2);
}

#[test]
fn text_expansion() {
    let r = TextRange::new((5, 0), (10, 0));
    assert_eq!(r.expand_pos((4, 0).into()), (4, 0).into());
    assert_eq!(r.expand_pos((5, 0).into()), (10, 0).into());
    assert_eq!(r.expand_pos((6, 0).into()), (11, 0).into());
    assert_eq!(r.expand_pos((10, 0).into()), (15, 0).into());
    assert_eq!(r.expand_pos((11, 0).into()), (16, 0).into());

    let r = TextRange::new((5, 0), (0, 1));
    assert_eq!(r.expand_pos((4, 0).into()), (4, 0).into());
    assert_eq!(r.expand_pos((5, 0).into()), (0, 1).into());
    assert_eq!(r.expand_pos((6, 0).into()), (1, 1).into());
    assert_eq!(r.expand_pos((10, 0).into()), (5, 1).into());
    assert_eq!(r.expand_pos((11, 0).into()), (6, 1).into());
    assert_eq!(r.expand_pos((0, 1).into()), (0, 2).into());
    assert_eq!(r.expand_pos((1, 1).into()), (1, 2).into());

    let r = TextRange::new((5, 0), (3, 1));
    assert_eq!(r.expand_pos((4, 0).into()), (4, 0).into());
    assert_eq!(r.expand_pos((5, 0).into()), (3, 1).into());
    assert_eq!(r.expand_pos((6, 0).into()), (4, 1).into());
    assert_eq!(r.expand_pos((10, 0).into()), (8, 1).into());
    assert_eq!(r.expand_pos((11, 0).into()), (9, 1).into());
    assert_eq!(r.expand_pos((0, 1).into()), (0, 2).into());
    assert_eq!(r.expand_pos((1, 1).into()), (1, 2).into());

    let r = TextRange::new((10, 5), (10, 6));
    assert_eq!(r.expand_pos((10, 7).into()), (10, 8).into());
}

#[test]
fn test_shrinking() {
    let r = TextRange::new((5, 0), (10, 0));
    assert_eq!(r.shrink_pos((4, 0).into()), (4, 0).into());
    assert_eq!(r.shrink_pos((5, 0).into()), (5, 0).into());
    assert_eq!(r.shrink_pos((6, 0).into()), (5, 0).into());
    assert_eq!(r.shrink_pos((10, 0).into()), (5, 0).into());
    assert_eq!(r.shrink_pos((11, 0).into()), (6, 0).into());

    let r = TextRange::new((5, 0), (0, 1));
    assert_eq!(r.shrink_pos((4, 0).into()), (4, 0).into());
    assert_eq!(r.shrink_pos((5, 0).into()), (5, 0).into());
    assert_eq!(r.shrink_pos((6, 0).into()), (5, 0).into());
    assert_eq!(r.shrink_pos((10, 0).into()), (5, 0).into());
    assert_eq!(r.shrink_pos((11, 0).into()), (5, 0).into());
    assert_eq!(r.shrink_pos((0, 1).into()), (5, 0).into());
    assert_eq!(r.shrink_pos((1, 1).into()), (6, 0).into());
    assert_eq!(r.shrink_pos((0, 2).into()), (0, 1).into());
    assert_eq!(r.shrink_pos((1, 2).into()), (1, 1).into());

    let r = TextRange::new((10, 5), (10, 6));
    assert_eq!(r.shrink_pos((10, 7).into()), (10, 6).into());
}

#[test]
fn test_ordering() {
    let r0 = TextRange::new((10, 5), (20, 5));
    let r1 = TextRange::new((10, 6), (20, 6));
    assert!(r0.before(r1));
    assert!(r1.after(r0));

    let r0 = TextRange::new((10, 4), (20, 5));
    let r1 = TextRange::new((30, 5), (20, 6));
    assert!(r0.before(r1));
    assert!(r1.after(r0));

    let r0 = TextRange::new((10, 4), (19, 5));
    let r1 = TextRange::new((20, 5), (20, 6));
    assert!(r0.before(r1));
    assert!(r1.after(r0));

    let r0 = TextRange::new((10, 4), (20, 5));
    let r1 = TextRange::new((20, 5), (20, 6));
    assert!(!r0.before(r1));
    assert!(!r1.after(r0));

    let r0 = TextRange::new((10, 4), (21, 5));
    let r1 = TextRange::new((20, 5), (20, 6));
    assert!(!r0.before(r1));
    assert!(!r1.after(r0));

    //
    let r0 = TextRange::new((10, 5), (20, 5));
    let r1 = TextRange::new((10, 6), (20, 6));
    assert!(!r0.intersects(r1));
    assert!(!r1.intersects(r0));

    let r0 = TextRange::new((10, 4), (20, 5));
    let r1 = TextRange::new((30, 5), (20, 6));
    assert!(!r0.intersects(r1));
    assert!(!r1.intersects(r0));

    let r0 = TextRange::new((10, 4), (19, 5));
    let r1 = TextRange::new((20, 5), (20, 6));
    assert!(!r0.intersects(r1));
    assert!(!r1.intersects(r0));

    let r0 = TextRange::new((10, 4), (20, 5));
    let r1 = TextRange::new((20, 5), (20, 6));
    assert!(r0.intersects(r1));
    assert!(r1.intersects(r0));

    let r0 = TextRange::new((10, 4), (21, 5));
    let r1 = TextRange::new((20, 5), (20, 6));
    assert!(r0.intersects(r1));
    assert!(r1.intersects(r0));
}
