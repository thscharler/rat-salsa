#[allow(unused_imports)]
use log::debug;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use std::cmp::min;
use std::iter::once;
use std::mem;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Create a Line from the given text. The first '_' marks
/// the navigation-char.
pub(crate) fn menu_str(txt: &str) -> (Line<'_>, Option<char>) {
    let mut line = Line::default();

    let mut idx_underscore = None;
    let mut idx_navchar_start = None;
    let mut navchar = None;
    let mut idx_navchar_end = None;
    let cit = txt.char_indices();
    for (idx, c) in cit {
        if idx_underscore.is_none() && c == '_' {
            idx_underscore = Some(idx);
        } else if idx_underscore.is_some() && idx_navchar_start.is_none() {
            navchar = Some(c.to_ascii_lowercase());
            idx_navchar_start = Some(idx);
        } else if idx_navchar_start.is_some() && idx_navchar_end.is_none() {
            idx_navchar_end = Some(idx);
        }
    }
    if idx_navchar_start.is_some() && idx_navchar_end.is_none() {
        idx_navchar_end = Some(txt.len());
    }

    if let Some(idx_underscore) = idx_underscore {
        if let Some(idx_navchar_start) = idx_navchar_start {
            if let Some(idx_navchar_end) = idx_navchar_end {
                line.spans.push(Span::from(&txt[0..idx_underscore]));
                line.spans
                    .push(Span::from(&txt[idx_navchar_start..idx_navchar_end]).underlined());
                line.spans.push(Span::from(&txt[idx_navchar_end..]));

                return (line, navchar);
            }
        }
    }

    line.spans.push(Span::from(txt));

    (line, None)
}

pub(crate) fn revert_style(mut style: Style) -> Style {
    if style.fg.is_some() && style.bg.is_some() {
        mem::swap(&mut style.fg, &mut style.bg);
        style
    } else {
        style.black().on_white()
    }
}

/// Select previous.
pub(crate) fn prev_opt(select: Option<usize>, change: usize, len: usize) -> Option<usize> {
    if let Some(select) = select {
        Some(prev(select, change))
    } else {
        Some(len.saturating_sub(1))
    }
}

/// Select next.
pub(crate) fn next_opt(selected: Option<usize>, change: usize, len: usize) -> Option<usize> {
    if let Some(select) = selected {
        Some(next(select, change, len))
    } else {
        Some(0)
    }
}

/// Select previous.
pub(crate) fn prev(select: usize, change: usize) -> usize {
    select.saturating_sub(change)
}

/// Select next.
pub(crate) fn next(select: usize, change: usize, len: usize) -> usize {
    min(select + change, len.saturating_sub(1))
}

/// Length in graphemes.
pub(crate) fn gr_len(s: &str) -> usize {
    s.graphemes(true).count()
}

/// Drop first graphem.
/// If s is empty do nothing.
pub(crate) fn drop_first(s: &str) -> &str {
    if s.is_empty() {
        s
    } else {
        split_at(s, 1).1
    }
}

/// Drop last graphem.
/// If s is empty do nothing.
pub(crate) fn drop_last(s: &str) -> &str {
    if s.is_empty() {
        s
    } else {
        let end = s.graphemes(true).count();
        split_at(s, end - 1).0
    }
}

/// Split selection for removal along the mask boundaries.
pub(crate) fn split_remove_mask(
    value: &str,
    selection: Range<usize>,
    mask: Range<usize>,
) -> (&str, &str, &str, &str, &str) {
    let mut byte_mask_start = None;
    let mut byte_mask_end = None;
    let mut byte_sel_start = None;
    let mut byte_sel_end = None;

    for (cidx, (idx, _c)) in value
        .grapheme_indices(true)
        .chain(once((value.len(), "")))
        .enumerate()
    {
        if cidx == selection.start {
            byte_sel_start = Some(idx);
        }
        if cidx == selection.end {
            byte_sel_end = Some(idx);
        }
        if cidx == mask.start {
            byte_mask_start = Some(idx);
        }
        if cidx == mask.end {
            byte_mask_end = Some(idx);
        }
    }

    let byte_sel_start = if selection.start <= mask.start {
        byte_mask_start.expect("mask")
    } else if selection.start >= mask.end {
        byte_mask_end.expect("mask")
    } else {
        byte_sel_start.expect("mask")
    };

    let byte_sel_end = if selection.end <= mask.start {
        byte_mask_start.expect("mask")
    } else if selection.end >= mask.end {
        byte_mask_end.expect("mask")
    } else {
        byte_sel_end.expect("mask")
    };

    let byte_mask_start = byte_mask_start.expect("mask");
    let byte_mask_end = byte_mask_end.expect("mask");

    (
        &value[..byte_mask_start],
        &value[byte_mask_start..byte_sel_start],
        &value[byte_sel_start..byte_sel_end],
        &value[byte_sel_end..byte_mask_end],
        &value[byte_mask_end..],
    )
}

