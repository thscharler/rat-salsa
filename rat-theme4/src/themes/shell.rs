use crate::RatWidgetColor;
use crate::palette::{Colors, Palette};
use crate::theme::SalsaTheme;
use crate::{StyleName, WidgetStyle};
use rat_widget::button::ButtonStyle;
use rat_widget::calendar::CalendarStyle;
use rat_widget::checkbox::CheckboxStyle;
use rat_widget::choice::ChoiceStyle;
use rat_widget::clipper::ClipperStyle;
use rat_widget::combobox::ComboboxStyle;
use rat_widget::dialog_frame::DialogFrameStyle;
use rat_widget::file_dialog::FileDialogStyle;
use rat_widget::form::FormStyle;
use rat_widget::line_number::LineNumberStyle;
use rat_widget::list::ListStyle;
use rat_widget::menu::MenuStyle;
use rat_widget::msgdialog::MsgDialogStyle;
use rat_widget::paragraph::ParagraphStyle;
use rat_widget::radio::{RadioLayout, RadioStyle};
use rat_widget::scrolled::{ScrollStyle, ScrollSymbols};
use rat_widget::shadow::{ShadowDirection, ShadowStyle};
use rat_widget::slider::SliderStyle;
use rat_widget::splitter::SplitStyle;
use rat_widget::statusline::StatusLineStyle;
use rat_widget::tabbed::TabbedStyle;
use rat_widget::table::TableStyle;
use rat_widget::text::TextStyle;
use rat_widget::view::ViewStyle;
#[cfg(feature = "color-input")]
use rat_widget_extra::color_input::ColorInputStyle;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders};
use std::time::Duration;

/// A 'shell'-theme.
///
/// It uses almost no background colors and lets your shell
/// bleed through.
pub fn create_shell(p: Palette) -> SalsaTheme {
    let mut th = SalsaTheme::new(p);

    th.define_style(Style::LABEL_FG, th.p.fg_style_alias(Color::LABEL_FG));
    th.define_style(Style::INPUT, th.p.style_alias(Color::INPUT_BG));
    th.define_style(Style::FOCUS, th.p.high_style_alias(Color::FOCUS_BG));
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
        th.p.fg_bg_style_alias(Color::FOOTER_FG, Color::HEADER_BG),
    );
    th.define_style(
        Style::WEEK_HEADER_FG,
        th.p.fg_style_alias(Color::WEEK_HEADER_FG),
    );
    th.define_style(
        Style::MONTH_HEADER_FG,
        th.p.fg_style_alias(Color::MONTH_HEADER_FG),
    );
    th.define_style(Style::SHADOWS, th.p.style_alias(Color::SHADOW_BG));
    th.define_style(
        Style::INPUT_FOCUS,
        th.p.high_style_alias(Color::INPUT_FOCUS_BG),
    );
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
        th.p.fg_style_alias(Color::CONTAINER_BORDER_FG),
    );
    th.define_style(
        Style::CONTAINER_ARROW_FG,
        th.p.fg_style_alias(Color::CONTAINER_ARROW_FG),
    );

    th.define_style(
        Style::DOCUMENT_BASE,
        th.p.style_alias(Color::DOCUMENT_BASE_BG),
    );
    th.define_style(
        Style::DOCUMENT_BORDER_FG,
        th.p.fg_style_alias(Color::DOCUMENT_BORDER_FG),
    );
    th.define_style(
        Style::DOCUMENT_ARROW_FG,
        th.p.fg_style_alias(Color::DOCUMENT_ARROW_FG),
    );

    th.define_style(Style::POPUP_BASE, th.p.style_alias(Color::POPUP_BASE_BG));
    th.define_style(
        Style::POPUP_BORDER_FG,
        th.p.fg_style_alias(Color::POPUP_BORDER_FG),
    );
    th.define_style(
        Style::POPUP_ARROW_FG,
        th.p.fg_style_alias(Color::POPUP_ARROW_FG),
    );

    th.define_style(Style::DIALOG_BASE, th.p.style_alias(Color::DIALOG_BASE_BG));
    th.define_style(
        Style::DIALOG_BORDER_FG,
        th.p.fg_style_alias(Color::DIALOG_BORDER_FG),
    );
    th.define_style(
        Style::DIALOG_ARROW_FG,
        th.p.fg_style_alias(Color::DIALOG_ARROW_FG),
    );

    th.define_fn(WidgetStyle::BUTTON, button);
    th.define_fn(WidgetStyle::CALENDAR, month);
    th.define_fn(WidgetStyle::CHECKBOX, checkbox);
    th.define_fn(WidgetStyle::CHOICE, choice);
    th.define_fn(WidgetStyle::CLIPPER, clipper);
    th.define_fn(WidgetStyle::COMBOBOX, combobox);
    #[cfg(feature = "color-input")]
    th.define_fn(WidgetStyle::COLOR_INPUT, color_input);
    th.define_fn(WidgetStyle::DIALOG_FRAME, dialog_frame);
    th.define_fn(WidgetStyle::FILE_DIALOG, file_dialog);
    th.define_fn(WidgetStyle::FORM, form);
    th.define_fn(WidgetStyle::LINE_NR, line_nr);
    th.define_fn(WidgetStyle::LIST, list);
    th.define_fn(WidgetStyle::MENU, menu);
    th.define_fn(WidgetStyle::MONTH, month);
    th.define_fn(WidgetStyle::MSG_DIALOG, msg_dialog);
    th.define_fn(WidgetStyle::PARAGRAPH, paragraph);
    th.define_fn(WidgetStyle::RADIO, radio);
    th.define_fn(WidgetStyle::SCROLL, scroll);
    th.define_fn(WidgetStyle::SCROLL_DIALOG, dialog_scroll);
    th.define_fn(WidgetStyle::SCROLL_POPUP, popup_scroll);
    th.define_fn(WidgetStyle::SHADOW, shadow);
    th.define_fn(WidgetStyle::SLIDER, slider);
    th.define_fn(WidgetStyle::SPLIT, split);
    th.define_fn(WidgetStyle::STATUSLINE, statusline);
    th.define_fn(WidgetStyle::TABBED, tabbed);
    th.define_fn(WidgetStyle::TABLE, table);
    th.define_fn(WidgetStyle::TEXT, text);
    th.define_fn(WidgetStyle::TEXTAREA, textarea);
    th.define_fn(WidgetStyle::TEXTVIEW, textview);
    th.define_fn(WidgetStyle::VIEW, view);

    th
}

