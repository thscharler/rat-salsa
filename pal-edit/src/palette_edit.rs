use crate::proc::{color_array, color_array_no_text};
use crate::widget::color_span::{ColorSpan, ColorSpanState};
use crate::{Config, Global, PalEvent};
use anyhow::Error;
use indexmap::IndexMap;
use rat_salsa::SalsaContext;
use rat_theme4::palette::{ColorIdx, Colors, Palette};
use rat_theme4::{RatWidgetColor, WidgetStyle};
use rat_widget::choice::{Choice, ChoiceState};
use rat_widget::clipper::{Clipper, ClipperState};
use rat_widget::color_input::{ColorInput, ColorInputState, Mode};
use rat_widget::combobox::{Combobox, ComboboxState};
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
use std::borrow::Cow;

#[derive(Debug)]
pub struct PaletteEdit {
    pub form: ClipperState,
    pub theme_name: TextInputState,
    pub theme: ComboboxState,
    pub name: TextInputState,
    pub doc: TextAreaState,
    pub dark: NumberInputState,

    pub color: [(ColorInputState, (), (), ColorInputState); Colors::LEN],
    pub aliased: IndexMap<String, ChoiceState<ColorIdx>>,
    pub unknown: IndexMap<String, ColorIdx>,
}

impl PaletteEdit {
    pub fn new(cfg: &Config) -> Self {
        let mut z = Self {
            form: ClipperState::named("form"),
            theme_name: TextInputState::named("theme_name"),
            theme: ComboboxState::named("theme"),
            name: TextInputState::named("name"),
            doc: TextAreaState::named("docs"),
            dark: NumberInputState::named("dark"),
            color: array::from_fn(|i| {
                (
                    ColorInputState::named(format!("{}-0", color_array()[i]).as_str()),
                    (),
                    (),
                    ColorInputState::named(format!("{}-3", color_array()[i]).as_str()),
                )
            }),
            aliased: {
                let mut map = IndexMap::new();
                for n in cfg.aliased_vec() {
                    map.insert(n.clone(), ChoiceState::named(&n));
                }
                map
            },
            unknown: Default::default(),
        };
        z.dark.set_format_loc("999", cfg.loc).expect("format");
        z
    }

    pub fn width(_cfg: &Config) -> u16 {
        64
    }

    pub fn theme_name(&self) -> String {
        self.theme_name.text().into()
    }

    pub fn theme(&self) -> String {
        self.theme.value()
    }

    pub fn name(&self) -> String {
        self.name.text().into()
    }

    pub fn doc(&self) -> String {
        self.doc.text()
    }

    pub fn generator(&self) -> String {
        format!("light-dark:{}", self.dark.value().unwrap_or(63))
    }

