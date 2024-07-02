use rat_widget::textarea::core::{InputCore, TextRange};
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
    match v.binary_search_by(|v| v.ordering(p)) {
        Ok(i) => Some(i),
        Err(_) => None,
    }
}

fn find(v: &Vec<TextRange>, p: (usize, usize)) -> Option<usize> {
    // eprintln!("find {:?}", p);
    match v.binary_search_by(|v| v.ordering(p)) {
        Ok(mut i) => loop {
            // eprintln!("i={}", i);
            if i == 0 {
                // eprintln!("break 0");
                return Some(i);
            }
            // eprintln!("order {:?}", v[i - 1].ordering(p));
            if !v[i - 1].contains(p) {
                // eprintln!("break !{:?}.contains({:?})", v[i - 1], p);
                return Some(i);
            }
            i -= 1;
        },
        Err(_) => None,
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

    assert_eq!(r.ordering((0, 0)), Ordering::Greater);
    assert_eq!(r.ordering((1, 0)), Ordering::Equal);
    assert_eq!(r.ordering((2, 0)), Ordering::Equal);
    assert_eq!(r.ordering((5, 0)), Ordering::Equal);
    assert_eq!(r.ordering((9, 0)), Ordering::Equal);
    assert_eq!(r.ordering((10, 0)), Ordering::Less);
    assert_eq!(r.ordering((11, 0)), Ordering::Less);

    let r = TextRange::new((9, 0), (11, 0));
    assert_eq!(r.ordering((8, 0)), Ordering::Greater);
    assert_eq!(r.ordering((9, 0)), Ordering::Equal);
    assert_eq!(r.ordering((10, 0)), Ordering::Equal);
    assert_eq!(r.ordering((11, 0)), Ordering::Less);
    assert_eq!(r.ordering((11, 0)), Ordering::Less);

    let r = TextRange::new((10, 0), (20, 0));
    assert_eq!(r.ordering((9, 0)), Ordering::Greater);
    assert_eq!(r.ordering((10, 0)), Ordering::Equal);
    assert_eq!(r.ordering((11, 0)), Ordering::Equal);
    assert_eq!(r.ordering((19, 0)), Ordering::Equal);
    assert_eq!(r.ordering((20, 0)), Ordering::Less);
    assert_eq!(r.ordering((21, 0)), Ordering::Less);
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

    let mut txt = InputCore::default();
    txt.add_style(TextRange::new((0, 0), (13, 0)), 0);
    txt.add_style(TextRange::new((0, 1), (13, 1)), 0);
    txt.add_style(TextRange::new((4, 3), (17, 3)), 0);
    txt.add_style(TextRange::new((31, 44), (44, 44)), 0);
    // overlapping styles
    txt.add_style(TextRange::new((30, 7), (42, 7)), 0);
    txt.add_style(TextRange::new((37, 7), (41, 7)), 1);

    let mut r = Vec::new();
    txt.styles_at((37, 7), &mut r);
    assert_eq!(r.len(), 2);
}

#[test]
fn text_expansion() {
    let r = TextRange::new((5, 0), (10, 0));
    assert_eq!(r.expand((4, 0)), (4, 0));
    assert_eq!(r.expand((5, 0)), (10, 0));
    assert_eq!(r.expand((6, 0)), (11, 0));
    assert_eq!(r.expand((10, 0)), (15, 0));
    assert_eq!(r.expand((11, 0)), (16, 0));

    let r = TextRange::new((5, 0), (0, 1));
    assert_eq!(r.expand((4, 0)), (4, 0));
    assert_eq!(r.expand((5, 0)), (0, 1));
    assert_eq!(r.expand((6, 0)), (1, 1));
    assert_eq!(r.expand((10, 0)), (5, 1));
    assert_eq!(r.expand((11, 0)), (6, 1));
    assert_eq!(r.expand((0, 1)), (0, 2));
    assert_eq!(r.expand((1, 1)), (1, 2));

    let r = TextRange::new((5, 0), (3, 1));
    assert_eq!(r.expand((4, 0)), (4, 0));
    assert_eq!(r.expand((5, 0)), (3, 1));
    assert_eq!(r.expand((6, 0)), (4, 1));
    assert_eq!(r.expand((10, 0)), (8, 1));
    assert_eq!(r.expand((11, 0)), (9, 1));
    assert_eq!(r.expand((0, 1)), (0, 2));
    assert_eq!(r.expand((1, 1)), (1, 2));

    let r = TextRange::new((10, 5), (10, 6));
    assert_eq!(r.expand((10, 7)), (10, 8));
}

#[test]
fn test_shrinking() {
    let r = TextRange::new((5, 0), (10, 0));
    assert_eq!(r.shrink((4, 0)), (4, 0));
    assert_eq!(r.shrink((5, 0)), (5, 0));
    assert_eq!(r.shrink((6, 0)), (5, 0));
    assert_eq!(r.shrink((10, 0)), (5, 0));
    assert_eq!(r.shrink((11, 0)), (6, 0));

    let r = TextRange::new((5, 0), (0, 1));
    assert_eq!(r.shrink((4, 0)), (4, 0));
    assert_eq!(r.shrink((5, 0)), (5, 0));
    assert_eq!(r.shrink((6, 0)), (5, 0));
    assert_eq!(r.shrink((10, 0)), (5, 0));
    assert_eq!(r.shrink((11, 0)), (5, 0));
    assert_eq!(r.shrink((0, 1)), (5, 0));
    assert_eq!(r.shrink((1, 1)), (6, 0));
    assert_eq!(r.shrink((0, 2)), (0, 1));
    assert_eq!(r.shrink((1, 2)), (1, 1));

    let r = TextRange::new((10, 5), (10, 6));
    assert_eq!(r.shrink((10, 7)), (10, 6));
}

#[test]
fn test_ordering() {
    // let r = TextRange::new((5, 0), (0, 1));
    //
    // let t = Instant::now();
    // for _ in 0..1000000 {
    //     black_box(r.ordering_inclusive((6, 8)));
    // }
    // eprintln!("ordering_inclusive {:?}", t.elapsed());
    //
    // let t = Instant::now();
    // for _ in 0..1000000 {
    //     black_box(r.ordering((6, 8)));
    // }
    // eprintln!("ordering {:?}", t.elapsed());
    //
    // let r = TextRange::new((5, 0), (0, 1));
    // let t = Instant::now();
    // for _ in 0..1000000 {
    //     black_box((r.start.1, r.start.0).cmp(&(8, 6)));
    // }
    // eprintln!("tuple cmp {:?}", t.elapsed());
}
