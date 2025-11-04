use crate::map_theme::MapTheme;
use crate::palette::Palette;
use crate::{SalsaTheme, StyleName, WidgetStyle, make_dyn};
use rat_widget::button::ButtonStyle;
use rat_widget::calendar::CalendarStyle;
use rat_widget::checkbox::CheckboxStyle;
use rat_widget::choice::ChoiceStyle;
use rat_widget::clipper::ClipperStyle;
use rat_widget::container::ContainerStyle;
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
use rat_widget::text::TextStyle;
use rat_widget::view::ViewStyle;
use ratatui::layout::Alignment;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders};
use std::time::Duration;

pub fn shell_theme(name: &str, p: Palette) -> Box<dyn SalsaTheme> {
    let mut th = MapTheme::new(name, p);

    th.define(Style::INPUT, th.p.high_contrast(p.gray[0]));
    th.define(Style::FOCUS, th.p.high_contrast(p.primary[2]));
    th.define(Style::SELECT, th.p.high_contrast(p.secondary[1]));
    th.define(Style::TEXT_FOCUS, th.p.high_contrast(p.gray[3]));
    th.define(Style::TEXT_SELECT, th.p.high_contrast(p.secondary[0]));

    th.define(Style::CONTAINER_BASE, Style::default());
    th.define(Style::CONTAINER_BORDER, Style::default());
    th.define(Style::CONTAINER_ARROWS, Style::default());

    th.define(Style::POPUP_BASE, th.p.normal_contrast(p.gray[0]));
    th.define(Style::POPUP_BORDER, th.p.normal_contrast(p.gray[0]));
    th.define(Style::POPUP_ARROW, th.p.normal_contrast(p.gray[0]));

    th.define(Style::DIALOG_BASE, th.p.high_contrast(p.gray[1]));
    th.define(Style::DIALOG_BORDER, th.p.normal_contrast(p.gray[1]));
    th.define(Style::DIALOG_ARROW, th.p.normal_contrast(p.gray[1]));

    th.define(Style::STATUS_BASE, Style::default());

    th.define_fn(WidgetStyle::BUTTON, make_dyn!(button));
    th.define_fn(WidgetStyle::CHECKBOX, make_dyn!(checkbox));
    th.define_fn(WidgetStyle::CHOICE, make_dyn!(choice));
    th.define_fn(WidgetStyle::CLIPPER, make_dyn!(clipper));
    th.define_fn(WidgetStyle::CONTAINER, make_dyn!(container));
    th.define_fn(WidgetStyle::FILE_DIALOG, make_dyn!(file_dialog));
    th.define_fn(WidgetStyle::FORM, make_dyn!(form));
    th.define_fn(WidgetStyle::LINE_NR, make_dyn!(line_nr));
    th.define_fn(WidgetStyle::LIST, make_dyn!(list));
    th.define_fn(WidgetStyle::MENU, make_dyn!(menu));
    th.define_fn(WidgetStyle::MONTH, make_dyn!(month));
    th.define_fn(WidgetStyle::MSG_DIALOG, make_dyn!(msg_dialog));
    th.define_fn(WidgetStyle::PARAGRAPH, make_dyn!(paragraph));
    th.define_fn(WidgetStyle::RADIO, make_dyn!(radio));
    th.define_fn(WidgetStyle::SCROLL, make_dyn!(scroll));
    th.define_fn(WidgetStyle::SCROLL_DIALOG, make_dyn!(dialog_scroll));
    th.define_fn(WidgetStyle::SCROLL_POPUP, make_dyn!(popup_scroll));
    th.define_fn(WidgetStyle::SHADOW, make_dyn!(shadow));
    th.define_fn(WidgetStyle::SLIDER, make_dyn!(slider));
    th.define_fn(WidgetStyle::SPLIT, make_dyn!(split));
    th.define_fn(WidgetStyle::STATUSLINE, make_dyn!(statusline));
    th.define_fn(WidgetStyle::TABBED, make_dyn!(tabbed));
    th.define_fn(WidgetStyle::TABLE, make_dyn!(table));
    th.define_fn(WidgetStyle::TEXT, make_dyn!(text));
    th.define_fn(WidgetStyle::TEXTAREA, make_dyn!(textarea));
    th.define_fn(WidgetStyle::TEXTVIEW, make_dyn!(textview));
    th.define_fn(WidgetStyle::VIEW, make_dyn!(view));

    Box::new(th)
}

