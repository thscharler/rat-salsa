use crate::palette::Palette;
use crate::{Category, Colors, ColorsExt, Theme};
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
use rat_widget::radio::{RadioLayout, RadioStyle};
use rat_widget::scrolled::{ScrollStyle, ScrollSymbols};
use rat_widget::shadow::{ShadowDirection, ShadowStyle};
use rat_widget::slider::SliderStyle;
use rat_widget::splitter::SplitStyle;
use rat_widget::statusline::StatusLineStyle;
use rat_widget::tabbed::TabbedStyle;
use rat_widget::table::TableStyle;
use rat_widget::text::{TextFocusGained, TextFocusLost, TextStyle};
use rat_widget::view::ViewStyle;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders};
use std::time::Duration;

pub const SHELL: Palette = {
    let mut p = Palette {
        name: "Shell",

        color: [
            [
                Color::Gray,
                Color::Gray,
                Color::White,
                Color::White,
                Color::Gray,
                Color::Gray,
                Color::White,
                Color::White,
            ], // text light
            [
                Color::DarkGray,
                Color::DarkGray,
                Color::Black,
                Color::Black,
                Color::DarkGray,
                Color::DarkGray,
                Color::Black,
                Color::Black,
            ], // text dark
            [Color::Cyan; 8],   // primary
            [Color::Yellow; 8], // secondary
            [Color::White; 8],  // white
            [Color::Black; 8],
            [
                Color::Gray,
                Color::Gray,
                Color::DarkGray,
                Color::DarkGray,
                Color::Gray,
                Color::Gray,
                Color::DarkGray,
                Color::DarkGray,
            ],
            [Color::Red; 8],
            [Color::Yellow; 8],
            [Color::LightYellow; 8],
            [Color::LightGreen; 8],
            [Color::Green; 8],
            [Color::Cyan; 8],
            [Color::LightCyan; 8],
            [Color::LightBlue; 8],
            [Color::Blue; 8],
            [Color::Magenta; 8],
            [Color::LightMagenta; 8],
            [Color::LightRed; 8],
        ],
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::LabelFg as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][0];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][0];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::Red as usize][0];
    p.color_ext[ColorsExt::Hover as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Title as usize] = p.color[Colors::Red as usize][0];
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Header as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Footer as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::Shadows as usize] = p.color[Colors::TextDark as usize][0];
    p.color_ext[ColorsExt::TextFocus as usize] = p.color[Colors::Primary as usize][0];
    p.color_ext[ColorsExt::TextSelect as usize] = p.color[Colors::Secondary as usize][0];
    p.color_ext[ColorsExt::ButtonBase as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::MenuBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::KeyBinding as usize] = p.color[Colors::BlueGreen as usize][0];
    p.color_ext[ColorsExt::StatusBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::ContainerBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::ContainerBorderFg as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::ContainerArrowFg as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::PopupBase as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::PopupBorderFg as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::PopupArrowFg as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::DialogBase as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::DialogBorderFg as usize] = p.color[Colors::Black as usize][3];
    p.color_ext[ColorsExt::DialogArrowFg as usize] = p.color[Colors::Black as usize][3];

    p
};

