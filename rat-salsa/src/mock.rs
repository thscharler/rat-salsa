
/// Empty placeholder for [run_tui](crate::run_tui).
pub fn init<State, Global, Error>(
    _state: &mut State, //
    _ctx: &mut Global,
) -> Result<(), Error> {
    Ok(())
}

/// Empty placeholder for [run_tui](crate::run_tui).
pub fn error<Global, State, Event, Error>(
    _error: Error,
    _state: &mut State,
    _ctx: &mut Global,
) -> Result<crate::Control<Event>, Error> {
    Ok(crate::Control::Continue)
}