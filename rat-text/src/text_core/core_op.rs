use crate::core::{TextCore, TextStore};
use crate::{Cursor, TextError, TextPosition, TextRange};

/// Auto-quote the selected text.
#[allow(clippy::needless_bool)]
pub fn insert_quotes<Store: TextStore + Default>(
    core: &mut TextCore<Store>,
    mut sel: TextRange,
    c: char,
) -> Result<bool, TextError> {
    core.begin_undo_seq();

    // remove matching quotes/brackets
    if sel.end.x > 0 {
        let first = TextRange::new(sel.start, (sel.start.x + 1, sel.start.y));
        let last = TextRange::new((sel.end.x - 1, sel.end.y), sel.end);
        let c0 = core.str_slice(first).expect("valid_slice");
        let c1 = core.str_slice(last).expect("valid_slice");
        let remove_quote = if c == '\'' || c == '`' || c == '"' {
            if c0 == "'" && c1 == "'" {
                true
            } else if c0 == "\"" && c1 == "\"" {
                true
            } else if c0 == "`" && c1 == "`" {
                true
            } else {
                false
            }
        } else {
            if c0 == "<" && c1 == ">" {
                true
            } else if c0 == "(" && c1 == ")" {
                true
            } else if c0 == "[" && c1 == "]" {
                true
            } else if c0 == "{" && c1 == "}" {
                true
            } else {
                false
            }
        };
        if remove_quote {
            core.remove_char_range(last)?;
            core.remove_char_range(first)?;
            if sel.start.y == sel.end.y {
                sel = TextRange::new(sel.start, TextPosition::new(sel.end.x - 2, sel.end.y));
            } else {
                sel = TextRange::new(sel.start, TextPosition::new(sel.end.x - 1, sel.end.y));
            }
        }
    }

    let cc = match c {
        '\'' => '\'',
        '`' => '`',
        '"' => '"',
        '<' => '>',
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => unreachable!("invalid quotes"),
    };
    core.insert_char(sel.end, cc)?;
    core.insert_char(sel.start, c)?;
    if sel.start.y == sel.end.y {
        sel = TextRange::new(sel.start, TextPosition::new(sel.end.x + 2, sel.end.y));
    } else {
        sel = TextRange::new(sel.start, TextPosition::new(sel.end.x + 1, sel.end.y));
    }
    core.set_selection(sel.start, sel.end);
    core.end_undo_seq();

    Ok(true)
}

/// Insert a tab, either expanded or literally.
pub fn insert_tab<Store: TextStore + Default>(
    core: &mut TextCore<Store>,
    mut pos: TextPosition,
    expand_tabs: bool,
    tab_width: u32,
) -> Result<bool, TextError> {
    if expand_tabs {
        let n = tab_width - (pos.x % tab_width);
        for _ in 0..n {
            core.insert_char(pos, ' ')?;
            pos.x += 1;
        }
    } else {
        core.insert_char(pos, '\t')?;
    }

    Ok(true)
}

/// Remove the previous character
pub fn remove_prev_char<Store: TextStore + Default>(
    core: &mut TextCore<Store>,
    pos: TextPosition,
) -> Result<bool, TextError> {
    let (sx, sy) = if pos.y == 0 && pos.x == 0 {
        (0, 0)
    } else if pos.y > 0 && pos.x == 0 {
        let prev_line_width = core.line_width(pos.y - 1).expect("line_width");
        (prev_line_width, pos.y - 1)
    } else {
        (pos.x - 1, pos.y)
    };
    let range = TextRange::new((sx, sy), (pos.x, pos.y));

    core.remove_char_range(range)
}

/// Remove the next characters.
pub fn remove_next_char<Store: TextStore + Default>(
    core: &mut TextCore<Store>,
    pos: TextPosition,
) -> Result<bool, TextError> {
    let c_line_width = core.line_width(pos.y)?;
    let c_last_line = core.len_lines() - 1;

    let (ex, ey) = if pos.y == c_last_line && pos.x == c_line_width {
        (pos.x, pos.y)
    } else if pos.y != c_last_line && pos.x == c_line_width {
        (0, pos.y + 1)
    } else {
        (pos.x + 1, pos.y)
    };
    let range = TextRange::new((pos.x, pos.y), (ex, ey));

    core.remove_char_range(range)
}

