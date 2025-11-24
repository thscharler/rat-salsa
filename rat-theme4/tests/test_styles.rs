use rat_theme4::RatWidgetColor;
use rat_theme4::theme::SalsaTheme;
use rat_theme4::{
    StyleName, WidgetStyle, create_palette, create_theme, salsa_palettes, salsa_themes,
};
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
use ratatui::style::{Color, Style};
use std::io::Write;

#[test]
fn test_palette() {
    let palettes = salsa_palettes();
    for pal in palettes {
        eprintln!();
        eprintln!("PALETTE {:?}", pal);
        eprintln!();
        let pal = create_palette(pal).expect("pal");

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
        let th = create_theme(theme);
        verify_theme(&th);
    }
}

#[test]
fn test_fallback() {
    let th = create_theme("Fallback");
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
    th.style_style(Style::POPUP_BASE);
    th.style_style(Style::POPUP_BORDER_FG);
    th.style_style(Style::POPUP_ARROW_FG);
    th.style_style(Style::DIALOG_BASE);
    th.style_style(Style::DIALOG_BORDER_FG);
    th.style_style(Style::DIALOG_ARROW_FG);
    th.style_style(Style::STATUS_BASE);

    th.style::<ButtonStyle>(WidgetStyle::BUTTON);
    th.style::<CalendarStyle>(WidgetStyle::CALENDAR);
    th.style::<CheckboxStyle>(WidgetStyle::CHECKBOX);
    th.style::<ChoiceStyle>(WidgetStyle::CHOICE);
    th.style::<ClipperStyle>(WidgetStyle::CLIPPER);
    th.style::<ColorInputStyle>(WidgetStyle::COLOR_INPUT);
    th.style::<ComboboxStyle>(WidgetStyle::COMBOBOX);
    th.style::<DialogFrameStyle>(WidgetStyle::DIALOG_FRAME);
    th.style::<FileDialogStyle>(WidgetStyle::FILE_DIALOG);
    th.style::<FormStyle>(WidgetStyle::FORM);
    th.style::<LineNumberStyle>(WidgetStyle::LINE_NR);
    th.style::<ListStyle>(WidgetStyle::LIST);
    th.style::<MenuStyle>(WidgetStyle::MENU);
    th.style::<CalendarStyle>(WidgetStyle::MONTH);
    th.style::<MsgDialogStyle>(WidgetStyle::MSG_DIALOG);
    th.style::<ParagraphStyle>(WidgetStyle::PARAGRAPH);
    th.style::<RadioStyle>(WidgetStyle::RADIO);
    th.style::<ScrollStyle>(WidgetStyle::SCROLL);
    th.style::<ScrollStyle>(WidgetStyle::SCROLL_DIALOG);
    th.style::<ScrollStyle>(WidgetStyle::SCROLL_POPUP);
    th.style::<ShadowStyle>(WidgetStyle::SHADOW);
    th.style::<SliderStyle>(WidgetStyle::SLIDER);
    th.style::<SplitStyle>(WidgetStyle::SPLIT);
    th.style::<StatusLineStyle>(WidgetStyle::STATUSLINE);
    th.style::<TabbedStyle>(WidgetStyle::TABBED);
    th.style::<TableStyle>(WidgetStyle::TABLE);
    th.style::<TextStyle>(WidgetStyle::TEXT);
    th.style::<TextStyle>(WidgetStyle::TEXTAREA);
    th.style::<TextStyle>(WidgetStyle::TEXTVIEW);
    th.style::<ViewStyle>(WidgetStyle::VIEW);
}
