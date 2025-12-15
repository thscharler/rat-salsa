#![allow(unused_imports)]
use crate::RatWidgetColor;
use crate::palette::{Colors, Palette};
use crate::theme::SalsaTheme;
use crate::{StyleName, WidgetStyle};
#[cfg(feature = "rat-widget")]
use rat_widget;
#[cfg(feature = "color-input")]
use rat_widget_extra::color_input::ColorInputStyle;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders};
use std::time::Duration;

/// A dark theme.
pub fn create_dark(p: Palette) -> SalsaTheme {
    let mut th = SalsaTheme::new(p);

    th.define_style(Style::LABEL_FG, th.p.fg_style_alias(Color::LABEL_FG));
    th.define_style(Style::INPUT, th.p.style_alias(Color::INPUT_BG));
    th.define_style(Style::FOCUS, th.p.style_alias(Color::FOCUS_BG));
    th.define_style(Style::SELECT, th.p.style_alias(Color::SELECT_BG));
    th.define_style(Style::DISABLED, th.p.style_alias(Color::DISABLED_BG));
    th.define_style(Style::INVALID, th.p.style_alias(Color::INVALID_BG));
    th.define_style(Style::HOVER, th.p.style_alias(Color::HOVER_BG));
    th.define_style(
        Style::TITLE,
        th.p.fg_bg_style_alias(Color::TITLE_FG, Color::TITLE_BG),
    );
    th.define_style(
        Style::HEADER,
        th.p.fg_bg_style_alias(Color::HEADER_FG, Color::HEADER_BG),
    );
    th.define_style(
        Style::FOOTER,
        th.p.fg_bg_style_alias(Color::FOOTER_FG, Color::FOOTER_BG),
    );
    th.define_style(Style::SHADOWS, th.p.style_alias(Color::SHADOW_BG));
    th.define_style(
        Style::WEEK_HEADER_FG,
        th.p.fg_style_alias(Color::WEEK_HEADER_FG),
    );
    th.define_style(
        Style::MONTH_HEADER_FG,
        th.p.fg_style_alias(Color::MONTH_HEADER_FG),
    );
    th.define_style(Style::SHADOWS, th.p.style_alias(Color::SHADOW_BG));
    th.define_style(Style::INPUT_FOCUS, th.p.style_alias(Color::INPUT_FOCUS_BG));
    th.define_style(Style::INPUT_SELECT, th.p.style_alias(Color::SELECT_BG));
    th.define_style(Style::KEY_BINDING, th.p.style_alias(Color::KEY_BINDING_BG));

    th.define_style(Style::BUTTON_BASE, th.p.style_alias(Color::BUTTON_BASE_BG));
    th.define_style(Style::MENU_BASE, th.p.style_alias(Color::MENU_BASE_BG));
    th.define_style(Style::STATUS_BASE, th.p.style_alias(Color::STATUS_BASE_BG));

    th.define_style(
        Style::CONTAINER_BASE,
        th.p.style_alias(Color::CONTAINER_BASE_BG),
    );
    th.define_style(
        Style::CONTAINER_BORDER_FG,
        th.p.fg_bg_style_alias(Color::CONTAINER_BORDER_FG, Color::CONTAINER_BASE_BG),
    );
    th.define_style(
        Style::CONTAINER_ARROW_FG,
        th.p.fg_bg_style_alias(Color::CONTAINER_ARROW_FG, Color::CONTAINER_BASE_BG),
    );

    th.define_style(
        Style::DOCUMENT_BASE,
        th.p.style_alias(Color::DOCUMENT_BASE_BG),
    );
    th.define_style(
        Style::DOCUMENT_BORDER_FG,
        th.p.fg_bg_style_alias(Color::DOCUMENT_BORDER_FG, Color::DOCUMENT_BASE_BG),
    );
    th.define_style(
        Style::DOCUMENT_ARROW_FG,
        th.p.fg_bg_style_alias(Color::DOCUMENT_ARROW_FG, Color::DOCUMENT_BASE_BG),
    );

    th.define_style(Style::POPUP_BASE, th.p.style_alias(Color::POPUP_BASE_BG));
    th.define_style(
        Style::POPUP_BORDER_FG,
        th.p.fg_bg_style_alias(Color::POPUP_BORDER_FG, Color::POPUP_BASE_BG),
    );
    th.define_style(
        Style::POPUP_ARROW_FG,
        th.p.fg_bg_style_alias(Color::POPUP_ARROW_FG, Color::POPUP_BASE_BG),
    );

    th.define_style(Style::DIALOG_BASE, th.p.style_alias(Color::DIALOG_BASE_BG));
    th.define_style(
        Style::DIALOG_BORDER_FG,
        th.p.fg_bg_style_alias(Color::DIALOG_BORDER_FG, Color::DIALOG_BASE_BG),
    );
    th.define_style(
        Style::DIALOG_ARROW_FG,
        th.p.fg_bg_style_alias(Color::DIALOG_ARROW_FG, Color::DIALOG_BASE_BG),
    );

    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::BUTTON, button);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::CALENDAR, month);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::CHECKBOX, checkbox);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::CHOICE, choice);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::CLIPPER, clipper);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::COMBOBOX, combobox);
    #[cfg(feature = "color-input")]
    th.define_fn(WidgetStyle::COLOR_INPUT, color_input);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::DIALOG_FRAME, dialog_frame);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::FILE_DIALOG, file_dialog);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::FORM, form);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::LINE_NR, line_nr);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::LIST, list);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::MENU, menu);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::MONTH, month);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::MSG_DIALOG, msg_dialog);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::PARAGRAPH, paragraph);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::RADIO, radio);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::SCROLL, scroll);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::SCROLL_DIALOG, dialog_scroll);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::SCROLL_POPUP, popup_scroll);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::SHADOW, shadow);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::SLIDER, slider);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::SPLIT, split);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::STATUSLINE, statusline);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::TABBED, tabbed);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::TABLE, table);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::TEXT, text);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::TEXTAREA, textarea);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::TEXTVIEW, textview);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::TOOLBAR, toolbar);
    #[cfg(feature = "rat-widget")]
    th.define_fn(WidgetStyle::VIEW, view);

    th
}

