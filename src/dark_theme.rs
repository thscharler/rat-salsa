use crate::Scheme;
use rat_widget::button::ButtonStyle;
use rat_widget::input::TextInputStyle;
use rat_widget::list::ListStyle;
use rat_widget::masked_input::MaskedInputStyle;
use rat_widget::menuline::MenuStyle;
use rat_widget::msgdialog::MsgDialogStyle;
use rat_widget::scrolled::ScrolledStyle;
use rat_widget::table::FTableStyle;
use rat_widget::textarea::TextAreaStyle;
use ratatui::prelude::Style;

/// One sample theme which prefers dark colors from the color-scheme
/// and generates styles for widgets.
///
/// The widget set fits for the widgets provided by
/// [rat-widget](https://www.docs.rs/rat-widget), for other needs
/// take it as an idea for your own implementation.
///
#[derive(Debug, Clone)]
pub struct DarkTheme {
    s: Scheme,
    name: String,
}

impl DarkTheme {
    pub fn new(name: String, s: Scheme) -> Self {
        Self { s, name }
    }
}

impl DarkTheme {
    /// Some display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Hint at dark.
    pub fn dark_theme(&self) -> bool {
        true
    }

    /// The underlying scheme.
    pub fn scheme(&self) -> &Scheme {
        &self.s
    }

    /// Create a style from the given white shade.
    /// n is `0..=3`
    pub fn white(&self, n: usize) -> Style {
        self.s.style(self.s.white[n])
    }

    /// Create a style from the given black shade.
    /// n is `0..=3`
    pub fn black(&self, n: usize) -> Style {
        self.s.style(self.s.black[n])
    }

    /// Create a style from the given gray shade.
    /// n is `0..=3`
    pub fn gray(&self, n: usize) -> Style {
        self.s.style(self.s.gray[n])
    }

    /// Create a style from the given red shade.
    /// n is `0..=3`
    pub fn red(&self, n: usize) -> Style {
        self.s.style(self.s.red[n])
    }

    /// Create a style from the given orange shade.
    /// n is `0..=3`
    pub fn orange(&self, n: usize) -> Style {
        self.s.style(self.s.orange[n])
    }

    /// Create a style from the given yellow shade.
    /// n is `0..=3`
    pub fn yellow(&self, n: usize) -> Style {
        self.s.style(self.s.yellow[n])
    }

    /// Create a style from the given limegreen shade.
    /// n is `0..=3`
    pub fn limegreen(&self, n: usize) -> Style {
        self.s.style(self.s.limegreen[n])
    }

    /// Create a style from the given green shade.
    /// n is `0..=3`
    pub fn green(&self, n: usize) -> Style {
        self.s.style(self.s.green[n])
    }

    /// Create a style from the given bluegreen shade.
    /// n is `0..=3`
    pub fn bluegreen(&self, n: usize) -> Style {
        self.s.style(self.s.bluegreen[n])
    }

    /// Create a style from the given cyan shade.
    /// n is `0..=3`
    pub fn cyan(&self, n: usize) -> Style {
        self.s.style(self.s.cyan[n])
    }

    /// Create a style from the given blue shade.
    /// n is `0..=3`
    pub fn blue(&self, n: usize) -> Style {
        self.s.style(self.s.blue[n])
    }

    /// Create a style from the given deepblue shade.
    /// n is `0..=3`
    pub fn deepblue(&self, n: usize) -> Style {
        self.s.style(self.s.deepblue[n])
    }

    /// Create a style from the given purple shade.
    /// n is `0..=3`
    pub fn purple(&self, n: usize) -> Style {
        self.s.style(self.s.purple[n])
    }

    /// Create a style from the given magenta shade.
    /// n is `0..=3`
    pub fn magenta(&self, n: usize) -> Style {
        self.s.style(self.s.magenta[n])
    }

    /// Create a style from the given redpink shade.
    /// n is `0..=3`
    pub fn redpink(&self, n: usize) -> Style {
        self.s.style(self.s.redpink[n])
    }

    /// Create a style from the given primary shade.
    /// n is `0..=3`
    pub fn primary(&self, n: usize) -> Style {
        self.s.style(self.s.primary[n])
    }

    /// Create a style from the given secondary shade.
    /// n is `0..=3`
    pub fn secondary(&self, n: usize) -> Style {
        self.s.style(self.s.secondary[n])
    }

    /// Focus style
    pub fn focus(&self) -> Style {
        let bg = self.s.primary[2];
        Style::default().fg(self.s.text_color(bg)).bg(bg)
    }

    /// Selection style
    pub fn select(&self) -> Style {
        let bg = self.s.secondary[1];
        Style::default().fg(self.s.text_color(bg)).bg(bg)
    }

