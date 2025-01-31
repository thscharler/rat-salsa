use rat_scrolled::block_style::{BlockStyle, StyleBorderType};
use rat_widget::button::ButtonStyle;
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::Borders;

#[test]
fn test_serde() {
    let style = ButtonStyle {
        style: Style::new().fg(Color::Rgb(192, 172, 152)),
        block_style: Some(BlockStyle {
            borders: Some(Borders::all()),
            border_style: Some(Style::new().yellow()),
            border_type: Some(StyleBorderType::Plain),
            ..Default::default()
        }),
        ..Default::default()
    };

    let s = serde_json::to_string_pretty(&style).unwrap();
    println!("{}", s);
    let v: ButtonStyle = serde_json::from_str(&s).unwrap();
    println!("{:#?}", v);
}