#[cfg(feature = "rat-widget")]
fn button(th: &SalsaTheme) -> rat_widget::button::ButtonStyle {
    rat_widget::button::ButtonStyle {
        style: th.style(Style::BUTTON_BASE),
        focus: Some(th.style(Style::FOCUS)),
        armed: Some(th.style(Style::SELECT)),
        hover: Some(th.style(Style::HOVER)),
        armed_delay: Some(Duration::from_millis(50)),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn checkbox(th: &SalsaTheme) -> rat_widget::checkbox::CheckboxStyle {
    rat_widget::checkbox::CheckboxStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::INPUT_FOCUS)),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn combobox(th: &SalsaTheme) -> rat_widget::combobox::ComboboxStyle {
    rat_widget::combobox::ComboboxStyle {
        choice: th.style(WidgetStyle::CHOICE),
        text: th.style(WidgetStyle::TEXT),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn choice(th: &SalsaTheme) -> rat_widget::choice::ChoiceStyle {
    rat_widget::choice::ChoiceStyle {
        style: th.style(Style::INPUT),
        select: Some(th.style(Style::INPUT_SELECT)),
        focus: Some(th.style(Style::INPUT_FOCUS)),
        popup_style: Some(th.style(Style::POPUP_BASE)),
        popup_border: Some(th.style(Style::POPUP_BORDER_FG)),
        popup_scroll: Some(th.style(WidgetStyle::SCROLL_POPUP)),
        popup_block: Some(
            Block::bordered()
                .borders(Borders::LEFT)
                .border_style(th.style::<Style>(Style::POPUP_BORDER_FG)),
        ),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn clipper(th: &SalsaTheme) -> rat_widget::clipper::ClipperStyle {
    rat_widget::clipper::ClipperStyle {
        style: th.style(Style::CONTAINER_BASE),
        label_style: Some(th.style(Style::LABEL_FG)),
        scroll: Some(th.style(WidgetStyle::SCROLL)),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn dialog_frame(th: &SalsaTheme) -> rat_widget::dialog_frame::DialogFrameStyle {
    rat_widget::dialog_frame::DialogFrameStyle {
        style: th.style(Style::DIALOG_BASE),
        border_style: Some(th.style::<Style>(Style::DIALOG_BORDER_FG)),
        button_style: Some(th.style(WidgetStyle::BUTTON)),
        ..rat_widget::dialog_frame::DialogFrameStyle::default()
    }
}

#[cfg(feature = "rat-widget")]
fn file_dialog(th: &SalsaTheme) -> rat_widget::file_dialog::FileDialogStyle {
    rat_widget::file_dialog::FileDialogStyle {
        style: th.style(Style::DIALOG_BASE),
        list: Some(th.style(WidgetStyle::LIST)),
        roots: Some(rat_widget::list::ListStyle {
            style: th.style(Style::DIALOG_BASE),
            ..th.style(WidgetStyle::LIST)
        }),
        text: Some(th.style(WidgetStyle::TEXT)),
        button: Some(th.style(WidgetStyle::BUTTON)),
        block: Some(Block::bordered()),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn form(th: &SalsaTheme) -> rat_widget::form::FormStyle {
    rat_widget::form::FormStyle {
        style: th.style(Style::CONTAINER_BASE),
        label_style: Some(th.style(Style::LABEL_FG)),
        navigation: Some(th.style(Style::CONTAINER_ARROW_FG)),
        navigation_hover: Some(th.style(Style::HOVER)),
        block: Some(
            Block::bordered()
                .borders(Borders::TOP | Borders::BOTTOM)
                .border_set(border::EMPTY)
                .border_style(th.style::<Style>(Style::CONTAINER_BORDER_FG)),
        ),
        border_style: Some(th.style::<Style>(Style::CONTAINER_BORDER_FG)),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn line_nr(th: &SalsaTheme) -> rat_widget::line_number::LineNumberStyle {
    rat_widget::line_number::LineNumberStyle {
        style: th.style(Style::CONTAINER_BORDER_FG),
        cursor: Some(th.style(Style::INPUT_SELECT)),
        ..rat_widget::line_number::LineNumberStyle::default()
    }
}

#[cfg(feature = "rat-widget")]
fn list(th: &SalsaTheme) -> rat_widget::list::ListStyle {
    rat_widget::list::ListStyle {
        style: th.style(Style::CONTAINER_BASE),
        select: Some(th.style(Style::SELECT)),
        focus: Some(th.style(Style::FOCUS)),
        scroll: Some(th.style(WidgetStyle::SCROLL)),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn menu(th: &SalsaTheme) -> rat_widget::menu::MenuStyle {
    rat_widget::menu::MenuStyle {
        style: th.style(Style::MENU_BASE),
        title: Some(th.style(Style::TITLE)),
        focus: Some(th.style(Style::FOCUS)),
        right: Some(th.style(Style::KEY_BINDING)),
        disabled: Some(th.style(Style::DISABLED)),
        highlight: Some(Style::default().underlined()),
        popup_block: Some(Block::bordered()),
        popup: Default::default(),
        popup_border: Some(th.style(Style::MENU_BASE)),
        popup_style: Some(th.style(Style::MENU_BASE)),
        popup_focus: Some(th.style(Style::FOCUS)),
        popup_right: Some(th.style(Style::KEY_BINDING)),
        popup_disabled: Some(th.style(Style::DISABLED)),
        popup_highlight: Some(Style::default().underlined()),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn month(th: &SalsaTheme) -> rat_widget::calendar::CalendarStyle {
    rat_widget::calendar::CalendarStyle {
        style: th.style(Style::CONTAINER_BASE),
        title: Some(th.style(Style::MONTH_HEADER_FG)),
        weeknum: Some(th.style(Style::WEEK_HEADER_FG)),
        weekday: Some(th.style(Style::WEEK_HEADER_FG)),
        day: None,
        select: Some(th.style(Style::SELECT)),
        focus: Some(th.style(Style::FOCUS)),
        ..rat_widget::calendar::CalendarStyle::default()
    }
}

#[cfg(feature = "rat-widget")]
fn msg_dialog(th: &SalsaTheme) -> rat_widget::msgdialog::MsgDialogStyle {
    rat_widget::msgdialog::MsgDialogStyle {
        style: th.style(Style::DIALOG_BASE),
        button: Some(th.style(WidgetStyle::BUTTON)),
        markdown_header_1: Some(th.style_style(Style::TITLE)),
        markdown_header_n: Some(th.style_style(Style::HEADER)),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn paragraph(th: &SalsaTheme) -> rat_widget::paragraph::ParagraphStyle {
    rat_widget::paragraph::ParagraphStyle {
        style: th.style(Style::CONTAINER_BASE),
        focus: Some(th.style(Style::FOCUS)),
        scroll: Some(th.style(WidgetStyle::SCROLL)),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn radio(th: &SalsaTheme) -> rat_widget::radio::RadioStyle {
    rat_widget::radio::RadioStyle {
        layout: Some(rat_widget::radio::RadioLayout::Stacked),
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::INPUT_FOCUS)),
        ..Default::default()
    }
}

/// Scroll style
#[cfg(feature = "rat-widget")]
fn scroll(th: &SalsaTheme) -> rat_widget::scrolled::ScrollStyle {
    rat_widget::scrolled::ScrollStyle {
        thumb_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        track_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        min_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        begin_style: Some(th.style(Style::CONTAINER_ARROW_FG)),
        end_style: Some(th.style(Style::CONTAINER_ARROW_FG)),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn popup_scroll(th: &SalsaTheme) -> rat_widget::scrolled::ScrollStyle {
    rat_widget::scrolled::ScrollStyle {
        thumb_style: Some(th.style(Style::POPUP_BORDER_FG)),
        track_style: Some(th.style(Style::POPUP_BORDER_FG)),
        min_style: Some(th.style(Style::POPUP_BORDER_FG)),
        begin_style: Some(th.style(Style::POPUP_ARROW_FG)),
        end_style: Some(th.style(Style::POPUP_ARROW_FG)),
        horizontal: Some(rat_widget::scrolled::ScrollSymbols {
            track: "",
            thumb: "▄",
            begin: "▗",
            end: "▖",
            min: " ",
        }),
        vertical: Some(rat_widget::scrolled::ScrollSymbols {
            track: " ",
            thumb: "█",
            begin: "▄",
            end: "▀",
            min: " ",
        }),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn dialog_scroll(th: &SalsaTheme) -> rat_widget::scrolled::ScrollStyle {
    rat_widget::scrolled::ScrollStyle {
        thumb_style: Some(th.style(Style::DIALOG_BORDER_FG)),
        track_style: Some(th.style(Style::DIALOG_BORDER_FG)),
        min_style: Some(th.style(Style::DIALOG_BORDER_FG)),
        begin_style: Some(th.style(Style::POPUP_ARROW_FG)),
        end_style: Some(th.style(Style::POPUP_ARROW_FG)),
        horizontal: Some(rat_widget::scrolled::ScrollSymbols {
            track: "",
            thumb: "▄",
            begin: "▗",
            end: "▖",
            min: " ",
        }),
        vertical: Some(rat_widget::scrolled::ScrollSymbols {
            track: " ",
            thumb: "█",
            begin: "▄",
            end: "▀",
            min: " ",
        }),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn shadow(th: &SalsaTheme) -> rat_widget::shadow::ShadowStyle {
    rat_widget::shadow::ShadowStyle {
        style: th.style(Style::SHADOWS),
        dir: rat_widget::shadow::ShadowDirection::BottomRight,
        ..rat_widget::shadow::ShadowStyle::default()
    }
}

#[cfg(feature = "rat-widget")]
fn slider(th: &SalsaTheme) -> rat_widget::slider::SliderStyle {
    rat_widget::slider::SliderStyle {
        style: th.style(Style::INPUT),
        bounds: Some(th.style(Style::INPUT)),
        knob: Some(th.style(Style::INPUT_SELECT)),
        focus: Some(th.style(Style::INPUT_FOCUS)),
        text_align: Some(Alignment::Center),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn split(th: &SalsaTheme) -> rat_widget::splitter::SplitStyle {
    rat_widget::splitter::SplitStyle {
        style: th.style(Style::CONTAINER_BORDER_FG),
        arrow_style: Some(th.style(Style::CONTAINER_ARROW_FG)),
        drag_style: Some(th.style(Style::HOVER)),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn statusline(th: &SalsaTheme) -> rat_widget::statusline::StatusLineStyle {
    rat_widget::statusline::StatusLineStyle {
        styles: vec![
            th.style(Style::STATUS_BASE),
            th.p.style(Colors::Blue, 3),
            th.p.style(Colors::Blue, 2),
            th.p.style(Colors::Blue, 1),
        ],
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn tabbed(th: &SalsaTheme) -> rat_widget::tabbed::TabbedStyle {
    rat_widget::tabbed::TabbedStyle {
        style: th.style(Style::CONTAINER_BASE),
        border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        tab: Some(th.style(Style::INPUT)),
        hover: Some(th.style(Style::HOVER)),
        select: Some(th.style(Style::SELECT)),
        focus: Some(th.style(Style::FOCUS)),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn table(th: &SalsaTheme) -> rat_widget::table::TableStyle {
    rat_widget::table::TableStyle {
        style: th.style(Style::CONTAINER_BASE),
        select_row: Some(th.style(Style::SELECT)),
        show_row_focus: true,
        focus_style: Some(th.style(Style::FOCUS)),
        border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        scroll: Some(th.style(WidgetStyle::SCROLL)),
        header: Some(th.style(Style::HEADER)),
        footer: Some(th.style(Style::FOOTER)),
        show_empty: true,
        ..Default::default()
    }
}

#[cfg(feature = "color-input")]
fn color_input(th: &SalsaTheme) -> ColorInputStyle {
    ColorInputStyle {
        text: rat_widget::text::TextStyle {
            style: th.style(Style::INPUT),
            focus: Some(th.style(Style::INPUT_FOCUS)),
            select: Some(th.style(Style::INPUT_SELECT)),
            invalid: Some(th.style(Style::INVALID)),
            ..rat_widget::text::TextStyle::default()
        },
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn text(th: &SalsaTheme) -> rat_widget::text::TextStyle {
    rat_widget::text::TextStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::INPUT_FOCUS)),
        select: Some(th.style(Style::INPUT_SELECT)),
        invalid: Some(th.style(Style::INVALID)),
        ..rat_widget::text::TextStyle::default()
    }
}

#[cfg(feature = "rat-widget")]
fn textarea(th: &SalsaTheme) -> rat_widget::text::TextStyle {
    rat_widget::text::TextStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::INPUT)),
        select: Some(th.style(Style::INPUT_SELECT)),
        scroll: Some(th.style(WidgetStyle::SCROLL)),
        border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        ..rat_widget::text::TextStyle::default()
    }
}

#[cfg(feature = "rat-widget")]
fn textview(th: &SalsaTheme) -> rat_widget::text::TextStyle {
    rat_widget::text::TextStyle {
        style: th.style(Style::CONTAINER_BASE),
        focus: Some(th.style(Style::CONTAINER_BASE)),
        select: Some(th.style(Style::INPUT_SELECT)),
        scroll: Some(th.style(WidgetStyle::SCROLL)),
        border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        ..rat_widget::text::TextStyle::default()
    }
}

#[cfg(feature = "rat-widget")]
fn toolbar(th: &SalsaTheme) -> rat_widget::toolbar::ToolbarStyle {
    rat_widget::toolbar::ToolbarStyle {
        style: th.style_style(Style::MENU_BASE),
        key_style: Some(th.style_style(Style::KEY_BINDING)),
        button: Some(rat_widget::button::ButtonStyle {
            style: th.style_style(Style::BUTTON_BASE),
            armed: Some(th.style_style(Style::SELECT)),
            hover: Some(th.style_style(Style::HOVER)),
            ..Default::default()
        }),
        checkbox: Some(rat_widget::checkbox::CheckboxStyle {
            style: th.style_style(Style::BUTTON_BASE),
            behave_check: Some(rat_widget::checkbox::CheckboxCheck::SingleClick),
            ..Default::default()
        }),
        choice: Some(rat_widget::choice::ChoiceStyle {
            style: th.style_style(Style::BUTTON_BASE),
            button: Some(th.style_style(Style::BUTTON_BASE)),
            select: Some(th.style_style(Style::SELECT)),
            focus: Some(th.style_style(Style::BUTTON_BASE)),
            popup: rat_widget::popup::PopupStyle {
                placement: Some(rat_widget::popup::Placement::BelowOrAbove),
                ..Default::default()
            },
            popup_style: Some(th.style_style(Style::POPUP_BASE)),
            popup_border: Some(th.style_style(Style::POPUP_BORDER_FG)),
            behave_focus: Some(rat_widget::choice::ChoiceFocus::OpenOnFocusGained),
            behave_select: Some(rat_widget::choice::ChoiceSelect::MouseClick),
            behave_close: Some(rat_widget::choice::ChoiceClose::SingleClick),
            ..Default::default()
        }),
        collapsed: Some(rat_widget::choice::ChoiceStyle {
            style: th.style_style(Style::BUTTON_BASE),
            button: Some(th.style_style(Style::BUTTON_BASE)),
            select: Some(th.style_style(Style::SELECT)),
            focus: Some(th.style_style(Style::BUTTON_BASE)),
            popup: rat_widget::popup::PopupStyle {
                placement: Some(rat_widget::popup::Placement::BelowOrAbove),
                ..Default::default()
            },
            popup_style: Some(th.style_style(Style::POPUP_BASE)),
            popup_border: Some(th.style_style(Style::POPUP_BORDER_FG)),
            behave_focus: Some(rat_widget::choice::ChoiceFocus::OpenOnFocusGained),
            behave_select: Some(rat_widget::choice::ChoiceSelect::MouseClick),
            behave_close: Some(rat_widget::choice::ChoiceClose::SingleClick),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(feature = "rat-widget")]
fn view(th: &SalsaTheme) -> rat_widget::view::ViewStyle {
    rat_widget::view::ViewStyle {
        scroll: Some(th.style(WidgetStyle::SCROLL)),
        ..Default::default()
    }
}
