use crate::{Global, PalEvent};
use anyhow::Error;
use log::debug;
use rat_salsa::Control;
use rat_theme4::WidgetStyle;
use rat_widget::clipper::{Clipper, ClipperState};
use rat_widget::color_input::{ColorInput, ColorInputState};
use rat_widget::event::{HandleEvent, Outcome, Regular, event_flow};
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_widget::layout::LayoutForm;
use rat_widget::scrolled::Scroll;
use rat_widget::text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::{Flex, Rect};
use ratatui::style::Color;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct Foreign {
    pub form: ClipperState,
    pub name: String,
    pub color: Vec<ColorInputState>,
}

impl Foreign {
    pub fn color(&self, name: &str) -> Option<Color> {
        for c in &self.color {
            if c.focus().name().as_ref() == name {
                return Some(c.value());
            }
        }
        None
    }

    pub fn load_from_file(&mut self, path: &Path) -> Result<Control<PalEvent>, Error> {
        let mut buf = String::new();
        {
            let mut f = File::open(path)?;
            f.read_to_string(&mut buf)?;
        }

        self.name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        self.form.set_layout(Default::default());
        self.color.clear();
        if buf.contains("M.base_30") {
            parse_base46(self, buf);
        } else {
            parse_kv(self, buf);
        }

        Ok(Control::Changed)
    }
}

fn parse_kv(state: &mut Foreign, buf: String) {
    for l in buf.lines() {
        if l.contains([':', '=']) {
            let mut it = l.split([':', '=']);
            let Some(name) = it.next() else {
                continue;
            };
            let Some(color) = it.next() else {
                continue;
            };
            debug!("parse kv {:?} {:?}", name, color);
            let name = name.trim_matches([' ', '"', '\'', ',']);
            let color = color.trim_matches([' ', '"', '\'', ',']);
            debug!("    trimmed {:?} {:?}", name, color);
            let color = if color.starts_with("#") {
                let Ok(color) = Color::from_str(color) else {
                    continue;
                };
                color
            } else if color.len() == 6 || color.len() == 8 {
                let Ok(r) = u8::from_str_radix(&color[0..2], 16) else {
                    continue;
                };
                let Ok(g) = u8::from_str_radix(&color[2..4], 16) else {
                    continue;
                };
                let Ok(b) = u8::from_str_radix(&color[4..6], 16) else {
                    continue;
                };
                Color::Rgb(r, g, b)
            } else {
                continue;
            };

            let mut cs = ColorInputState::named(name);
            cs.set_value(color);
            state.color.push(cs);
        }
    }
}

fn parse_base46(state: &mut Foreign, buf: String) {
    // quick and dirty parser
    let mut mode = 0;
    for l in buf.lines() {
        if l.starts_with("M.base_30") {
            mode = 1;
        } else if l.starts_with("M.base_16") {
            mode = 1;
        } else if l.starts_with("}") {
            mode = 0;
        } else if mode == 1 {
            let l = l.trim();
            let mut it = l.split(['=', ',']);
            let Some(name) = it.next() else {
                continue;
            };
            let name = name.trim();
            let Some(color) = it.next() else {
                continue;
            };
            let color = color.trim_matches([' ', '"']);
            let Ok(color) = Color::from_str(color) else {
                continue;
            };

            let mut cs = ColorInputState::named(name);
            cs.set_value(color);
            state.color.push(cs);
        }
    }
}

impl HasFocus for Foreign {
    fn build(&self, builder: &mut FocusBuilder) {
        let tag = builder.start(&self.form);
        for c in &self.color {
            builder.widget(c);
        }
        builder.end(tag);
    }

    fn focus(&self) -> FocusFlag {
        self.form.focus()
    }

    fn area(&self) -> Rect {
        self.form.area()
    }
}

impl HasScreenCursor for Foreign {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        for c in &self.color {
            if let Some(sc) = c.screen_cursor() {
                return Some(sc);
            }
        }
        None
    }
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Foreign,
    ctx: &mut Global,
) -> Result<(), Error> {
    let mut form = Clipper::new()
        .buffer_uses_view_size()
        .vscroll(Scroll::new())
        .styles(ctx.theme.style(WidgetStyle::CLIPPER));
    let layout_size = form.layout_size(area, &mut state.form);

    if !state.form.valid_layout(layout_size) {
        use rat_widget::layout::{FormLabel as L, FormWidget as W};
        let mut layout = LayoutForm::<usize>::new().spacing(1).flex(Flex::Start);

        for c in &state.color {
            layout.widget(
                c.id(),
                L::String(c.focus().name().to_string()),
                W::Width(17),
            )
        }

        form = form.layout(layout.build_endless(layout_size.width));
    }

    let mut form = form.into_buffer(area, &mut state.form);
    for c in &mut state.color {
        form.render(
            c.id(),
            || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
            c,
        );
    }
    form.finish(buf, &mut state.form);

    Ok(())
}

pub fn event(
    event: &crossterm::event::Event,
    state: &mut Foreign,
    _ctx: &mut Global,
) -> Result<Outcome, Error> {
    let mut master_mode = None;

    let r = 'f: {
        for c in &mut state.color {
            let mode = c.mode();
            let r = c.handle(event, Regular);
            if c.mode() != mode {
                master_mode = Some(c.mode());
            }
            event_flow!(break 'f r);
        }
        event_flow!(break 'f state.form.handle(event, Regular));
        Outcome::Continue
    };

    if let Some(master_mode) = master_mode {
        for c in &mut state.color {
            c.set_mode(master_mode);
        }
    }

    Ok(r)
}
