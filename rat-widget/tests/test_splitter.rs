use rat_widget::splitter::SplitState;

#[test]
fn test_0() {
    let mut sp = SplitState::new();
    sp.set_area_lengths(vec![]);

    assert_eq!(sp.len(), 0);
    assert_eq!(sp.area_lengths(), &[]);
    assert_eq!(sp.hidden_lengths(), &[]);
    assert_eq!(sp.total_area_len(), 0);
    // rest will panic
}

#[test]
fn test_1() {
    let mut sp = SplitState::new();
    sp.set_area_lengths(vec![10]);

    assert_eq!(sp.len(), 1);
    assert_eq!(sp.area_lengths(), &[10]);
    assert_eq!(sp.hidden_lengths(), &[0]);
    assert_eq!(sp.total_area_len(), 10);

    assert_eq!(sp.area_len(0), 10);
    sp.set_area_len(0, 20);
    assert_eq!(sp.area_len(0), 20);

    assert_eq!(sp.is_hidden(0), false);
    assert_eq!(sp.hide_split(0), false);
    assert_eq!(sp.is_hidden(0), false);

    sp.set_hidden_lengths(vec![5]);
    // don't expand in this case. true is fine.
    assert_eq!(sp.show_split(0), true);
    assert_eq!(sp.hidden_lengths(), &[0]);
    assert_eq!(sp.area_lengths(), &[20]);
}

#[test]
fn test_2() {
    let mut sp = SplitState::new();
    sp.set_area_lengths(vec![10, 10]);

    assert_eq!(sp.len(), 2);
    assert_eq!(sp.area_lengths(), &[10, 10]);
    assert_eq!(sp.hidden_lengths(), &[0, 0]);
    assert_eq!(sp.total_area_len(), 20);

    assert_eq!(sp.area_len(0), 10);
    sp.set_area_len(0, 20);
    assert_eq!(sp.area_len(0), 20);

    assert_eq!(sp.split_pos(0), 20);
    sp.set_split_pos(0, 0);
    assert_eq!(sp.area_lengths(), &[1, 29]);
    sp.set_split_pos(0, 255);
    assert_eq!(sp.area_lengths(), &[30, 0]);
    sp.set_split_pos(0, 32768);
    assert_eq!(sp.area_lengths(), &[30, 0]);
    sp.set_split_pos(0, 1);
    assert_eq!(sp.area_lengths(), &[1, 29]);
    sp.set_split_pos(0, 20);
    assert_eq!(sp.area_lengths(), &[20, 10]);

    assert_eq!(sp.is_hidden(0), false);
    assert_eq!(sp.hide_split(0), true);
    assert_eq!(sp.area_lengths(), &[1, 29]);
    assert_eq!(sp.hidden_lengths(), &[19, 0]);
    assert_eq!(sp.is_hidden(0), true);
    assert_eq!(sp.show_split(0), true);
    assert_eq!(sp.area_lengths(), &[20, 10]);
    assert_eq!(sp.hidden_lengths(), &[0, 0]);
    assert_eq!(sp.is_hidden(0), false);

    assert_eq!(sp.is_hidden(1), false);
    assert_eq!(sp.hide_split(1), true);
    assert_eq!(sp.area_lengths(), &[30, 0]);
    assert_eq!(sp.hidden_lengths(), &[0, 10]);
    assert_eq!(sp.is_hidden(1), true);
    assert_eq!(sp.show_split(1), true);
    assert_eq!(sp.area_lengths(), &[20, 10]);
    assert_eq!(sp.hidden_lengths(), &[0, 0]);
    assert_eq!(sp.is_hidden(1), false);

    sp.set_hidden_lengths(vec![0, 5]);
    assert_eq!(sp.show_split(1), true);
    assert_eq!(sp.hidden_lengths(), &[0, 0]);
    assert_eq!(sp.area_lengths(), &[15, 15]);

    sp.set_hidden_lengths(vec![5, 0]);
    assert_eq!(sp.show_split(0), true);
    assert_eq!(sp.hidden_lengths(), &[0, 0]);
    assert_eq!(sp.area_lengths(), &[20, 10]);

    sp.set_hidden_lengths(vec![255, 0]);
    assert_eq!(sp.show_split(0), true);
    assert_eq!(sp.hidden_lengths(), &[0, 0]);
    assert_eq!(sp.area_lengths(), &[29, 1]);
}

