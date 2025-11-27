use crate::Global;
use anyhow::Error;
use rat_theme4::WidgetStyle;
use rat_widget::choice::{Choice, ChoiceState};
use rat_widget::event::{HandleEvent, Outcome, Popup, Regular, event_flow};
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_widget::form::{Form, FormState};
use rat_widget::layout::LayoutForm;
use rat_widget::paragraph::{Paragraph, ParagraphState};
use rat_widget::text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::{Flex, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Padding, Wrap};

#[derive(Debug)]
pub struct SampleCustom {
    pub form: FormState<usize>,
    pub bg_1: ChoiceState<Option<String>>,
    pub fg_1: ChoiceState<Option<String>>,
    pub para1: ParagraphState,
    pub bg_2: ChoiceState<Option<String>>,
    pub fg_2: ChoiceState<Option<String>>,
    pub para2: ParagraphState,
    pub bg_3: ChoiceState<Option<String>>,
    pub fg_3: ChoiceState<Option<String>>,
    pub para3: ParagraphState,
}

impl SampleCustom {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for SampleCustom {
    fn default() -> Self {
        let mut z = Self {
            form: Default::default(),
            bg_1: Default::default(),
            fg_1: Default::default(),
            para1: Default::default(),
            bg_2: Default::default(),
            fg_2: Default::default(),
            para2: Default::default(),
            bg_3: Default::default(),
            fg_3: Default::default(),
            para3: Default::default(),
        };
        z.fg_1.set_default_value(Some(None));
        z.bg_1.set_default_value(Some(None));
        z.fg_2.set_default_value(Some(None));
        z.bg_2.set_default_value(Some(None));
        z.fg_3.set_default_value(Some(None));
        z.bg_3.set_default_value(Some(None));
        z
    }
}

impl HasFocus for SampleCustom {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.fg_1);
        builder.widget(&self.bg_1);
        builder.widget(&self.fg_2);
        builder.widget(&self.bg_2);
        builder.widget(&self.fg_3);
        builder.widget(&self.bg_3);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not available")
    }

    fn area(&self) -> Rect {
        unimplemented!("not available")
    }
}