/// Split along mask-sections, search within the mask.
pub(crate) fn split_mask_match<'a>(
    value: &'a str,
    search: &str,
    mask: Range<usize>,
) -> (&'a str, &'a str, &'a str, &'a str, &'a str) {
    let mut byte_mask_start = None;
    let mut byte_mask_end = None;
    let mut byte_find_start = None;
    let mut byte_find_end = None;

    for (cidx, (idx, c)) in value
        .grapheme_indices(true)
        .chain(once((value.len(), "")))
        .enumerate()
    {
        if cidx == mask.start {
            byte_mask_start = Some(idx);
        }
        if cidx >= mask.start && cidx < mask.end && c == search {
            byte_find_start = Some(idx);
            byte_find_end = Some(idx + c.len());
        }
        if cidx == mask.end {
            byte_mask_end = Some(idx);
        }
    }

    #[allow(clippy::unnecessary_unwrap)]
    let (byte_find_start, byte_find_end) = if byte_find_start.is_some() {
        (byte_find_start.expect("find"), byte_find_end.expect("find"))
    } else {
        (
            byte_mask_start.expect("mask"),
            byte_mask_start.expect("mask"),
        )
    };
    let byte_mask_start = byte_mask_start.expect("mask");
    let byte_mask_end = byte_mask_end.expect("mask");

    (
        &value[..byte_mask_start],
        &value[byte_mask_start..byte_find_start],
        &value[byte_find_start..byte_find_end],
        &value[byte_find_end..byte_mask_end],
        &value[byte_mask_end..],
    )
}

/// Split along mask bounds and again at the cursor.
pub(crate) fn split_mask(
    value: &str,
    cursor: usize,
    mask: Range<usize>,
) -> (&str, &str, &str, &str) {
    let mut byte_mask_start = None;
    let mut byte_mask_end = None;
    let mut byte_cursor = None;

    for (cidx, (idx, _c)) in value
        .grapheme_indices(true)
        .chain(once((value.len(), "")))
        .enumerate()
    {
        if cidx == cursor {
            byte_cursor = Some(idx);
        }
        if cidx == mask.start {
            byte_mask_start = Some(idx);
        }
        if cidx == mask.end {
            byte_mask_end = Some(idx);
        }
    }

    let byte_cursor = if cursor <= mask.start {
        byte_mask_start.expect("mask")
    } else if cursor >= mask.end {
        byte_mask_end.expect("mask")
    } else {
        byte_cursor.expect("mask")
    };
    let byte_mask_start = byte_mask_start.expect("mask");
    let byte_mask_end = byte_mask_end.expect("mask");

    (
        &value[..byte_mask_start],
        &value[byte_mask_start..byte_cursor],
        &value[byte_cursor..byte_mask_end],
        &value[byte_mask_end..],
    )
}

pub(crate) fn split_at(value: &str, cursor: usize) -> (&str, &str) {
    let mut byte_cursor = None;

    for (cidx, (idx, _c)) in value
        .grapheme_indices(true)
        .chain(once((value.len(), "")))
        .enumerate()
    {
        if cidx == cursor {
            byte_cursor = Some(idx);
        }
    }

    let byte_cursor = byte_cursor.expect("cursor");

    (&value[..byte_cursor], &value[byte_cursor..])
}

/// Split off selection
pub(crate) fn split3(value: &str, selection: Range<usize>) -> (&str, &str, &str) {
    let mut byte_selection_start = None;
    let mut byte_selection_end = None;

    for (cidx, (idx, _c)) in value
        .grapheme_indices(true)
        .chain(once((value.len(), "")))
        .enumerate()
    {
        if cidx == selection.start {
            byte_selection_start = Some(idx);
        }
        if cidx == selection.end {
            byte_selection_end = Some(idx)
        }
    }

    let byte_selection_start = byte_selection_start.expect("byte_selection_start_not_found");
    let byte_selection_end = byte_selection_end.expect("byte_selection_end_not_found");

    (
        &value[0..byte_selection_start],
        &value[byte_selection_start..byte_selection_end],
        &value[byte_selection_end..value.len()],
    )
}