#[test]
fn test_3() {
    let mut sp = SplitState::new();
    sp.set_area_lengths(vec![10, 10, 10]);

    assert_eq!(sp.len(), 3);
    assert_eq!(sp.area_lengths(), &[10, 10, 10]);
    assert_eq!(sp.hidden_lengths(), &[0, 0, 0]);
    assert_eq!(sp.total_area_len(), 30);

    assert_eq!(sp.area_len(0), 10);
    sp.set_area_len(0, 20);
    assert_eq!(sp.area_len(0), 20);

    assert_eq!(sp.split_pos(0), 20);
    sp.set_split_pos(0, 0);
    assert_eq!(sp.area_lengths(), &[1, 29, 10]);
    sp.set_split_pos(0, 255);
    assert_eq!(sp.area_lengths(), &[39, 1, 0]);
    sp.set_split_pos(0, 32768);
    assert_eq!(sp.area_lengths(), &[39, 1, 0]);
    sp.set_split_pos(0, 1);
    assert_eq!(sp.area_lengths(), &[1, 39, 0]);
    sp.set_split_pos(0, 20);
    assert_eq!(sp.area_lengths(), &[20, 20, 0]);

    sp.set_split_pos(1, 25);
    assert_eq!(sp.area_lengths(), &[20, 5, 15]);
    sp.set_split_pos(1, 20);
    assert_eq!(sp.area_lengths(), &[19, 1, 20]);
    sp.set_split_pos(1, 15);
    assert_eq!(sp.area_lengths(), &[14, 1, 25]);
    sp.set_split_pos(1, 0);
    assert_eq!(sp.area_lengths(), &[1, 1, 38]);
    sp.set_split_pos(1, 1);
    assert_eq!(sp.area_lengths(), &[1, 1, 38]);
    sp.set_split_pos(1, 25);
    assert_eq!(sp.area_lengths(), &[1, 24, 15]);
    sp.set_split_pos(1, 39);
    assert_eq!(sp.area_lengths(), &[1, 38, 1]);
    sp.set_split_pos(1, 40);
    assert_eq!(sp.area_lengths(), &[1, 39, 0]);
    sp.set_split_pos(1, 41);
    assert_eq!(sp.area_lengths(), &[1, 39, 0]);
    sp.set_split_pos(1, 25);
    assert_eq!(sp.area_lengths(), &[1, 24, 15]);
    sp.set_split_pos(1, 255);
    assert_eq!(sp.area_lengths(), &[1, 39, 0]);

    sp.set_split_pos(0, 20);
    sp.set_split_pos(1, 30);
    assert_eq!(sp.area_lengths(), &[20, 10, 10]);

    assert_eq!(sp.is_hidden(0), false);
    assert_eq!(sp.hide_split(0), true);
    assert_eq!(sp.area_lengths(), &[1, 29, 10]);
    assert_eq!(sp.hidden_lengths(), &[19, 0, 0]);
    assert_eq!(sp.is_hidden(0), true);
    assert_eq!(sp.show_split(0), true);
    assert_eq!(sp.area_lengths(), &[20, 10, 10]);
    assert_eq!(sp.hidden_lengths(), &[0, 0, 0]);
    assert_eq!(sp.is_hidden(0), false);

    assert_eq!(sp.is_hidden(1), false);
    assert_eq!(sp.hide_split(1), true);
    assert_eq!(sp.area_lengths(), &[20, 1, 19]);
    assert_eq!(sp.hidden_lengths(), &[0, 9, 0]);
    assert_eq!(sp.is_hidden(1), true);
    assert_eq!(sp.show_split(1), true);
    assert_eq!(sp.area_lengths(), &[20, 10, 10]);
    assert_eq!(sp.hidden_lengths(), &[0, 0, 0]);
    assert_eq!(sp.is_hidden(1), false);

    sp.set_hidden_lengths(vec![0, 5, 0]);
    assert_eq!(sp.show_split(1), true);
    assert_eq!(sp.hidden_lengths(), &[0, 0, 0]);
    assert_eq!(sp.area_lengths(), &[20, 15, 5]);

    sp.set_hidden_lengths(vec![5, 0, 0]);
    assert_eq!(sp.show_split(0), true);
    assert_eq!(sp.hidden_lengths(), &[0, 0, 0]);
    assert_eq!(sp.area_lengths(), &[25, 10, 5]);

    sp.set_hidden_lengths(vec![255, 0, 0]);
    assert_eq!(sp.show_split(0), true);
    assert_eq!(sp.hidden_lengths(), &[0, 0, 0]);
    assert_eq!(sp.area_lengths(), &[38, 1, 1]);
}

