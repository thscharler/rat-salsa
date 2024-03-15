use crossterm::event::{Event, KeyCode, KeyEvent};
use log::debug;
use ratatui::text::Span;

/// Sum all widths.
pub(crate) fn span_width(spans: &[Span<'_>]) -> u16 {
    spans.iter().map(|v| v.width() as u16).sum()
}

/// Clamp the selection, invalid select values change to None.
pub(crate) fn clamp_opt(select: Option<usize>, max: usize) -> Option<usize> {
    if let Some(select) = select {
        if max == 0 {
            None
        } else if select >= max {
            Some(max - 1)
        } else {
            Some(select)
        }
    } else {
        None
    }
}

/// Select previous.
pub(crate) fn prev_opt(select: Option<usize>) -> Option<usize> {
    if let Some(select) = select {
        if select > 0 {
            Some(select - 1)
        } else {
            Some(0)
        }
    } else {
        Some(0)
    }
}

/// Select next.
pub(crate) fn next_opt(select: Option<usize>, max: usize) -> Option<usize> {
    if let Some(select) = select {
        if select + 1 < max {
            Some(select + 1)
        } else {
            Some(select)
        }
    } else {
        Some(0)
    }
}

/// Select previous.
pub(crate) fn prev_pg_opt(select: Option<usize>, pg: usize) -> Option<usize> {
    if let Some(select) = select {
        if select >= pg {
            Some(select - pg)
        } else {
            Some(0)
        }
    } else {
        Some(0)
    }
}

/// Select next.
pub(crate) fn next_pg_opt(select: Option<usize>, pg: usize, max: usize) -> Option<usize> {
    if let Some(select) = select {
        if select + pg < max {
            Some(select + pg)
        } else {
            Some(max - 1)
        }
    } else {
        Some(0)
    }
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