fn button(th: &MapTheme) -> ButtonStyle {
    ButtonStyle {
        style: th.p.gray(2),
        focus: Some(th.style(Style::FOCUS)),
        armed: Some(th.style(Style::SELECT)),
        hover: Some(th.style(Style::SELECT)),
        armed_delay: Some(Duration::from_millis(50)),
        ..Default::default()
    }
}

fn checkbox(th: &MapTheme) -> CheckboxStyle {
    CheckboxStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        ..Default::default()
    }
}

fn choice(th: &MapTheme) -> ChoiceStyle {
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

fn clipper(th: &MapTheme) -> ClipperStyle {
    ClipperStyle {
        style: th.style(Style::CONTAINER_BASE),
        scroll: Some(scroll(th)),
        ..Default::default()
    }
}

fn container(th: &MapTheme) -> ContainerStyle {
    ContainerStyle {
        style: th.style(Style::CONTAINER_BASE),
        symbol: None,
        block: None,
        ..Default::default()
    }
}

fn file_dialog(th: &MapTheme) -> FileDialogStyle {
    FileDialogStyle {
        style: th.style(Style::DIALOG_BASE),
        list: Some(list(th)),
        roots: Some(ListStyle {
            style: th.style(Style::DIALOG_BASE),
            ..list(th)
        }),
        text: Some(th.style(Style::INPUT)),
        button: Some(button(th)),
        block: Some(Block::bordered()),
        ..Default::default()
    }
}

fn form(th: &MapTheme) -> FormStyle {
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

fn line_nr(th: &MapTheme) -> LineNumberStyle {
    LineNumberStyle {
        style: th.style(Style::CONTAINER_BASE),
        cursor: Some(th.style(Style::TEXT_SELECT)),
        ..LineNumberStyle::default()
    }
}

fn list(th: &MapTheme) -> ListStyle {
    ListStyle {
        style: th.style(Style::CONTAINER_BASE),
        select: Some(th.style(Style::SELECT)),
        focus: Some(th.style(Style::FOCUS)),
        scroll: Some(scroll(th)),
        ..Default::default()
    }
}

fn menu(th: &MapTheme) -> MenuStyle {
    MenuStyle {
        style: th.style(Style::STATUS_BASE),
        title: Some(th.p.yellow(2)),
        focus: Some(th.style(Style::FOCUS)),
        right: Some(th.p.green(3)),
        disabled: Some(th.p.fg_gray(0)),
        highlight: Some(Style::default().underlined()),
        popup_style: Some(th.style(Style::STATUS_BASE)),
        block: Some(Block::bordered()),
        popup: Default::default(),
        ..Default::default()
    }
}

fn month(th: &MapTheme) -> CalendarStyle {
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

fn msg_dialog(th: &MapTheme) -> MsgDialogStyle {
    MsgDialogStyle {
        style: th.style(Style::DIALOG_BASE),
        button: Some(button(th)),
        ..Default::default()
    }
}

fn paragraph(th: &MapTheme) -> ParagraphStyle {
    ParagraphStyle {
        style: th.style(Style::CONTAINER_BASE),
        focus: Some(th.style(Style::FOCUS)),
        scroll: Some(scroll(th)),
        ..Default::default()
    }
}

fn radio(th: &MapTheme) -> RadioStyle {
    RadioStyle {
        layout: Some(RadioLayout::Stacked),
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        ..Default::default()
    }
}

/// Scroll style
fn scroll(th: &MapTheme) -> ScrollStyle {
    ScrollStyle {
        thumb_style: Some(th.style(Style::CONTAINER_BORDER)),
        track_style: Some(th.style(Style::CONTAINER_BORDER)),
        min_style: Some(th.style(Style::CONTAINER_BORDER)),
        begin_style: Some(th.style(Style::CONTAINER_ARROWS)),
        end_style: Some(th.style(Style::CONTAINER_ARROWS)),
        ..Default::default()
    }
}

fn dialog_scroll(th: &MapTheme) -> ScrollStyle {
    ScrollStyle {
        thumb_style: Some(th.style(Style::DIALOG_BORDER)),
        track_style: Some(th.style(Style::DIALOG_BORDER)),
        min_style: Some(th.style(Style::DIALOG_BORDER)),
        begin_style: Some(th.style(Style::DIALOG_ARROW)),
        end_style: Some(th.style(Style::DIALOG_ARROW)),
        ..Default::default()
    }
}

fn popup_scroll(th: &MapTheme) -> ScrollStyle {
    ScrollStyle {
        thumb_style: Some(th.style(Style::POPUP_BORDER)),
        track_style: Some(th.style(Style::POPUP_BORDER)),
        min_style: Some(th.style(Style::POPUP_BORDER)),
        begin_style: Some(th.style(Style::POPUP_ARROW)),
        end_style: Some(th.style(Style::POPUP_ARROW)),
        ..Default::default()
    }
}

fn shadow(th: &MapTheme) -> ShadowStyle {
    ShadowStyle {
        style: th.p.normal_contrast(th.p.black[0]),
        dir: ShadowDirection::BottomRight,
        ..ShadowStyle::default()
    }
}

fn slider(th: &MapTheme) -> SliderStyle {
    SliderStyle {
        style: th.style(Style::INPUT),
        bounds: Some(th.p.gray(2)),
        knob: Some(th.style(Style::TEXT_SELECT)),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        text_align: Some(Alignment::Center),
        ..Default::default()
    }
}

fn split(th: &MapTheme) -> SplitStyle {
    SplitStyle {
        style: th.style(Style::CONTAINER_BORDER),
        arrow_style: Some(th.style(Style::CONTAINER_ARROWS)),
        drag_style: Some(th.style(Style::FOCUS)),
        ..Default::default()
    }
}

fn statusline(th: &MapTheme) -> StatusLineStyle {
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

fn tabbed(th: &MapTheme) -> TabbedStyle {
    TabbedStyle {
        style: th.style(Style::CONTAINER_BASE),
        tab: Some(th.p.gray(2)),
        select: Some(th.p.secondary(0)),
        focus: Some(th.style(Style::FOCUS)),
        ..Default::default()
    }
}

fn table(th: &MapTheme) -> TableStyle {
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

fn text(th: &MapTheme) -> TextStyle {
    TextStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        select: Some(th.style(Style::TEXT_SELECT)),
        invalid: Some(th.p.fg_red(3)),
        ..TextStyle::default()
    }
}

fn textarea(th: &MapTheme) -> TextStyle {
    TextStyle {
        style: th.style(Style::INPUT),
        focus: Some(th.style(Style::TEXT_FOCUS)),
        select: Some(th.style(Style::TEXT_SELECT)),
        scroll: Some(scroll(th)),
        border_style: Some(th.style(Style::CONTAINER_BORDER)),
        ..TextStyle::default()
    }
}

fn textview(th: &MapTheme) -> TextStyle {
    TextStyle {
        style: th.style(Style::CONTAINER_BASE),
        focus: Some(th.style(Style::CONTAINER_BASE)),
        select: Some(th.style(Style::TEXT_SELECT)),
        scroll: Some(scroll(th)),
        border_style: Some(th.style(Style::CONTAINER_BORDER)),
        ..TextStyle::default()
    }
}

fn view(th: &MapTheme) -> ViewStyle {
    ViewStyle {
        scroll: Some(scroll(th)),
        ..Default::default()
    }
}
