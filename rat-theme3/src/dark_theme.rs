//!
//! Implements a dark theme.
//!

use crate::{Contrast, Palette, SalsaTheme};
use rat_widget::button::ButtonStyle;
use rat_widget::calendar::CalendarStyle;
use rat_widget::checkbox::CheckboxStyle;
use rat_widget::choice::ChoiceStyle;
use rat_widget::clipper::ClipperStyle;
use rat_widget::file_dialog::FileDialogStyle;
use rat_widget::form::FormStyle;
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
#[cfg(feature = "serde")]
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
#[cfg(feature = "serde")]
use serde::ser::SerializeStruct;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;
use std::time::Duration;

/// One sample theme which prefers dark colors from the color-palette
/// and generates styles for widgets.
///
/// The widget set fits for the widgets provided by
/// [rat-widget](https://www.docs.rs/rat-widget), for other needs
/// take it as an idea for your own implementation.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DarkTheme {
    name: Box<str>,
    p: Palette,
}

#[cfg(feature = "serde")]
impl Serialize for DarkTheme {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut theme = ser.serialize_struct("DarkTheme", 2)?;
        theme.serialize_field("name", &self.name)?;
        theme.serialize_field("p", &self.p)?;
        theme.end()
    }
}

#[cfg(feature = "serde")]
struct DarkThemeVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for DarkThemeVisitor {
    type Value = DarkTheme;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "struct DarkTheme")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let name = seq
            .next_element::<Box<str>>()?
            .ok_or(A::Error::invalid_length(0, &"DarkTheme.name"))?;
        let p = seq
            .next_element::<Palette>()?
            .ok_or(A::Error::invalid_length(0, &"DarkTheme.p"))?;
        Ok(DarkTheme { name, p })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut name = None;
        let mut p = None;

        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "name" => name = Some(map.next_value::<Box<str>>()?),
                "p" => p = Some(map.next_value::<Palette>()?),
                _ => {}
            }
        }
        let name = name.ok_or(A::Error::missing_field("name"))?;
        let p = p.ok_or(A::Error::missing_field("p"))?;

        Ok(DarkTheme { name, p })
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for DarkTheme {
    fn deserialize<D>(des: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["name", "p"];
        des.deserialize_struct("DarkTheme", FIELDS, DarkThemeVisitor)
    }
}

impl DarkTheme {
    pub fn new(name: &str, s: Palette) -> Self {
        Self {
            name: Box::from(name),
            p: s,
        }
    }

    /// Create a style from a background color
    fn style(&self, bg: Color) -> Style {
        self.p.style(bg, Contrast::Normal)
    }

    /// Create a style from a background color
    fn high_style(&self, bg: Color) -> Style {
        self.p.style(bg, Contrast::High)
    }

    fn table_header(&self) -> Style {
        self.style(self.p.blue[2])
    }

    fn table_footer(&self) -> Style {
        self.style(self.p.blue[2])
    }
}

impl SalsaTheme for DarkTheme {
    /// Some display name.
    fn name(&self) -> &str {
        &self.name
    }

    /// The underlying palette.
    fn palette(&self) -> &Palette {
        &self.p
    }

    /// Create a style from the given white shade.
    /// n is `0..8`
    fn white(&self, n: usize) -> Style {
        self.p.white(n, Contrast::Normal)
    }

    /// Create a style from the given black shade.
    /// n is `0..8`
    fn black(&self, n: usize) -> Style {
        self.p.black(n, Contrast::Normal)
    }

    /// Create a style from the given gray shade.
    /// n is `0..8`
    fn gray(&self, n: usize) -> Style {
        self.p.gray(n, Contrast::Normal)
    }

    /// Create a style from the given red shade.
    /// n is `0..8`
    fn red(&self, n: usize) -> Style {
        self.p.red(n, Contrast::Normal)
    }

    /// Create a style from the given orange shade.
    /// n is `0..8`
    fn orange(&self, n: usize) -> Style {
        self.p.orange(n, Contrast::Normal)
    }

    /// Create a style from the given yellow shade.
    /// n is `0..8`
    fn yellow(&self, n: usize) -> Style {
        self.p.yellow(n, Contrast::Normal)
    }

    /// Create a style from the given limegreen shade.
    /// n is `0..8`
    fn limegreen(&self, n: usize) -> Style {
        self.p.limegreen(n, Contrast::Normal)
    }

    /// Create a style from the given green shade.
    /// n is `0..8`
    fn green(&self, n: usize) -> Style {
        self.p.green(n, Contrast::Normal)
    }

    /// Create a style from the given bluegreen shade.
    /// n is `0..8`
    fn bluegreen(&self, n: usize) -> Style {
        self.p.bluegreen(n, Contrast::Normal)
    }

