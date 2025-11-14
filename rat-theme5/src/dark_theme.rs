use crate::palette::{Colors, ColorsExt, Palette};
use crate::{Category, Theme};
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
use rat_widget::scrolled::ScrollStyle;
use rat_widget::shadow::{ShadowDirection, ShadowStyle};
use rat_widget::slider::SliderStyle;
use rat_widget::splitter::SplitStyle;
use rat_widget::statusline::StatusLineStyle;
use rat_widget::tabbed::TabbedStyle;
use rat_widget::table::TableStyle;
use rat_widget::text::{TextFocusGained, TextFocusLost, TextStyle};
use rat_widget::view::ViewStyle;
use ratatui::layout::Alignment;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders};
use std::time::Duration;

/// A dark theme.
pub fn dark_theme(name: &str, p: Palette) -> Theme {
    let mut th = Theme::new(name, Category::Dark, p);

    th.define(Style::INPUT, th.p.style_ext(ColorsExt::Input));
    th.define(Style::FOCUS, th.p.style_ext(ColorsExt::Focus));
    th.define(Style::SELECT, th.p.style_ext(ColorsExt::Select));
    th.define(Style::DISABLED, th.p.style_ext(ColorsExt::Disabled));
    th.define(Style::INVALID, th.p.style_ext(ColorsExt::Invalid));
    th.define(Style::TITLE, th.p.style_ext(ColorsExt::Title));
    th.define(Style::HEADER, th.p.style_ext(ColorsExt::Header));
    th.define(Style::FOOTER, th.p.style_ext(ColorsExt::Footer));
    th.define(Style::SHADOW, th.p.style_ext(ColorsExt::Shadow));
    th.define(Style::TEXT_FOCUS, th.p.style_ext(ColorsExt::TextFocus));
    th.define(Style::TEXT_SELECT, th.p.style_ext(ColorsExt::Select));
    th.define(Style::KEY_BINDING, th.p.style_ext(ColorsExt::KeyBinding));

    th.define(Style::BUTTON_BASE, th.p.style_ext(ColorsExt::ButtonBase));
    th.define(Style::MENU_BASE, th.p.style_ext(ColorsExt::MenuBase));
    th.define(Style::STATUS_BASE, th.p.style_ext(ColorsExt::StatusBase));

    th.define(
        Style::CONTAINER_BASE,
        th.p.style_ext(ColorsExt::ContainerBase),
    );
    th.define(
        Style::CONTAINER_BORDER,
        th.p.style_ext(ColorsExt::ContainerBorder),
    );
    th.define(
        Style::CONTAINER_ARROW,
        th.p.style_ext(ColorsExt::ContainerArrow),
    );

    th.define(Style::POPUP_BASE, th.p.style_ext(ColorsExt::PopupBase));
    th.define(Style::POPUP_BORDER, th.p.style_ext(ColorsExt::PopupBorder));
    th.define(Style::POPUP_ARROW, th.p.style_ext(ColorsExt::PopupArrow));

    th.define(Style::DIALOG_BASE, th.p.style_ext(ColorsExt::DialogBase));
    th.define(
        Style::DIALOG_BORDER,
        th.p.style_ext(ColorsExt::DialogBorder),
    );
    th.define(Style::DIALOG_ARROW, th.p.style_ext(ColorsExt::DialogArrow));

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

fn button(th: &Theme) -> ButtonStyle {
    ButtonStyle {
        style: th.style(Style::BUTTON_BASE),
        focus: Some(th.style(Style::FOCUS)),
        armed: Some(th.style(Style::SELECT)),
        hover: Some(th.style(Style::SELECT)),
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

fn clipper(th: &Theme) -> ClipperStyle {
    ClipperStyle {
        style: th.style(Style::CONTAINER_BASE),
        scroll: Some(scroll(th)),
        ..Default::default()
    }
}

fn dialog_frame(th: &Theme) -> DialogFrameStyle {
    DialogFrameStyle {
        style: th.style(Style::DIALOG_BASE),
        block: Some(Block::bordered().style(th.style::<Style>(Style::DIALOG_BORDER))),
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
        navigation: Some(th.style(Style::CONTAINER_ARROW)),
        block: Some(
            Block::bordered()
                .borders(Borders::TOP | Borders::BOTTOM)
                .border_style(th.style::<Style>(Style::CONTAINER_BORDER)),
        ),
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
        style: th.style(Style::INPUT),
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
        thumb_style: Some(th.style(Style::CONTAINER_BORDER)),
        track_style: Some(th.style(Style::CONTAINER_BORDER)),
        min_style: Some(th.style(Style::CONTAINER_BORDER)),
        begin_style: Some(th.style(Style::CONTAINER_ARROW)),
        end_style: Some(th.style(Style::CONTAINER_ARROW)),
        ..Default::default()
    }
}

fn popup_scroll(th: &Theme) -> ScrollStyle {
    ScrollStyle {
        thumb_style: Some(th.style(Style::POPUP_BORDER)),
        track_style: Some(th.style(Style::POPUP_BORDER)),
        min_style: Some(th.style(Style::POPUP_BORDER)),
        begin_style: Some(th.style(Style::POPUP_ARROW)),
        end_style: Some(th.style(Style::POPUP_ARROW)),
        ..Default::default()
    }
}

fn dialog_scroll(th: &Theme) -> ScrollStyle {
    ScrollStyle {
        thumb_style: Some(th.style(Style::DIALOG_BORDER)),
        track_style: Some(th.style(Style::DIALOG_BORDER)),
        min_style: Some(th.style(Style::DIALOG_BORDER)),
        begin_style: Some(th.style(Style::POPUP_ARROW)),
        end_style: Some(th.style(Style::POPUP_ARROW)),
        ..Default::default()
    }
}

fn shadow(th: &Theme) -> ShadowStyle {
    ShadowStyle {
        style: th.style(Style::SHADOW),
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
        style: th.style(Style::CONTAINER_BORDER),
        arrow_style: Some(th.style(Style::CONTAINER_ARROW)),
        drag_style: Some(th.style(Style::FOCUS)),
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
        tab: Some(th.style(Style::INPUT)),
        select: Some(th.style(Style::SELECT)),
        focus: Some(th.style(Style::FOCUS)),
        ..Default::default()
    }
}

fn table(th: &Theme) -> TableStyle {
    TableStyle {
        style: th.style(Style::CONTAINER_BASE),
        select_row: Some(th.style(Style::SELECT)),
        show_row_focus: true,
        focus_style: Some(th.style(Style::FOCUS)),
        border_style: Some(th.style(Style::CONTAINER_BORDER)),
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
        border_style: Some(th.style(Style::CONTAINER_BORDER)),
        ..TextStyle::default()
    }
}

fn textview(th: &Theme) -> TextStyle {
    TextStyle {
        style: th.style(Style::CONTAINER_BASE),
        focus: Some(th.style(Style::CONTAINER_BASE)),
        select: Some(th.style(Style::TEXT_SELECT)),
        scroll: Some(scroll(th)),
        border_style: Some(th.style(Style::CONTAINER_BORDER)),
        ..TextStyle::default()
    }
}

fn view(th: &Theme) -> ViewStyle {
    ViewStyle {
        scroll: Some(scroll(th)),
        ..Default::default()
    }
}