fn button(th: &SalsaTheme) -> ButtonStyle {
    ButtonStyle {
        style: th.style(Style::BUTTON_BASE),
        focus: Some(th.style(Style::FOCUS)),
        armed: Some(th.style(Style::SELECT)),
        hover: Some(th.p.style_alias(Color::HOVER_BG)),
        armed_delay: Some(Duration::from_millis(50)),
        ..Default::default()
    }
}

fn checkbox(th: &SalsaTheme) -> CheckboxStyle {
    CheckboxStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::INPUT_FOCUS)),
        ..Default::default()
    }
}

fn combobox(th: &SalsaTheme) -> ComboboxStyle {
    ComboboxStyle {
        choice: th.style(WidgetStyle::CHOICE),
        text: th.style(WidgetStyle::TEXT),
        ..Default::default()
    }
}

fn choice(th: &SalsaTheme) -> ChoiceStyle {
    ChoiceStyle {
        style: th.style(Style::INPUT),
        select: Some(th.style(Style::INPUT_SELECT)),
        focus: Some(th.style(Style::INPUT_FOCUS)),
        popup_style: Some(th.style(Style::POPUP_BASE)),
        popup_border: Some(th.style(Style::POPUP_BORDER_FG)),
        popup_scroll: Some(th.style(WidgetStyle::SCROLL_POPUP)),
        popup_block: Some(
            Block::bordered()
                .borders(Borders::LEFT | Borders::BOTTOM | Borders::RIGHT)
                .border_set(border::Set {
                    top_left: "X",
                    top_right: "X",
                    bottom_left: "▀",
                    bottom_right: "▀",
                    vertical_left: "▌",
                    vertical_right: "▐",
                    horizontal_top: "X",
                    horizontal_bottom: "▀",
                })
                .border_style(th.style::<Style>(Style::POPUP_BORDER_FG)),
        ),
        ..Default::default()
    }
}

fn clipper(th: &SalsaTheme) -> ClipperStyle {
    ClipperStyle {
        style: th.style(Style::CONTAINER_BASE),
        label_style: Some(th.style(Style::LABEL_FG)),
        scroll: Some(th.style(WidgetStyle::SCROLL)),
        ..Default::default()
    }
}

fn dialog_frame(th: &SalsaTheme) -> DialogFrameStyle {
    DialogFrameStyle {
        style: th.style(Style::DIALOG_BASE),
        border_style: Some(th.style::<Style>(Style::DIALOG_BORDER_FG)),
        button_style: Some(th.style(WidgetStyle::BUTTON)),
        ..DialogFrameStyle::default()
    }
}

