use rat_text::text_input_mask::{MaskedInputState, mask_op};

#[test]
fn test_number1() {
    let mut m = MaskedInputState::new();
    m.set_mask("#").expect("ok");
    assert_eq!(m.cursor(), 1);
    mask_op::advance_cursor(&mut m, '1');
    assert_eq!(m.cursor(), 1);
    m.insert_char('1');
    assert_eq!(m.text(), "1");
}

#[test]
fn test_number2() {
    let mut m = MaskedInputState::new();

    m.set_mask("##").expect("ok");
    assert_eq!(m.cursor(), 2);
    mask_op::advance_cursor(&mut m, '1');
    assert_eq!(m.cursor(), 2);
    m.insert_char('1');
    assert_eq!(m.cursor(), 2);
    assert_eq!(m.text(), " 1");
    m.insert_char('2');
    assert_eq!(m.cursor(), 2);
    assert_eq!(m.text(), "12");
    m.insert_char('3');
    assert_eq!(m.cursor(), 2);
    assert_eq!(m.text(), "12");
    mask_op::remove_prev(&mut m);
    assert_eq!(m.text(), " 1");
    mask_op::remove_prev(&mut m);
    assert_eq!(m.text(), "  ");

    m.set_text("12");
    m.set_cursor(1, false);
    mask_op::remove_prev(&mut m);
    assert_eq!(m.cursor(), 1);
    assert_eq!(m.text(), " 2");

    m.set_text("12");
    m.set_cursor(1, false);
    mask_op::remove_next(&mut m);
    assert_eq!(m.cursor(), 2);
    assert_eq!(m.text(), " 1");

    m.set_text("12");
    m.set_cursor(2, false);
    mask_op::remove_prev(&mut m);
    assert_eq!(m.cursor(), 2);
    assert_eq!(m.text(), " 1");
    mask_op::remove_prev(&mut m);
    assert_eq!(m.cursor(), 2);
    assert_eq!(m.text(), "  ");
    mask_op::remove_prev(&mut m);
    assert_eq!(m.cursor(), 0);
    assert_eq!(m.text(), "  ");
    mask_op::remove_prev(&mut m);
    assert_eq!(m.cursor(), 0);
    assert_eq!(m.text(), "  ");

    m.set_text("12");
    m.set_cursor(2, false);
    mask_op::remove_next(&mut m);
    assert_eq!(m.cursor(), 2);
    assert_eq!(m.text(), "12");
}

#[test]
fn test_number3() {
    let mut m = MaskedInputState::new();

    m.set_mask("##0").expect("ok");
    m.set_cursor(0, false);
    assert_eq!(m.cursor(), 0);
    mask_op::advance_cursor(&mut m, '1');
    assert_eq!(m.cursor(), 3);
    m.insert_char('1');
    assert_eq!(m.text(), "  1");
    m.set_cursor(0, false);
    assert_eq!(m.cursor(), 0);
    mask_op::advance_cursor(&mut m, '2');
    assert_eq!(m.cursor(), 2);
    m.insert_char('2');
    assert_eq!(m.text(), " 21");
    assert_eq!(m.cursor(), 2);

    mask_op::remove_prev(&mut m);
    assert_eq!(m.text(), "  1");
    assert_eq!(m.cursor(), 2);
    mask_op::remove_next(&mut m);
    assert_eq!(m.text(), "  0");
    assert_eq!(m.cursor(), 3);
    mask_op::remove_prev(&mut m);
    assert_eq!(m.cursor(), 0);
}

#[test]
fn test_number4() {
    let mut m = MaskedInputState::new();

    m.set_mask("###.##").expect("ok");
    m.set_cursor(0, false);
    assert_eq!(m.cursor(), 0);
    mask_op::advance_cursor(&mut m, '1');
    assert_eq!(m.cursor(), 3);
    m.insert_char('1');
    assert_eq!(m.text(), "  1.  ");
    mask_op::advance_cursor(&mut m, '2');
    m.insert_char('2');
    assert_eq!(m.text(), " 12.  ");
    mask_op::advance_cursor(&mut m, '3');
    m.insert_char('3');
    assert_eq!(m.text(), "123.  ");
    assert_eq!(m.cursor(), 3);
    mask_op::advance_cursor(&mut m, '4');
    assert_eq!(m.cursor(), 3);
    m.insert_char('4');
    assert_eq!(m.text(), "123.  ");
}

