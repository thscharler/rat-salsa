use crate::palette::Palette;
use crate::theme::SalsaTheme;
use crate::themes::create_dark;

/// A dark theme.
pub fn create_light(p: Palette) -> SalsaTheme {
    // currently the same as dark
    create_dark(p)
}
