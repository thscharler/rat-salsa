/// Timer event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerEvent(pub crate::timer::TimeOut);

/// Event sent immediately after rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderedEvent;

/// Event sent immediately before quitting the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuitEvent;
