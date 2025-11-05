use crate::palette::Palette;
use crate::{Category, SalsaTheme};
use crate::{StyleName, WidgetStyle};
use rat_widget::button::ButtonStyle;
use rat_widget::calendar::CalendarStyle;
use rat_widget::checkbox::CheckboxStyle;
use rat_widget::choice::ChoiceStyle;
use rat_widget::clipper::ClipperStyle;
use rat_widget::container::ContainerStyle;
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

    th.define(Style::INPUT, Style::default());
    th.define(Style::FOCUS, Style::default());
    th.define(Style::SELECT, Style::default());
    th.define(Style::TEXT_FOCUS, Style::default());
    th.define(Style::TEXT_SELECT, Style::default());

    th.define(Style::CONTAINER_BASE, Style::default());
    th.define(Style::CONTAINER_BORDER, Style::default());
    th.define(Style::CONTAINER_ARROWS, Style::default());

    th.define(Style::POPUP_BASE, Style::default());
    th.define(Style::POPUP_BORDER, Style::default());
    th.define(Style::POPUP_ARROW, Style::default());

    th.define(Style::DIALOG_BASE, Style::default());
    th.define(Style::DIALOG_BORDER, Style::default());
    th.define(Style::DIALOG_ARROW, Style::default());

    th.define(Style::STATUS_BASE, Style::default());

    th.define_fn(WidgetStyle::BUTTON, |_| Box::new(ButtonStyle::default()));
    th.define_fn(
        WidgetStyle::CALENDAR,
        |_| Box::new(CalendarStyle::default()),
    );
    th.define_fn(
        WidgetStyle::CHECKBOX,
        |_| Box::new(CheckboxStyle::default()),
    );
    th.define_fn(WidgetStyle::CHOICE, |_| Box::new(ChoiceStyle::default()));
    th.define_fn(WidgetStyle::CLIPPER, |_| Box::new(ClipperStyle::default()));
    th.define_fn(WidgetStyle::CONTAINER, |_| {
        Box::new(ContainerStyle::default())
    });
    th.define_fn(WidgetStyle::DIALOG_FRAME, |_| {
        Box::new(FileDialogStyle::default())
    });
    th.define_fn(WidgetStyle::FILE_DIALOG, |_| {
        Box::new(FileDialogStyle::default())
    });
    th.define_fn(WidgetStyle::FORM, |_| Box::new(FormStyle::default()));
    th.define_fn(WidgetStyle::LINE_NR, |_| {
        Box::new(LineNumberStyle::default())
    });
    th.define_fn(WidgetStyle::LIST, |_| Box::new(ListStyle::default()));
    th.define_fn(WidgetStyle::MENU, |_| Box::new(MenuStyle::default()));
    th.define_fn(WidgetStyle::MONTH, |_| Box::new(CalendarStyle::default()));

    th.define_fn(WidgetStyle::MSG_DIALOG, |_| {
        Box::new(MsgDialogStyle::default())
    });
    th.define_fn(WidgetStyle::PARAGRAPH, |_| {
        Box::new(ParagraphStyle::default())
    });
    th.define_fn(WidgetStyle::RADIO, |_| Box::new(RadioStyle::default()));
    th.define_fn(WidgetStyle::SCROLL, |_| Box::new(ScrollStyle::default()));
    th.define_fn(WidgetStyle::SCROLL_DIALOG, |_| {
        Box::new(ScrollStyle::default())
    });
    th.define_fn(WidgetStyle::SCROLL_POPUP, |_| {
        Box::new(ScrollStyle::default())
    });
    th.define_fn(WidgetStyle::SHADOW, |_| Box::new(ShadowStyle::default()));
    th.define_fn(WidgetStyle::SLIDER, |_| Box::new(SliderStyle::default()));
    th.define_fn(WidgetStyle::SPLIT, |_| Box::new(SplitStyle::default()));
    th.define_fn(WidgetStyle::STATUSLINE, |_| {
        Box::new(StatusLineStyle::default())
    });
    th.define_fn(WidgetStyle::TABBED, |_| Box::new(TabbedStyle::default()));
    th.define_fn(WidgetStyle::TABLE, |_| Box::new(TableStyle::default()));
    th.define_fn(WidgetStyle::TEXT, |_| Box::new(TextStyle::default()));
    th.define_fn(WidgetStyle::TEXTAREA, |_| Box::new(TextStyle::default()));
    th.define_fn(WidgetStyle::TEXTVIEW, |_| Box::new(TextStyle::default()));
    th.define_fn(WidgetStyle::VIEW, |_| Box::new(ViewStyle::default()));

    th
}
