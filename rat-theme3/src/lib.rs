use crate::palettes::{
    BASE16, BASE16_RELAX, BLACKWHITE, IMPERIAL, MONEKAI, MONOCHROME, OCEAN, OXOCARBON, RADIUM,
    RUST, SOLARIZED, TUNDRA, VSCODE_DARK,
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
#[allow(deprecated)]
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
use rat_widget::form::FormStyle;
pub use shell_theme::*;

/// Trait for a theme.
pub trait SalsaTheme {
    /// Theme name.
    fn name(&self) -> &str;

    /// Color palette.
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

    /// Text input base style.
    fn text_input(&self) -> Style;
    /// Text input with focus.
    fn text_focus(&self) -> Style;
    /// Text selection.
    fn text_select(&self) -> Style;

    /// Scroll style
    fn scroll_style(&self) -> ScrollStyle;

    /// Container base.
    fn container_base(&self) -> Style;
    /// Container border.
    fn container_border(&self) -> Style;
    /// Container scrollbar arrows.
    fn container_arrow(&self) -> Style;

    /// Background for popups.
    fn popup_base(&self) -> Style;
    /// Border for popups.
    fn popup_border(&self) -> Style;
    /// Popup scrollbar arrows.
    fn popup_arrow(&self) -> Style;
    /// Popup scroll style
    fn popup_scroll_style(&self) -> ScrollStyle;

    /// Background for dialogs.
    fn dialog_base(&self) -> Style;
    /// Border for dialogs.
    fn dialog_border(&self) -> Style;
    /// Dialog scrollbar arrows.
    fn dialog_arrow(&self) -> Style;
    /// Dialog scroll style
    fn dialog_scroll_style(&self) -> ScrollStyle;

    /// Style for the status line.
    fn status_base(&self) -> Style;

    /// Base style for buttons.
    fn button_base(&self) -> Style;
    /// Armed style for buttons.
    fn button_armed(&self) -> Style;

    /// Field label style.
    fn label_style(&self) -> Style;

    /// Any text fields as input widgets.
    fn text_style(&self) -> TextStyle;

    /// TextArea as input widget.
    fn textarea_style(&self) -> TextStyle;

    /// Choice.
    fn choice_style(&self) -> ChoiceStyle;

    /// Radiobutton.
    fn radio_style(&self) -> RadioStyle;

    /// Checkbox.
    fn checkbox_style(&self) -> CheckboxStyle;

    /// Slider.
    fn slider_style(&self) -> SliderStyle;

    /// Calendar.
    fn month_style(&self) -> CalendarStyle;

    /// Line numbers.
    fn line_nr_style(&self) -> LineNumberStyle;

    /// Buttons.
    fn button_style(&self) -> ButtonStyle;

    /// Table.
    fn table_style(&self) -> TableStyle;

    /// List.
    fn list_style(&self) -> ListStyle;

    /// Text style for view-only TextAreas.
    fn textview_style(&self) -> TextStyle;

    /// Paragraph.
    fn paragraph_style(&self) -> ParagraphStyle;

    /// Shadow.
    fn shadow_style(&self) -> ShadowStyle;

    /// Menus.
    fn menu_style(&self) -> MenuStyle;

    /// Split.
    fn split_style(&self) -> SplitStyle;

    /// View.
    fn view_style(&self) -> ViewStyle;

    /// Tabbed.
    fn tabbed_style(&self) -> TabbedStyle;

    /// Old school statusline styling.
    fn statusline_style(&self) -> Vec<Style>;

    /// Style for a StatusLine with 3 indicator fields.
    fn statusline_style_ext(&self) -> StatusLineStyle;

    /// FileDialog.
    fn file_dialog_style(&self) -> FileDialogStyle;

    /// MsgDialog.
    fn msg_dialog_style(&self) -> MsgDialogStyle;

    /// Pager.
    #[allow(deprecated)]
    fn pager_style(&self) -> PagerStyle;

    /// Form.
    fn form_style(&self) -> FormStyle;

    /// Clipper.
    fn clipper_style(&self) -> ClipperStyle;
}

// Create a theme + palette.
pub fn create_theme(theme: &str) -> Option<Box<dyn SalsaTheme>> {
    let theme: Box<dyn SalsaTheme> = match theme {
        "Imperial Dark" => Box::new(DarkTheme::new(theme, IMPERIAL)),
        "Radium Dark" => Box::new(DarkTheme::new(theme, RADIUM)),
        "Tundra Dark" => Box::new(DarkTheme::new(theme, TUNDRA)),
        "Ocean Dark" => Box::new(DarkTheme::new(theme, OCEAN)),
        "Monochrome Dark" => Box::new(DarkTheme::new(theme, MONOCHROME)),
        "Black&White Dark" => Box::new(DarkTheme::new(theme, BLACKWHITE)),
        "Base16 Dark" => Box::new(DarkTheme::new(theme, BASE16)),
        "Base16 Relax Dark" => Box::new(DarkTheme::new(theme, BASE16_RELAX)),
        "Monekai Dark" => Box::new(DarkTheme::new(theme, MONEKAI)),
        "Solarized Dark" => Box::new(DarkTheme::new(theme, SOLARIZED)),
        "OxoCarbon Dark" => Box::new(DarkTheme::new(theme, OXOCARBON)),
        "Rust Dark" => Box::new(DarkTheme::new(theme, RUST)),
        "VSCode Dark" => Box::new(DarkTheme::new(theme, VSCODE_DARK)),
        //
        "Imperial Shell" => Box::new(ShellTheme::new(theme, IMPERIAL)),
        "Radium Shell" => Box::new(ShellTheme::new(theme, RADIUM)),
        "Tundra Shell" => Box::new(ShellTheme::new(theme, TUNDRA)),
        "Ocean Shell" => Box::new(ShellTheme::new(theme, OCEAN)),
        "Monochrome Shell" => Box::new(ShellTheme::new(theme, MONOCHROME)),
        "Black&White Shell" => Box::new(ShellTheme::new(theme, BLACKWHITE)),
        "Base16 Shell" => Box::new(ShellTheme::new(theme, BASE16)),
        "Base16 Relax Shell" => Box::new(ShellTheme::new(theme, BASE16_RELAX)),
        "Monekai Shell" => Box::new(ShellTheme::new(theme, MONEKAI)),
        "Solarized Shell" => Box::new(ShellTheme::new(theme, SOLARIZED)),
        "OxoCarbon Shell" => Box::new(ShellTheme::new(theme, OXOCARBON)),
        "Rust Shell" => Box::new(ShellTheme::new(theme, RUST)),
        "VSCode Shell" => Box::new(ShellTheme::new(theme, VSCODE_DARK)),

        _ => return None,
    };

    Some(theme)
}

/// Get a Palette by name.
pub fn create_palette(name: &str) -> Option<Palette> {
    match name {
        "Imperial" => Some(IMPERIAL),
        "Radium" => Some(RADIUM),
        "Tundra" => Some(TUNDRA),
        "Ocean" => Some(OCEAN),
        "Monochrome" => Some(MONOCHROME),
        "Black & White" => Some(BLACKWHITE),
        "Base16" => Some(BASE16),
        "Base16 Relax" => Some(BASE16_RELAX),
        "Monekai" => Some(MONEKAI),
        "Solarized" => Some(SOLARIZED),
        "OxoCarbon" => Some(OXOCARBON),
        "Rust" => Some(RUST),
        "VSCode" => Some(VSCODE_DARK),
        _ => return None,
    }
}

const PALETTES: &[&str] = &[
    "Imperial",
    "Radium",
    "Tundra",
    "Ocean",
    "Monochrome",
    "Black & White",
    "Base16",
    "Base16 Relax",
    "Monekai",
    "Solarized",
    "OxoCarbon",
    "Rust",
    "VSCode",
];

const THEMES: &[&str] = &[
    "Imperial Dark",
    "Radium Dark",
    "Tundra Dark",
    "Ocean Dark",
    "Monochrome Dark",
    "Black & White Dark",
    "Base16 Dark",
    "Base16 Relax Dark",
    "Monekai Dark",
    "Solarized Dark",
    "OxoCarbon Dark",
    "Rust Dark",
    "VSCode Dark",
    //
    "Imperial Shell",
    "Radium Shell",
    "Tundra Shell",
    "Ocean Shell",
    "Monochrome Shell",
    "Black & White Shell",
    "Base16 Shell",
    "Base16 Relax Shell",
    "Monekai Shell",
    "Solarized Shell",
    "OxoCarbon Shell",
    "Rust Shell",
    "VSCode Shell",
];

pub fn salsa_themes() -> Vec<&'static str> {
    Vec::from(THEMES)
}

/// All currently existing color palettes.
pub fn salsa_palettes() -> Vec<&'static str> {
    Vec::from(PALETTES)
}
