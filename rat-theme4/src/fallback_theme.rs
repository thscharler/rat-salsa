use crate::palette::Palette;
use crate::{Category, SalsaTheme};
use crate::{StyleName, WidgetStyle};
use rat_widget::button::ButtonStyle;
use rat_widget::calendar::CalendarStyle;
use rat_widget::checkbox::CheckboxStyle;
use rat_widget::choice::ChoiceStyle;
use rat_widget::clipper::ClipperStyle;
use rat_widget::color_input::ColorInputStyle;
use rat_widget::combobox::ComboboxStyle;
use rat_widget::dialog_frame::DialogFrameStyle;
use rat_widget::file_dialog::FileDialogStyle;
use rat_widget::form::FormStyle;
use rat_widget::line_number::LineNumberStyle;
use rat_widget::list::ListStyle;
use rat_widget::menu::MenuStyle;
use rat_widget::msgdialog::MsgDialogStyle;
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
use ratatui::style::Style;

/// A theme to test the fallback-styles of each widget.
pub fn fallback_theme(name: &str, p: Palette) -> SalsaTheme {
    let mut th = SalsaTheme::new(name, Category::Other, p);

    th.define(Style::LABEL_FG, Style::default());
    th.define(Style::INPUT, Style::default());
    th.define(Style::FOCUS, Style::default());
    th.define(Style::SELECT, Style::default());
    th.define(Style::DISABLED, Style::default());
    th.define(Style::INVALID, Style::default());
    th.define(Style::HOVER, Style::default());
    th.define(Style::TITLE, Style::default());
    th.define(Style::HEADER, Style::default());
    th.define(Style::FOOTER, Style::default());
    th.define(Style::SHADOWS, Style::default());
    th.define(Style::TEXT_FOCUS, Style::default());
    th.define(Style::TEXT_SELECT, Style::default());
    th.define(Style::KEY_BINDING, Style::default());

    th.define(Style::BUTTON_BASE, Style::default());
    th.define(Style::MENU_BASE, Style::default());
    th.define(Style::STATUS_BASE, Style::default());

    th.define(Style::CONTAINER_BASE, Style::default());
    th.define(Style::CONTAINER_BORDER_FG, Style::default());
    th.define(Style::CONTAINER_ARROW_FG, Style::default());

    th.define(Style::POPUP_BASE, Style::default());
    th.define(Style::POPUP_BORDER_FG, Style::default());
    th.define(Style::POPUP_ARROW_FG, Style::default());

    th.define(Style::DIALOG_BASE, Style::default());
    th.define(Style::DIALOG_BORDER_FG, Style::default());
    th.define(Style::DIALOG_ARROW_FG, Style::default());

    th.define_fn0(WidgetStyle::BUTTON, ButtonStyle::default);
    th.define_fn0(WidgetStyle::CALENDAR, CalendarStyle::default);
    th.define_fn0(WidgetStyle::CHECKBOX, CheckboxStyle::default);
    th.define_fn0(WidgetStyle::CHOICE, ChoiceStyle::default);
    th.define_fn0(WidgetStyle::CLIPPER, ClipperStyle::default);
    th.define_fn0(WidgetStyle::COMBOBOX, ComboboxStyle::default);
    th.define_fn0(WidgetStyle::COLOR_INPUT, ColorInputStyle::default);
    th.define_fn0(WidgetStyle::DIALOG_FRAME, DialogFrameStyle::default);
    th.define_fn0(WidgetStyle::FILE_DIALOG, FileDialogStyle::default);
    th.define_fn0(WidgetStyle::FORM, FormStyle::default);
    th.define_fn0(WidgetStyle::LINE_NR, LineNumberStyle::default);
    th.define_fn0(WidgetStyle::LIST, ListStyle::default);
    th.define_fn0(WidgetStyle::MENU, MenuStyle::default);
    th.define_fn0(WidgetStyle::MONTH, CalendarStyle::default);
    th.define_fn0(WidgetStyle::MSG_DIALOG, MsgDialogStyle::default);
    th.define_fn0(WidgetStyle::PARAGRAPH, ParagraphStyle::default);
    th.define_fn0(WidgetStyle::RADIO, RadioStyle::default);
    th.define_fn0(WidgetStyle::SCROLL, ScrollStyle::default);
    th.define_fn0(WidgetStyle::SCROLL_DIALOG, ScrollStyle::default);
    th.define_fn0(WidgetStyle::SCROLL_POPUP, ScrollStyle::default);
    th.define_fn0(WidgetStyle::SHADOW, ShadowStyle::default);
    th.define_fn0(WidgetStyle::SLIDER, SliderStyle::default);
    th.define_fn0(WidgetStyle::SPLIT, SplitStyle::default);
    th.define_fn0(WidgetStyle::STATUSLINE, StatusLineStyle::default);
    th.define_fn0(WidgetStyle::TABBED, TabbedStyle::default);
    th.define_fn0(WidgetStyle::TABLE, TableStyle::default);
    th.define_fn0(WidgetStyle::TEXT, TextStyle::default);
    th.define_fn0(WidgetStyle::TEXTAREA, TextStyle::default);
    th.define_fn0(WidgetStyle::TEXTVIEW, TextStyle::default);
    th.define_fn0(WidgetStyle::VIEW, ViewStyle::default);

    th
}
