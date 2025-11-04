use crate::map_theme::MapTheme;
use crate::palette::Palette;
use crate::{NamedStyle, SalsaTheme, WidgetStyle, make_dyn};
use rat_widget::container::ContainerStyle;
use rat_widget::scrolled::ScrollStyle;
use ratatui::style::Style;

pub fn dark_theme(name: &str, p: Palette) -> Box<dyn SalsaTheme> {
    let mut th = MapTheme::new(name, p);

    th.add_named(NamedStyle::FOCUS, th.p.high_contrast(p.primary[2]));
    th.add_named(NamedStyle::SELECT, th.p.high_contrast(p.secondary[1]));

    th.add_style(WidgetStyle::CONTAINER, make_dyn!(container));
    th.add_style(WidgetStyle::SCROLL, make_dyn!(scroll));

    Box::new(th)
}

/// Focus style
fn focus(th: &MapTheme) -> Style {
    th.p.high_contrast(th.p.primary[2])
}

/// Selection style
fn select(th: &MapTheme) -> Style {
    th.p.high_contrast(th.p.secondary[1])
}

/// Container base
fn container_base(th: &MapTheme) -> Style {
    th.p.normal_contrast(th.p.black[1])
}

/// Container border
fn container_border(th: &MapTheme) -> Style {
    container_base(th).fg(th.p.gray[0])
}

/// Container arrows
fn container_arrow(th: &MapTheme) -> Style {
    container_base(th).fg(th.p.gray[0])
}

fn container(th: &MapTheme) -> ContainerStyle {
    ContainerStyle {
        style: container_base(th),
        symbol: None,
        block: None,
        ..Default::default()
    }
}

/// Scroll style
fn scroll(th: &MapTheme) -> ScrollStyle {
    ScrollStyle {
        thumb_style: Some(container_border(th)),
        track_style: Some(container_border(th)),
        min_style: Some(container_border(th)),
        begin_style: Some(container_arrow(th)),
        end_style: Some(container_arrow(th)),
        ..Default::default()
    }
}