    /// Create a style from the given cyan shade.
    /// n is `0..8`
    fn cyan(&self, n: usize) -> Style {
        self.p.cyan(n, Contrast::Normal)
    }

    /// Create a style from the given blue shade.
    /// n is `0..8`
    fn blue(&self, n: usize) -> Style {
        self.p.blue(n, Contrast::Normal)
    }

    /// Create a style from the given deepblue shade.
    /// n is `0..8`
    fn deepblue(&self, n: usize) -> Style {
        self.p.deepblue(n, Contrast::Normal)
    }

    /// Create a style from the given purple shade.
    /// n is `0..8`
    fn purple(&self, n: usize) -> Style {
        self.p.purple(n, Contrast::Normal)
    }

    /// Create a style from the given magenta shade.
    /// n is `0..8`
    fn magenta(&self, n: usize) -> Style {
        self.p.magenta(n, Contrast::Normal)
    }

    /// Create a style from the given redpink shade.
    /// n is `0..8`
    fn redpink(&self, n: usize) -> Style {
        self.p.redpink(n, Contrast::Normal)
    }

    /// Create a style from the given primary shade.
    /// n is `0..8`
    fn primary(&self, n: usize) -> Style {
        self.p.primary(n, Contrast::Normal)
    }

    /// Create a style from the given secondary shade.
    /// n is `0..8`
    fn secondary(&self, n: usize) -> Style {
        self.p.secondary(n, Contrast::Normal)
    }

    /// Focus style
    fn focus(&self) -> Style {
        self.high_style(self.p.primary[2])
    }

    /// Selection style
    fn select(&self) -> Style {
        self.high_style(self.p.secondary[1])
    }

    /// Text field style.
    fn text_input(&self) -> Style {
        self.high_style(self.p.gray[3])
    }

    /// Focused text field style.
    fn text_focus(&self) -> Style {
        self.high_style(self.p.primary[1])
    }

    /// Text selection style.
    fn text_select(&self) -> Style {
        self.high_style(self.p.secondary[1])
    }

    /// Container base
    fn container_base(&self) -> Style {
        self.style(self.p.black[1])
    }

    /// Container border
    fn container_border(&self) -> Style {
        self.container_base().fg(self.p.gray[0])
    }

    /// Container arrows
    fn container_arrow(&self) -> Style {
        self.container_base().fg(self.p.gray[0])
    }

    /// Background for popups.
    fn popup_base(&self) -> Style {
        self.style(self.p.white[0])
    }

    /// Dialog arrows
    fn popup_border(&self) -> Style {
        self.popup_base().fg(self.p.gray[0])
    }

    /// Dialog arrows
    fn popup_arrow(&self) -> Style {
        self.popup_base().fg(self.p.gray[0])
    }

    /// Background for dialogs.
    fn dialog_base(&self) -> Style {
        self.style(self.p.gray[1])
    }

    /// Dialog arrows
    fn dialog_border(&self) -> Style {
        self.dialog_base().fg(self.p.white[0])
    }

    /// Dialog arrows
    fn dialog_arrow(&self) -> Style {
        self.dialog_base().fg(self.p.white[0])
    }

    /// Style for the status line.
    fn status_base(&self) -> Style {
        self.style(self.p.black[2])
    }

    /// Base style for buttons.
    fn button_base(&self) -> Style {
        self.style(self.p.gray[2])
    }

    /// Armed style for buttons.
    fn button_armed(&self) -> Style {
        self.style(self.p.secondary[0])
    }

