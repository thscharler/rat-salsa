use rat_event::util::{column_at, column_at_drag, row_at, row_at_drag};
use ratatui::layout::Rect;

#[test]
fn test_rows0() {
    let area = Rect::new(0, 0, 0, 4);
    let rows = vec![
        Rect::new(0, 0, 0, 1),
        Rect::new(0, 1, 0, 1),
        Rect::new(0, 2, 0, 1),
        Rect::new(0, 3, 0, 1),
    ];

    assert_eq!(row_at(&rows, 0), Some(0));
    assert_eq!(row_at(&rows, 1), Some(1));
    assert_eq!(row_at(&rows, 2), Some(2));
    assert_eq!(row_at(&rows, 3), Some(3));
    assert_eq!(row_at(&rows, 4), None);
    assert_eq!(row_at(&rows, 16384), None);

    assert_eq!(row_at_drag(area, &rows, 0), Ok(0));
    assert_eq!(row_at_drag(area, &rows, 1), Ok(1));
    assert_eq!(row_at_drag(area, &rows, 2), Ok(2));
    assert_eq!(row_at_drag(area, &rows, 3), Ok(3));
    assert_eq!(row_at_drag(area, &rows, 4), Err(1));
    assert_eq!(row_at_drag(area, &rows, 5), Err(2));
    assert_eq!(row_at_drag(area, &rows, 16384), Err(16381));
}

#[test]
fn test_rows1() {
    let area = Rect::new(0, 10, 0, 4);
    let rows = vec![
        Rect::new(0, 10, 0, 1),
        Rect::new(0, 11, 0, 1),
        Rect::new(0, 12, 0, 1),
        Rect::new(0, 13, 0, 1),
    ];

    assert_eq!(row_at(&rows, 0), None);
    assert_eq!(row_at(&rows, 9), None);
    assert_eq!(row_at(&rows, 10), Some(0));
    assert_eq!(row_at(&rows, 11), Some(1));
    assert_eq!(row_at(&rows, 12), Some(2));
    assert_eq!(row_at(&rows, 13), Some(3));
    assert_eq!(row_at(&rows, 14), None);
    assert_eq!(row_at(&rows, 16384), None);

    assert_eq!(row_at_drag(area, &rows, 0), Err(-10));
    assert_eq!(row_at_drag(area, &rows, 9), Err(-1));
    assert_eq!(row_at_drag(area, &rows, 10), Ok(0));
    assert_eq!(row_at_drag(area, &rows, 11), Ok(1));
    assert_eq!(row_at_drag(area, &rows, 12), Ok(2));
    assert_eq!(row_at_drag(area, &rows, 13), Ok(3));
    assert_eq!(row_at_drag(area, &rows, 14), Err(1));
    assert_eq!(row_at_drag(area, &rows, 15), Err(2));
    assert_eq!(row_at_drag(area, &rows, 16384), Err(16371));
}

#[test]
fn test_rows_empty0() {
    let area = Rect::new(0, 0, 0, 10);
    let rows = vec![];
    assert_eq!(row_at(&rows, 23), None);

    assert_eq!(row_at_drag(area, &rows, 0), Err(0));
    assert_eq!(row_at_drag(area, &rows, 1), Err(1));
    assert_eq!(row_at_drag(area, &rows, 16384), Err(16384));
}

#[test]
fn test_rows_empty1() {
    let area = Rect::new(0, 10, 0, 10);
    let rows = vec![];
    assert_eq!(row_at(&rows, 23), None);

    assert_eq!(row_at_drag(area, &rows, 0), Err(-10));
    assert_eq!(row_at_drag(area, &rows, 1), Err(-9));
    assert_eq!(row_at_drag(area, &rows, 10), Err(0));
    assert_eq!(row_at_drag(area, &rows, 11), Err(1));
    assert_eq!(row_at_drag(area, &rows, 19), Err(9));
    assert_eq!(row_at_drag(area, &rows, 16384), Err(16374));
}