/// Find the start of the next word. If the position is at the start
/// or inside a word, the same position is returned.
pub fn next_word_start<Store: TextStore + Default>(
    core: &TextCore<Store>,
    pos: TextPosition,
) -> Result<TextPosition, TextError> {
    let mut it = core.text_graphemes(pos)?;
    let mut last_pos = it.text_offset();
    loop {
        let Some(c) = it.next() else {
            break;
        };
        last_pos = c.text_bytes().start;
        if !c.is_whitespace() {
            break;
        }
    }

    Ok(core.byte_pos(last_pos).expect("valid_pos"))
}

/// Find the end of the next word. Skips whitespace first, then goes on
/// until it finds the next whitespace.
pub fn next_word_end<Store: TextStore + Default>(
    core: &TextCore<Store>,
    pos: TextPosition,
) -> Result<TextPosition, TextError> {
    let mut it = core.text_graphemes(pos)?;
    let mut last_pos = it.text_offset();
    let mut init = true;
    loop {
        let Some(c) = it.next() else {
            break;
        };
        last_pos = c.text_bytes().start;
        if init {
            if !c.is_whitespace() {
                init = false;
            }
        } else {
            if c.is_whitespace() {
                break;
            }
        }
        last_pos = c.text_bytes().end;
    }

    Ok(core.byte_pos(last_pos).expect("valid_pos"))
}

/// Find the start of the prev word. Skips whitespace first, then goes on
/// until it finds the next whitespace.
///
/// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
/// both return start<=end!
pub fn prev_word_start<Store: TextStore + Default>(
    core: &TextCore<Store>,
    pos: TextPosition,
) -> Result<TextPosition, TextError> {
    let mut it = core.text_graphemes(pos)?;
    let mut last_pos = it.text_offset();
    let mut init = true;
    loop {
        let Some(c) = it.prev() else {
            break;
        };
        if init {
            if !c.is_whitespace() {
                init = false;
            }
        } else {
            if c.is_whitespace() {
                break;
            }
        }
        last_pos = c.text_bytes().start;
    }

    Ok(core.byte_pos(last_pos).expect("valid_pos"))
}

/// Find the end of the previous word. Word is everything that is not whitespace.
/// Attention: start/end are mirrored here compared to next_word_start/next_word_end,
/// both return start<=end!
pub fn prev_word_end<Store: TextStore + Default>(
    core: &TextCore<Store>,
    pos: TextPosition,
) -> Result<TextPosition, TextError> {
    let mut it = core.text_graphemes(pos)?;
    let mut last_pos = it.text_offset();
    loop {
        let Some(c) = it.prev() else {
            break;
        };
        if !c.is_whitespace() {
            break;
        }
        last_pos = c.text_bytes().start;
    }

    Ok(core.byte_pos(last_pos).expect("valid_pos"))
}

/// Is the position at a word boundary?
pub fn is_word_boundary<Store: TextStore + Default>(
    core: &TextCore<Store>,
    pos: TextPosition,
) -> Result<bool, TextError> {
    let mut it = core.text_graphemes(pos)?;
    if let Some(c0) = it.prev() {
        it.next();
        if let Some(c1) = it.next() {
            Ok(c0.is_whitespace() && !c1.is_whitespace()
                || !c0.is_whitespace() && c1.is_whitespace())
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

/// Find the start of the word at pos.
/// Returns pos if the position is not inside a word.
pub fn word_start<Store: TextStore + Default>(
    core: &TextCore<Store>,
    pos: TextPosition,
) -> Result<TextPosition, TextError> {
    let mut it = core.text_graphemes(pos)?;
    let mut last_pos = it.text_offset();
    loop {
        let Some(c) = it.prev() else {
            break;
        };
        if c.is_whitespace() {
            break;
        }
        last_pos = c.text_bytes().start;
    }

    Ok(core.byte_pos(last_pos).expect("valid_pos"))
}

/// Find the end of the word at pos.
/// Returns pos if the position is not inside a word.
pub fn word_end<Store: TextStore + Default>(
    core: &TextCore<Store>,
    pos: TextPosition,
) -> Result<TextPosition, TextError> {
    let mut it = core.text_graphemes(pos)?;
    let mut last_pos = it.text_offset();
    loop {
        let Some(c) = it.next() else {
            break;
        };
        last_pos = c.text_bytes().start;
        if c.is_whitespace() {
            break;
        }
        last_pos = c.text_bytes().end;
    }

    Ok(core.byte_pos(last_pos).expect("valid_pos"))
}
