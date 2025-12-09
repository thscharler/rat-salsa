use crate::mini_salsa::{MiniSalsaState, mock_init, run_ui, setup_logging};
use crate::text_samples::{
    add_range_styles, sample_emoji, sample_lorem_rustum, sample_scott_0, sample_tabs,
};
use rat_event::{Outcome, ct_event, try_flow};
use rat_scrolled::Scroll;
use rat_text::text_area::{TextArea, TextAreaState};
use rat_text::{HasScreenCursor, text_area};
use rat_theme4::StyleName;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};
use std::fmt;

mod mini_salsa;
mod text_samples;

fn main() -> Result<(), anyhow::Error> {
    setup_logging()?;

    let mut state = State {
        info: true,
        textarea: Default::default(),
    };
    state.textarea.set_auto_indent(false);

    run_ui("textarea1", mock_init, event, render, &mut state)
}

struct State {
    pub(crate) info: bool,
    pub(crate) textarea: TextAreaState,
}

fn render(
    buf: &mut Buffer,
    area: Rect,
    ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<(), anyhow::Error> {
    let l1 = Layout::vertical([
        Constraint::Length(7),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    let l2 = Layout::horizontal([
        Constraint::Length(15),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(25),
    ])
    .split(l1[1]);

    TextArea::new()
        .block(Block::bordered().style(ctx.theme.style_style(Style::DOCUMENT_BASE)))
        .scroll(
            Scroll::new()
                .scroll_by(1)
                .style(ctx.theme.style_style(Style::DOCUMENT_BASE)),
        )
        .set_horizontal_max_offset(256)
        .style(ctx.theme.style_style(Style::DOCUMENT_BASE))
        .select_style(ctx.theme.style_style(Style::INPUT_SELECT))
        .text_style([
            Style::new().red(),
            Style::new().underlined(),
            Style::new().green(),
            Style::new().on_yellow(),
        ])
        .render(l2[1], buf, &mut state.textarea);

    if let Some((cx, cy)) = state.textarea.screen_cursor() {
        ctx.cursor = Some((cx, cy));
    }

    if state.info {
        use fmt::Write;
        let mut stats = String::new();
        _ = writeln!(&mut stats);
        _ = writeln!(&mut stats, "cursor: {:?}", state.textarea.cursor(),);
        _ = writeln!(&mut stats, "anchor: {:?}", state.textarea.anchor());
        if let Some((scx, scy)) = state.textarea.screen_cursor() {
            _ = writeln!(&mut stats, "screen: {}:{}", scx, scy);
        } else {
            _ = writeln!(&mut stats, "screen: None",);
        }
        _ = writeln!(
            &mut stats,
            "width: {:?} ",
            state.textarea.line_width(state.textarea.cursor().y)
        );
        _ = writeln!(
            &mut stats,
            "next word: {:?} {:?}",
            state.textarea.next_word_start(state.textarea.cursor()),
            state.textarea.next_word_end(state.textarea.cursor())
        );
        _ = writeln!(
            &mut stats,
            "prev word: {:?} {:?}",
            state.textarea.prev_word_start(state.textarea.cursor()),
            state.textarea.prev_word_end(state.textarea.cursor())
        );

        _ = write!(&mut stats, "cursor-styles: ",);
        let mut styles = Vec::new();
        let cursor_byte = state.textarea.byte_at(state.textarea.cursor());
        state.textarea.styles_at(cursor_byte.start, &mut styles);
        for (_, s) in styles {
            _ = write!(&mut stats, "{}, ", s);
        }
        _ = writeln!(&mut stats);

        if let Some(st) = state.textarea.value.styles() {
            _ = writeln!(&mut stats, "text-styles: {}", st.count());
        }
        if let Some(st) = state.textarea.value.styles() {
            for r in st.take(20) {
                _ = writeln!(&mut stats, "    {:?}", r);
            }
        }
        Paragraph::new(stats).render(l2[3], buf);
    }

    let ccursor = state.textarea.selection();
    ctx.status[0] = format!(
        "{}:{} - {}:{}",
        ccursor.start.y, ccursor.start.x, ccursor.end.y, ccursor.end.x,
    );

    Ok(())
}

fn event(
    event: &crossterm::event::Event,
    _ctx: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {
    try_flow!(text_area::handle_events(&mut state.textarea, true, event));

    try_flow!(match event {
        ct_event!(key press ALT-'0') => {
            state.info = !state.info;
            Outcome::Changed
        }
        ct_event!(key press ALT-'1') => {
            state.textarea.clear();
            Outcome::Changed
        }
        ct_event!(key press ALT-'2') => {
            let (text, styles) = sample_scott_0();
            state.textarea.set_rope(text);
            add_range_styles(&mut state.textarea, styles);
            Outcome::Changed
        }
        ct_event!(key press ALT-'3') => {
            let (text, styles) = sample_emoji();
            state.textarea.set_rope(text);
            add_range_styles(&mut state.textarea, styles);
            Outcome::Changed
        }
        ct_event!(key press ALT-'4') => {
            let (text, styles) = sample_tabs();
            state.textarea.set_rope(text);
            add_range_styles(&mut state.textarea, styles);
            Outcome::Changed
        }
        ct_event!(key press ALT-'5') => {
            let (text, styles) = sample_lorem_rustum();
            state.textarea.set_rope(text);
            state.textarea.set_styles(styles);
            Outcome::Changed
        }
        _ => Outcome::Continue,
    });

    Ok(Outcome::Continue)
}
