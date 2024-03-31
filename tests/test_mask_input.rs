use rat_salsa::widget::mask_input::core::EditDirection::*;
use rat_salsa::widget::mask_input::core::Mask::*;
use rat_salsa::widget::mask_input::core::{assert_eq_core, test_input_mask_core};

#[test]
fn test_ip4() {
    // ADVANCE CURSOR "1"
    let mut b = test_input_mask_core(
        "999\\.999\\.999\\.999",
        "   .   .   .   ",
        "   .   .   .   ",
        15,
        0,
        16,
        0,
        0,
        Some(",|.|-|+|E|e"),
    );
    b.advance_cursor('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "   .   .   .   ", "   .   .   .   ", 15, 0, 16, 3, 3,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #2:2:3-4 <9 | \. "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "   .   .   .   ", "   .   .   .   ", 15, 0, 16, 3, 3,Some(",|.|-|+|E|e"));
    b.insert_char('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "  1.   .   .   ", "   .   .   .   ", 15, 0, 16, 3, 3,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "  1.   .   .   ", "  1.   .   .   ", 15, 0, 16, 3, 3,Some(",|.|-|+|E|e"));
    b.advance_cursor('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "  1.   .   .   ", "  1.   .   .   ", 15, 0, 16, 3, 3,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #2:2:3-4 <9 | \. "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "  1.   .   .   ", "  1.   .   .   ", 15, 0, 16, 3, 3,Some(",|.|-|+|E|e"));
    b.insert_char('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", " 12.   .   .   ", "  1.   .   .   ", 15, 0, 16, 3, 3,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "3"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", " 12.   .   .   ", " 12.   .   .   ", 15, 0, 16, 3, 3,Some(",|.|-|+|E|e"));
    b.advance_cursor('3');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", " 12.   .   .   ", " 12.   .   .   ", 15, 0, 16, 3, 3,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #2:2:3-4 <9 | \. "3"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", " 12.   .   .   ", " 12.   .   .   ", 15, 0, 16, 3, 3,Some(",|.|-|+|E|e"));
    b.insert_char('3');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.   .   .   ", " 12.   .   .   ", 15, 0, 16, 3, 3,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "4"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.   .   .   ", "123.   .   .   ", 15, 0, 16, 3, 3,Some(",|.|-|+|E|e"));
    b.advance_cursor('4');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.   .   .   ", "123.   .   .   ", 15, 0, 16, 7, 7,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #4:4:7-8 <9 | \. "4"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.   .   .   ", "123.   .   .   ", 15, 0, 16, 7, 7,Some(",|.|-|+|E|e"));
    b.insert_char('4');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.  4.   .   ", "123.   .   .   ", 15, 0, 16, 7, 7,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "5"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.  4.   .   ", "123.  4.   .   ", 15, 0, 16, 7, 7,Some(",|.|-|+|E|e"));
    b.advance_cursor('5');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.  4.   .   ", "123.  4.   .   ", 15, 0, 16, 7, 7,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #4:4:7-8 <9 | \. "5"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.  4.   .   ", "123.  4.   .   ", 15, 0, 16, 7, 7,Some(",|.|-|+|E|e"));
    b.insert_char('5');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123. 45.   .   ", "123.  4.   .   ", 15, 0, 16, 7, 7,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "6"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123. 45.   .   ", "123. 45.   .   ", 15, 0, 16, 7, 7,Some(",|.|-|+|E|e"));
    b.advance_cursor('6');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123. 45.   .   ", "123. 45.   .   ", 15, 0, 16, 7, 7,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #4:4:7-8 <9 | \. "6"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123. 45.   .   ", "123. 45.   .   ", 15, 0, 16, 7, 7,Some(",|.|-|+|E|e"));
    b.insert_char('6');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456.   .   ", "123. 45.   .   ", 15, 0, 16, 7, 7,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "."
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456.   .   ", "123.456.   .   ", 15, 0, 16, 7, 7,Some(",|.|-|+|E|e"));
    b.advance_cursor('.');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456.   .   ", "123.456.   .   ", 15, 0, 16, 8, 8,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #5:5:8-11 \. | <9 "."
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456.   .   ", "123.456.   .   ", 15, 0, 16, 8, 8,Some(",|.|-|+|E|e"));
    b.insert_char('.');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456.   .   ", "123.456.   .   ", 15, 0, 16, 8, 8,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456.   .   ", "123.456.   .   ", 15, 0, 16, 8, 8,Some(",|.|-|+|E|e"));
    b.advance_cursor('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456.   .   ", "123.456.   .   ", 15, 0, 16, 11, 11,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #6:6:11-12 <9 | \. "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456.   .   ", "123.456.   .   ", 15, 0, 16, 11, 11,Some(",|.|-|+|E|e"));
    b.insert_char('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456.  1.   ", "123.456.   .   ", 15, 0, 16, 11, 11,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456.  1.   ", "123.456.  1.   ", 15, 0, 16, 11, 11,Some(",|.|-|+|E|e"));
    b.advance_cursor('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456.  1.   ", "123.456.  1.   ", 15, 0, 16, 11, 11,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #6:6:11-12 <9 | \. "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456.  1.   ", "123.456.  1.   ", 15, 0, 16, 11, 11,Some(",|.|-|+|E|e"));
    b.insert_char('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.   ", "123.456.  1.   ", 15, 0, 16, 11, 11,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "."
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.   ", "123.456. 12.   ", 15, 0, 16, 11, 11,Some(",|.|-|+|E|e"));
    b.advance_cursor('.');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.   ", "123.456. 12.   ", 15, 0, 16, 12, 12,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #7:7:12-15 \. | <9 "."
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.   ", "123.456. 12.   ", 15, 0, 16, 12, 12,Some(",|.|-|+|E|e"));
    b.insert_char('.');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.   ", "123.456. 12.   ", 15, 0, 16, 12, 12,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.   ", "123.456. 12.   ", 15, 0, 16, 12, 12,Some(",|.|-|+|E|e"));
    b.advance_cursor('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.   ", "123.456. 12.   ", 15, 0, 16, 15, 15,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #8:8:15-18 <9 |  "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.   ", "123.456. 12.   ", 15, 0, 16, 15, 15,Some(",|.|-|+|E|e"));
    b.insert_char('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.  1", "123.456. 12.   ", 15, 0, 16, 15, 15,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "4"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.  1", "123.456. 12.  1", 15, 0, 16, 15, 15,Some(",|.|-|+|E|e"));
    b.advance_cursor('4');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.  1", "123.456. 12.  1", 15, 0, 16, 15, 15,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #8:8:15-18 <9 |  "4"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.  1", "123.456. 12.  1", 15, 0, 16, 15, 15,Some(",|.|-|+|E|e"));
    b.insert_char('4');
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12. 14", "123.456. 12.  1", 15, 0, 16, 15, 15,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE NEXT Mask #7:7:12-15 <9 | <9
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12. 14", "123.456. 12. 14", 15, 0, 16, 14, 14,Some(",|.|-|+|E|e"));
    b.remove_next();
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.  1", "123.456. 12. 14", 15, 0, 16, 15, 15,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #7:7:12-15 <9 | <9
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.  1", "123.456. 12.  1", 15, 0, 16, 15, 15,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.   ", "123.456. 12.  1", 15, 0, 16, 12, 12,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #6:6:11-12 <9 | \.
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.   ", "123.456. 12.   ", 15, 0, 16, 12, 12,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.   ", "123.456. 12.   ", 15, 0, 16, 11, 11,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #5:5:8-11 <9 | <9
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456. 12.   ", "123.456. 12.   ", 15, 0, 16, 11, 11,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456.  1.   ", "123.456. 12.   ", 15, 0, 16, 11, 11,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #5:5:8-11 <9 | <9
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456.  1.   ", "123.456.  1.   ", 15, 0, 16, 11, 11,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", "123.456.   .   ", "123.456.  1.   ", 15, 0, 16, 8, 8,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE SELECTION Mask #1:1:0-3 <9 | <9 2..5
    #[rustfmt::skip]
        let mut b = test_input_mask_core("999\\.999\\.999\\.999", "123.456.   .   ", "123.456.   .   ", 15, 0, 16, 2, 5,Some(",|.|-|+|E|e"));
    b.remove_selection(2..5);
    #[rustfmt::skip]
        let a = test_input_mask_core("999\\.999\\.999\\.999", " 12. 56.   .   ", "123.456.   .   ", 15, 0, 16, 2, 2,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
}

#[test]
fn test_visa() {
    // ADVANCE CURSOR "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "                   ", "                   ", 19, 0, 20, 0, 0,Some(",|.|-|+|E|e"));
    b.advance_cursor('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "                   ", "                   ", 19, 0, 20, 0, 0,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:1:0-4  | d "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "                   ", "                   ", 19, 0, 20, 0, 0,Some(",|.|-|+|E|e"));
    b.insert_char('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1                  ", "                   ", 19, 0, 20, 1, 1,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1                  ", "1                  ", 19, 0, 20, 1, 1,Some(",|.|-|+|E|e"));
    b.advance_cursor('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1                  ", "1                  ", 19, 0, 20, 1, 1,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:1:0-4 d | d "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1                  ", "1                  ", 19, 0, 20, 1, 1,Some(",|.|-|+|E|e"));
    b.insert_char('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "12                 ", "1                  ", 19, 0, 20, 2, 2,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "3"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "12                 ", "12                 ", 19, 0, 20, 2, 2,Some(",|.|-|+|E|e"));
    b.advance_cursor('3');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "12                 ", "12                 ", 19, 0, 20, 2, 2,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:1:0-4 d | d "3"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "12                 ", "12                 ", 19, 0, 20, 2, 2,Some(",|.|-|+|E|e"));
    b.insert_char('3');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "123                ", "12                 ", 19, 0, 20, 3, 3,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "4"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "123                ", "123                ", 19, 0, 20, 3, 3,Some(",|.|-|+|E|e"));
    b.advance_cursor('4');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "123                ", "123                ", 19, 0, 20, 3, 3,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:1:0-4 d | d "4"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "123                ", "123                ", 19, 0, 20, 3, 3,Some(",|.|-|+|E|e"));
    b.insert_char('4');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234               ", "123                ", 19, 0, 20, 4, 4,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "5"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234               ", "1234               ", 19, 0, 20, 4, 4,Some(",|.|-|+|E|e"));
    b.advance_cursor('5');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234               ", "1234               ", 19, 0, 20, 5, 5,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #3:3:5-9   | d "5"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234               ", "1234               ", 19, 0, 20, 5, 5,Some(",|.|-|+|E|e"));
    b.insert_char('5');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5             ", "1234               ", 19, 0, 20, 6, 6,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "6"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5             ", "1234 5             ", 19, 0, 20, 6, 6,Some(",|.|-|+|E|e"));
    b.advance_cursor('6');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5             ", "1234 5             ", 19, 0, 20, 6, 6,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #3:3:5-9 d | d "6"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5             ", "1234 5             ", 19, 0, 20, 6, 6,Some(",|.|-|+|E|e"));
    b.insert_char('6');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 56            ", "1234 5             ", 19, 0, 20, 7, 7,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "7"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 56            ", "1234 56            ", 19, 0, 20, 7, 7,Some(",|.|-|+|E|e"));
    b.advance_cursor('7');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 56            ", "1234 56            ", 19, 0, 20, 7, 7,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #3:3:5-9 d | d "7"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 56            ", "1234 56            ", 19, 0, 20, 7, 7,Some(",|.|-|+|E|e"));
    b.insert_char('7');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 567           ", "1234 56            ", 19, 0, 20, 8, 8,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "8"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 567           ", "1234 567           ", 19, 0, 20, 8, 8,Some(",|.|-|+|E|e"));
    b.advance_cursor('8');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 567           ", "1234 567           ", 19, 0, 20, 8, 8,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #3:3:5-9 d | d "8"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 567           ", "1234 567           ", 19, 0, 20, 8, 8,Some(",|.|-|+|E|e"));
    b.insert_char('8');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678          ", "1234 567           ", 19, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR " "
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678          ", "1234 5678          ", 19, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.advance_cursor(' ');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678          ", "1234 5678          ", 19, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #5:5:10-14   | d " "
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678          ", "1234 5678          ", 19, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    b.insert_char(' ');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678          ", "1234 5678          ", 19, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "9"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678          ", "1234 5678          ", 19, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    b.advance_cursor('9');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678          ", "1234 5678          ", 19, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #5:5:10-14 d | d "9"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678          ", "1234 5678          ", 19, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    b.insert_char('9');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  9       ", "1234 5678          ", 19, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "9"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  9       ", "1234 5678  9       ", 19, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    b.advance_cursor('9');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  9       ", "1234 5678  9       ", 19, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #5:5:10-14 d | d "9"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  9       ", "1234 5678  9       ", 19, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    b.insert_char('9');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  99      ", "1234 5678  9       ", 19, 0, 20, 13, 13,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "9"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  99      ", "1234 5678  99      ", 19, 0, 20, 13, 13,Some(",|.|-|+|E|e"));
    b.advance_cursor('9');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  99      ", "1234 5678  99      ", 19, 0, 20, 13, 13,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #5:5:10-14 d | d "9"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  99      ", "1234 5678  99      ", 19, 0, 20, 13, 13,Some(",|.|-|+|E|e"));
    b.insert_char('9');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999     ", "1234 5678  99      ", 19, 0, 20, 14, 14,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR " "
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999     ", "1234 5678  999     ", 19, 0, 20, 14, 14,Some(",|.|-|+|E|e"));
    b.advance_cursor(' ');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999     ", "1234 5678  999     ", 19, 0, 20, 15, 15,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #7:7:15-19   | d " "
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999     ", "1234 5678  999     ", 19, 0, 20, 15, 15,Some(",|.|-|+|E|e"));
    b.insert_char(' ');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999     ", "1234 5678  999     ", 19, 0, 20, 16, 16,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "0"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999     ", "1234 5678  999     ", 19, 0, 20, 16, 16,Some(",|.|-|+|E|e"));
    b.advance_cursor('0');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999     ", "1234 5678  999     ", 19, 0, 20, 16, 16,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #7:7:15-19 d | d "0"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999     ", "1234 5678  999     ", 19, 0, 20, 16, 16,Some(",|.|-|+|E|e"));
    b.insert_char('0');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999  0  ", "1234 5678  999     ", 19, 0, 20, 17, 17,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "0"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999  0  ", "1234 5678  999  0  ", 19, 0, 20, 17, 17,Some(",|.|-|+|E|e"));
    b.advance_cursor('0');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999  0  ", "1234 5678  999  0  ", 19, 0, 20, 17, 17,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #7:7:15-19 d | d "0"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999  0  ", "1234 5678  999  0  ", 19, 0, 20, 17, 17,Some(",|.|-|+|E|e"));
    b.insert_char('0');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999  00 ", "1234 5678  999  0  ", 19, 0, 20, 18, 18,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "0"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999  00 ", "1234 5678  999  00 ", 19, 0, 20, 18, 18,Some(",|.|-|+|E|e"));
    b.advance_cursor('0');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999  00 ", "1234 5678  999  00 ", 19, 0, 20, 18, 18,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #7:7:15-19 d | d "0"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999  00 ", "1234 5678  999  00 ", 19, 0, 20, 18, 18,Some(",|.|-|+|E|e"));
    b.insert_char('0');
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999  000", "1234 5678  999  00 ", 19, 0, 20, 19, 19,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE SELECTION Mask #5:5:10-14 d | d 12..18
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  999  000", "1234 5678  999  000", 19, 0, 20, 12, 18,Some(",|.|-|+|E|e"));
    b.remove_selection(12..18);
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  9   0   ", "1234 5678  999  000", 19, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE SELECTION Mask #1:1:0-4  | d 0..9
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "1234 5678  9   0   ", "1234 5678  9   0   ", 19, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.remove_selection(0..9);
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "           9   0   ", "1234 5678  9   0   ", 19, 0, 20, 0, 0,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE SELECTION Mask #1:1:0-4  | d 0..19
    #[rustfmt::skip]
        let mut b = test_input_mask_core("dddd dddd dddd dddd", "           9   0   ", "           9   0   ", 19, 0, 20, 0, 0,Some(",|.|-|+|E|e"));
    b.remove_selection(0..19);
    #[rustfmt::skip]
        let a = test_input_mask_core("dddd dddd dddd dddd", "                   ", "           9   0   ", 19, 0, 20, 0, 0,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
}