    /// Text field style.
    pub fn text_input(&self) -> Style {
        Style::default().fg(self.s.black[0]).bg(self.s.gray[3])
    }

    /// Focused text field style.
    pub fn text_focus(&self) -> Style {
        let bg = self.s.primary[0];
        Style::default().fg(self.s.text_color(bg)).bg(bg)
    }

    /// Text selection style.
    pub fn text_select(&self) -> Style {
        let bg = self.s.secondary[0];
        Style::default().fg(self.s.text_color(bg)).bg(bg)
    }

    /// Data display style. Used for lists, tables, ...
    pub fn data(&self) -> Style {
        Style::default().fg(self.s.white[0]).bg(self.s.black[1])
    }

    /// Background for dialogs.
    pub fn dialog_style(&self) -> Style {
        Style::default().fg(self.s.white[2]).bg(self.s.gray[1])
    }

    /// Style for the status line.
    pub fn status_style(&self) -> Style {
        Style::default().fg(self.s.white[0]).bg(self.s.black[2])
    }

    /// Complete TextAreaStyle
    pub fn textarea_style(&self) -> TextAreaStyle {
        TextAreaStyle {
            style: self.data(),
            focus: Some(self.focus()),
            select: Some(self.text_select()),
            ..TextAreaStyle::default()
        }
    }

    /// Complete TextInputStyle
    pub fn input_style(&self) -> TextInputStyle {
        TextInputStyle {
            style: self.text_input(),
            focus: Some(self.text_focus()),
            select: Some(self.text_select()),
            ..TextInputStyle::default()
        }
    }

    /// Complete MaskedInputStyle
    pub fn inputmask_style(&self) -> MaskedInputStyle {
        MaskedInputStyle {
            style: self.text_input(),
            focus: Some(self.text_focus()),
            select: Some(self.text_select()),
            invalid: Some(Style::default().bg(self.s.red[3])),
            ..Default::default()
        }
    }

    /// Complete MenuStyle
    pub fn menu_style(&self) -> MenuStyle {
        let menu = Style::default().fg(self.s.white[3]).bg(self.s.black[2]);
        MenuStyle {
            style: menu,
            title: Some(Style::default().fg(self.s.black[0]).bg(self.s.yellow[2])),
            select: Some(menu),
            focus: Some(self.focus()),
            ..Default::default()
        }
    }

    /// Complete FTableStyle
    pub fn table_style(&self) -> FTableStyle {
        FTableStyle {
            style: self.data(),
            select_row_style: Some(self.select()),
            show_row_focus: true,
            focus_style: Some(self.focus()),
            ..Default::default()
        }
    }

    /// Complete ListStyle
    pub fn list_style(&self) -> ListStyle {
        ListStyle {
            style: self.data(),
            select_style: self.select(),
            focus_style: self.focus(),
            ..Default::default()
        }
    }

    /// Complete ButtonStyle
    pub fn button_style(&self) -> ButtonStyle {
        ButtonStyle {
            style: Style::default().fg(self.s.white[0]).bg(self.s.primary[0]),
            focus: Some(self.focus()),
            armed: Some(Style::default().fg(self.s.black[0]).bg(self.s.secondary[0])),
            ..Default::default()
        }
    }

    /// Complete ScrolledStyle
    pub fn scrolled_style(&self) -> ScrolledStyle {
        let style = Style::default().fg(self.s.gray[0]).bg(self.s.black[1]);
        let arrow_style = Style::default().fg(self.s.secondary[0]).bg(self.s.black[1]);
        ScrolledStyle {
            thumb_style: Some(style),
            track_style: Some(style),
            begin_style: Some(arrow_style),
            end_style: Some(arrow_style),
            ..Default::default()
        }
    }

    /// Complete StatusLineStyle for a StatusLine with 3 indicator fields.
    /// This is what I need for the
    /// [minimal](https://github.com/thscharler/rat-salsa/blob/master/examples/minimal.rs)
    /// example, which shows timings for Render/Event/Action.
    pub fn statusline_style(&self) -> Vec<Style> {
        let s = &self.s;
        vec![
            self.status_style(),
            Style::default().fg(s.text_color(s.white[0])).bg(s.blue[3]),
            Style::default().fg(s.text_color(s.white[0])).bg(s.blue[2]),
            Style::default().fg(s.text_color(s.white[0])).bg(s.blue[1]),
        ]
    }

    /// Complete MsgDialogStyle.
    pub fn msg_dialog_style(&self) -> MsgDialogStyle {
        MsgDialogStyle {
            style: self.status_style(),
            button: self.button_style(),
            ..Default::default()
        }
    }
}