    pub fn file_name(&self) -> String {
        let name = self
            .theme_name
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
            .theme_name
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
    pub fn aliased_for(&self, names: Vec<String>) -> Vec<(Cow<'static, str>, ColorIdx)> {
        let mut aliased = Vec::new();
        for (n, s) in self.aliased.iter() {
            if names.contains(n) {
                aliased.push((Cow::Owned(n.to_string()), s.value()))
            }
        }
        aliased.sort();
        aliased
    }

    pub fn aliased(&self) -> Vec<(Cow<'static, str>, ColorIdx)> {
        let mut aliased = Vec::new();
        for (n, s) in self.aliased.iter() {
            aliased.push((Cow::Owned(n.clone()), s.value()))
        }
        for (n, c) in self.unknown.iter() {
            aliased.push((Cow::Owned(n.clone()), *c));
        }
        aliased.sort();
        aliased
    }

    pub fn set_palette(&mut self, p: Palette) {
        self.theme_name.set_value(p.theme_name.as_ref());
        self.theme.set_value(p.theme.as_ref());
        self.name.set_value(p.name.as_ref());
        self.doc.set_value(p.doc.as_ref());
        if p.generator.starts_with("light-dark") {
            if p.generator.starts_with("light-dark") {
                if let Some(s) = p.generator.split(':').nth(1) {
                    let dark = s.trim().parse::<u8>().unwrap_or(63);
                    _ = self.dark.set_value(dark);
                }
            }
        }
        for c in color_array() {
            self.color[c as usize].0.set_value(p.color[c as usize][0]);
            self.color[c as usize].3.set_value(p.color[c as usize][3]);
        }
        for (n, s) in self.aliased.iter_mut() {
            if let Some(c_idx) = p.try_aliased(n) {
                s.set_value(c_idx);
            }
        }
        for (n, c) in p.aliased.as_ref() {
            if !self.aliased.contains_key(n.as_ref()) {
                self.unknown.insert(n.to_string(), *c);
            }
        }
    }

    pub fn palette(&self) -> Palette {
        let mut palette = Palette::default();
        palette.theme_name = Cow::Owned(self.theme_name.value());
        palette.theme = Cow::Owned(self.theme.value());
        palette.name = Cow::Owned(self.name.value());
        palette.doc = Cow::Owned(self.doc.value());
        let dark = self.dark.value().unwrap_or(64);
        palette.generator = Cow::Owned(format!("light-dark:{}", dark));

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
        for c in color_array_no_text() {
            palette.color[c as usize] = Palette::interpolatec(
                self.color[c as usize].0.value(),
                self.color[c as usize].3.value(),
                dark,
            );
        }
        palette.aliased = Cow::Owned(self.aliased());

        palette
    }
}

impl HasFocus for PaletteEdit {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.theme_name);
        builder.widget(&self.theme);
        builder.widget(&self.name);
        builder.widget_navigate(&self.doc, Navigation::Regular);
        builder.widget(&self.dark);
        for c in color_array() {
            builder.widget(&self.color[c as usize].0);
            builder.widget(&self.color[c as usize].3);
        }
        for (_, s) in self.aliased.iter() {
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
        None.or(self.theme_name.screen_cursor())
            .or(self.name.screen_cursor())
            .or(self.doc.screen_cursor())
            .or(self.dark.screen_cursor())
            .or_else(|| {
                for c in color_array() {
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
        let mut layout = LayoutForm::<usize>::new() //
            .spacing(1)
            .flex(Flex::Start);
        layout.widget(state.theme_name.id(), L::Str("Theme Name"), W::Width(20));
        layout.widget(state.theme.id(), L::Str("Theme"), W::Width(20));
        layout.widget(state.name.id(), L::Str("Name"), W::Width(20));
        layout.widget(state.doc.id(), L::Str("Doc"), W::StretchX(20, 3));
        layout.widget(state.dark.id(), L::Str("Dark"), W::Width(4));
        layout.gap(1);
        for c in color_array() {
            layout.widget(
                state.color[c as usize].0.id(),
                L::String(c.to_string()),
                W::Width(51),
            );
        }
        layout.gap(1);
        layout.widget(0, L::None, W::Wide(1, 1)); // placeholder
        let mut layout = layout.build_endless(layout_size.width);

        // build alias list with 2 columns and append.
        let first_extra = ctx.cfg.extra_alias.get(0).map(|v| v.as_str());
        let mut layout2 = LayoutForm::<usize>::new()
            .spacing(1)
            .columns(2)
            .flex(Flex::Legacy);
        for (n, s) in state.aliased.iter() {
            if Some(n.as_str()) == first_extra {
                layout2.column_break();
            }
            layout2.widget(s.id(), L::String(n.to_string()), W::Width(15));
        }
        let layout2 = layout2.build_endless(layout_size.width);
        layout.append(layout.widget_for(0).as_position(), layout2);

        form = form.layout(layout);
    }
    let mut form = form.into_buffer(area, &mut state.form);

    form.render(
        state.theme_name.id(),
        || TextInput::new().styles(ctx.theme.style(WidgetStyle::TEXT)),
        &mut state.theme_name,
    );
    let theme_popup = form.render2(
        state.theme.id(),
        || {
            Combobox::new()
                .item("Dark", "Dark")
                .item("Light", "Light")
                .item("Shell", "Shell")
                .styles(ctx.theme.style(WidgetStyle::CHOICE))
                .into_widgets()
        },
        &mut state.theme,
    );
    form.render(
        state.name.id(),
        || TextInput::new().styles(ctx.theme.style(WidgetStyle::TEXT)),
        &mut state.name,
    );
    form.render(
        state.doc.id(),
        || {
            TextArea::new()
                .vscroll(Scroll::new())
                .styles(ctx.theme.style(WidgetStyle::TEXTAREA))
        },
        &mut state.doc,
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

    for c in color_array() {
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
    let pal_choice = crate::proc::pal_choice(pal);

    let mut popup_ext = Vec::new();
    let mut popup_place = Placement::Right;
    let first_extra = ctx.cfg.extra_alias.get(0).map(|v| v.as_str());
    for (n, s) in state.aliased.iter_mut() {
        if Some(n.as_str()) == first_extra {
            popup_place = Placement::Left;
        }
        let popup = form.render2(
            s.id(),
            || {
                Choice::new()
                    .items(pal_choice.iter().cloned())
                    .select_marker('*')
                    .popup_len(8)
                    .popup_scroll(Scroll::default())
                    .popup_placement(popup_place)
                    .styles(ctx.theme.style(WidgetStyle::CHOICE))
                    .into_widgets()
            },
            s,
        );
        popup_ext.push((n.clone(), popup));
    }

    form.render_popup(state.theme.id(), || theme_popup, &mut state.theme);
    for (n, popup) in popup_ext {
        let s = state.aliased.get_mut(&n).expect("state");
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
        for (n, s) in state.aliased.iter_mut() {
            event_flow!(
                break 'f match s.handle(event, Popup) {
                    ChoiceOutcome::Value => {
                        if *n == Color::CONTAINER_BASE_BG {
                            ctx.queue_event(PalEvent::ContainerBase(s.value()));
                        }
                        ChoiceOutcome::Value
                    }
                    r => r,
                }
            )
        }
        event_flow!(break 'f state.theme.handle(event, Popup));

        event_flow!(break 'f state.theme_name.handle(event, Regular));
        event_flow!(break 'f state.name.handle(event, Regular));
        event_flow!(break 'f state.doc.handle(event, Regular));
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
        for c in color_array() {
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
        for c in color_array() {
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
