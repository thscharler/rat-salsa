use crate::palettes::{
    BASE16, BASE16_RELAXED, BLACKWHITE, IMPERIAL, MONEKAI, MONOCHROME, OCEAN, OXOCARBON, RADIUM,
    SOLARIZED, TUNDRA, VSCODE_DARK,
};
use rat_widget::button::ButtonStyle;
use rat_widget::calendar::CalendarStyle;
use rat_widget::checkbox::CheckboxStyle;
use rat_widget::choice::ChoiceStyle;
use rat_widget::clipper::ClipperStyle;
use rat_widget::file_dialog::FileDialogStyle;
use rat_widget::line_number::LineNumberStyle;
use rat_widget::list::ListStyle;
use rat_widget::menu::MenuStyle;
use rat_widget::msgdialog::MsgDialogStyle;
use rat_widget::pager::PagerStyle;
use rat_widget::paragraph::ParagraphStyle;
use rat_widget::radio::RadioStyle;
use rat_widget::scrolled::ScrollStyle;
use rat_widget::shadow::ShadowStyle;
use rat_widget::slider::SliderStyle;
use rat_widget::splitter::SplitStyle;
use rat_widget::statusline::StatusLineStyle;
use rat_widget::tabbed::TabbedStyle;
use rat_widget::table::TableStyle;
use rat_widget::text::TextStyle;
use rat_widget::view::ViewStyle;
use ratatui::prelude::Style;

mod dark_theme;
mod palette;
pub mod palettes;
mod shell_theme;

pub use dark_theme::*;
pub use palette::*;
pub use shell_theme::*;

/// Trait for a theme.
pub trait SalsaTheme {
    fn name(&self) -> &str;

    fn palette(&self) -> &Palette;

    /// Create a style from the given white shade.
    /// n is `0..8`
    fn white(&self, n: usize) -> Style;

    /// Create a style from the given black shade.
    /// n is `0..8`
    fn black(&self, n: usize) -> Style;

    /// Create a style from the given gray shade.
    /// n is `0..8`
    fn gray(&self, n: usize) -> Style;

    /// Create a style from the given red shade.
    /// n is `0..8`
    fn red(&self, n: usize) -> Style;

    /// Create a style from the given orange shade.
    /// n is `0..8`
    fn orange(&self, n: usize) -> Style;

    /// Create a style from the given yellow shade.
    /// n is `0..8`
    fn yellow(&self, n: usize) -> Style;

    /// Create a style from the given limegreen shade.
    /// n is `0..8`
    fn limegreen(&self, n: usize) -> Style;

    /// Create a style from the given green shade.
    /// n is `0..8`
    fn green(&self, n: usize) -> Style;

    /// Create a style from the given bluegreen shade.
    /// n is `0..8`
    fn bluegreen(&self, n: usize) -> Style;

    /// Create a style from the given cyan shade.
    /// n is `0..8`
    fn cyan(&self, n: usize) -> Style;

    /// Create a style from the given blue shade.
    /// n is `0..8`
    fn blue(&self, n: usize) -> Style;

    /// Create a style from the given deepblue shade.
    /// n is `0..8`
    fn deepblue(&self, n: usize) -> Style;

    /// Create a style from the given purple shade.
    /// n is `0..8`
    fn purple(&self, n: usize) -> Style;

    /// Create a style from the given magenta shade.
    /// n is `0..8`
    fn magenta(&self, n: usize) -> Style;

    /// Create a style from the given redpink shade.
    /// n is `0..8`
    fn redpink(&self, n: usize) -> Style;

    /// Create a style from the given primary shade.
    /// n is `0..8`
    fn primary(&self, n: usize) -> Style;

    /// Create a style from the given secondary shade.
    /// n is `0..8`
    fn secondary(&self, n: usize) -> Style;

    /// Focused style.
    fn focus(&self) -> Style;
    /// Selection style.
    fn select(&self) -> Style;

    /// Text input.
    fn text_input(&self) -> Style;
    /// Text input with focus.
    fn text_focus(&self) -> Style;
    /// Text selection.
    fn text_select(&self) -> Style;

    /// Container base
    fn container_base(&self) -> Style;

    /// Container border
    fn container_border(&self) -> Style;

    /// Container arrows
    fn container_arrow(&self) -> Style;

    /// Background for popups.
    fn popup_base(&self) -> Style;

    /// Dialog arrows
    fn popup_border(&self) -> Style;

    /// Dialog arrows
    fn popup_arrow(&self) -> Style;

    /// Background for dialogs.
    fn dialog_base(&self) -> Style;

    /// Dialog arrows
    fn dialog_border(&self) -> Style;

    /// Dialog arrows
    fn dialog_arrow(&self) -> Style;

    /// Style for the status line.
    fn status_base(&self) -> Style;

    /// Base style for buttons.
    fn button_base(&self) -> Style;

    /// Armed style for buttons.
    fn button_armed(&self) -> Style;

    /// Complete MonthStyle.
    fn month_style(&self) -> CalendarStyle;

    /// Style for shadows.
    fn shadow_style(&self) -> ShadowStyle;

    /// Style for LineNumbers.
    fn line_nr_style(&self) -> LineNumberStyle;

