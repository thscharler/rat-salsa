use crate::color_span::{ColorSpan, ColorSpanState};
use crate::{Config, Global, PalEvent};
use anyhow::Error;
use indexmap::IndexMap;
use rat_salsa::SalsaContext;
use rat_theme4::{ColorIdx, Colors, Palette, RatWidgetColor, WidgetStyle};
use rat_widget::choice::{Choice, ChoiceState};
use rat_widget::clipper::{Clipper, ClipperState};
use rat_widget::color_input::{ColorInput, ColorInputState, Mode};
use rat_widget::event::{
    ChoiceOutcome, HandleEvent, MouseOnly, Outcome, Popup, Regular, TextOutcome, event_flow,
};
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_widget::layout::LayoutForm;
use rat_widget::number_input::{NumberInput, NumberInputState};
use rat_widget::popup::Placement;
use rat_widget::scrolled::Scroll;
use rat_widget::text::HasScreenCursor;
use rat_widget::text_input::{TextInput, TextInputState};
use rat_widget::textarea::{TextArea, TextAreaState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Flex, Rect};
use ratatui::style::Color;
use ratatui::widgets::{Block, BorderType};
use std::array;

#[derive(Debug)]
pub struct PaletteEdit {
    pub palette: Palette,

    pub form: ClipperState,
    pub name: TextInputState,
    pub docs: TextAreaState,
    pub dark: NumberInputState,

    pub color: [(ColorInputState, (), (), ColorInputState); Colors::LEN],
    pub color_ext: IndexMap<String, ChoiceState<ColorIdx>>,
}

impl PaletteEdit {
    pub fn new(cfg: &Config) -> Self {
        let mut z = Self {
            palette: Default::default(),
            form: ClipperState::named("form"),
            name: TextInputState::named("name"),
            docs: TextAreaState::named("docs"),
            dark: NumberInputState::named("dark"),
            color: array::from_fn(|i| {
                (
                    ColorInputState::named(format!("{}-0", Colors::array()[i].name()).as_str()),
                    (),
                    (),
                    ColorInputState::named(format!("{}-3", Colors::array()[i].name()).as_str()),
                )
            }),
            color_ext: {
                let mut map = IndexMap::new();
                for n in cfg.aliases() {
                    map.insert(n.clone(), ChoiceState::named(&n));
                }
                map
            },
        };
        z.dark.set_format_loc("999", cfg.loc).expect("format");
        z
    }

    pub fn name(&self) -> String {
        self.name.text().into()
    }

    pub fn file_name(&self) -> String {
        let name = self
            .name
            .text()
            .chars()
            .filter_map(|v| {
                if v.is_alphanumeric() {
                    Some(v)
                } else {
                    Some('_')
                }
            })
            .collect::<String>();
        name.to_lowercase()
    }

    pub fn const_name(&self) -> String {
        let name = self
            .name
            .text()
            .chars()
            .filter_map(|v| {
                if v.is_alphanumeric() {
                    Some(v)
                } else {
                    Some('_')
                }
            })
            .collect::<String>();
        name.to_uppercase()
    }
}

impl PaletteEdit {
    pub fn aliased(&self) -> Vec<(&'static str, ColorIdx)> {
        let mut aliased = Vec::new();
        for (n, s) in self.color_ext.iter() {
            let n = n.clone().into_boxed_str();
            let n = Box::leak(n);
            aliased.push((&*n, s.value()))
        }
        aliased.sort();
        aliased
    }

    pub fn palette(&self) -> Palette {
        let mut palette = Palette::default();
        let name = Box::from(self.name.text());
        let name = Box::leak(name);
        palette.name = name;

        let dark = self.dark.value().unwrap_or(64);

        palette.color[Colors::TextLight as usize] = Palette::interpolatec2(
            self.color[Colors::TextLight as usize].0.value(),
            self.color[Colors::TextLight as usize].3.value(),
            Color::default(),
            Color::default(),
        );
        palette.color[Colors::TextDark as usize] = Palette::interpolatec2(
            self.color[Colors::TextDark as usize].0.value(),
            self.color[Colors::TextDark as usize].3.value(),
            Color::default(),
            Color::default(),
        );
        for c in Colors::array_no_text() {
            palette.color[c as usize] = Palette::interpolatec(
                self.color[c as usize].0.value(),
                self.color[c as usize].3.value(),
                dark,
            );
        }
        palette.aliased = self.aliased().leak();

        palette
    }
}

impl HasFocus for PaletteEdit {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.name);
        builder.widget_navigate(&self.docs, Navigation::Regular);
        builder.widget(&self.dark);
        for c in Colors::array() {
            builder.widget(&self.color[c as usize].0);
            builder.widget(&self.color[c as usize].3);
        }
        for (_, s) in self.color_ext.iter() {
            builder.widget(s);
        }
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not defined")
    }

    fn area(&self) -> Rect {
        unimplemented!("not defined")
    }
}