#[test]
fn test_4() {
    let mut sp = SplitState::new();
    sp.set_area_lengths(vec![10, 10, 10, 10]);

    assert_eq!(sp.len(), 4);
    assert_eq!(sp.area_lengths(), &[10, 10, 10, 10]);
    assert_eq!(sp.hidden_lengths(), &[0, 0, 0, 0]);
    assert_eq!(sp.total_area_len(), 40);

    sp.set_area_lengths(vec![10, 10, 10, 10]);
    sp.set_split_pos(0, 0);
    assert_eq!(sp.area_lengths(), &[1, 19, 10, 10]);
    sp.set_split_pos(0, 255);
    assert_eq!(sp.area_lengths(), &[38, 1, 1, 0]);
    sp.set_split_pos(0, 32768);
    assert_eq!(sp.area_lengths(), &[38, 1, 1, 0]);
    sp.set_split_pos(0, 1);
    assert_eq!(sp.area_lengths(), &[1, 38, 1, 0]);
    sp.set_split_pos(0, 10);
    assert_eq!(sp.area_lengths(), &[10, 29, 1, 0]);

    sp.set_area_lengths(vec![10, 10, 10, 10]);
    sp.set_split_pos(1, 15);
    assert_eq!(sp.area_lengths(), &[10, 5, 15, 10]);
    sp.set_split_pos(1, 20);
    assert_eq!(sp.area_lengths(), &[10, 10, 10, 10]);
    sp.set_split_pos(1, 15);
    assert_eq!(sp.area_lengths(), &[10, 5, 15, 10]);
    sp.set_split_pos(1, 0);
    assert_eq!(sp.area_lengths(), &[1, 1, 28, 10]);
    sp.set_split_pos(1, 1);
    assert_eq!(sp.area_lengths(), &[1, 1, 28, 10]);
    sp.set_split_pos(1, 25);
    assert_eq!(sp.area_lengths(), &[1, 24, 5, 10]);
    sp.set_split_pos(1, 39);
    assert_eq!(sp.area_lengths(), &[1, 38, 1, 0]);
    sp.set_split_pos(1, 40);
    assert_eq!(sp.area_lengths(), &[1, 38, 1, 0]);
    sp.set_split_pos(1, 41);
    assert_eq!(sp.area_lengths(), &[1, 38, 1, 0]);
    sp.set_split_pos(1, 25);
    assert_eq!(sp.area_lengths(), &[1, 24, 15, 0]);
    sp.set_split_pos(1, 255);
    assert_eq!(sp.area_lengths(), &[1, 38, 1, 0]);

    sp.set_area_lengths(vec![10, 10, 10, 10]);
    sp.set_split_pos(2, 15);
    assert_eq!(sp.area_lengths(), &[10, 4, 1, 25]);
    sp.set_split_pos(2, 20);
    assert_eq!(sp.area_lengths(), &[10, 4, 6, 20]);
    sp.set_split_pos(2, 15);
    assert_eq!(sp.area_lengths(), &[10, 4, 1, 25]);
    sp.set_split_pos(2, 0);
    assert_eq!(sp.area_lengths(), &[1, 1, 1, 37]);
    sp.set_split_pos(2, 1);
    assert_eq!(sp.area_lengths(), &[1, 1, 1, 37]);
    sp.set_split_pos(2, 25);
    assert_eq!(sp.area_lengths(), &[1, 1, 23, 15]);
    sp.set_split_pos(2, 39);
    assert_eq!(sp.area_lengths(), &[1, 1, 37, 1]);
    sp.set_split_pos(2, 40);
    assert_eq!(sp.area_lengths(), &[1, 1, 38, 0]);
    sp.set_split_pos(2, 41);
    assert_eq!(sp.area_lengths(), &[1, 1, 38, 0]);
    sp.set_split_pos(2, 25);
    assert_eq!(sp.area_lengths(), &[1, 1, 23, 15]);
    sp.set_split_pos(2, 255);
    assert_eq!(sp.area_lengths(), &[1, 1, 38, 0]);

    sp.set_area_lengths(vec![10, 10, 10, 10]);

    assert_eq!(sp.is_hidden(0), false);
    assert_eq!(sp.hide_split(0), true);
    assert_eq!(sp.area_lengths(), &[1, 19, 10, 10]);
    assert_eq!(sp.hidden_lengths(), &[9, 0, 0, 0]);
    assert_eq!(sp.is_hidden(0), true);
    assert_eq!(sp.show_split(0), true);
    assert_eq!(sp.area_lengths(), &[10, 10, 10, 10]);
    assert_eq!(sp.hidden_lengths(), &[0, 0, 0, 0]);
    assert_eq!(sp.is_hidden(0), false);

    assert_eq!(sp.is_hidden(1), false);
    assert_eq!(sp.hide_split(1), true);
    assert_eq!(sp.area_lengths(), &[10, 1, 19, 10]);
    assert_eq!(sp.hidden_lengths(), &[0, 9, 0, 0]);
    assert_eq!(sp.is_hidden(1), true);
    assert_eq!(sp.show_split(1), true);
    assert_eq!(sp.area_lengths(), &[10, 10, 10, 10]);
    assert_eq!(sp.hidden_lengths(), &[0, 0, 0, 0]);
    assert_eq!(sp.is_hidden(1), false);

    assert_eq!(sp.is_hidden(2), false);
    assert_eq!(sp.hide_split(2), true);
    assert_eq!(sp.area_lengths(), &[10, 10, 1, 19]);
    assert_eq!(sp.hidden_lengths(), &[0, 0, 9, 0]);
    assert_eq!(sp.is_hidden(2), true);
    assert_eq!(sp.show_split(2), true);
    assert_eq!(sp.area_lengths(), &[10, 10, 10, 10]);
    assert_eq!(sp.hidden_lengths(), &[0, 0, 0, 0]);
    assert_eq!(sp.is_hidden(2), false);

    assert_eq!(sp.is_hidden(3), false);
    assert_eq!(sp.hide_split(3), true);
    assert_eq!(sp.area_lengths(), &[10, 10, 20, 0]);
    assert_eq!(sp.hidden_lengths(), &[0, 0, 0, 10]);
    assert_eq!(sp.is_hidden(3), true);
    assert_eq!(sp.show_split(3), true);
    assert_eq!(sp.area_lengths(), &[10, 10, 10, 10]);
    assert_eq!(sp.hidden_lengths(), &[0, 0, 0, 0]);
    assert_eq!(sp.is_hidden(3), false);
}