    /// Complete TextAreaStyle
    fn textarea_style(&self) -> TextStyle;

    /// Complete TextInputStyle
    fn text_style(&self) -> TextStyle;

    /// Text-label style.
    fn label_style(&self) -> Style;

    /// Paragraph style
    fn paragraph_style(&self) -> ParagraphStyle;

    /// Choice style.
    fn choice_style(&self) -> ChoiceStyle;

    /// Radiobutton style.
    fn radio_style(&self) -> RadioStyle;

    /// Complete CheckboxStyle
    fn checkbox_style(&self) -> CheckboxStyle;

    /// Slider Style
    fn slider_style(&self) -> SliderStyle;

    /// Complete MenuStyle
    fn menu_style(&self) -> MenuStyle;

    /// Complete ButtonStyle
    fn button_style(&self) -> ButtonStyle;

    /// Complete TableStyle
    fn table_style(&self) -> TableStyle;

    /// Complete ListStyle
    fn list_style(&self) -> ListStyle;

    /// Scroll style
    fn scroll_style(&self) -> ScrollStyle;

    /// Popup scroll style
    fn popup_scroll_style(&self) -> ScrollStyle;

    /// Dialog scroll style
    fn dialog_scroll_style(&self) -> ScrollStyle;

    /// Split style
    fn split_style(&self) -> SplitStyle;

    /// View style
    fn view_style(&self) -> ViewStyle;

    /// Tabbed style
    fn tabbed_style(&self) -> TabbedStyle;

    /// StatusLineStyle for a StatusLine with 3 indicator fields.
    fn statusline_style_ext(&self) -> StatusLineStyle;

    /// FileDialog style.
    fn file_dialog_style(&self) -> FileDialogStyle;

    /// Complete MsgDialogStyle.
    fn msg_dialog_style(&self) -> MsgDialogStyle;

    /// Pager style.
    fn pager_style(&self) -> PagerStyle;

    /// Clipper style.
    fn clipper_style(&self) -> ClipperStyle;
}

pub fn create_theme(theme: &str, palette: &str) -> Box<dyn SalsaTheme> {
    match palette {
        "Imperial"=> IMPERIAL,
        "Radium"=> RADIUM,
        "Tundra"=> TUNDRA,
        "Ocean"=> OCEAN,
        "Monochrome"=> MONOCHROME,
        "Black&White"=> BLACKWHITE,
        "Base16"=> BASE16,
        "Base16Relaxed"=> BASE16_RELAXED,
        "Monekai"=> MONEKAI,
        "Solarized"=> SOLARIZED,
        "OxoCarbon"=> OXOCARBON,
        "VSCodeDark"=> VSCODE_DARK,
    }

    match theme {
        "Dark" =>
        "Shell" =>
    }

}

pub fn salsa_themes() -> Vec<String> {
    vec!["Dark".to_string(), "Shell".to_string()]
}

/// All currently existing color palettes.
pub fn salsa_palettes() -> Vec<String> {
    vec![
        ("Imperial".to_string(),
        ("Radium".to_string(),
        ("Tundra".to_string(),
        ("Ocean".to_string(),
        ("Monochrome".to_string(),
        ("Black&White".to_string(),
        ("Base16".to_string(),
        ("Base16Relaxed".to_string(),
        ("Monekai".to_string(),
        ("Solarized".to_string(),
        ("OxoCarbon".to_string(),
        ("VSCodeDark".to_string(),
    ]
}

/// A list of DarkTheme for all color palettes.
pub fn dark_themes() -> Vec<DarkTheme> {
    vec![
        DarkTheme::new("Imperial", IMPERIAL),
        DarkTheme::new("Radium", RADIUM),
        DarkTheme::new("Tundra", TUNDRA),
        DarkTheme::new("Ocean", OCEAN),
        DarkTheme::new("Monochrome", MONOCHROME),
        DarkTheme::new("Black&White", BLACKWHITE),
        DarkTheme::new("Base16", BASE16),
        DarkTheme::new("Base16Relaxed", BASE16_RELAXED),
        DarkTheme::new("Monekai", MONEKAI),
        DarkTheme::new("Solarized", SOLARIZED),
        DarkTheme::new("Oxocarbon", OXOCARBON),
        DarkTheme::new("VSCodeDark", VSCODE_DARK),
    ]
}

/// A list of ShellTheme for all color palettes.
pub fn shell_themes() -> Vec<ShellTheme> {
    vec![
        ShellTheme::new("Imperial", IMPERIAL),
        ShellTheme::new("Radium", RADIUM),
        ShellTheme::new("Tundra", TUNDRA),
        ShellTheme::new("Ocean", OCEAN),
        ShellTheme::new("Monochrome", MONOCHROME),
        ShellTheme::new("Black&White", BLACKWHITE),
        ShellTheme::new("Base16", BASE16),
        ShellTheme::new("Base16Relaxed", BASE16_RELAXED),
        ShellTheme::new("Monekai", MONEKAI),
        ShellTheme::new("Solarized", SOLARIZED),
        ShellTheme::new("Oxocarbon", OXOCARBON),
        ShellTheme::new("VSCodeDark", VSCODE_DARK),
    ]
}