fn file_dialog(th: &SalsaTheme) -> FileDialogStyle {
    FileDialogStyle {
        style: th.style(Style::DIALOG_BASE),
        list: Some(th.style(WidgetStyle::LIST)),
        roots: Some(ListStyle {
            style: th.style(Style::DIALOG_BASE),
            ..th.style(WidgetStyle::LIST)
        }),
        text: Some(th.style(WidgetStyle::TEXT)),
        button: Some(th.style(WidgetStyle::BUTTON)),
        block: Some(Block::bordered()),
        ..Default::default()
    }
}

fn form(th: &SalsaTheme) -> FormStyle {
    FormStyle {
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

fn line_nr(th: &SalsaTheme) -> LineNumberStyle {
    LineNumberStyle {
        style: th.style(Style::CONTAINER_BASE),
        cursor: Some(th.style(Style::INPUT_SELECT)),
        ..LineNumberStyle::default()
    }
}

fn list(th: &SalsaTheme) -> ListStyle {
    ListStyle {
        style: th.style(Style::CONTAINER_BASE),
        select: Some(th.style(Style::SELECT)),
        focus: Some(th.style(Style::FOCUS)),
        scroll: Some(th.style(WidgetStyle::SCROLL)),
        ..Default::default()
    }
}

fn menu(th: &SalsaTheme) -> MenuStyle {
    MenuStyle {
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

fn month(th: &SalsaTheme) -> CalendarStyle {
    CalendarStyle {
        style: th.style(Style::CONTAINER_BASE),
        title: Some(th.style(Style::MONTH_HEADER_FG)),
        weeknum: Some(th.style(Style::WEEK_HEADER_FG)),
        weekday: Some(th.style(Style::WEEK_HEADER_FG)),
        day: None,
        select: Some(th.style(Style::SELECT)),
        focus: Some(th.style(Style::FOCUS)),
        ..CalendarStyle::default()
    }
}

fn msg_dialog(th: &SalsaTheme) -> MsgDialogStyle {
    MsgDialogStyle {
        style: th.style(Style::DIALOG_BASE),
        button: Some(th.style(WidgetStyle::BUTTON)),
        markdown_header_1: Some(th.style_style(Style::TITLE)),
        markdown_header_n: Some(th.style_style(Style::HEADER)),
        ..Default::default()
    }
}

fn paragraph(th: &SalsaTheme) -> ParagraphStyle {
    ParagraphStyle {
        style: th.style(Style::CONTAINER_BASE),
        focus: Some(th.style(Style::FOCUS)),
        scroll: Some(th.style(WidgetStyle::SCROLL)),
        ..Default::default()
    }
}

fn radio(th: &SalsaTheme) -> RadioStyle {
    RadioStyle {
        layout: Some(RadioLayout::Stacked),
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::INPUT_FOCUS)),
        ..Default::default()
    }
}

/// Scroll style
fn scroll(th: &SalsaTheme) -> ScrollStyle {
    ScrollStyle {
        thumb_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        track_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        min_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        begin_style: Some(th.style(Style::CONTAINER_ARROW_FG)),
        end_style: Some(th.style(Style::CONTAINER_ARROW_FG)),
        horizontal: Some(ScrollSymbols {
            track: "▒",
            thumb: symbols::block::FULL,
            begin: "←",
            end: "→",
            min: "░",
        }),
        vertical: Some(ScrollSymbols {
            track: "▒",
            thumb: symbols::block::FULL,
            begin: "↑",
            end: "↓",
            min: "░",
        }),
        ..Default::default()
    }
}

fn popup_scroll(th: &SalsaTheme) -> ScrollStyle {
    ScrollStyle {
        thumb_style: Some(th.style(Style::POPUP_BORDER_FG)),
        track_style: Some(th.style(Style::POPUP_BORDER_FG)),
        min_style: Some(th.style(Style::POPUP_BORDER_FG)),
        begin_style: Some(th.style(Style::POPUP_ARROW_FG)),
        end_style: Some(th.style(Style::POPUP_ARROW_FG)),
        horizontal: Some(ScrollSymbols {
            track: "▒",
            thumb: symbols::block::FULL,
            begin: "←",
            end: "→",
            min: "░",
        }),
        vertical: Some(ScrollSymbols {
            track: "▒",
            thumb: symbols::block::FULL,
            begin: "↑",
            end: "↓",
            min: "░",
        }),
        ..Default::default()
    }
}

fn dialog_scroll(th: &SalsaTheme) -> ScrollStyle {
    ScrollStyle {
        thumb_style: Some(th.style(Style::DIALOG_BORDER_FG)),
        track_style: Some(th.style(Style::DIALOG_BORDER_FG)),
        min_style: Some(th.style(Style::DIALOG_BORDER_FG)),
        begin_style: Some(th.style(Style::POPUP_ARROW_FG)),
        end_style: Some(th.style(Style::POPUP_ARROW_FG)),
        horizontal: Some(ScrollSymbols {
            track: "▒",
            thumb: symbols::block::FULL,
            begin: "←",
            end: "→",
            min: "░",
        }),
        vertical: Some(ScrollSymbols {
            track: "▒",
            thumb: symbols::block::FULL,
            begin: "↑",
            end: "↓",
            min: "░",
        }),
        ..Default::default()
    }
}

fn shadow(th: &SalsaTheme) -> ShadowStyle {
    ShadowStyle {
        style: th.style(Style::SHADOWS),
        dir: ShadowDirection::BottomRight,
        ..ShadowStyle::default()
    }
}

fn slider(th: &SalsaTheme) -> SliderStyle {
    SliderStyle {
        style: th.style(Style::INPUT),
        bounds: Some(th.style(Style::INPUT)),
        knob: Some(th.style(Style::INPUT_SELECT)),
        focus: Some(th.style(Style::INPUT_FOCUS)),
        text_align: Some(Alignment::Center),
        ..Default::default()
    }
}

fn split(th: &SalsaTheme) -> SplitStyle {
    SplitStyle {
        style: th.style(Style::CONTAINER_BORDER_FG),
        arrow_style: Some(th.style(Style::CONTAINER_ARROW_FG)),
        drag_style: Some(th.style(Style::HOVER)),
        ..Default::default()
    }
}

fn statusline(th: &SalsaTheme) -> StatusLineStyle {
    StatusLineStyle {
        styles: vec![
            th.style(Style::STATUS_BASE),
            th.p.style(Colors::Blue, 3),
            th.p.style(Colors::Blue, 2),
            th.p.style(Colors::Blue, 1),
        ],
        ..Default::default()
    }
}

fn tabbed(th: &SalsaTheme) -> TabbedStyle {
    TabbedStyle {
        style: th.style(Style::CONTAINER_BASE),
        border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        tab: Some(th.style(Style::CONTAINER_BASE)),
        hover: Some(th.style(Style::HOVER)),
        select: Some(th.style(Style::SELECT)),
        focus: Some(th.style(Style::FOCUS)),
        ..Default::default()
    }
}

fn table(th: &SalsaTheme) -> TableStyle {
    TableStyle {
        style: th.style(Style::CONTAINER_BASE),
        select_row: Some(th.style(Style::SELECT)),
        show_row_focus: true,
        focus_style: Some(th.style(Style::FOCUS)),
        border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        scroll: Some(th.style(WidgetStyle::SCROLL)),
        header: Some(th.style(Style::HEADER)),
        footer: Some(th.style(Style::FOOTER)),
        ..Default::default()
    }
}

#[cfg(feature = "color-input")]
fn color_input(th: &SalsaTheme) -> ColorInputStyle {
    ColorInputStyle {
        text: TextStyle {
            style: th.style(Style::INPUT),
            focus: Some(th.style(Style::INPUT_FOCUS)),
            select: Some(th.style(Style::INPUT_SELECT)),
            invalid: Some(th.style(Style::INVALID)),
            ..TextStyle::default()
        },
        ..Default::default()
    }
}

fn text(th: &SalsaTheme) -> TextStyle {
    TextStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::INPUT_FOCUS)),
        select: Some(th.style(Style::INPUT_SELECT)),
        invalid: Some(th.style(Style::INVALID)),
        ..TextStyle::default()
    }
}

fn textarea(th: &SalsaTheme) -> TextStyle {
    TextStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::INPUT)),
        select: Some(th.style(Style::INPUT_SELECT)),
        scroll: Some(th.style(WidgetStyle::SCROLL)),
        border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        ..TextStyle::default()
    }
}

fn textview(th: &SalsaTheme) -> TextStyle {
    TextStyle {
        style: th.style(Style::CONTAINER_BASE),
        focus: Some(th.style(Style::CONTAINER_BASE)),
        select: Some(th.style(Style::INPUT_SELECT)),
        scroll: Some(th.style(WidgetStyle::SCROLL)),
        border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        ..TextStyle::default()
    }
}

fn view(th: &SalsaTheme) -> ViewStyle {
    ViewStyle {
        scroll: Some(th.style(WidgetStyle::SCROLL)),
        ..Default::default()
    }
}
