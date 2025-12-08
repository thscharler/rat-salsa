use crate::palette::{ColorIdx, Palette};
use crate::theme::SalsaTheme;
use crate::{RatWidgetColor, StyleName, WidgetStyle};
#[cfg(feature = "rat-widget")]
use rat_widget;
#[cfg(feature = "color-input")]
use rat_widget_extra::color_input::ColorInputStyle;
use ratatui::style::{Color, Style};

/// A theme to test the fallback-styles of each widget.
pub fn create_fallback(p: Palette) -> SalsaTheme {
    let mut th = SalsaTheme::new(p);
    th.name = "Fallback".to_string();
    th.theme = "Fallback".to_string();

    th.p.add_aliased(Color::LABEL_FG, ColorIdx::default());
    th.p.add_aliased(Color::INPUT_BG, ColorIdx::default());
    th.p.add_aliased(Color::FOCUS_BG, ColorIdx::default());
    th.p.add_aliased(Color::SELECT_BG, ColorIdx::default());
    th.p.add_aliased(Color::DISABLED_BG, ColorIdx::default());
    th.p.add_aliased(Color::INVALID_BG, ColorIdx::default());
    th.p.add_aliased(Color::HOVER_BG, ColorIdx::default());
    th.p.add_aliased(Color::TITLE_FG, ColorIdx::default());
    th.p.add_aliased(Color::TITLE_BG, ColorIdx::default());
    th.p.add_aliased(Color::HEADER_FG, ColorIdx::default());
    th.p.add_aliased(Color::HEADER_BG, ColorIdx::default());
    th.p.add_aliased(Color::FOOTER_FG, ColorIdx::default());
    th.p.add_aliased(Color::FOOTER_BG, ColorIdx::default());
    th.p.add_aliased(Color::SHADOW_BG, ColorIdx::default());
    th.p.add_aliased(Color::WEEK_HEADER_FG, ColorIdx::default());
    th.p.add_aliased(Color::MONTH_HEADER_FG, ColorIdx::default());
    th.p.add_aliased(Color::INPUT_FOCUS_BG, ColorIdx::default());
    th.p.add_aliased(Color::INPUT_SELECT_BG, ColorIdx::default());
    th.p.add_aliased(Color::BUTTON_BASE_BG, ColorIdx::default());
    th.p.add_aliased(Color::MENU_BASE_BG, ColorIdx::default());
    th.p.add_aliased(Color::KEY_BINDING_BG, ColorIdx::default());
    th.p.add_aliased(Color::STATUS_BASE_BG, ColorIdx::default());
    th.p.add_aliased(Color::CONTAINER_BASE_BG, ColorIdx::default());
    th.p.add_aliased(Color::CONTAINER_BORDER_FG, ColorIdx::default());
    th.p.add_aliased(Color::CONTAINER_ARROW_FG, ColorIdx::default());
    th.p.add_aliased(Color::DOCUMENT_BASE_BG, ColorIdx::default());
    th.p.add_aliased(Color::DOCUMENT_BORDER_FG, ColorIdx::default());
    th.p.add_aliased(Color::DOCUMENT_ARROW_FG, ColorIdx::default());
    th.p.add_aliased(Color::POPUP_BASE_BG, ColorIdx::default());
    th.p.add_aliased(Color::POPUP_BORDER_FG, ColorIdx::default());
    th.p.add_aliased(Color::POPUP_ARROW_FG, ColorIdx::default());
    th.p.add_aliased(Color::DIALOG_BASE_BG, ColorIdx::default());
    th.p.add_aliased(Color::DIALOG_BORDER_FG, ColorIdx::default());
    th.p.add_aliased(Color::DIALOG_ARROW_FG, ColorIdx::default());

    th.define_style(Style::LABEL_FG, Style::default());
    th.define_style(Style::INPUT, Style::default());
    th.define_style(Style::FOCUS, Style::default());
    th.define_style(Style::SELECT, Style::default());
    th.define_style(Style::DISABLED, Style::default());
    th.define_style(Style::INVALID, Style::default());
    th.define_style(Style::HOVER, Style::default());
    th.define_style(Style::TITLE, Style::default());
    th.define_style(Style::HEADER, Style::default());
    th.define_style(Style::FOOTER, Style::default());
    th.define_style(Style::SHADOWS, Style::default());
    th.define_style(Style::WEEK_HEADER_FG, Style::default());
    th.define_style(Style::MONTH_HEADER_FG, Style::default());
    th.define_style(Style::INPUT_FOCUS, Style::default());
    th.define_style(Style::INPUT_SELECT, Style::default());
    th.define_style(Style::KEY_BINDING, Style::default());

    th.define_style(Style::BUTTON_BASE, Style::default());
    th.define_style(Style::MENU_BASE, Style::default());
    th.define_style(Style::STATUS_BASE, Style::default());

    th.define_style(Style::CONTAINER_BASE, Style::default());
    th.define_style(Style::CONTAINER_BORDER_FG, Style::default());
    th.define_style(Style::CONTAINER_ARROW_FG, Style::default());

    th.define_style(Style::DOCUMENT_BASE, Style::default());
    th.define_style(Style::DOCUMENT_BORDER_FG, Style::default());
    th.define_style(Style::DOCUMENT_ARROW_FG, Style::default());

    th.define_style(Style::POPUP_BASE, Style::default());
    th.define_style(Style::POPUP_BORDER_FG, Style::default());
    th.define_style(Style::POPUP_ARROW_FG, Style::default());

    th.define_style(Style::DIALOG_BASE, Style::default());
    th.define_style(Style::DIALOG_BORDER_FG, Style::default());
    th.define_style(Style::DIALOG_ARROW_FG, Style::default());

    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::BUTTON,
        rat_widget::button::ButtonStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::CALENDAR,
        rat_widget::calendar::CalendarStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::CHECKBOX,
        rat_widget::checkbox::CheckboxStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::CHOICE,
        rat_widget::choice::ChoiceStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::CLIPPER,
        rat_widget::clipper::ClipperStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::COMBOBOX,
        rat_widget::combobox::ComboboxStyle::default,
    );
    #[cfg(feature = "color-input")]
    th.define_fn0(WidgetStyle::COLOR_INPUT, ColorInputStyle::default);
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::DIALOG_FRAME,
        rat_widget::dialog_frame::DialogFrameStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::FILE_DIALOG,
        rat_widget::file_dialog::FileDialogStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(WidgetStyle::FORM, rat_widget::form::FormStyle::default);
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::LINE_NR,
        rat_widget::line_number::LineNumberStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(WidgetStyle::LIST, rat_widget::list::ListStyle::default);
    #[cfg(feature = "rat-widget")]
    th.define_fn0(WidgetStyle::MENU, rat_widget::menu::MenuStyle::default);
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::MONTH,
        rat_widget::calendar::CalendarStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::MSG_DIALOG,
        rat_widget::msgdialog::MsgDialogStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::PARAGRAPH,
        rat_widget::paragraph::ParagraphStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(WidgetStyle::RADIO, rat_widget::radio::RadioStyle::default);
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::SCROLL,
        rat_widget::scrolled::ScrollStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::SCROLL_DIALOG,
        rat_widget::scrolled::ScrollStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::SCROLL_POPUP,
        rat_widget::scrolled::ScrollStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::SHADOW,
        rat_widget::shadow::ShadowStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::SLIDER,
        rat_widget::slider::SliderStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::SPLIT,
        rat_widget::splitter::SplitStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::STATUSLINE,
        rat_widget::statusline::StatusLineStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(
        WidgetStyle::TABBED,
        rat_widget::tabbed::TabbedStyle::default,
    );
    #[cfg(feature = "rat-widget")]
    th.define_fn0(WidgetStyle::TABLE, rat_widget::table::TableStyle::default);
    #[cfg(feature = "rat-widget")]
    th.define_fn0(WidgetStyle::TEXT, rat_widget::text::TextStyle::default);
    #[cfg(feature = "rat-widget")]
    th.define_fn0(WidgetStyle::TEXTAREA, rat_widget::text::TextStyle::default);
    #[cfg(feature = "rat-widget")]
    th.define_fn0(WidgetStyle::TEXTVIEW, rat_widget::text::TextStyle::default);
    #[cfg(feature = "rat-widget")]
    th.define_fn0(WidgetStyle::VIEW, rat_widget::view::ViewStyle::default);

    th
}