#[test]
fn test_cols0() {
    let area = Rect::new(0, 0, 4, 0);
    let cols = vec![
        Rect::new(0, 0, 1, 0),
        Rect::new(1, 0, 1, 0),
        Rect::new(2, 0, 1, 0),
        Rect::new(3, 0, 1, 0),
    ];

    assert_eq!(column_at(&cols, 0), Some(0));
    assert_eq!(column_at(&cols, 1), Some(1));
    assert_eq!(column_at(&cols, 2), Some(2));
    assert_eq!(column_at(&cols, 3), Some(3));
    assert_eq!(column_at(&cols, 4), None);
    assert_eq!(column_at(&cols, 16384), None);

    assert_eq!(column_at_drag(area, &cols, 0), Ok(0));
    assert_eq!(column_at_drag(area, &cols, 1), Ok(1));
    assert_eq!(column_at_drag(area, &cols, 2), Ok(2));
    assert_eq!(column_at_drag(area, &cols, 3), Ok(3));
    assert_eq!(column_at_drag(area, &cols, 4), Err(1));
    assert_eq!(column_at_drag(area, &cols, 5), Err(2));
    assert_eq!(column_at_drag(area, &cols, 16384), Err(16381));
}

#[test]
fn test_cols1() {
    let area = Rect::new(10, 0, 4, 0);
    let cols = vec![
        Rect::new(10, 0, 1, 0),
        Rect::new(11, 0, 1, 0),
        Rect::new(12, 0, 1, 0),
        Rect::new(13, 0, 1, 0),
    ];

    assert_eq!(column_at(&cols, 0), None);
    assert_eq!(column_at(&cols, 9), None);
    assert_eq!(column_at(&cols, 10), Some(0));
    assert_eq!(column_at(&cols, 11), Some(1));
    assert_eq!(column_at(&cols, 12), Some(2));
    assert_eq!(column_at(&cols, 13), Some(3));
    assert_eq!(column_at(&cols, 14), None);
    assert_eq!(column_at(&cols, 16384), None);

    assert_eq!(column_at_drag(area, &cols, 0), Err(-10));
    assert_eq!(column_at_drag(area, &cols, 9), Err(-1));
    assert_eq!(column_at_drag(area, &cols, 10), Ok(0));
    assert_eq!(column_at_drag(area, &cols, 11), Ok(1));
    assert_eq!(column_at_drag(area, &cols, 12), Ok(2));
    assert_eq!(column_at_drag(area, &cols, 13), Ok(3));
    assert_eq!(column_at_drag(area, &cols, 14), Err(1));
    assert_eq!(column_at_drag(area, &cols, 15), Err(2));
    assert_eq!(column_at_drag(area, &cols, 16384), Err(16371));
}

#[test]
fn test_cols_empty0() {
    let area = Rect::new(0, 0, 10, 0);
    let cols = vec![];
    assert_eq!(column_at(&cols, 23), None);

    assert_eq!(column_at_drag(area, &cols, 0), Err(0));
    assert_eq!(column_at_drag(area, &cols, 1), Err(1));
    assert_eq!(column_at_drag(area, &cols, 16384), Err(16384));
}

#[test]
fn test_cols_empty1() {
    let area = Rect::new(10, 0, 10, 0);
    let cols = vec![];
    assert_eq!(column_at(&cols, 23), None);

    assert_eq!(column_at_drag(area, &cols, 0), Err(-10));
    assert_eq!(column_at_drag(area, &cols, 1), Err(-9));
    assert_eq!(column_at_drag(area, &cols, 10), Err(0));
    assert_eq!(column_at_drag(area, &cols, 11), Err(1));
    assert_eq!(column_at_drag(area, &cols, 19), Err(9));
    assert_eq!(column_at_drag(area, &cols, 16384), Err(16374));
}
