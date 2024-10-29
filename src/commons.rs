/// Handle for an area.
/// Can be used to get a stored area.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct AreaHandle(pub(crate) usize);
