use rat_text::clipboard::LocalClipboard;
use rat_text::core::{TextCore, TextRope, TextStore};
use rat_text::undo_buffer::UndoVec;
use rat_text::{TextPosition, TextRange};

#[test]
fn test_undo() {
    let mut s = TextCore::<TextRope>::new(
        Some(Box::new(UndoVec::new(40))),
        Some(Box::new(LocalClipboard::new())),
    );
    s.set_newline("\n".into());

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

    s.remove_next_char(TextPosition::new(0, 2)).unwrap();
    s.remove_next_char(TextPosition::new(0, 2)).unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\nuiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\nqwer\nuiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\nuiop\n");

    s.insert_newline(TextPosition::new(0, 3)).unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n\nuiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\nuiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n\nuiop\n");

    s.insert_tab(TextPosition::new(0, 3)).unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        \nuiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n\nuiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        \nuiop\n");

    s.insert_str(TextPosition::new(8, 3), "567").unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        567\nuiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        \nuiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        567\nuiop\n");

    s.remove_prev_char(TextPosition::new(2, 1)).unwrap();
    s.remove_prev_char(TextPosition::new(1, 1)).unwrap();
    assert_eq!(s.text().string(), "asdf\nxjklö\ner\n        567\nuiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        567\nuiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nxjklö\ner\n        567\nuiop\n");

    s.remove_str_range(TextRange::new((0, 2), (0, 4))).unwrap();
    assert_eq!(s.text().string(), "asdf\nxjklö\nuiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nxjklö\ner\n        567\nuiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nxjklö\nuiop\n");
}

#[test]
fn test_undo2() {
    let mut s = TextCore::<TextRope>::new(
        Some(Box::new(UndoVec::new(40))),
        Some(Box::new(LocalClipboard::new())),
    );
    s.set_newline("\n".into());

    s.set_text(TextRope::new_text("asdf\njklö\nqwer\nuiop\n"));
    assert_eq!(s.text().string(), "asdf\njklö\nqwer\nuiop\n");

    s.insert_char(TextPosition::new(0, 1), 'x').unwrap();
    s.insert_char(TextPosition::new(0, 1), 'y').unwrap();
    s.insert_char(TextPosition::new(0, 1), 'z').unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\nqwer\nuiop\n");

    s.remove_next_char(TextPosition::new(0, 2)).unwrap();
    s.remove_next_char(TextPosition::new(0, 2)).unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\nuiop\n");

    s.insert_newline(TextPosition::new(0, 3)).unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n\nuiop\n");

    s.insert_tab(TextPosition::new(0, 3)).unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        \nuiop\n");

    s.insert_str(TextPosition::new(8, 3), "567").unwrap();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        567\nuiop\n");

    s.remove_prev_char(TextPosition::new(2, 1)).unwrap();
    s.remove_prev_char(TextPosition::new(1, 1)).unwrap();
    assert_eq!(s.text().string(), "asdf\nxjklö\ner\n        567\nuiop\n");

    s.remove_str_range(TextRange::new((0, 2), (0, 4))).unwrap();
    assert_eq!(s.text().string(), "asdf\nxjklö\nuiop\n");

    s.undo();
    assert_eq!(s.text().string(), "asdf\nxjklö\ner\n        567\nuiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        567\nuiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        \nuiop\n");
    s.undo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n\nuiop\n");
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
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n\nuiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        \nuiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nzyxjklö\ner\n        567\nuiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nxjklö\ner\n        567\nuiop\n");
    s.redo();
    assert_eq!(s.text().string(), "asdf\nxjklö\nuiop\n");
}
