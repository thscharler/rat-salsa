use crate::{Scheme, Theme};
use rat_widget::button::ButtonStyle;
use rat_widget::input::TextInputStyle;
use rat_widget::list::ListStyle;
use rat_widget::masked_input::MaskedInputStyle;
use rat_widget::menuline::MenuStyle;
use rat_widget::msgdialog::MsgDialogStyle;
use rat_widget::scrolled::ScrolledStyle;
use rat_widget::table::FTableStyle;
use ratatui::prelude::{Color, Style};
use ratatui::style::Stylize;

#[derive(Debug)]
pub struct DarkTheme {
    s: Scheme,
    name: String,
}

impl DarkTheme {
    pub fn new(name: String, s: Scheme) -> Self {
        Self { s, name }
    }
}

impl Theme for DarkTheme {
    fn name(&self) -> &str {
        &self.name
    }

    fn dark_theme(&self) -> bool {
        true
    }

    fn scheme(&self) -> &Scheme {
        &self.s
    }

    fn focus(&self) -> Color {
        self.s.primary[2]
    }

    fn focus_fg(&self) -> Color {
        self.s.text_color(self.s.primary[2])
    }

    fn select(&self) -> Color {
        self.s.secondary[1]
    }

    fn select_fg(&self) -> Color {
        self.s.text_color(self.s.secondary[0])
    }

    fn input_style(&self) -> TextInputStyle {
        TextInputStyle {
            style: Style::default().fg(self.s.black[0]).bg(self.s.gray[3]),
            focus: Some(Style::default().fg(self.focus_fg()).bg(self.focus())),
            select: Some(Style::default().fg(self.select_fg()).bg(self.select())),
            ..TextInputStyle::default()
        }
    }

    fn inputmask_style(&self) -> MaskedInputStyle {
        MaskedInputStyle {
            style: Style::default().fg(self.s.black[0]).bg(self.s.gray[3]),
            focus: Some(Style::default().fg(self.focus_fg()).bg(self.focus())),
            select: Some(Style::default().fg(self.select_fg()).bg(self.select())),
            invalid: Some(Style::default().bg(Color::Red)),
            ..Default::default()
        }
    }

    fn menu_style(&self) -> MenuStyle {
        MenuStyle {
            style: Style::default().fg(self.s.white[3]).bg(self.s.black[2]),
            title: Some(Style::default().fg(self.s.black[0]).bg(self.s.yellow[2])),
            select: Some(Style::default().fg(self.s.white[3]).bg(self.s.black[2])),
            focus: Some(Style::default().fg(self.focus_fg()).bg(self.focus())),
            ..Default::default()
        }
    }

    fn table_style(&self) -> FTableStyle {
        FTableStyle {
            style: Style::default().fg(self.s.white[0]).bg(self.s.black[1]),
            select_row_style: Some(Style::default().fg(self.select_fg()).bg(self.select())),
            show_row_focus: true,
            focus_style: Some(Style::default().fg(self.focus_fg()).bg(self.focus())),
            ..Default::default()
        }
    }

    fn list_style(&self) -> ListStyle {
        ListStyle {
            style: Style::default().fg(self.s.white[0]).bg(self.s.black[1]),
            select_style: Style::default().fg(self.select_fg()).bg(self.select()),
            focus_style: Style::default().fg(self.focus_fg()).bg(self.focus()),
            ..Default::default()
        }
    }

    fn button_style(&self) -> ButtonStyle {
        ButtonStyle {
            style: Style::default()
                .fg(self.s.white[0])
                .bg(self.s.primary[0])
                .bold(),
            focus: Some(Style::default().fg(self.focus_fg()).bg(self.focus()).bold()),
            armed: Some(
                Style::default()
                    .fg(self.s.black[0])
                    .bg(self.s.secondary[0])
                    .bold(),
            ),
            ..Default::default()
        }
    }

    fn scrolled_style(&self) -> ScrolledStyle {
        ScrolledStyle {
            thumb_style: Some(Style::default().fg(self.s.gray[0]).bg(self.s.black[1])),
            track_style: Some(Style::default().fg(self.s.gray[0]).bg(self.s.black[1])),
            begin_style: Some(Style::default().fg(self.s.gray[0]).bg(self.s.black[1])),
            end_style: Some(Style::default().fg(self.s.gray[0]).bg(self.s.black[1])),
            ..Default::default()
        }
    }

    fn dialog_style(&self) -> Style {
        Style::default().fg(self.s.white[2]).bg(self.s.gray[1])
    }

    fn status_style(&self) -> Style {
        Style::default().fg(self.s.white[0]).bg(self.s.black[2])
    }

    fn statusline_style(&self) -> Vec<Style> {
        vec![
            self.status_style(),
            Style::default().fg(self.s.white[0]).bg(self.s.blue[3]),
            Style::default().fg(self.s.white[0]).bg(self.s.blue[2]),
            Style::default().fg(self.s.white[0]).bg(self.s.blue[1]),
        ]
    }

    fn msg_dialog_style(&self) -> MsgDialogStyle {
        MsgDialogStyle {
            style: self.status_style(),
            button: self.button_style(),
            ..Default::default()
        }
    }
}
