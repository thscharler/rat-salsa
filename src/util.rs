use crossterm::event::{Event, KeyCode, KeyEvent};
use log::debug;
use ratatui::text::Span;
use std::mem;
use std::mem::MaybeUninit;

/// Sum all widths.
pub fn span_width(spans: &Vec<Span>) -> u16 {
    spans.iter().map(|v| v.width() as u16).sum()
}

/// Clamp
pub fn clamp_u16(v: u16, max: u16) -> u16 {
    if v >= max {
        max - 1
    } else {
        v
    }
}

/// Clamp the selection, invalid select values change to None.
pub fn clamp_opt(select: Option<usize>, max: usize) -> Option<usize> {
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
pub fn prev_opt(select: Option<usize>) -> Option<usize> {
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
pub fn next_opt(select: Option<usize>, max: usize) -> Option<usize> {
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
pub fn prev_pg_opt(select: Option<usize>, pg: usize) -> Option<usize> {
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
pub fn next_pg_opt(select: Option<usize>, pg: usize, max: usize) -> Option<usize> {
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
pub fn next_circular(select: usize, max: usize) -> usize {
    if select + 1 < max {
        select + 1
    } else {
        0
    }
}

/// Prev but circle around.
pub fn prev_circular(select: usize, max: usize) -> usize {
    if select > 0 {
        select - 1
    } else {
        max - 1
    }
}

#[inline]
pub(super) fn split_tuple<A, B, const N: usize>(t: [(A, B); N]) -> ([A; N], [B; N]) {
    let mut a: [MaybeUninit<A>; N] = uninit_array();
    let mut b: [MaybeUninit<B>; N] = uninit_array();

    for (i, (va, vb)) in t.into_iter().enumerate() {
        a[i].write(va);
        b[i].write(vb);
    }

    let aa = unsafe { assume_init_array(a) };
    let bb = unsafe { assume_init_array(b) };

    (aa, bb)
}

#[inline]
pub(super) fn uninit_array<T: Sized, const N: usize>() -> [MaybeUninit<T>; N] {
    unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() }
}

#[inline]
pub(super) unsafe fn assume_init_array<T: Sized, const N: usize>(a: [MaybeUninit<T>; N]) -> [T; N] {
    // copy is not optimal in the general case, but for this i'm fine with it.
    unsafe { mem::transmute_copy(&a) }
}

pub fn log_key(s: &str, evt: &Event) {
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
