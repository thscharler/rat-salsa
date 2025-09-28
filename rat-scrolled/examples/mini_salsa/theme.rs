use crate::mini_salsa::palette::{Contrast, Palette};
use rat_scrolled::ScrollStyle;
use ratatui::prelude::Style;
use ratatui::style::Color;

/// A sample theme for shell usage.
#[derive(Debug, Clone)]
pub struct ShellTheme {
    p: Palette,
    name: &'static str,
}

impl ShellTheme {
    pub const fn new(name: &'static str, p: Palette) -> Self {
        Self { p, name }
    }

    /// Create a style with only a text foreground color
    fn fg_style(&self, color: Color) -> Style {
        Style::new().fg(color)
    }
}

impl ShellTheme {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn palette(&self) -> &Palette {
        &self.p
    }

    /// Create a style from the given white shade.
    /// n is `0..8`
    pub fn white(&self, n: usize) -> Style {
        self.p.white(n, Contrast::Normal)
    }

    /// Create a style from the given black shade.
    /// n is `0..8`
    pub fn black(&self, n: usize) -> Style {
        self.p.black(n, Contrast::Normal)
    }

    /// Create a style from the given gray shade.
    /// n is `0..8`
    pub fn gray(&self, n: usize) -> Style {
        self.p.gray(n, Contrast::Normal)
    }

    /// Create a style from the given red shade.
    /// n is `0..8`
    pub fn red(&self, n: usize) -> Style {
        self.p.red(n, Contrast::Normal)
    }

    /// Create a style from the given orange shade.
    /// n is `0..8`
    pub fn orange(&self, n: usize) -> Style {
        self.p.orange(n, Contrast::Normal)
    }

    /// Create a style from the given yellow shade.
    /// n is `0..8`
    pub fn yellow(&self, n: usize) -> Style {
        self.p.yellow(n, Contrast::Normal)
    }

    /// Create a style from the given limegreen shade.
    /// n is `0..8`
    pub fn limegreen(&self, n: usize) -> Style {
        self.p.limegreen(n, Contrast::Normal)
    }

    /// Create a style from the given green shade.
    /// n is `0..8`
    pub fn green(&self, n: usize) -> Style {
        self.p.green(n, Contrast::Normal)
    }

    /// Create a style from the given bluegreen shade.
    /// n is `0..8`
    pub fn bluegreen(&self, n: usize) -> Style {
        self.p.bluegreen(n, Contrast::Normal)
    }

    /// Create a style from the given cyan shade.
    /// n is `0..8`
    pub fn cyan(&self, n: usize) -> Style {
        self.p.cyan(n, Contrast::Normal)
    }

    /// Create a style from the given blue shade.
    /// n is `0..8`
    pub fn blue(&self, n: usize) -> Style {
        self.p.blue(n, Contrast::Normal)
    }

    /// Create a style from the given deepblue shade.
    /// n is `0..8`
    pub fn deepblue(&self, n: usize) -> Style {
        self.p.deepblue(n, Contrast::Normal)
    }

    /// Create a style from the given purple shade.
    /// n is `0..8`
    pub fn purple(&self, n: usize) -> Style {
        self.p.purple(n, Contrast::Normal)
    }

    /// Create a style from the given magenta shade.
    /// n is `0..8`
    pub fn magenta(&self, n: usize) -> Style {
        self.p.magenta(n, Contrast::Normal)
    }

    /// Create a style from the given redpink shade.
    /// n is `0..8`
    pub fn redpink(&self, n: usize) -> Style {
        self.p.redpink(n, Contrast::Normal)
    }

    /// Create a style from the given primary shade.
    /// n is `0..8`
    pub fn primary(&self, n: usize) -> Style {
        self.p.primary(n, Contrast::Normal)
    }

    /// Create a style from the given secondary shade.
    /// n is `0..8`
    pub fn secondary(&self, n: usize) -> Style {
        self.p.secondary(n, Contrast::Normal)
    }

    pub fn focus(&self) -> Style {
        self.p.normal_contrast(self.p.primary[Palette::BRIGHT_1])
    }

    pub fn select(&self) -> Style {
        self.p.normal_contrast(self.p.secondary[Palette::BRIGHT_2])
    }

    pub fn text_input(&self) -> Style {
        self.p.normal_contrast(self.p.gray[Palette::BRIGHT_0])
    }

