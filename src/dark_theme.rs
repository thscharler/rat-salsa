use crate::Scheme;
use rat_widget::button::ButtonStyle;
use rat_widget::input::TextInputStyle;
use rat_widget::list::ListStyle;
use rat_widget::masked_input::MaskedInputStyle;
use rat_widget::menuline::MenuStyle;
use rat_widget::msgdialog::MsgDialogStyle;
use rat_widget::scrolled::ScrolledStyle;
use rat_widget::table::FTableStyle;
use ratatui::prelude::Style;
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

impl DarkTheme {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dark_theme(&self) -> bool {
        true
    }

    pub fn scheme(&self) -> &Scheme {
        &self.s
    }

    pub fn focus(&self) -> Style {
        let bg = self.s.primary[2];
        Style::default().fg(self.s.text_color(bg)).bg(bg)
    }

    pub fn select(&self) -> Style {
        let bg = self.s.secondary[1];
        Style::default().fg(self.s.text_color(bg)).bg(bg)
    }

    pub fn text_input(&self) -> Style {
        Style::default().fg(self.s.black[0]).bg(self.s.gray[3])
    }

    pub fn text_focus(&self) -> Style {
        let bg = self.s.primary[0];
        Style::default().fg(self.s.text_color(bg)).bg(bg)
    }

    pub fn text_select(&self) -> Style {
        let bg = self.s.secondary[0];
        Style::default().fg(self.s.text_color(bg)).bg(bg)
    }

    pub fn data(&self) -> Style {
        Style::default().fg(self.s.white[0]).bg(self.s.black[1])
    }

    pub fn dialog_style(&self) -> Style {
        Style::default().fg(self.s.white[2]).bg(self.s.gray[1])
    }

    pub fn status_style(&self) -> Style {
        Style::default().fg(self.s.white[0]).bg(self.s.black[2])
    }

    pub fn input_style(&self) -> TextInputStyle {
        TextInputStyle {
            style: self.text_input(),
            focus: Some(self.text_focus()),
            select: Some(self.text_select()),
            ..TextInputStyle::default()
        }
    }

    pub fn inputmask_style(&self) -> MaskedInputStyle {
        MaskedInputStyle {
            style: self.text_input(),
            focus: Some(self.text_focus()),
            select: Some(self.text_select()),
            invalid: Some(Style::default().bg(self.s.red[3])),
            ..Default::default()
        }
    }

    pub fn menu_style(&self) -> MenuStyle {
        let menu = Style::default().fg(self.s.white[3]).bg(self.s.black[2]);
        MenuStyle {
            style: menu,
            title: Some(Style::default().fg(self.s.black[0]).bg(self.s.yellow[2])),
            select: Some(menu),
            focus: Some(self.focus()),
            ..Default::default()
        }
    }

    pub fn table_style(&self) -> FTableStyle {
        FTableStyle {
            style: self.data(),
            select_row_style: Some(self.select()),
            show_row_focus: true,
            focus_style: Some(self.focus()),
            ..Default::default()
        }
    }

    pub fn list_style(&self) -> ListStyle {
        ListStyle {
            style: self.data(),
            select_style: self.select(),
            focus_style: self.focus(),
            ..Default::default()
        }
    }

    pub fn button_style(&self) -> ButtonStyle {
        ButtonStyle {
            style: Style::default().fg(self.s.white[0]).bg(self.s.primary[0]),
            focus: Some(self.focus()),
            armed: Some(Style::default().fg(self.s.black[0]).bg(self.s.secondary[0])),
            ..Default::default()
        }
    }

    pub fn scrolled_style(&self) -> ScrolledStyle {
        let style = Style::default().fg(self.s.gray[0]).bg(self.s.black[1]);
        ScrolledStyle {
            thumb_style: Some(style),
            track_style: Some(style),
            begin_style: Some(style),
            end_style: Some(style),
            ..Default::default()
        }
    }

    pub fn statusline_style(&self) -> Vec<Style> {
        vec![
            self.status_style(),
            Style::default().fg(self.s.white[0]).bg(self.s.blue[3]),
            Style::default().fg(self.s.white[0]).bg(self.s.blue[2]),
            Style::default().fg(self.s.white[0]).bg(self.s.blue[1]),
        ]
    }

    pub fn msg_dialog_style(&self) -> MsgDialogStyle {
        MsgDialogStyle {
            style: self.status_style(),
            button: self.button_style(),
            ..Default::default()
        }
    }
}