#[test]
fn test_number5() {
    let mut m = MaskedInputState::new();

    m.set_mask("###.0##").expect("ok");
    assert_eq!(m.text(), "   .0  ");
    m.set_cursor(0, false);
    assert_eq!(m.cursor(), 0);
    mask_op::advance_cursor(&mut m, '.');
    assert_eq!(m.cursor(), 3);
    m.insert_char('.');
    assert_eq!(m.cursor(), 4);
    m.insert_char('1');
    assert_eq!(m.text(), "   .1  ");
    mask_op::advance_cursor(&mut m, '2');
    m.insert_char('2');
    assert_eq!(m.text(), "   .12 ");
    mask_op::advance_cursor(&mut m, '3');
    m.insert_char('3');
    assert_eq!(m.text(), "   .123");
    mask_op::advance_cursor(&mut m, '4');
    m.insert_char('4');
    assert_eq!(m.text(), "   .123");

    mask_op::remove_prev(&mut m);
    assert_eq!(m.text(), "   .12 ");
    mask_op::remove_prev(&mut m);
    assert_eq!(m.text(), "   .1  ");
    mask_op::remove_prev(&mut m);
    assert_eq!(m.text(), "   .0  ");
    assert_eq!(m.cursor(), 4);
    mask_op::remove_prev(&mut m);
    assert_eq!(m.cursor(), 3);
    mask_op::remove_prev(&mut m);
    assert_eq!(m.cursor(), 0);

    m.set_text("123.456");
    m.set_cursor(3, false);
    mask_op::remove_next(&mut m);
    assert_eq!(m.cursor(), 4);
    assert_eq!(m.text(), "123.456");
    mask_op::remove_next(&mut m);
    assert_eq!(m.cursor(), 4);
    assert_eq!(m.text(), "123.56 ");
    mask_op::remove_next(&mut m);
    assert_eq!(m.cursor(), 4);
    assert_eq!(m.text(), "123.6  ");
    mask_op::remove_next(&mut m);
    assert_eq!(m.cursor(), 4);
    assert_eq!(m.text(), "123.0  ");
    mask_op::remove_next(&mut m);
    assert_eq!(m.cursor(), 7);
    assert_eq!(m.text(), "123.0  ");
}

#[test]
fn test_number6() {
    let mut m = MaskedInputState::new();

    m.set_mask("###.0##").expect("ok");
    m.set_text("123.456");
    m.select_all();
    assert_eq!(m.selection(), 0..7);
    mask_op::remove_range(&mut m, 0..7).expect("ok");
    assert_eq!(m.text(), "   .0  ");

    m.set_text("123.456");
    mask_op::remove_range(&mut m, 2..5).expect("ok");
    assert_eq!(m.text(), " 12.56 ");
}

#[test]
fn test_number7() {
    let mut m = MaskedInputState::new();

    m.set_mask("###.0##").expect("ok");
    m.set_text("   .0  ");
    m.set_cursor(5, false);
    assert_eq!(m.cursor(), 5);
    mask_op::advance_cursor(&mut m, '1');
    assert_eq!(m.cursor(), 5);
    m.insert_char('1');
    assert_eq!(m.text(), "   .01 ");
}

#[test]
fn test_sign1() {
    let mut m = MaskedInputState::new();

    m.set_mask("###.###").expect("ok");
    m.set_text("  1.0  ");

    mask_op::advance_cursor(&mut m, '-');
    assert_eq!(m.cursor(), 3);
    m.insert_char('-');
    assert_eq!(m.text(), " -1.0  ");

    m.insert_char('-');
    assert_eq!(m.text(), "  1.0  ");
}