    /// Complete MonthStyle.
    fn month_style(&self) -> CalendarStyle {
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
    fn shadow_style(&self) -> ShadowStyle {
        ShadowStyle {
            style: Style::new().bg(self.p.black[0]),
            dir: ShadowDirection::BottomRight,
            ..ShadowStyle::default()
        }
    }

    /// Style for LineNumbers.
    fn line_nr_style(&self) -> LineNumberStyle {
        LineNumberStyle {
            style: self.container_base().fg(self.p.gray[1]),
            cursor: Some(self.text_select()),
            ..LineNumberStyle::default()
        }
    }

    /// Complete TextAreaStyle
    fn textarea_style(&self) -> TextStyle {
        TextStyle {
            style: self.text_input(),
            focus: Some(self.focus()),
            select: Some(self.text_select()),
            scroll: Some(self.scroll_style()),
            border_style: Some(self.container_border()),
            ..TextStyle::default()
        }
    }

    /// Complete TextInputStyle
    fn text_style(&self) -> TextStyle {
        TextStyle {
            style: self.text_input(),
            focus: Some(self.text_focus()),
            select: Some(self.text_select()),
            invalid: Some(Style::default().bg(self.p.red[3])),
            ..TextStyle::default()
        }
    }

    /// Text-label style.
    fn label_style(&self) -> Style {
        self.container_base()
    }

    fn paragraph_style(&self) -> ParagraphStyle {
        ParagraphStyle {
            style: self.container_base(),
            focus: Some(self.focus()),
            scroll: Some(self.scroll_style()),
            ..Default::default()
        }
    }

    fn choice_style(&self) -> ChoiceStyle {
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

    fn radio_style(&self) -> RadioStyle {
        RadioStyle {
            layout: Some(RadioLayout::Stacked),
            style: self.text_input(),
            focus: Some(self.text_focus()),
            ..Default::default()
        }
    }

    /// Complete CheckboxStyle
    fn checkbox_style(&self) -> CheckboxStyle {
        CheckboxStyle {
            style: self.text_input(),
            focus: Some(self.text_focus()),
            ..Default::default()
        }
    }

    /// Slider Style
    fn slider_style(&self) -> SliderStyle {
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
    fn menu_style(&self) -> MenuStyle {
        MenuStyle {
            style: self.status_base(),
            title: Some(self.style(self.p.yellow[2])),
            focus: Some(self.focus()),
            right: Some(Style::default().fg(self.p.bluegreen[0])),
            disabled: Some(Style::default().fg(self.p.gray[0])),
            highlight: Some(Style::default().underlined()),
            popup_style: Some(self.status_base()),
            popup_block: Some(Block::bordered()),
            popup_focus: Some(self.focus()),
            popup_right: Some(Style::default().fg(self.p.bluegreen[0])),
            popup_disabled: Some(Style::default().fg(self.p.gray[0])),
            popup_highlight: Some(Style::default().underlined()),
            popup: Default::default(),
            ..Default::default()
        }
    }

    /// Complete ButtonStyle
    fn button_style(&self) -> ButtonStyle {
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
    fn table_style(&self) -> TableStyle {
        TableStyle {
            style: self.container_base(),
            select_row: Some(self.select()),
            show_row_focus: true,
            focus_style: Some(self.focus()),
            border_style: Some(self.container_border()),
            scroll: Some(self.scroll_style()),
            header: Some(self.table_header()),
            footer: Some(self.table_footer()),
            ..Default::default()
        }
    }

    /// Complete ListStyle
    fn list_style(&self) -> ListStyle {
        ListStyle {
            style: self.container_base(),
            select: Some(self.select()),
            focus: Some(self.focus()),
            scroll: Some(self.scroll_style()),
            ..Default::default()
        }
    }

    /// Scroll style
    fn scroll_style(&self) -> ScrollStyle {
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
    fn popup_scroll_style(&self) -> ScrollStyle {
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
    fn dialog_scroll_style(&self) -> ScrollStyle {
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
    fn split_style(&self) -> SplitStyle {
        SplitStyle {
            style: self.container_border(),
            arrow_style: Some(self.container_arrow()),
            drag_style: Some(self.focus()),
            ..Default::default()
        }
    }

    /// View style
    fn view_style(&self) -> ViewStyle {
        ViewStyle {
            scroll: Some(self.scroll_style()),
            ..Default::default()
        }
    }

    /// Tabbed style
    fn tabbed_style(&self) -> TabbedStyle {
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
    fn statusline_style(&self) -> Vec<Style> {
        vec![
            self.status_base(),
            self.p.normal_contrast(self.p.white[0]).bg(self.p.blue[3]),
            self.p.normal_contrast(self.p.white[0]).bg(self.p.blue[2]),
            self.p.normal_contrast(self.p.white[0]).bg(self.p.blue[1]),
        ]
    }

    /// StatusLineStyle for a StatusLine with 3 indicator fields.
    fn statusline_style_ext(&self) -> StatusLineStyle {
        StatusLineStyle {
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
    fn file_dialog_style(&self) -> FileDialogStyle {
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
    fn msg_dialog_style(&self) -> MsgDialogStyle {
        MsgDialogStyle {
            style: self.dialog_base(),
            button: Some(self.button_style()),
            ..Default::default()
        }
    }

    fn form_style(&self) -> FormStyle {
        FormStyle {
            style: self.container_base(),
            navigation: Some(self.container_arrow()),
            block: Some(
                Block::bordered()
                    .borders(Borders::TOP | Borders::BOTTOM)
                    .border_style(self.container_border()),
            ),
            ..Default::default()
        }
    }

    /// Clipper style.
    fn clipper_style(&self) -> ClipperStyle {
        ClipperStyle {
            style: self.container_base(),
            scroll: Some(self.scroll_style()),
            ..Default::default()
        }
    }

    fn textview_style(&self) -> TextStyle {
        TextStyle {
            style: self.container_base(),
            focus: Some(self.container_base()),
            select: Some(self.text_select()),
            scroll: Some(self.scroll_style()),
            border_style: Some(self.container_border()),
            ..TextStyle::default()
        }
    }
}
