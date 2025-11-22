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
use ratatui::style::Style;
use std::io::Write;

#[test]
fn test_palette() {
    use rat_theme4::palette::Colors::*;

    let palettes = salsa_palettes();
    for pal in palettes {
        eprintln!();
        eprintln!("PALETTE {:?}", pal);
        eprintln!();
        let pal = create_palette(pal).expect("pal");

        for n in [
            TextLight, TextDark, Primary, Secondary, White, Black, Gray, Red, Orange, Yellow,
            LimeGreen, Green, BlueGreen, Cyan, Blue, DeepBlue, Purple, Magenta, RedPink, None,
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
    th.style_style(Style::TEXT_FOCUS);
    th.style_style(Style::TEXT_SELECT);
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