#[test]
fn test_sign2() {
    let mut m = MaskedInputState::new();

    m.set_mask("###.###-").expect("ok");
    m.set_text("  1.0   ");

    mask_op::advance_cursor(&mut m, '-');
    m.insert_char('-');
    assert_eq!(m.text(), "  1.0  -");

    m.set_mask("###.###+").expect("ok");
    m.set_text("  1.0   ");

    mask_op::advance_cursor(&mut m, '-');
    m.insert_char('-');
    assert_eq!(m.text(), "  1.0  -");
}

#[test]
fn test_sign3() {
    let mut m = MaskedInputState::new();

    m.set_mask("###.###").expect("ok");
    m.set_text("  1.0  ");
    m.set_cursor(0, false);

    m.insert_char('-');
    assert_eq!(m.text(), " -1.0  ");
    assert_eq!(m.cursor(), 0);

    m.set_cursor(1, false);
    m.insert_char('-');
    assert_eq!(m.text(), "  1.0  ");
    assert_eq!(m.cursor(), 1);

    m.set_cursor(2, false);
    m.insert_char('-');
    assert_eq!(m.text(), " -1.0  ");
    assert_eq!(m.cursor(), 2);

    m.set_cursor(3, false);
    m.insert_char('-');
    assert_eq!(m.text(), "  1.0  ");
    assert_eq!(m.cursor(), 3);

    m.set_cursor(4, false);
    m.insert_char('-');
    assert_eq!(m.text(), " -1.0  ");
    assert_eq!(m.cursor(), 4);

    m.set_cursor(5, false);
    m.insert_char('-');
    assert_eq!(m.text(), "  1.0  ");
    assert_eq!(m.cursor(), 5);

    m.set_cursor(6, false);
    m.insert_char('-');
    assert_eq!(m.text(), " -1.0  ");
    assert_eq!(m.cursor(), 6);

    m.set_cursor(7, false);
    m.insert_char('-');
    assert_eq!(m.text(), "  1.0  ");
    assert_eq!(m.cursor(), 7);

    // rev order
    m.insert_char('-');

    m.set_cursor(0, false);
    m.insert_char('-');
    assert_eq!(m.text(), "  1.0  ");
    assert_eq!(m.cursor(), 0);

    m.set_cursor(1, false);
    m.insert_char('-');
    assert_eq!(m.text(), " -1.0  ");
    assert_eq!(m.cursor(), 1);

    m.set_cursor(2, false);
    m.insert_char('-');
    assert_eq!(m.text(), "  1.0  ");
    assert_eq!(m.cursor(), 2);

    m.set_cursor(3, false);
    m.insert_char('-');
    assert_eq!(m.text(), " -1.0  ");
    assert_eq!(m.cursor(), 3);

    m.set_cursor(4, false);
    m.insert_char('-');
    assert_eq!(m.text(), "  1.0  ");
    assert_eq!(m.cursor(), 4);

    m.set_cursor(5, false);
    m.insert_char('-');
    assert_eq!(m.text(), " -1.0  ");
    assert_eq!(m.cursor(), 5);

    m.set_cursor(6, false);
    m.insert_char('-');
    assert_eq!(m.text(), "  1.0  ");
    assert_eq!(m.cursor(), 6);

    m.set_cursor(7, false);
    m.insert_char('-');
    assert_eq!(m.text(), " -1.0  ");
    assert_eq!(m.cursor(), 7);
}

#[test]
fn test_sign4() {
    let mut m = MaskedInputState::new();

    m.set_mask("\\X###.###-").expect("ok");
    m.set_text("   1.0   ");

    mask_op::advance_cursor(&mut m, '-');
    assert_eq!(m.cursor(), 4);
    m.insert_char('-');
    assert_eq!(m.text(), "   1.0  -");
}

