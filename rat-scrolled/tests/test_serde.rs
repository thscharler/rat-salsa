use rat_scrolled::{ScrollStyle, SCROLLBAR_HORIZONTAL, SCROLLBAR_VERTICAL};
use ratatui::style::{Style, Stylize};

#[test]
fn test_ser() {
    let style = ScrollStyle {
        thumb_style: Some(Style::new().blue().on_black()),
        track_style: None,
        begin_style: None,
        end_style: None,
        min_style: None,
        horizontal: Some(SCROLLBAR_HORIZONTAL),
        vertical: Some(SCROLLBAR_VERTICAL),
        ..Default::default()
    };

    let str = serde_json::to_string(&style).unwrap();
    println!("{}", str);
    let x: ScrollStyle = serde_json::from_str(&str).unwrap();
    println!("{:?}", x);
}
