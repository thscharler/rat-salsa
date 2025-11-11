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
use ratatui::layout::Alignment;
use ratatui::style::{Style, Stylize};
use ratatui::symbols;
use ratatui::widgets::{Block, Borders};
use std::time::Duration;

/// A 'shell'-theme.
///
/// It uses almost no background colors and lets your shell
/// bleed through.
pub fn shell_theme(name: &str, p: Palette) -> SalsaTheme {
    let mut th = SalsaTheme::new(name, Category::Shell, p);

    th.define(Style::INPUT, th.p.gray(0));
    th.define(Style::FOCUS, th.p.high_contrast(th.p.primary[2]));
    th.define(Style::SELECT, th.p.high_contrast(th.p.secondary[1]));
    th.define(Style::TEXT_FOCUS, th.p.high_contrast(th.p.gray[3]));
    th.define(Style::TEXT_SELECT, th.p.high_contrast(th.p.secondary[0]));
    th.define(Style::TEXT_SELECT, th.p.gray(2));
    th.define(Style::BUTTON_BASE, th.p.gray(2));

    th.define(Style::CONTAINER_BASE, Style::default());
    th.define(Style::CONTAINER_BORDER, Style::default());
    th.define(Style::CONTAINER_ARROWS, Style::default());

    th.define(Style::POPUP_BASE, th.p.bg_gray(0));
    th.define(Style::POPUP_BORDER, th.p.bg_gray(0));
    th.define(Style::POPUP_ARROW, th.p.bg_gray(0));

    th.define(Style::DIALOG_BASE, th.p.bg_gray(1));
    th.define(Style::DIALOG_BORDER, th.p.bg_gray(1));
    th.define(Style::DIALOG_ARROW, th.p.bg_gray(1));

    th.define(Style::STATUS_BASE, Style::default());

    th.define_fn(WidgetStyle::BUTTON, button);
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

fn button(th: &SalsaTheme) -> ButtonStyle {
    ButtonStyle {
        style: th.style(Style::BUTTON_BASE),
        focus: Some(th.style(Style::FOCUS)),
        armed: Some(th.style(Style::SELECT)),
        hover: Some(th.style(Style::SELECT)),
        armed_delay: Some(Duration::from_millis(50)),
        ..Default::default()
    }
}

fn checkbox(th: &SalsaTheme) -> CheckboxStyle {
    CheckboxStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        ..Default::default()
    }
}

fn choice(th: &SalsaTheme) -> ChoiceStyle {
    ChoiceStyle {
        style: th.style(Style::INPUT),
        select: Some(th.style(Style::TEXT_SELECT)),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        popup_style: Some(th.style(Style::POPUP_BASE)),
        popup_border: Some(th.style(Style::POPUP_BORDER)),
        popup_scroll: Some(popup_scroll(th)),
        popup_block: Some(
            Block::bordered()
                .borders(Borders::LEFT)
                .border_style(th.style::<Style>(Style::POPUP_BORDER)),
        ),
        ..Default::default()
    }
}

fn clipper(th: &SalsaTheme) -> ClipperStyle {
    ClipperStyle {
        style: th.style(Style::CONTAINER_BASE),
        scroll: Some(scroll(th)),
        ..Default::default()
    }
}

fn combobox(th: &SalsaTheme) -> ComboboxStyle {
    ComboboxStyle {
        choice: choice(th),
        text: text(th),
        ..Default::default()
    }
}

fn dialog_frame(th: &SalsaTheme) -> DialogFrameStyle {
    DialogFrameStyle {
        style: th.style(Style::DIALOG_BASE),
        block: Some(Block::bordered().style(th.style::<Style>(Style::DIALOG_BORDER))),
        button_style: Some(button(th)),
        ..DialogFrameStyle::default()
    }
}

