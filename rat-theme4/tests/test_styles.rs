use rat_theme4::{
    SalsaTheme, StyleName, WidgetStyle, create_palette, create_theme, log_style_define,
    salsa_palettes, salsa_themes,
};
use rat_widget::button::ButtonStyle;
use rat_widget::calendar::CalendarStyle;
use rat_widget::checkbox::CheckboxStyle;
use rat_widget::choice::ChoiceStyle;
use rat_widget::clipper::ClipperStyle;
use rat_widget::combobox::ComboboxStyle;
use rat_widget::container::ContainerStyle;
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

#[test]
fn test_palette() {
    let palettes = salsa_palettes();
    for pal in palettes {
        create_palette(pal);
    }
}

#[test]
fn test_styles() {
    let themes = salsa_themes();

    for theme in themes {
        eprintln!("THEME {:?}", theme);
        let th = create_theme(theme).expect("theme");
        verify_theme(&th);
    }
}

#[test]
fn test_fallback() {
    let th = create_theme("Fallback").expect("theme");
    verify_theme(&th);
}

fn verify_theme(th: &SalsaTheme) {
    th.style_style(Style::INPUT);
    th.style_style(Style::FOCUS);
    th.style_style(Style::SELECT);
    th.style_style(Style::TEXT_FOCUS);
    th.style_style(Style::TEXT_SELECT);
    th.style_style(Style::BUTTON_BASE);
    th.style_style(Style::CONTAINER_BASE);
    th.style_style(Style::CONTAINER_BORDER);
    th.style_style(Style::CONTAINER_ARROWS);
    th.style_style(Style::POPUP_BASE);
    th.style_style(Style::POPUP_BORDER);
    th.style_style(Style::POPUP_ARROW);
    th.style_style(Style::DIALOG_BASE);
    th.style_style(Style::DIALOG_BORDER);
    th.style_style(Style::DIALOG_ARROW);
    th.style_style(Style::STATUS_BASE);

    th.style::<ButtonStyle>(WidgetStyle::BUTTON);
    th.style::<CheckboxStyle>(WidgetStyle::CHECKBOX);
    th.style::<ChoiceStyle>(WidgetStyle::CHOICE);
    th.style::<ClipperStyle>(WidgetStyle::CLIPPER);
    th.style::<ContainerStyle>(WidgetStyle::CONTAINER);
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