/// A 'shell'-theme.
///
/// It uses almost no background colors and lets your shell
/// bleed through.
pub fn shell_shell_theme(name: &str) -> Theme {
    let p = SHELL;
    let mut th = Theme::new(name, Category::Shell, p);

    th.define(Style::LABEL_FG, th.p.fg_style_ext(ColorsExt::LabelFg));
    th.define(Style::INPUT, th.p.style_ext(ColorsExt::Input));
    th.define(Style::FOCUS, th.p.high_style_ext(ColorsExt::Focus));
    th.define(Style::SELECT, th.p.high_style_ext(ColorsExt::Select));
    th.define(Style::DISABLED, th.p.style_ext(ColorsExt::Disabled));
    th.define(Style::INVALID, th.p.style_ext(ColorsExt::Invalid));
    th.define(Style::HOVER, th.p.fg_style_ext(ColorsExt::Hover));
    th.define(Style::TITLE, th.p.fg_style_ext(ColorsExt::TitleFg));
    th.define(Style::HEADER, th.p.fg_style_ext(ColorsExt::HeaderFg));
    th.define(Style::FOOTER, th.p.fg_style_ext(ColorsExt::FooterFg));
    th.define(Style::SHADOWS, th.p.style_ext(ColorsExt::Shadows));
    th.define(Style::TEXT_FOCUS, th.p.high_style_ext(ColorsExt::TextFocus));
    th.define(Style::TEXT_SELECT, th.p.high_style_ext(ColorsExt::Select));
    th.define(
        Style::KEY_BINDING,
        th.p.high_style_ext(ColorsExt::KeyBinding),
    );

    th.define(
        Style::BUTTON_BASE,
        th.p.high_style_ext(ColorsExt::ButtonBase),
    );
    th.define(Style::MENU_BASE, th.p.fg_style(Colors::TextLight, 0));
    th.define(Style::STATUS_BASE, th.p.fg_style(Colors::TextLight, 0));

    th.define(Style::CONTAINER_BASE, th.p.fg_style(Colors::TextLight, 0));
    th.define(
        Style::CONTAINER_BORDER_FG,
        th.p.fg_style_ext(ColorsExt::ContainerBorderFg),
    );
    th.define(
        Style::CONTAINER_ARROW_FG,
        th.p.fg_style_ext(ColorsExt::ContainerArrowFg),
    );

    th.define(Style::POPUP_BASE, th.p.fg_style(Colors::TextLight, 0));
    th.define(
        Style::POPUP_BORDER_FG,
        th.p.fg_style_ext(ColorsExt::PopupBorderFg),
    );
    th.define(
        Style::POPUP_ARROW_FG,
        th.p.fg_style_ext(ColorsExt::PopupArrowFg),
    );

    th.define(Style::DIALOG_BASE, th.p.fg_style(Colors::TextLight, 0));
    th.define(
        Style::DIALOG_BORDER_FG,
        th.p.fg_style_ext(ColorsExt::DialogBorderFg),
    );
    th.define(
        Style::DIALOG_ARROW_FG,
        th.p.fg_style_ext(ColorsExt::DialogArrowFg),
    );

    th.define_fn(WidgetStyle::BUTTON, button);
    th.define_fn(WidgetStyle::CALENDAR, month);
    th.define_fn(WidgetStyle::CHECKBOX, checkbox);
    th.define_fn(WidgetStyle::CHOICE, choice);
    th.define_fn(WidgetStyle::CLIPPER, clipper);
    th.define_fn(WidgetStyle::COMBOBOX, combobox);
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

fn button(th: &Theme) -> ButtonStyle {
    ButtonStyle {
        style: th.style(Style::BUTTON_BASE),
        focus: Some(th.style(Style::FOCUS)),
        armed: Some(th.style(Style::SELECT)),
        hover: Some(th.p.style_ext(ColorsExt::Hover)),
        armed_delay: Some(Duration::from_millis(50)),
        ..Default::default()
    }
}

fn checkbox(th: &Theme) -> CheckboxStyle {
    CheckboxStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        ..Default::default()
    }
}

fn combobox(th: &Theme) -> ComboboxStyle {
    ComboboxStyle {
        choice: choice(th),
        text: text(th),
        ..Default::default()
    }
}