fn file_dialog(th: &SalsaTheme) -> FileDialogStyle {
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

fn form(th: &SalsaTheme) -> FormStyle {
    FormStyle {
        style: th.style(Style::CONTAINER_BASE),
        navigation: Some(th.style(Style::CONTAINER_ARROWS)),
        block: Some(
            Block::bordered()
                .borders(Borders::TOP | Borders::BOTTOM)
                .border_style(th.style::<Style>(Style::CONTAINER_BORDER)),
        ),
        ..Default::default()
    }
}

fn line_nr(th: &SalsaTheme) -> LineNumberStyle {
    LineNumberStyle {
        style: th.style(Style::CONTAINER_BASE),
        cursor: Some(th.style(Style::TEXT_SELECT)),
        ..LineNumberStyle::default()
    }
}

fn list(th: &SalsaTheme) -> ListStyle {
    ListStyle {
        style: th.style(Style::CONTAINER_BASE),
        select: Some(th.style(Style::SELECT)),
        focus: Some(th.style(Style::FOCUS)),
        scroll: Some(scroll(th)),
        ..Default::default()
    }
}

fn menu(th: &SalsaTheme) -> MenuStyle {
    MenuStyle {
        style: th.style(Style::STATUS_BASE),
        title: Some(th.p.bg_yellow(2)),
        focus: Some(th.style(Style::FOCUS)),
        right: Some(th.p.fg_green(3)),
        disabled: Some(th.p.fg_gray(0)),
        highlight: Some(Style::default().underlined()),
        block: Some(Block::bordered()),
        popup: Default::default(),
        popup_border: Some(th.style(Style::STATUS_BASE)),
        popup_style: Some(th.style(Style::STATUS_BASE)),
        ..Default::default()
    }
}

fn month(th: &SalsaTheme) -> CalendarStyle {
    CalendarStyle {
        style: Default::default(),
        title: None,
        weeknum: Some(th.p.fg_limegreen(0)),
        weekday: Some(th.p.fg_limegreen(0)),
        day: None,
        select: Some(th.style(Style::SELECT)),
        focus: Some(th.style(Style::FOCUS)),
        ..CalendarStyle::default()
    }
}

fn msg_dialog(th: &SalsaTheme) -> MsgDialogStyle {
    MsgDialogStyle {
        style: th.style(Style::DIALOG_BASE),
        button: Some(button(th)),
        ..Default::default()
    }
}

fn paragraph(th: &SalsaTheme) -> ParagraphStyle {
    ParagraphStyle {
        style: th.style(Style::CONTAINER_BASE),
        focus: Some(th.style(Style::FOCUS)),
        scroll: Some(scroll(th)),
        ..Default::default()
    }
}

fn radio(th: &SalsaTheme) -> RadioStyle {
    RadioStyle {
        layout: Some(RadioLayout::Stacked),
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        ..Default::default()
    }
}

/// Scroll style
fn scroll(th: &SalsaTheme) -> ScrollStyle {
    ScrollStyle {
        thumb_style: Some(th.style(Style::CONTAINER_BORDER)),
        track_style: Some(th.style(Style::CONTAINER_BORDER)),
        min_style: Some(th.style(Style::CONTAINER_BORDER)),
        begin_style: Some(th.style(Style::CONTAINER_ARROWS)),
        end_style: Some(th.style(Style::CONTAINER_ARROWS)),
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
        thumb_style: Some(th.style(Style::DIALOG_BORDER)),
        track_style: Some(th.style(Style::DIALOG_BORDER)),
        min_style: Some(th.style(Style::DIALOG_BORDER)),
        begin_style: Some(th.style(Style::DIALOG_ARROW)),
        end_style: Some(th.style(Style::DIALOG_ARROW)),
        ..Default::default()
    }
}

fn popup_scroll(th: &SalsaTheme) -> ScrollStyle {
    ScrollStyle {
        thumb_style: Some(th.style(Style::POPUP_BORDER)),
        track_style: Some(th.style(Style::POPUP_BORDER)),
        min_style: Some(th.style(Style::POPUP_BORDER)),
        begin_style: Some(th.style(Style::POPUP_ARROW)),
        end_style: Some(th.style(Style::POPUP_ARROW)),
        ..Default::default()
    }
}

fn shadow(th: &SalsaTheme) -> ShadowStyle {
    ShadowStyle {
        style: th.p.normal_contrast(th.p.black[0]),
        dir: ShadowDirection::BottomRight,
        ..ShadowStyle::default()
    }
}

fn slider(th: &SalsaTheme) -> SliderStyle {
    SliderStyle {
        style: th.style(Style::INPUT),
        bounds: Some(th.p.gray(2)),
        knob: Some(th.style(Style::TEXT_SELECT)),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        text_align: Some(Alignment::Center),
        ..Default::default()
    }
}

fn split(th: &SalsaTheme) -> SplitStyle {
    SplitStyle {
        style: th.style(Style::CONTAINER_BORDER),
        arrow_style: Some(th.style(Style::CONTAINER_ARROWS)),
        drag_style: Some(th.style(Style::FOCUS)),
        ..Default::default()
    }
}

fn statusline(th: &SalsaTheme) -> StatusLineStyle {
    StatusLineStyle {
        styles: vec![
            th.style(Style::STATUS_BASE),
            th.p.normal_contrast(th.p.blue[2]),
            th.p.normal_contrast(th.p.blue[2]),
            th.p.normal_contrast(th.p.blue[2]),
        ],
        ..Default::default()
    }
}

fn tabbed(th: &SalsaTheme) -> TabbedStyle {
    TabbedStyle {
        style: th.style(Style::CONTAINER_BASE),
        tab: Some(th.p.gray(2)),
        select: Some(th.p.secondary(0)),
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
        border_style: Some(th.style(Style::CONTAINER_BORDER)),
        scroll: Some(scroll(th)),
        header: Some(th.p.green(2)),
        footer: Some(th.p.green(2)),
        ..Default::default()
    }
}

fn color_input(th: &SalsaTheme) -> ColorInputStyle {
    ColorInputStyle {
        text: TextStyle {
            style: th.style(Style::INPUT),
            focus: Some(th.style(Style::TEXT_FOCUS)),
            select: Some(th.style(Style::TEXT_SELECT)),
            invalid: Some(th.p.fg_red(3)),
            ..TextStyle::default()
        },
        ..Default::default()
    }
}

fn text(th: &SalsaTheme) -> TextStyle {
    TextStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        select: Some(th.style(Style::TEXT_SELECT)),
        invalid: Some(th.p.fg_red(3)),
        ..TextStyle::default()
    }
}

fn textarea(th: &SalsaTheme) -> TextStyle {
    TextStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        select: Some(th.style(Style::TEXT_SELECT)),
        scroll: Some(scroll(th)),
        border_style: Some(th.style(Style::CONTAINER_BORDER)),
        ..TextStyle::default()
    }
}

fn textview(th: &SalsaTheme) -> TextStyle {
    TextStyle {
        style: th.style(Style::CONTAINER_BASE),
        focus: Some(th.style(Style::CONTAINER_BASE)),
        select: Some(th.style(Style::TEXT_SELECT)),
        scroll: Some(scroll(th)),
        border_style: Some(th.style(Style::CONTAINER_BORDER)),
        ..TextStyle::default()
    }
}

fn view(th: &SalsaTheme) -> ViewStyle {
    ViewStyle {
        scroll: Some(scroll(th)),
        ..Default::default()
    }
}
