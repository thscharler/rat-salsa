use crate::palette::Palette;
use crate::theme::SalsaTheme;
use crate::themes::create_dark;

/// A dark theme.
pub fn create_light(name: &str, p: Palette) -> SalsaTheme {
    // currently the same
    let mut theme = create_dark(name, p);
    theme.theme = "Light".into();
    theme
}