fn choice(th: &Theme) -> ChoiceStyle {
    ChoiceStyle {
        style: th.style(Style::INPUT),
        select: Some(th.style(Style::TEXT_SELECT)),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        popup_style: Some(th.style(Style::POPUP_BASE)),
        popup_border: Some(th.style(Style::POPUP_BORDER_FG)),
        popup_scroll: Some(popup_scroll(th)),
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

fn clipper(th: &Theme) -> ClipperStyle {
    ClipperStyle {
        style: th.style(Style::CONTAINER_BASE),
        label_style: Some(th.style(Style::LABEL_FG)),
        scroll: Some(scroll(th)),
        ..Default::default()
    }
}

fn dialog_frame(th: &Theme) -> DialogFrameStyle {
    DialogFrameStyle {
        style: th.style(Style::DIALOG_BASE),
        border_style: Some(th.style::<Style>(Style::DIALOG_BORDER_FG)),
        button_style: Some(button(th)),
        ..DialogFrameStyle::default()
    }
}

fn file_dialog(th: &Theme) -> FileDialogStyle {
    FileDialogStyle {
        style: th.style(Style::DIALOG_BASE),
        list: Some(list(th)),
        roots: Some(ListStyle {
            style: th.style(Style::DIALOG_BASE),
            ..list(th)
        }),
        text: Some(text(th)),
        button: Some(button(th)),
        block: Some(Block::bordered()),
        ..Default::default()
    }
}

fn form(th: &Theme) -> FormStyle {
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

fn line_nr(th: &Theme) -> LineNumberStyle {
    LineNumberStyle {
        style: th.style(Style::CONTAINER_BASE),
        cursor: Some(th.style(Style::TEXT_SELECT)),
        ..LineNumberStyle::default()
    }
}

fn list(th: &Theme) -> ListStyle {
    ListStyle {
        style: th.style(Style::CONTAINER_BASE),
        select: Some(th.style(Style::SELECT)),
        focus: Some(th.style(Style::FOCUS)),
        scroll: Some(scroll(th)),
        ..Default::default()
    }
}

fn menu(th: &Theme) -> MenuStyle {
    MenuStyle {
        style: th.style(Style::STATUS_BASE),
        title: Some(th.style(Style::TITLE)),
        focus: Some(th.style(Style::FOCUS)),
        right: Some(th.style(Style::KEY_BINDING)),
        disabled: Some(th.style(Style::DISABLED)),
        highlight: Some(Style::default().underlined()),
        block: Some(Block::bordered()),
        popup: Default::default(),
        popup_border: Some(th.style(Style::STATUS_BASE)),
        popup_style: Some(th.style(Style::STATUS_BASE)),
        ..Default::default()
    }
}

fn month(th: &Theme) -> CalendarStyle {
    CalendarStyle {
        style: th.style(Style::CONTAINER_BASE),
        title: None,
        weeknum: Some(th.style(Style::HEADER)),
        weekday: Some(th.style(Style::HEADER)),
        day: None,
        select: Some(th.style(Style::SELECT)),
        focus: Some(th.style(Style::FOCUS)),
        ..CalendarStyle::default()
    }
}

fn msg_dialog(th: &Theme) -> MsgDialogStyle {
    MsgDialogStyle {
        style: th.style(Style::DIALOG_BASE),
        button: Some(button(th)),
        ..Default::default()
    }
}

fn paragraph(th: &Theme) -> ParagraphStyle {
    ParagraphStyle {
        style: th.style(Style::CONTAINER_BASE),
        focus: Some(th.style(Style::FOCUS)),
        scroll: Some(scroll(th)),
        ..Default::default()
    }
}

fn radio(th: &Theme) -> RadioStyle {
    RadioStyle {
        layout: Some(RadioLayout::Stacked),
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        ..Default::default()
    }
}

/// Scroll style
fn scroll(th: &Theme) -> ScrollStyle {
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

fn popup_scroll(th: &Theme) -> ScrollStyle {
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

fn dialog_scroll(th: &Theme) -> ScrollStyle {
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

fn shadow(th: &Theme) -> ShadowStyle {
    ShadowStyle {
        style: th.style(Style::SHADOWS),
        dir: ShadowDirection::BottomRight,
        ..ShadowStyle::default()
    }
}

fn slider(th: &Theme) -> SliderStyle {
    SliderStyle {
        style: th.style(Style::INPUT),
        bounds: Some(th.style(Style::INPUT)),
        knob: Some(th.style(Style::TEXT_SELECT)),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        text_align: Some(Alignment::Center),
        ..Default::default()
    }
}

fn split(th: &Theme) -> SplitStyle {
    SplitStyle {
        style: th.style(Style::CONTAINER_BORDER_FG),
        arrow_style: Some(th.style(Style::CONTAINER_ARROW_FG)),
        drag_style: Some(th.style(Style::HOVER)),
        ..Default::default()
    }
}

fn statusline(th: &Theme) -> StatusLineStyle {
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

fn tabbed(th: &Theme) -> TabbedStyle {
    TabbedStyle {
        style: th.style(Style::CONTAINER_BASE),
        border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        tab: Some(th.p.fg_style_ext(ColorsExt::Input)),
        hover: Some(th.p.fg_style_ext(ColorsExt::Hover)),
        select: Some(th.p.fg_style_ext(ColorsExt::Select)),
        focus: Some(th.p.fg_style_ext(ColorsExt::Focus)),
        ..Default::default()
    }
}

fn table(th: &Theme) -> TableStyle {
    TableStyle {
        style: th.style(Style::CONTAINER_BASE),
        select_row: Some(th.style(Style::SELECT)),
        show_row_focus: true,
        focus_style: Some(th.style(Style::FOCUS)),
        border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        scroll: Some(scroll(th)),
        header: Some(th.style(Style::HEADER)),
        footer: Some(th.style(Style::FOOTER)),
        ..Default::default()
    }
}

fn color_input(th: &Theme) -> ColorInputStyle {
    ColorInputStyle {
        text: TextStyle {
            style: th.style(Style::INPUT),
            focus: Some(th.style(Style::TEXT_FOCUS)),
            select: Some(th.style(Style::TEXT_SELECT)),
            invalid: Some(th.style(Style::INVALID)),
            on_focus_gained: Some(TextFocusGained::Overwrite),
            on_focus_lost: Some(TextFocusLost::Position0),
            ..TextStyle::default()
        },
        ..Default::default()
    }
}

fn text(th: &Theme) -> TextStyle {
    TextStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        select: Some(th.style(Style::TEXT_SELECT)),
        invalid: Some(th.style(Style::INVALID)),
        ..TextStyle::default()
    }
}

fn textarea(th: &Theme) -> TextStyle {
    TextStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::INPUT)),
        select: Some(th.style(Style::TEXT_SELECT)),
        scroll: Some(scroll(th)),
        border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        ..TextStyle::default()
    }
}

fn textview(th: &Theme) -> TextStyle {
    TextStyle {
        style: th.style(Style::CONTAINER_BASE),
        focus: Some(th.style(Style::CONTAINER_BASE)),
        select: Some(th.style(Style::TEXT_SELECT)),
        scroll: Some(scroll(th)),
        border_style: Some(th.style(Style::CONTAINER_BORDER_FG)),
        ..TextStyle::default()
    }
}

fn view(th: &Theme) -> ViewStyle {
    ViewStyle {
        scroll: Some(scroll(th)),
        ..Default::default()
    }
}