impl HasScreenCursor for SampleCustom {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        None
    }
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut SampleCustom,
    ctx: &mut Global,
) -> Result<(), Error> {
    let form = Form::new() //
        .styles(ctx.show_theme.style(WidgetStyle::FORM))
        .show_navigation(false);
    let size = form.layout_size(area);
    {
        use rat_widget::layout::{FormLabel as L, FormWidget as W};
        let mut l = LayoutForm::new()
            .spacing(1)
            .flex(Flex::Legacy)
            .padding(Padding::new(1, 1, 1, 1));

        l.widget(state.fg_1.id(), L::Str("FG"), W::Width(25));
        l.widget(state.bg_1.id(), L::Str("BG"), W::Width(25));
        l.gap(1);
        l.widget(state.para1.id(), L::None, W::WideStretchX(25, 3));
        l.gap(1);
        l.widget(state.fg_2.id(), L::Str("FG"), W::Width(25));
        l.widget(state.bg_2.id(), L::Str("BG"), W::Width(25));
        l.gap(1);
        l.widget(state.para2.id(), L::None, W::WideStretchX(25, 3));
        l.gap(1);
        l.widget(state.fg_3.id(), L::Str("FG"), W::Width(25));
        l.widget(state.bg_3.id(), L::Str("BG"), W::Width(25));
        l.gap(1);
        l.widget(state.para3.id(), L::None, W::WideStretchX(25, 3));

        state.form.set_layout(l.build_endless(size.width));
    }
    let mut form = form.into_buffer(area, buf, &mut state.form);
    let pal_choice = crate::pal_aliases(ctx.show_theme.p.clone());

    let fg_1_popup = form.render2(
        state.fg_1.id(),
        || {
            Choice::new()
                .items(pal_choice.clone())
                .select_marker('*')
                .styles(ctx.show_theme.style(WidgetStyle::CHOICE))
                .into_widgets()
        },
        &mut state.fg_1,
    );
    let bg_1_popup = form.render2(
        state.bg_1.id(),
        || {
            Choice::new()
                .items(pal_choice.clone())
                .select_marker('*')
                .styles(ctx.show_theme.style(WidgetStyle::CHOICE))
                .into_widgets()
        },
        &mut state.bg_1,
    );
    let fg_2_popup = form.render2(
        state.fg_2.id(),
        || {
            Choice::new()
                .items(pal_choice.clone())
                .select_marker('*')
                .styles(ctx.show_theme.style(WidgetStyle::CHOICE))
                .into_widgets()
        },
        &mut state.fg_2,
    );
    let bg_2_popup = form.render2(
        state.bg_2.id(),
        || {
            Choice::new()
                .items(pal_choice.clone())
                .select_marker('*')
                .styles(ctx.show_theme.style(WidgetStyle::CHOICE))
                .into_widgets()
        },
        &mut state.bg_2,
    );
    let fg_3_popup = form.render2(
        state.fg_3.id(),
        || {
            Choice::new()
                .items(pal_choice.clone())
                .select_marker('*')
                .styles(ctx.show_theme.style(WidgetStyle::CHOICE))
                .into_widgets()
        },
        &mut state.fg_3,
    );
    let bg_3_popup = form.render2(
        state.bg_3.id(),
        || {
            Choice::new()
                .items(pal_choice.clone())
                .select_marker('*')
                .styles(ctx.show_theme.style(WidgetStyle::CHOICE))
                .into_widgets()
        },
        &mut state.bg_3,
    );

    form.render(state.para1.id(),
        || Paragraph::new("Idol (アイドル Aidoru) ist ein Lied des japanischen Duos Yoasobi, das im April 2023 als Musikdownload und Stream veröffentlicht wurde. Im Mai erschien eine englischsprachige Version des Liedes mitsamt Musikvideo.")
            .wrap(Wrap{ trim: true })
            .styles(ctx.show_theme.style(WidgetStyle::PARAGRAPH))
            .style(style_opt(state.fg_1.value(), state.bg_1.value(), ctx)),
        &mut state.para1
    );
    form.render(state.para2.id(),
        || Paragraph::new("Idol (アイドル Aidoru) ist ein Lied des japanischen Duos Yoasobi, das im April 2023 als Musikdownload und Stream veröffentlicht wurde. Im Mai erschien eine englischsprachige Version des Liedes mitsamt Musikvideo.")
            .wrap(Wrap{ trim: true })
            .styles(ctx.show_theme.style(WidgetStyle::PARAGRAPH))
            .style(style_opt(state.fg_2.value(), state.bg_2.value(), ctx)),
        &mut state.para2
    );
    form.render(state.para3.id(),
        || Paragraph::new("Idol (アイドル Aidoru) ist ein Lied des japanischen Duos Yoasobi, das im April 2023 als Musikdownload und Stream veröffentlicht wurde. Im Mai erschien eine englischsprachige Version des Liedes mitsamt Musikvideo.")
            .wrap(Wrap{ trim: true })
            .styles(ctx.show_theme.style(WidgetStyle::PARAGRAPH))
            .style(style_opt(state.fg_3.value(), state.bg_3.value(), ctx)),
        &mut state.para3
    );

    // don't forget the popup ...
    form.render_popup(state.fg_1.id(), || fg_1_popup, &mut state.fg_1);
    form.render_popup(state.bg_1.id(), || bg_1_popup, &mut state.bg_1);
    form.render_popup(state.fg_2.id(), || fg_2_popup, &mut state.fg_2);
    form.render_popup(state.bg_2.id(), || bg_2_popup, &mut state.bg_2);
    form.render_popup(state.fg_3.id(), || fg_3_popup, &mut state.fg_3);
    form.render_popup(state.bg_3.id(), || bg_3_popup, &mut state.bg_3);

    Ok(())
}

fn style_opt(fg: Option<String>, bg: Option<String>, ctx: &mut Global) -> Style {
    if let Some(fg) = fg {
        if let Some(bg) = bg {
            ctx.show_theme.p.fg_bg_style_alias(fg.as_str(), bg.as_str())
        } else {
            ctx.show_theme.p.fg_style_alias(fg.as_str())
        }
    } else {
        if let Some(bg) = bg {
            ctx.show_theme.p.bg_style_alias(bg.as_str())
        } else {
            Style::default()
        }
    }
}

pub fn event(
    event: &crossterm::event::Event,
    state: &mut SampleCustom,
    _ctx: &mut Global,
) -> Result<Outcome, Error> {
    event_flow!(state.fg_1.handle(event, Popup));
    event_flow!(state.bg_1.handle(event, Popup));
    event_flow!(state.fg_2.handle(event, Popup));
    event_flow!(state.bg_2.handle(event, Popup));
    event_flow!(state.fg_3.handle(event, Popup));
    event_flow!(state.bg_3.handle(event, Popup));

    event_flow!(state.para1.handle(event, Regular));
    event_flow!(state.para2.handle(event, Regular));
    event_flow!(state.para3.handle(event, Regular));
    event_flow!(state.form.handle(event, Regular));
    Ok(Outcome::Continue)
}
