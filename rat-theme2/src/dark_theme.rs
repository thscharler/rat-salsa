//!
//! Implements a dark theme.
//!

use crate::{Contrast, Palette};
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
use rat_widget::paragraph::ParagraphStyle;
use rat_widget::radio::{RadioLayout, RadioStyle};
use rat_widget::scrolled::ScrollStyle;
use rat_widget::shadow::{ShadowDirection, ShadowStyle};
use rat_widget::slider::SliderStyle;
use rat_widget::splitter::SplitStyle;
use rat_widget::statusline::StatusLineStyle;
use rat_widget::tabbed::TabbedStyle;
use rat_widget::table::TableStyle;
use rat_widget::text::TextStyle;
use rat_widget::view::ViewStyle;
use ratatui::layout::Alignment;
use ratatui::style::Color;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders};
use std::borrow::Cow;
use std::time::Duration;

/// One sample theme which prefers dark colors from the color-palette
/// and generates styles for widgets.
///
/// The widget set fits for the widgets provided by
/// [rat-widget](https://www.docs.rs/rat-widget), for other needs
/// take it as an idea for your own implementation.
///
#[derive(Debug, Clone)]
pub struct DarkTheme {
    p: Palette,
    name: &'static str,
}

impl DarkTheme {
    pub const fn new(name: &'static str, s: Palette) -> Self {
        Self { p: s, name }
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

    /// The underlying palette.
    pub fn palette(&self) -> &Palette {
        &self.p
    }

    /// Create a style from a background color
    pub fn style(&self, bg: Color) -> Style {
        self.p.style(bg, Contrast::Normal)
    }

    /// Create a style from a background color
    pub fn high_style(&self, bg: Color) -> Style {
        self.p.style(bg, Contrast::High)
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

    /// Focus style
    pub fn focus(&self) -> Style {
        self.high_style(self.p.primary[2])
    }

    /// Selection style
    pub fn select(&self) -> Style {
        self.high_style(self.p.secondary[1])
    }

    /// Text field style.
    pub fn text_input(&self) -> Style {
        self.high_style(self.p.gray[3])
    }

    /// Focused text field style.
    pub fn text_focus(&self) -> Style {
        self.high_style(self.p.primary[1])
    }

    /// Text selection style.
    pub fn text_select(&self) -> Style {
        self.high_style(self.p.secondary[1])
    }

    /// Container base
    pub fn container_base(&self) -> Style {
        self.style(self.p.black[1])
    }

    /// Container border
    pub fn container_border(&self) -> Style {
        self.container_base().fg(self.p.gray[0])
    }

    /// Container arrows
    pub fn container_arrow(&self) -> Style {
        self.container_base().fg(self.p.gray[0])
    }

    /// Background for popups.
    pub fn popup_base(&self) -> Style {
        self.style(self.p.white[0])
    }

    /// Label text inside container.
    pub fn popup_label(&self) -> Style {
        self.style(self.p.white[0])
    }

    /// Dialog arrows
    pub fn popup_border(&self) -> Style {
        self.popup_base().fg(self.p.gray[0])
    }

    /// Dialog arrows
    pub fn popup_arrow(&self) -> Style {
        self.popup_base().fg(self.p.gray[0])
    }

    /// Background for dialogs.
    pub fn dialog_base(&self) -> Style {
        self.style(self.p.gray[1])
    }

    /// Label text inside container.
    pub fn dialog_label(&self) -> Style {
        self.dialog_base()
    }

    /// Dialog arrows
    pub fn dialog_border(&self) -> Style {
        self.dialog_base().fg(self.p.white[0])
    }

    /// Dialog arrows
    pub fn dialog_arrow(&self) -> Style {
        self.dialog_base().fg(self.p.white[0])
    }

    /// Style for the status line.
    pub fn status_base(&self) -> Style {
        self.style(self.p.black[2])
    }

    /// Base style for buttons.
    pub fn button_base(&self) -> Style {
        self.style(self.p.gray[2])
    }

    /// Armed style for buttons.
    pub fn button_armed(&self) -> Style {
        self.style(self.p.secondary[0])
    }

    /// Complete MonthStyle.
    pub fn month_style(&self) -> CalendarStyle {
        CalendarStyle {
            style: self.style(self.p.black[2]),
            title: None,
            weeknum: Some(Style::new().fg(self.p.limegreen[2])),
            weekday: Some(Style::new().fg(self.p.limegreen[2])),
            day: None,
            select: Some(self.select()),
            focus: Some(self.focus()),
            ..CalendarStyle::default()
        }
    }

    /// Style for shadows.
    pub fn shadow_style(&self) -> ShadowStyle {
        ShadowStyle {
            style: Style::new().bg(self.p.black[0]),
            dir: ShadowDirection::BottomRight,
            ..ShadowStyle::default()
        }
    }

    /// Style for LineNumbers.
    pub fn line_nr_style(&self) -> LineNumberStyle {
        LineNumberStyle {
            style: self.container_base().fg(self.p.gray[1]),
            cursor: Some(self.text_select()),
            ..LineNumberStyle::default()
        }
    }

    /// Complete TextAreaStyle
    pub fn textarea_style(&self) -> TextStyle {
        TextStyle {
            style: self.container_base(),
            focus: Some(self.focus()),
            select: Some(self.text_select()),
            scroll: Some(self.scroll_style()),
            border_style: Some(self.container_border()),
            ..TextStyle::default()
        }
    }

    /// Complete TextInputStyle
    pub fn text_style(&self) -> TextStyle {
        TextStyle {
            style: self.text_input(),
            focus: Some(self.text_focus()),
            select: Some(self.text_select()),
            invalid: Some(Style::default().bg(self.p.red[3])),
            ..TextStyle::default()
        }
    }

    /// Text-label style.
    pub fn label_style(&self) -> Style {
        self.container_base()
    }

    pub fn paragraph_style(&self) -> ParagraphStyle {
        ParagraphStyle {
            style: self.container_base(),
            focus: Some(self.focus()),
            scroll: Some(self.scroll_style()),
            ..Default::default()
        }
    }

    pub fn choice_style(&self) -> ChoiceStyle {
        ChoiceStyle {
            style: self.text_input(),
            select: Some(self.text_select()),
            focus: Some(self.text_focus()),
            popup_style: Some(self.popup_base()),
            popup_border: Some(self.popup_border()),
            popup_scroll: Some(self.popup_scroll_style()),
            popup_block: Some(
                Block::bordered()
                    .borders(Borders::LEFT)
                    .border_style(self.popup_border()),
            ),
            ..Default::default()
        }
    }

    pub fn radio_style(&self) -> RadioStyle {
        RadioStyle {
            layout: Some(RadioLayout::Stacked),
            style: self.text_input(),
            focus: Some(self.text_focus()),
            ..Default::default()
        }
    }

    /// Complete CheckboxStyle
    pub fn checkbox_style(&self) -> CheckboxStyle {
        CheckboxStyle {
            style: self.text_input(),
            focus: Some(self.text_focus()),
            ..Default::default()
        }
    }

    /// Slider Style
    pub fn slider_style(&self) -> SliderStyle {
        SliderStyle {
            style: self.text_input(),
            bounds: Some(self.gray(2)),
            knob: Some(self.select()),
            focus: Some(self.focus()),
            text_align: Some(Alignment::Center),
            ..Default::default()
        }
    }

    /// Complete MenuStyle
    pub fn menu_style(&self) -> MenuStyle {
        let menu = Style::default().fg(self.p.white[3]).bg(self.p.black[2]);
        MenuStyle {
            style: menu,
            title: Some(Style::default().fg(self.p.black[0]).bg(self.p.yellow[2])),
            focus: Some(self.focus()),
            right: Some(Style::default().fg(self.p.bluegreen[0])),
            disabled: Some(Style::default().fg(self.p.gray[0])),
            highlight: Some(Style::default().underlined()),
            popup_style: Some(menu),
            block: Some(Block::bordered()),
            popup: Default::default(),
            ..Default::default()
        }
    }

    /// Complete ButtonStyle
    pub fn button_style(&self) -> ButtonStyle {
        ButtonStyle {
            style: self.button_base(),
            focus: Some(self.focus()),
            armed: Some(self.select()),
            hover: Some(self.select()),
            armed_delay: Some(Duration::from_millis(50)),
            ..Default::default()
        }
    }

    /// Complete TableStyle
    pub fn table_style(&self) -> TableStyle {
        TableStyle {
            style: self.container_base(),
            select_row: Some(self.select()),
            show_row_focus: true,
            focus_style: Some(self.focus()),
            border_style: Some(self.container_border()),
            scroll: Some(self.scroll_style()),
            ..Default::default()
        }
    }

    pub fn table_header(&self) -> Style {
        self.style(self.p.blue[2])
    }

    pub fn table_footer(&self) -> Style {
        self.style(self.p.blue[2])
    }

    /// Complete ListStyle
    pub fn list_style(&self) -> ListStyle {
        ListStyle {
            style: self.container_base(),
            select: Some(self.select()),
            focus: Some(self.focus()),
            scroll: Some(self.scroll_style()),
            ..Default::default()
        }
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

    /// Split style
    pub fn split_style(&self) -> SplitStyle {
        SplitStyle {
            style: self.container_border(),
            arrow_style: Some(self.container_arrow()),
            drag_style: Some(self.focus()),
            ..Default::default()
        }
    }

    /// View style
    pub fn view_style(&self) -> ViewStyle {
        ViewStyle {
            scroll: Some(self.scroll_style()),
            ..Default::default()
        }
    }

    /// Tabbed style
    pub fn tabbed_style(&self) -> TabbedStyle {
        let style = self.high_style(self.p.black[1]);
        TabbedStyle {
            style,
            tab: Some(self.gray(1)),
            select: Some(self.style(self.p.primary[4])),
            focus: Some(self.focus()),
            ..Default::default()
        }
    }

    /// Complete StatusLineStyle for a StatusLine with 3 indicator fields.
    /// This is what I need for the
    /// [minimal](https://github.com/thscharler/rat-salsa/blob/master/examples/minimal.rs)
    /// example, which shows timings for Render/Event/Action.
    pub fn statusline_style(&self) -> Vec<Style> {
        vec![
            self.status_base(),
            self.p.normal_contrast(self.p.white[0]).bg(self.p.blue[3]),
            self.p.normal_contrast(self.p.white[0]).bg(self.p.blue[2]),
            self.p.normal_contrast(self.p.white[0]).bg(self.p.blue[1]),
        ]
    }

    /// StatusLineStyle for a StatusLine with 3 indicator fields.
    pub fn statusline_style_ext(&self) -> StatusLineStyle {
        StatusLineStyle {
            sep: Some(Cow::Borrowed("|")),
            styles: vec![
                self.status_base(),
                self.p.normal_contrast(self.p.white[0]).bg(self.p.blue[3]),
                self.p.normal_contrast(self.p.white[0]).bg(self.p.blue[2]),
                self.p.normal_contrast(self.p.white[0]).bg(self.p.blue[1]),
            ],
            ..Default::default()
        }
    }

    /// FileDialog style.
    pub fn file_dialog_style(&self) -> FileDialogStyle {
        FileDialogStyle {
            style: self.dialog_base(),
            list: Some(self.list_style()),
            roots: Some(ListStyle {
                style: self.dialog_base(),
                ..self.list_style()
            }),
            text: Some(self.text_style()),
            button: Some(self.button_style()),
            block: Some(Block::bordered()),
            ..Default::default()
        }
    }

    /// Complete MsgDialogStyle.
    pub fn msg_dialog_style(&self) -> MsgDialogStyle {
        MsgDialogStyle {
            style: self.dialog_base(),
            button: Some(self.button_style()),
            ..Default::default()
        }
    }

    /// Clipper style.
    pub fn clipper_style(&self) -> ClipperStyle {
        ClipperStyle {
            style: self.container_base(),
            scroll: Some(self.scroll_style()),
            ..Default::default()
        }
    }
}