    pub fn text_focus(&self) -> Style {
        self.p.normal_contrast(self.p.gray[Palette::BRIGHT_3])
    }

    pub fn text_select(&self) -> Style {
        self.p.normal_contrast(self.p.secondary[Palette::BRIGHT_0])
    }

    pub fn table_base(&self) -> Style {
        Style::default().fg(self.p.white[1]).bg(self.p.black[0])
    }

    pub fn table_header(&self) -> Style {
        Style::default().fg(self.p.white[1]).bg(self.p.blue[2])
    }

    pub fn table_footer(&self) -> Style {
        Style::default().fg(self.p.white[1]).bg(self.p.blue[2])
    }

    /// Container base
    pub fn container_base(&self) -> Style {
        Default::default()
    }

    /// Container border
    pub fn container_border(&self) -> Style {
        Default::default()
    }

    /// Container arrows
    pub fn container_arrow(&self) -> Style {
        Default::default()
    }

    /// Background for popups.
    pub fn popup_base(&self) -> Style {
        self.p
            .style(self.p.gray[Palette::BRIGHT_0], Contrast::Normal)
    }

    /// Dialog arrows
    pub fn popup_border(&self) -> Style {
        self.popup_base().fg(self.p.gray[Palette::BRIGHT_0])
    }

    /// Dialog arrows
    pub fn popup_arrow(&self) -> Style {
        self.popup_base().fg(self.p.gray[Palette::BRIGHT_0])
    }

    /// Background for dialogs.
    pub fn dialog_base(&self) -> Style {
        self.p
            .style(self.p.gray[Palette::BRIGHT_1], Contrast::Normal)
    }

    /// Dialog arrows
    pub fn dialog_border(&self) -> Style {
        self.dialog_base().fg(self.p.white[Palette::BRIGHT_0])
    }

    /// Dialog arrows
    pub fn dialog_arrow(&self) -> Style {
        self.dialog_base().fg(self.p.white[Palette::BRIGHT_0])
    }

    /// Style for the status line.
    pub fn status_base(&self) -> Style {
        Default::default()
    }

    /// Base style for buttons.
    pub fn button_base(&self) -> Style {
        self.p
            .style(self.p.gray[Palette::BRIGHT_2], Contrast::Normal)
    }

    /// Armed style for buttons.
    pub fn button_armed(&self) -> Style {
        self.p
            .style(self.p.secondary[Palette::BRIGHT_0], Contrast::Normal)
    }

    pub fn block(&self) -> Style {
        Style::default().fg(self.p.gray[1]).bg(self.p.black[1])
    }

    pub fn block_title(&self) -> Style {
        Style::default().fg(self.p.secondary[1])
    }

    /// Text-label style.
    pub fn label_style(&self) -> Style {
        self.container_base()
    }

    /// Scroll style
    pub fn scroll_style(&self) -> ScrollStyle {
        ScrollStyle {
            thumb_style: Some(self.container_border()),
            track_style: Some(self.container_border()),
            min_style: Some(self.container_border()),
            begin_style: Some(self.container_arrow()),
            end_style: Some(self.container_arrow()),
            ..Default::default()
        }
    }

    /// Popup scroll style
    pub fn popup_scroll_style(&self) -> ScrollStyle {
        ScrollStyle {
            thumb_style: Some(self.popup_border()),
            track_style: Some(self.popup_border()),
            min_style: Some(self.popup_border()),
            begin_style: Some(self.popup_arrow()),
            end_style: Some(self.popup_arrow()),
            ..Default::default()
        }
    }

    /// Dialog scroll style
    pub fn dialog_scroll_style(&self) -> ScrollStyle {
        ScrollStyle {
            thumb_style: Some(self.dialog_border()),
            track_style: Some(self.dialog_border()),
            min_style: Some(self.dialog_border()),
            begin_style: Some(self.dialog_arrow()),
            end_style: Some(self.dialog_arrow()),
            ..Default::default()
        }
    }

    /// Complete StatusLineStyle for a StatusLine with 3 indicator fields.
    pub fn statusline_style(&self) -> Vec<Style> {
        vec![
            self.status_base(),
            self.fg_style(self.p.blue[Palette::BRIGHT_2]),
            self.fg_style(self.p.blue[Palette::BRIGHT_2]),
            self.fg_style(self.p.blue[Palette::BRIGHT_2]),
        ]
    }
}
