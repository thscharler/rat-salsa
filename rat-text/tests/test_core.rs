use rat_text::clipboard::LocalClipboard;
use rat_text::core::core_op::{insert_tab, remove_next_char, remove_prev_char};
use rat_text::core::{TextCore, TextRope, TextStore};
use rat_text::undo_buffer::UndoVec;
use rat_text::{TextPosition, TextRange};

#[test]
fn test_undo() {
    let mut s = TextCore::<TextRope>::new(
        Some(Box::new(UndoVec::new(40))),
        Some(Box::new(LocalClipboard::new())),
    );

    s.set_text(TextRope::new_text("asdf\njklö\nqwer\nuiop\n"));
    assert_eq!(s.text().string(), "asdf\njklö\nqwer\nuiop\n");

    s.insert_char(TextPosition::new(0, 1), 'x').unwrap();
    s.insert_char(TextPosition::new(0, 1), 'y').unwrap();
    s.insert_char(TextPosition::new(0, 1), 'z').unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\nqwer\nuiop\n");
    s.undo();
    s.undo();
    s.undo();
    assert_eq!(s.text().string(), "asdf\njklö\nqwer\nuiop\n");
    s.redo();
    s.redo();
    s.redo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\nqwer\nuiop\n");

    remove_next_char(&mut s, TextPosition::new(0, 2)).unwrap();
    remove_next_char(&mut s, TextPosition::new(0, 2)).unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\nuiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\nqwer\nuiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\nuiop\n");

    insert_tab(&mut s, TextPosition::new(0, 3), true, 8).unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        uiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\nuiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        uiop\n");

    s.insert_str(TextPosition::new(8, 3), "567").unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        567uiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        uiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        567uiop\n");

    remove_prev_char(&mut s, TextPosition::new(2, 1)).unwrap();
    remove_prev_char(&mut s, TextPosition::new(1, 1)).unwrap();
    assert_eq!(s.text().string(), "asdf\nxjklö\ner\n        567uiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        567uiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nxjklö\ner\n        567uiop\n");

    s.remove_str_range(TextRange::new((0, 2), (0, 4))).unwrap();
    assert_eq!(s.text().string(), "asdf\nxjklö\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nxjklö\ner\n        567uiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nxjklö\n");
}

#[test]
fn test_undo2() {
    let mut s = TextCore::<TextRope>::new(
        Some(Box::new(UndoVec::new(40))),
        Some(Box::new(LocalClipboard::new())),
    );

    s.set_text(TextRope::new_text("asdf\njklö\nqwer\nuiop\n"));
    assert_eq!(s.text().string(), "asdf\njklö\nqwer\nuiop\n");

    s.insert_char(TextPosition::new(0, 1), 'x').unwrap();
    s.insert_char(TextPosition::new(0, 1), 'y').unwrap();
    s.insert_char(TextPosition::new(0, 1), 'z').unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\nqwer\nuiop\n");

    remove_next_char(&mut s, TextPosition::new(0, 2)).unwrap();
    remove_next_char(&mut s, TextPosition::new(0, 2)).unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\nuiop\n");

    insert_tab(&mut s, TextPosition::new(0, 3), true, 8).unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        uiop\n");

    s.insert_str(TextPosition::new(8, 3), "567").unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        567uiop\n");

    remove_prev_char(&mut s, TextPosition::new(2, 1)).unwrap();
    remove_prev_char(&mut s, TextPosition::new(1, 1)).unwrap();
    assert_eq!(s.text().string(), "asdf\nxjklö\ner\n        567uiop\n");

    s.remove_str_range(TextRange::new((0, 2), (0, 4))).unwrap();
    assert_eq!(s.text().string(), "asdf\nxjklö\n");

    s.undo();
    assert_eq!(s.text().string(), "asdf\nxjklö\ner\n        567uiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        567uiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        uiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\nuiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\nqwer\nuiop\n");
    s.undo();
    s.undo();
    s.undo();
    assert_eq!(s.text().string(), "asdf\njklö\nqwer\nuiop\n");

    s.redo();
    s.redo();
    s.redo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\nqwer\nuiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\nuiop\n");
    s.redo();

    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        uiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        567uiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nxjklö\ner\n        567uiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nxjklö\n");
}
