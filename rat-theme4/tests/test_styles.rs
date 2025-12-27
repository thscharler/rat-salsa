use rat_theme4::theme::SalsaTheme;
use rat_theme4::{RatWidgetColor, create_salsa_theme};
use rat_theme4::{StyleName, WidgetStyle, create_salsa_palette, salsa_themes};
#[cfg(feature = "rat-widget")]
use rat_widget;
#[cfg(feature = "color-input")]
use rat_widget_extra::color_input::ColorInputStyle;
use ratatui_core::style::{Color, Style};
use std::io::Write;

#[test]
fn test_palette() {
    let palettes = salsa_themes();
    for pal in palettes {
        eprintln!();
        eprintln!("PALETTE {:?}", pal);
        eprintln!();
        let pal = create_salsa_palette(pal).expect("pal");

        if !pal.aliased.iter().is_sorted() {
            dbg!(&pal.aliased);
            let mut v = pal
                .aliased
                .iter()
                .map(|v| &v.0)
                .cloned()
                .collect::<Vec<_>>();
            v.sort();
            dbg!(v);
        }

        assert!(pal.aliased.is_sorted());

        for n in [
            Color::LABEL_FG,
            Color::INPUT_BG,
            Color::FOCUS_BG,
            Color::SELECT_BG,
            Color::DISABLED_BG,
            Color::INVALID_BG,
            Color::HOVER_BG,
            Color::TITLE_FG,
            Color::TITLE_BG,
            Color::HEADER_FG,
            Color::HEADER_BG,
            Color::FOOTER_FG,
            Color::FOOTER_BG,
            Color::SHADOW_BG,
            Color::WEEK_HEADER_FG,
            Color::MONTH_HEADER_FG,
            Color::INPUT_FOCUS_BG,
            Color::INPUT_SELECT_BG,
            Color::BUTTON_BASE_BG,
            Color::MENU_BASE_BG,
            Color::KEY_BINDING_BG,
            Color::STATUS_BASE_BG,
            Color::CONTAINER_BASE_BG,
            Color::CONTAINER_BORDER_FG,
            Color::CONTAINER_ARROW_FG,
            Color::DOCUMENT_BASE_BG,
            Color::DOCUMENT_BORDER_FG,
            Color::DOCUMENT_ARROW_FG,
            Color::POPUP_BASE_BG,
            Color::POPUP_BORDER_FG,
            Color::POPUP_ARROW_FG,
            Color::DIALOG_BASE_BG,
            Color::DIALOG_BORDER_FG,
            Color::DIALOG_ARROW_FG,
        ] {
            _ = pal.color_alias(&n.to_string());
        }
    }
}

#[test]
fn test_styles() {
    let themes = salsa_themes();

    for theme in themes {
        eprintln!();
        eprintln!("THEME {:?}", theme);
        eprintln!();
        _ = std::io::stderr().flush();
        let th = create_salsa_theme(theme);
        verify_theme(&th);
    }
}

#[test]
fn test_fallback() {
    let th = create_salsa_theme("Fallback");
    verify_theme(&th);
}

fn verify_theme(th: &SalsaTheme) {
    th.style_style(Style::LABEL_FG);
    th.style_style(Style::INPUT);
    th.style_style(Style::FOCUS);
    th.style_style(Style::SELECT);
    th.style_style(Style::DISABLED);
    th.style_style(Style::INVALID);
    th.style_style(Style::HOVER);
    th.style_style(Style::TITLE);
    th.style_style(Style::HEADER);
    th.style_style(Style::FOOTER);
    th.style_style(Style::SHADOWS);
    th.style_style(Style::WEEK_HEADER_FG);
    th.style_style(Style::MONTH_HEADER_FG);
    th.style_style(Style::INPUT_FOCUS);
    th.style_style(Style::INPUT_SELECT);
    th.style_style(Style::KEY_BINDING);
    th.style_style(Style::BUTTON_BASE);
    th.style_style(Style::MENU_BASE);
    th.style_style(Style::STATUS_BASE);
    th.style_style(Style::CONTAINER_BASE);
    th.style_style(Style::CONTAINER_BORDER_FG);
    th.style_style(Style::CONTAINER_ARROW_FG);
    th.style_style(Style::DOCUMENT_BASE);
    th.style_style(Style::DOCUMENT_BORDER_FG);
    th.style_style(Style::DOCUMENT_ARROW_FG);
    th.style_style(Style::POPUP_BASE);
    th.style_style(Style::POPUP_BORDER_FG);
    th.style_style(Style::POPUP_ARROW_FG);
    th.style_style(Style::DIALOG_BASE);
    th.style_style(Style::DIALOG_BORDER_FG);
    th.style_style(Style::DIALOG_ARROW_FG);
    th.style_style(Style::STATUS_BASE);

    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::button::ButtonStyle>(WidgetStyle::BUTTON);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::calendar::CalendarStyle>(WidgetStyle::CALENDAR);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::checkbox::CheckboxStyle>(WidgetStyle::CHECKBOX);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::choice::ChoiceStyle>(WidgetStyle::CHOICE);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::clipper::ClipperStyle>(WidgetStyle::CLIPPER);
    #[cfg(feature = "color-input")]
    th.style::<ColorInputStyle>(WidgetStyle::COLOR_INPUT);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::combobox::ComboboxStyle>(WidgetStyle::COMBOBOX);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::dialog_frame::DialogFrameStyle>(WidgetStyle::DIALOG_FRAME);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::file_dialog::FileDialogStyle>(WidgetStyle::FILE_DIALOG);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::form::FormStyle>(WidgetStyle::FORM);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::line_number::LineNumberStyle>(WidgetStyle::LINE_NR);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::list::ListStyle>(WidgetStyle::LIST);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::menu::MenuStyle>(WidgetStyle::MENU);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::calendar::CalendarStyle>(WidgetStyle::MONTH);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::msgdialog::MsgDialogStyle>(WidgetStyle::MSG_DIALOG);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::paragraph::ParagraphStyle>(WidgetStyle::PARAGRAPH);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::radio::RadioStyle>(WidgetStyle::RADIO);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::scrolled::ScrollStyle>(WidgetStyle::SCROLL);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::scrolled::ScrollStyle>(WidgetStyle::SCROLL_DIALOG);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::scrolled::ScrollStyle>(WidgetStyle::SCROLL_POPUP);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::shadow::ShadowStyle>(WidgetStyle::SHADOW);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::slider::SliderStyle>(WidgetStyle::SLIDER);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::splitter::SplitStyle>(WidgetStyle::SPLIT);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::statusline::StatusLineStyle>(WidgetStyle::STATUSLINE);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::tabbed::TabbedStyle>(WidgetStyle::TABBED);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::table::TableStyle>(WidgetStyle::TABLE);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::text::TextStyle>(WidgetStyle::TEXT);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::text::TextStyle>(WidgetStyle::TEXTAREA);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::text::TextStyle>(WidgetStyle::TEXTVIEW);
    #[cfg(feature = "rat-widget")]
    th.style::<rat_widget::view::ViewStyle>(WidgetStyle::VIEW);
}