#[test]
fn test_section_cursor1() {
    let mut m = MaskedInputState::new();

    m.set_mask("").expect("ok");
    assert_eq!(m.section_cursor(0), None);
    m.set_mask("#").expect("ok");
    assert_eq!(m.section_cursor(0), Some(1));
    m.set_mask("##").expect("ok");
    assert_eq!(m.section_cursor(0), Some(2));
    m.set_mask("###").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("##0").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("#00").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("000").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("###.#").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("###.##").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("###.###").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("###.0").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("###.0##").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("###.00").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("###.00#").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("###.000").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("##0.000").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("#00.000").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("990.000-").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("990.000+").expect("ok");
    assert_eq!(m.section_cursor(0), Some(3));
    m.set_mask("-990.000-").expect("ok");
    assert_eq!(m.section_cursor(0), Some(4));
    m.set_mask("+990.000+").expect("ok");
    assert_eq!(m.section_cursor(0), Some(4));
    m.set_mask("##\\/##\\/####").expect("ok");
    assert_eq!(m.section_cursor(0), Some(2));
    m.set_mask("###,##0.0##").expect("ok");
    assert_eq!(m.section_cursor(0), Some(7));
    m.set_mask("###,##0.0##-").expect("ok");
    assert_eq!(m.section_cursor(0), Some(7));
    m.set_mask("###,##0.0##+").expect("ok");
    assert_eq!(m.section_cursor(0), Some(7));
    m.set_mask("\\€ ###,##0.0##+").expect("ok");
    assert_eq!(m.section_cursor(0), None);
    assert_eq!(m.next_section_cursor(0), Some(9));
    m.set_mask("HHH").expect("ok");
    assert_eq!(m.section_cursor(0), Some(0));
    m.set_mask("HH HH HH").expect("ok");
    assert_eq!(m.section_cursor(0), Some(0));
    m.set_mask("llllll").expect("ok");
    assert_eq!(m.section_cursor(0), Some(0));
    m.set_mask("aaaaaa").expect("ok");
    assert_eq!(m.section_cursor(0), Some(0));
    m.set_mask("cccccc").expect("ok");
    assert_eq!(m.section_cursor(0), Some(0));
    m.set_mask("______").expect("ok");
    assert_eq!(m.section_cursor(0), Some(0));
    m.set_mask("dd\\°dd\\'dd\\\"").expect("ok");
    assert_eq!(m.section_cursor(0), Some(0));
}

#[test]
fn test_section_cursor2() {
    let mut m = MaskedInputState::new();

    m.set_mask("###,##0.0##-").expect("ok");
    assert_eq!(m.section_cursor(12), None);
    assert_eq!(m.section_cursor(11), Some(7));
}

#[test]
fn test_section2() {
    let mut m = MaskedInputState::new();
    m.set_mask("##\\/##\\/####").expect("ok");
    m.set_cursor(0, false);
    assert_eq!(m.cursor(), 0);
    mask_op::advance_cursor(&mut m, '/');
    assert_eq!(m.cursor(), 2);
    m.insert_char('/');
    assert_eq!(m.cursor(), 5);

    m.set_mask("dd\\°dd\\'dd\\\"").expect("ok");
    m.set_cursor(0, false);
    assert_eq!(m.cursor(), 0);
    mask_op::advance_cursor(&mut m, '\'');
    assert_eq!(m.cursor(), 5);
    m.insert_char('\'');
    assert_eq!(m.cursor(), 6);

    m.set_mask("90\\°90\\'90\\\"").expect("ok");
    m.set_cursor(0, false);
    assert_eq!(m.cursor(), 0);
    mask_op::advance_cursor(&mut m, '\'');
    assert_eq!(m.cursor(), 5);
    m.insert_char('\'');
    assert_eq!(m.cursor(), 8);
    mask_op::advance_cursor(&mut m, '"');
    assert_eq!(m.cursor(), 8);
    m.insert_char('"');
    assert_eq!(m.cursor(), 9);

    m.set_mask("\\€ ###,##0.0##+").expect("ok");
    m.set_cursor(0, false);
    assert_eq!(m.cursor(), 0);
    mask_op::advance_cursor(&mut m, '€');
    assert_eq!(m.cursor(), 0);
    m.insert_char('€');
    assert_eq!(m.cursor(), 9);
}
