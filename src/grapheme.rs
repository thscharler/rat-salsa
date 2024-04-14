//!
//! Utility functions to work with graphemes.
//!

use std::iter::once;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

///
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

/// Split off selection and cursor.
pub(crate) fn split5(
    value: &str,
    cursor: usize,
    visible: Range<usize>,
    selection: Range<usize>,
) -> (&str, &str, &str, &str, &str) {
    let mut vis_sta = None;
    let mut vis_end = None;
    let mut sel_sta = None;
    let mut sel_end = None;
    let mut cur_sta = None;
    let mut cur_len = None;

    for (cidx, (idx, c)) in value
        .grapheme_indices(true)
        .chain(once((value.len(), "")))
        .enumerate()
    {
        if cidx == visible.start {
            vis_sta = Some(idx);
        }
        if cidx == visible.end {
            vis_end = Some(idx);
        }
        if cidx == selection.start {
            sel_sta = Some(idx);
        }
        if cidx == selection.end {
            sel_end = Some(idx);
        }
        if cidx == cursor {
            cur_sta = Some(idx);
            cur_len = Some(c.len())
        }
    }

    let vis_sta = vis_sta.expect("visible_start_not_found");
    let vis_end = vis_end.expect("visible_end_not_found");
    let sel_sta = sel_sta.expect("selection_start_not_found");
    let sel_end = sel_end.expect("selection_end_not_found");
    let cur_sta = cur_sta.expect("cursor_start_not_found");
    let cur_len = cur_len.expect("cursor_end_not_found");

    let before_str = &value[vis_sta..sel_sta];

    let (cursor1_str, sel_str) = if sel_sta == cur_sta && sel_sta + cur_len <= sel_end {
        (
            &value[cur_sta..cur_sta + cur_len],
            &value[sel_sta + cur_len..sel_end],
        )
    } else {
        (&value[sel_sta..sel_sta], &value[sel_sta..sel_end])
    };

    let (cursor2_str, after_str) = if cur_len == 0 {
        (" ", &value[sel_end..vis_end])
    } else if sel_end == cur_sta && sel_end + cur_len <= vis_end {
        (
            &value[cur_sta..cur_sta + cur_len],
            &value[sel_end + cur_len..vis_end],
        )
    } else {
        (&value[sel_end..sel_end], &value[sel_end..vis_end])
    };

    (before_str, cursor1_str, sel_str, cursor2_str, after_str)
}

/// Is the first char alphanumeric?
pub(crate) fn is_alphanumeric(s: &str) -> bool {
    if let Some(c) = s.chars().next() {
        c.is_alphanumeric()
    } else {
        false
    }
}