impl HasScreenCursor for PaletteEdit {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.name
            .screen_cursor()
            .or(self.docs.screen_cursor())
            .or_else(|| {
                for c in Colors::array() {
                    if let Some(s) = self.color[c as usize].0.screen_cursor() {
                        return Some(s);
                    }
                    if let Some(s) = self.color[c as usize].3.screen_cursor() {
                        return Some(s);
                    }
                }
                None
            })
    }
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut PaletteEdit,
    ctx: &mut Global,
) -> Result<(), Error> {
    let mut form = Clipper::new() //
        .buffer_uses_view_size()
        .block(Block::bordered().border_type(BorderType::Rounded))
        .vscroll(Scroll::new())
        .styles(ctx.theme.style(WidgetStyle::CLIPPER));

    let layout_size = form.layout_size(area, &mut state.form);

    if !state.form.valid_layout(layout_size) {
        use rat_widget::layout::{FormLabel as L, FormWidget as W};
        let mut layout = LayoutForm::<usize>::new().spacing(1).flex(Flex::Start);
        layout.widget(state.name.id(), L::Str("Name"), W::Width(20));
        layout.widget(state.docs.id(), L::Str("Doc"), W::StretchX(20, 3));
        layout.widget(state.dark.id(), L::Str("Dark"), W::Width(4));
        layout.gap(1);
        for c in Colors::array() {
            layout.widget(
                state.color[c as usize].0.id(),
                L::String(c.to_string()),
                W::Width(51),
            );
        }
        layout.gap(1);
        for (n, s) in state.color_ext.iter() {
            layout.widget(s.id(), L::String(n.to_string()), W::Width(15));
        }
        form = form.layout(layout.build_endless(layout_size.width));
    }
    let mut form = form.into_buffer(area, &mut state.form);

    form.render(
        state.name.id(),
        || TextInput::new().styles(ctx.theme.style(WidgetStyle::TEXT)),
        &mut state.name,
    );
    form.render(
        state.docs.id(),
        || {
            TextArea::new()
                .vscroll(Scroll::new())
                .styles(ctx.theme.style(WidgetStyle::TEXTAREA))
        },
        &mut state.docs,
    );
    form.render(
        state.dark.id(),
        || NumberInput::new().styles(ctx.theme.style(WidgetStyle::TEXT)),
        &mut state.dark,
    );
    form.render(
        state.color[Colors::TextLight as usize].0.id(),
        || {
            ColorSpan::new()
                .half()
                .dark(state.dark.value().unwrap_or(63))
                .color0(ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)))
                .color3(ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)))
        },
        &mut ColorSpanState {
            color0: &mut state.color[Colors::TextLight as usize].0,
            color3: &mut state.color[Colors::TextLight as usize].3,
        },
    );
    form.render(
        state.color[Colors::TextDark as usize].0.id(),
        || {
            ColorSpan::new()
                .half()
                .dark(state.dark.value().unwrap_or(63))
                .color0(ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)))
                .color3(ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)))
        },
        &mut ColorSpanState {
            color0: &mut state.color[Colors::TextDark as usize].0,
            color3: &mut state.color[Colors::TextDark as usize].3,
        },
    );

    for c in Colors::array() {
        form.render(
            state.color[c as usize].0.id(),
            || {
                ColorSpan::new()
                    .dark(state.dark.value().unwrap_or(63))
                    .color0(ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)))
                    .color3(ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)))
            },
            &mut ColorSpanState {
                color0: &mut state.color[c as usize].0,
                color3: &mut state.color[c as usize].3,
            },
        );
    }

    let pal = state.palette();
    let pal_choice = crate::pal_choice(pal);

    let mut popup_ext = Vec::new();
    for (n, s) in state.color_ext.iter_mut() {
        let popup = form.render2(
            s.id(),
            || {
                Choice::new()
                    .items(pal_choice.iter().cloned())
                    .select_marker('*')
                    .popup_len(8)
                    .popup_scroll(Scroll::default())
                    .popup_placement(Placement::Right)
                    .styles(ctx.theme.style(WidgetStyle::CHOICE))
                    .into_widgets()
            },
            s,
        );
        popup_ext.push((n.clone(), popup));
    }
    for (n, popup) in popup_ext {
        let s = state.color_ext.get_mut(&n).expect("state");
        form.render_popup(s.id(), || popup, s);
    }

    form.finish(buf, &mut state.form);

    Ok(())
}

#[allow(unused_variables)]
pub fn event(
    event: &crossterm::event::Event,
    state: &mut PaletteEdit,
    ctx: &mut Global,
) -> Result<Outcome, Error> {
    let mut mode_change = None;

    let r = 'f: {
        for (n, s) in state.color_ext.iter_mut() {
            event_flow!(
                break 'f match s.handle(event, Popup) {
                    ChoiceOutcome::Value => {
                        if *n == Color::CONTAINER_BASE {
                            ctx.queue_event(PalEvent::ContainerBase(s.value()));
                        }
                        ChoiceOutcome::Value
                    }
                    r => r,
                }
            )
        }

        event_flow!(break 'f state.name.handle(event, Regular));
        event_flow!(break 'f state.docs.handle(event, Regular));
        event_flow!(
            break 'f match state.dark.handle(event, Regular) {
                TextOutcome::TextChanged => {
                    if state.dark.value().unwrap_or(0) > 255 {
                        state.dark.set_invalid(true);
                    } else {
                        state.dark.set_invalid(false);
                    }
                    TextOutcome::TextChanged
                }
                r => r,
            }
        );
        for c in Colors::array() {
            event_flow!(
                break 'f handle_color(event, &mut state.color[c as usize].0, &mut mode_change)
            );
            event_flow!(
                break 'f handle_color(event, &mut state.color[c as usize].3, &mut mode_change)
            );
        }

        event_flow!(break 'f state.form.handle(event, MouseOnly));

        Outcome::Continue
    };

    if let Some(mode_change) = mode_change {
        for c in Colors::array() {
            state.color[c as usize].0.set_mode(mode_change);
            state.color[c as usize].3.set_mode(mode_change);
        }
    }

    Ok(r)
}

fn handle_color(
    event: &crossterm::event::Event,
    color: &mut ColorInputState,
    mode_change: &mut Option<Mode>,
) -> TextOutcome {
    let mode = color.mode();
    let r = color.handle(event, Regular);
    if color.mode() != mode {
        *mode_change = Some(color.mode());
    }
    r
}
