use crossterm::event::{Event, KeyCode, KeyEvent};
use log::debug;
use ratatui::text::Span;
use std::cmp::min;
use std::ops::Range;

/// Sum all widths.
pub(crate) fn span_width(spans: &[Span<'_>]) -> u16 {
    spans.iter().map(|v| v.width() as u16).sum()
}

/// Constrains the range to the visible range and shifts the result by offset.
pub(crate) fn clamp_shift(range: Range<usize>, offset: usize, width: usize) -> Range<usize> {
    let start = if range.start < offset {
        offset
    } else {
        range.start
    };
    let end = if range.end > offset + width {
        offset + width
    } else {
        range.end
    };

    start - offset..end - offset
}

/// Select previous.
pub(crate) fn prev_opt(select: Option<usize>, change: usize) -> Option<usize> {
    if let Some(select) = select {
        Some(prev(select, change))
    } else {
        Some(0)
    }
}

/// Select next.
pub(crate) fn next_opt(selected: Option<usize>, change: usize, max: usize) -> Option<usize> {
    if let Some(select) = selected {
        Some(next(select, change, max))
    } else {
        Some(0)
    }
}

/// Select previous.
pub(crate) fn prev(select: usize, change: usize) -> usize {
    select.saturating_sub(change)
}

/// Select next.
pub(crate) fn next(select: usize, change: usize, max: usize) -> usize {
    min(select + change, max)
}

/// Next but circle around.
pub(crate) fn next_circular(select: usize, max: usize) -> usize {
    if select + 1 < max {
        select + 1
    } else {
        0
    }
}

/// Prev but circle around.
pub(crate) fn prev_circular(select: usize, max: usize) -> usize {
    if select > 0 {
        select - 1
    } else {
        max - 1
    }
}

#[allow(dead_code)]
pub(crate) fn log_key(s: &str, evt: &Event) {
    if matches!(
        evt,
        Event::Key(KeyEvent {
            code: KeyCode::Esc,
            ..
        })
    ) {
        debug!("{} {:?}", s, evt);
    }
}