#[test]
fn test_date() {
    // ADVANCE CURSOR "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", "  /  /    ", "mm/dd/yyyy", 10, 0, 11, 1, 1,Some(",|.|-|+|E|e"));
    b.advance_cursor('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", "  /  /    ", "mm/dd/yyyy", 10, 0, 11, 2, 2,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #2:2:2-3 <9 | / "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", "  /  /    ", "mm/dd/yyyy", 10, 0, 11, 2, 2,Some(",|.|-|+|E|e"));
    b.insert_char('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/  /    ", "mm/dd/yyyy", 10, 0, 11, 2, 2,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "/"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/  /    ", " 1/dd/yyyy", 10, 0, 11, 2, 2,Some(",|.|-|+|E|e"));
    b.advance_cursor('/');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/  /    ", " 1/dd/yyyy", 10, 0, 11, 3, 3,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #3:3:3-5 / | <9 "/"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/  /    ", " 1/dd/yyyy", 10, 0, 11, 3, 3,Some(",|.|-|+|E|e"));
    b.insert_char('/');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/  /    ", " 1/dd/yyyy", 10, 0, 11, 3, 3,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/  /    ", " 1/dd/yyyy", 10, 0, 11, 3, 3,Some(",|.|-|+|E|e"));
    b.advance_cursor('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/  /    ", " 1/dd/yyyy", 10, 0, 11, 5, 5,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #4:4:5-6 <9 | / "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/  /    ", " 1/dd/yyyy", 10, 0, 11, 5, 5,Some(",|.|-|+|E|e"));
    b.insert_char('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/ 1/    ", " 1/dd/yyyy", 10, 0, 11, 5, 5,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/ 1/    ", " 1/ 1/yyyy", 10, 0, 11, 5, 5,Some(",|.|-|+|E|e"));
    b.advance_cursor('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/ 1/    ", " 1/ 1/yyyy", 10, 0, 11, 5, 5,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #4:4:5-6 <9 | / "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/ 1/    ", " 1/ 1/yyyy", 10, 0, 11, 5, 5,Some(",|.|-|+|E|e"));
    b.insert_char('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/    ", " 1/ 1/yyyy", 10, 0, 11, 5, 5,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "/"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/    ", " 1/12/yyyy", 10, 0, 11, 5, 5,Some(",|.|-|+|E|e"));
    b.advance_cursor('/');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/    ", " 1/12/yyyy", 10, 0, 11, 6, 6,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #5:5:6-10 / | <9 "/"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/    ", " 1/12/yyyy", 10, 0, 11, 6, 6,Some(",|.|-|+|E|e"));
    b.insert_char('/');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/    ", " 1/12/yyyy", 10, 0, 11, 6, 6,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/    ", " 1/12/yyyy", 10, 0, 11, 6, 6,Some(",|.|-|+|E|e"));
    b.advance_cursor('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/    ", " 1/12/yyyy", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #6:6:10-10 <9 |  "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/    ", " 1/12/yyyy", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    b.insert_char('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/   2", " 1/12/yyyy", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "0"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/   2", " 1/12/   2", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    b.advance_cursor('0');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/   2", " 1/12/   2", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #6:6:10-10 <9 |  "0"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/   2", " 1/12/   2", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    b.insert_char('0');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/  20", " 1/12/   2", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/  20", " 1/12/  20", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    b.advance_cursor('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/  20", " 1/12/  20", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #6:6:10-10 <9 |  "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/  20", " 1/12/  20", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    b.insert_char('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/ 201", " 1/12/  20", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/ 201", " 1/12/ 201", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    b.advance_cursor('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/ 201", " 1/12/ 201", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #6:6:10-10 <9 |  "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/ 201", " 1/12/ 201", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    b.insert_char('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/2012", " 1/12/ 201", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #5:5:6-10 <9 | <9
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/2012", " 1/12/2012", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/ 201", " 1/12/2012", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #5:5:6-10 <9 | <9
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/ 201", " 1/12/ 201", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/  20", " 1/12/ 201", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/  20", " 1/12/  20", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    b.advance_cursor('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/  20", " 1/12/  20", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #6:6:10-10 <9 |  "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/  20", " 1/12/  20", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    b.insert_char('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/ 202", " 1/12/  20", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "4"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/ 202", " 1/12/ 202", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    b.advance_cursor('4');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/ 202", " 1/12/ 202", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #6:6:10-10 <9 |  "4"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/ 202", " 1/12/ 202", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    b.insert_char('4');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 1/12/2024", " 1/12/ 202", 10, 0, 11, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE SELECTION Mask #1:1:0-2 <9 | <9 1..7
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 1/12/2024", " 1/12/2024", 10, 0, 11, 1, 7,Some(",|.|-|+|E|e"));
    b.remove_selection(1..7);
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", "  /  /  24", " 1/12/2024", 10, 0, 11, 1, 1,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "5"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", "  /  /  24", "mm/dd/  24", 10, 0, 11, 1, 1,Some(",|.|-|+|E|e"));
    b.advance_cursor('5');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", "  /  /  24", "mm/dd/  24", 10, 0, 11, 2, 2,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #2:2:2-3 <9 | / "5"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", "  /  /  24", "mm/dd/  24", 10, 0, 11, 2, 2,Some(",|.|-|+|E|e"));
    b.insert_char('5');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 5/  /  24", "mm/dd/  24", 10, 0, 11, 2, 2,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "/"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 5/  /  24", " 5/dd/  24", 10, 0, 11, 2, 2,Some(",|.|-|+|E|e"));
    b.advance_cursor('/');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 5/  /  24", " 5/dd/  24", 10, 0, 11, 3, 3,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #3:3:3-5 / | <9 "/"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 5/  /  24", " 5/dd/  24", 10, 0, 11, 3, 3,Some(",|.|-|+|E|e"));
    b.insert_char('/');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 5/  /  24", " 5/dd/  24", 10, 0, 11, 3, 3,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "9"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 5/  /  24", " 5/dd/  24", 10, 0, 11, 3, 3,Some(",|.|-|+|E|e"));
    b.advance_cursor('9');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 5/  /  24", " 5/dd/  24", 10, 0, 11, 5, 5,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #4:4:5-6 <9 | / "9"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("99/99/9999", " 5/  /  24", " 5/dd/  24", 10, 0, 11, 5, 5,Some(",|.|-|+|E|e"));
    b.insert_char('9');
    #[rustfmt::skip]
        let a = test_input_mask_core("99/99/9999", " 5/ 9/  24", " 5/dd/  24", 10, 0, 11, 5, 5,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
}

#[test]
fn test_num73() {
    // ADVANCE CURSOR "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "        0.00", "        0,00", 12, 0, 20, 5, 5,Some(",|.|-|+|E|e"));
    b.advance_cursor('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "        0.00", "        0,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:2:9-10 <0 | . "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "        0.00", "        0,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.insert_char('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "        1.00", "        0,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "        1.00", "        1,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.advance_cursor('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "        1.00", "        1,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:2:9-10 <0 | . "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "        1.00", "        1,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.insert_char('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "       12.00", "        1,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "3"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "       12.00", "       12,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.advance_cursor('3');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "       12.00", "       12,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:2:9-10 <0 | . "3"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "       12.00", "       12,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.insert_char('3');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "      123.00", "       12,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #1:1:0-9 <# | <#
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "      123.00", "      123,00", 12, 0, 20, 8, 8,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "       13.00", "      123,00", 12, 0, 20, 8, 8,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "5"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "       13.00", "       13,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.advance_cursor('5');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "       13.00", "       13,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:2:9-10 <0 | . "5"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "       13.00", "       13,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.insert_char('5');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "      135.00", "       13,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "7"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "      135.00", "      135,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.advance_cursor('7');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "      135.00", "      135,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:2:9-10 <0 | . "7"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "      135.00", "      135,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.insert_char('7');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "    1,357.00", "      135,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "9"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "    1,357.00", "    1.357,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.advance_cursor('9');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "    1,357.00", "    1.357,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:2:9-10 <0 | . "9"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "    1,357.00", "    1.357,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.insert_char('9');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "   13,579.00", "    1.357,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "-"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.advance_cursor('-');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:2:9-10 <0 | . "-"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.insert_char('-');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "-  13,579.00", "   13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "-"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "-  13,579.00", "-  13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.advance_cursor('-');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "-  13,579.00", "-  13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:2:9-10 <0 | . "-"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "-  13,579.00", "-  13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.insert_char('-');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "   13,579.00", "-  13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "-"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.advance_cursor('-');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:2:9-10 <0 | . "-"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.insert_char('-');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "-  13,579.00", "   13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "-"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "-  13,579.00", "-  13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.advance_cursor('-');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "-  13,579.00", "-  13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:2:9-10 <0 | . "-"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "-  13,579.00", "-  13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.insert_char('-');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "   13,579.00", "-  13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR ","
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.advance_cursor(',');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:3:10-12 . | >0 ","
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    b.insert_char(',');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    b.advance_cursor('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:3:10-12 . | >0 "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    b.insert_char('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "   13,579.20", "   13.579,00", 12, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "   13,579.20", "   13.579,20", 12, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    b.advance_cursor('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "   13,579.20", "   13.579,20", 12, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #1:3:10-12 >0 | >0 "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "   13,579.20", "   13.579,20", 12, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    b.insert_char('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "   13,579.22", "   13.579,20", 12, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #1:3:10-12 >0 | >0
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "   13,579.22", "   13.579,22", 12, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "   13,579.20", "   13.579,22", 12, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #1:3:10-12 . | >0
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "   13,579.20", "   13.579,20", 12, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,20", 12, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #1:2:9-10 <0 | .
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #1:1:0-9 <# | <0
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "   13,579.00", "   13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "    1,357.00", "   13.579,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #1:1:0-9 <# | <0
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "    1,357.00", "    1.357,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "      135.00", "    1.357,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #1:1:0-9 <# | <0
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "      135.00", "      135,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "       13.00", "      135,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #1:1:0-9 <# | <0
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "       13.00", "       13,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "        1.00", "       13,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #1:1:0-9 <# | <0
    #[rustfmt::skip]
        let mut b = test_input_mask_core("#,###,##0.00", "        1.00", "        1,00", 12, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("#,###,##0.00", "        0.00", "        1,00", 12, 0, 20, 0, 0,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
}

#[test]
fn test_euro() {
    // ADVANCE CURSOR "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€       0.00 ", "€       0,00 ", 13, 0, 20, 0, 0,Some(",|.|-|+|E|e"));
    b.advance_cursor('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€       0.00 ", "€       0,00 ", 13, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #3:4:9-10 <0 | . "1"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€       0.00 ", "€       0,00 ", 13, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.insert_char('1');
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€       1.00 ", "€       0,00 ", 13, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€       1.00 ", "€       1,00 ", 13, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.advance_cursor('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€       1.00 ", "€       1,00 ", 13, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #3:4:9-10 <0 | . "2"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€       1.00 ", "€       1,00 ", 13, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.insert_char('2');
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€      12.00 ", "€       1,00 ", 13, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR ","
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€      12.00 ", "€      12,00 ", 13, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.advance_cursor(',');
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€      12.00 ", "€      12,00 ", 13, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #3:5:10-12 . | >0 ","
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€      12.00 ", "€      12,00 ", 13, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    b.insert_char(',');
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€      12.00 ", "€      12,00 ", 13, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "4"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€      12.00 ", "€      12,00 ", 13, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    b.advance_cursor('4');
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€      12.00 ", "€      12,00 ", 13, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #3:5:10-12 . | >0 "4"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€      12.00 ", "€      12,00 ", 13, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    b.insert_char('4');
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€      12.40 ", "€      12,00 ", 13, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "5"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€      12.40 ", "€      12,40 ", 13, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    b.advance_cursor('5');
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€      12.40 ", "€      12,40 ", 13, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #3:5:10-12 >0 | >0 "5"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€      12.40 ", "€      12,40 ", 13, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    b.insert_char('5');
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€      12.45 ", "€      12,40 ", 13, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "-"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€      12.45 ", "€      12,45 ", 13, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    b.advance_cursor('-');
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€      12.45 ", "€      12,45 ", 13, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #3:6:12-13 >0 | - "-"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€      12.45 ", "€      12,45 ", 13, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    b.insert_char('-');
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€      12.45-", "€      12,45 ", 13, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #3:5:10-12 >0 | >0
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€      12.45-", "€      12,45-", 13, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€      12.40-", "€      12,45-", 13, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #3:5:10-12 . | >0
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€      12.40-", "€      12,40-", 13, 0, 20, 11, 11,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€      12.00-", "€      12,40-", 13, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #3:4:9-10 <0 | .
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€      12.00-", "€      12,00-", 13, 0, 20, 10, 10,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€      12.00-", "€      12,00-", 13, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #3:3:2-9 <# | <0
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€      12.00-", "€      12,00-", 13, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€       1.00-", "€      12,00-", 13, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE PREV Mask #3:3:2-9 <# | <0
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€       1.00-", "€       1,00-", 13, 0, 20, 9, 9,Some(",|.|-|+|E|e"));
    b.remove_prev();
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€       0.00-", "€       1,00-", 13, 0, 20, 2, 2,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // ADVANCE CURSOR "-"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€       0.00-", "€       0,00-", 13, 0, 20, 2, 2,Some(",|.|-|+|E|e"));
    b.advance_cursor('-');
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€       0.00-", "€       0,00-", 13, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // INSERT CHAR Mask #3:6:12-13 >0 | - "-"
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€       0.00-", "€       0,00-", 13, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    b.insert_char('-');
    #[rustfmt::skip]
        let a = test_input_mask_core("€ ###,##0.00-", "€       0.00 ", "€       0,00-", 13, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    assert_eq_core(&b, &a);
    // REMOVE NEXT Mask #3:6:12-13 >0 | -
    #[rustfmt::skip]
        let mut b = test_input_mask_core("€ ###,##0.00-", "€       0.00 ", "€       0,00 ", 13, 0, 20, 12, 12,Some(",|.|-|+|E|e"));
    b.remove_next();
}
