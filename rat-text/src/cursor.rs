use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorType {
    TerminalCursor,
    RenderedCursor,
}

impl TryFrom<u64> for CursorType {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => CursorType::TerminalCursor,
            1 => CursorType::RenderedCursor,
            _ => return Err(()),
        })
    }
}

static CURSOR_TYPE: AtomicU64 = AtomicU64::new(CursorType::TerminalCursor as u64);

pub fn set_cursor_type(c: CursorType) {
    CURSOR_TYPE.store(c as u64, Ordering::Release);
}

pub fn cursor_type() -> CursorType {
    let cursor_type = CURSOR_TYPE.load(Ordering::Acquire);
    cursor_type.try_into().expect("cursor-type")
}
