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
#[allow(deprecated)]
use rat_widget::pager::PagerStyle;
use rat_widget::paragraph::ParagraphStyle;
use rat_widget::popup::PopupStyle;
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
use ratatui::prelude::Style;
use ratatui::style::{Color, Stylize};
use ratatui::widgets::{Block, Borders};
use std::borrow::Cow;
use std::time::Duration;

/// A sample theme for shell usage.
#[derive(Debug, Clone)]
pub struct ShellTheme {
    p: Palette,
    name: Box<str>,
}

impl ShellTheme {
    pub fn new(name: &str, p: Palette) -> Self {
        Self {
            p,
            name: Box::from(name),
        }
    }

    /// Create a style with only a text foreground color
    fn fg_style(&self, color: Color) -> Style {
        Style::new().fg(color)
    }
}

impl SalsaTheme for ShellTheme {
    fn name(&self) -> &str {
        &self.name
    }

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

    fn focus(&self) -> Style {
        self.p.high_contrast(self.p.primary[Palette::BRIGHT_2])
    }

    fn select(&self) -> Style {
        self.p.high_contrast(self.p.secondary[Palette::BRIGHT_2])
    }

    fn text_input(&self) -> Style {
        self.p.normal_contrast(self.p.gray[Palette::BRIGHT_0])
    }

    fn text_focus(&self) -> Style {
        self.p.normal_contrast(self.p.gray[Palette::BRIGHT_3])
    }

    fn text_select(&self) -> Style {
        self.p.normal_contrast(self.p.secondary[Palette::BRIGHT_0])
    }

    /// Container base
    fn container_base(&self) -> Style {
        Default::default()
    }

    /// Container border
    fn container_border(&self) -> Style {
        Default::default()
    }

    /// Container arrows
    fn container_arrow(&self) -> Style {
        Default::default()
    }

    /// Background for popups.
    fn popup_base(&self) -> Style {
        self.p
            .style(self.p.gray[Palette::BRIGHT_0], Contrast::Normal)
    }

    /// Dialog arrows
    fn popup_border(&self) -> Style {
        self.popup_base().fg(self.p.gray[Palette::BRIGHT_0])
    }

    /// Dialog arrows
    fn popup_arrow(&self) -> Style {
        self.popup_base().fg(self.p.gray[Palette::BRIGHT_0])
    }

    /// Background for dialogs.
    fn dialog_base(&self) -> Style {
        self.p
            .style(self.p.gray[Palette::BRIGHT_1], Contrast::Normal)
    }

    /// Dialog arrows
    fn dialog_border(&self) -> Style {
        self.dialog_base().fg(self.p.white[Palette::BRIGHT_0])
    }

    /// Dialog arrows
    fn dialog_arrow(&self) -> Style {
        self.dialog_base().fg(self.p.white[Palette::BRIGHT_0])
    }

    /// Style for the status line.
    fn status_base(&self) -> Style {
        Default::default()
    }

    /// Base style for buttons.
    fn button_base(&self) -> Style {
        self.p
            .style(self.p.gray[Palette::BRIGHT_2], Contrast::Normal)
    }

    /// Armed style for buttons.
    fn button_armed(&self) -> Style {
        self.p
            .style(self.p.secondary[Palette::BRIGHT_0], Contrast::Normal)
    }

    /// Complete MonthStyle.
    fn month_style(&self) -> CalendarStyle {
        CalendarStyle {
            style: Default::default(),
            title: None,
            weeknum: Some(Style::new().fg(self.p.limegreen[Palette::BRIGHT_0])),
            weekday: Some(Style::new().fg(self.p.limegreen[Palette::BRIGHT_0])),
            day: None,
            select: Some(self.select()),
            focus: Some(self.focus()),
            ..CalendarStyle::default()
        }
    }

    /// Style for shadows.
    fn shadow_style(&self) -> ShadowStyle {
        ShadowStyle {
            style: Style::new().bg(self.p.black[Palette::DARK_0]),
            dir: ShadowDirection::BottomRight,
            ..ShadowStyle::default()
        }
    }

    /// Style for LineNumbers.
    fn line_nr_style(&self) -> LineNumberStyle {
        LineNumberStyle {
            style: self.container_base(),
            cursor: Some(self.text_select()),
            ..LineNumberStyle::default()
        }
    }

    /// Complete TextAreaStyle
    fn textarea_style(&self) -> TextStyle {
        TextStyle {
            style: self.text_input(),
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
            invalid: Some(self.fg_style(self.p.red[Palette::BRIGHT_3])),
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

    #[allow(deprecated)]
    fn choice_style(&self) -> ChoiceStyle {
        ChoiceStyle {
            style: self.text_input(),
            select: Some(self.text_select()),
            focus: Some(self.text_focus()),
            popup: PopupStyle::default(),
            popup_style: Some(self.popup_base()),
            popup_border_style: Some(self.popup_border()),
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
            title: Some(self.fg_style(self.p.yellow[Palette::BRIGHT_2])),
            focus: Some(self.focus()),
            right: Some(self.fg_style(self.p.green[Palette::BRIGHT_3])),
            disabled: Some(self.fg_style(self.p.gray[Palette::BRIGHT_2])),
            highlight: Some(Style::default().underlined()),
            block: Some(Block::bordered().style(self.popup_border())),
            ..Default::default()
        }
    }

    /// Complete ButtonStyle
    fn button_style(&self) -> ButtonStyle {
        ButtonStyle {
            style: self.button_base(),
            focus: Some(self.focus()),
            armed: Some(self.select()),
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
            header: Some(self.fg_style(self.p.green[Palette::BRIGHT_2])),
            footer: Some(self.fg_style(self.p.green[Palette::BRIGHT_2])),
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
        TabbedStyle {
            style: self.container_base(),
            tab: Some(self.button_base()),
            select: Some(self.button_armed()),
            focus: Some(self.focus()),
            ..Default::default()
        }
    }

    /// Complete StatusLineStyle for a StatusLine with 3 indicator fields.
    fn statusline_style(&self) -> Vec<Style> {
        vec![
            self.status_base(),
            self.fg_style(self.p.blue[Palette::BRIGHT_2]),
            self.fg_style(self.p.blue[Palette::BRIGHT_2]),
            self.fg_style(self.p.blue[Palette::BRIGHT_2]),
        ]
    }

    /// StatusLineStyle for a StatusLine with 3 indicator fields.
    fn statusline_style_ext(&self) -> StatusLineStyle {
        StatusLineStyle {
            sep: Some(Cow::Borrowed("|")),
            styles: vec![
                self.status_base(),
                self.status_base().fg(self.p.blue[Palette::BRIGHT_2]),
                self.status_base().fg(self.p.blue[Palette::BRIGHT_2]),
                self.status_base().fg(self.p.blue[Palette::BRIGHT_2]),
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

    /// Pager style.
    #[allow(deprecated)]
    fn pager_style(&self) -> PagerStyle {
        PagerStyle {
            style: self.container_base(),
            navigation: Some(self.container_arrow()),
            block: Some(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(self.container_border()),
            ),
            ..Default::default()
        }
    }

    /// Form style.
    fn form_style(&self) -> FormStyle {
        FormStyle {
            style: self.container_base(),
            navigation: Some(self.container_arrow()),
            block: Some(
                Block::default()
                    .borders(Borders::TOP)
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
            select: Some(self.text_select()),
            scroll: Some(self.scroll_style()),
            border_style: Some(self.container_border()),
            ..TextStyle::default()
        }
    }
}
