use crate::Global;
use anyhow::Error;
use rat_theme4::WidgetStyle;
use rat_widget::clipper::{Clipper, ClipperState};
use rat_widget::color_input::{ColorInput, ColorInputState, Mode};
use rat_widget::event::{HandleEvent, Outcome, Regular, TextOutcome, event_flow};
use rat_widget::focus::{HasFocus, impl_has_focus};
use rat_widget::layout::LayoutForm;
use rat_widget::scrolled::Scroll;
use rat_widget::text::impl_screen_cursor;
use ratatui::buffer::Buffer;
use ratatui::layout::{Flex, Rect};

#[allow(non_snake_case)]
#[derive(Debug, Default)]
pub struct Base46 {
    pub form: ClipperState,

    pub white: ColorInputState,
    pub darker_black: ColorInputState,
    pub black: ColorInputState,
    pub black2: ColorInputState,
    pub one_bg: ColorInputState,
    pub one_bg2: ColorInputState,
    pub one_bg3: ColorInputState,
    pub grey: ColorInputState,
    pub grey_fg: ColorInputState,
    pub grey_fg2: ColorInputState,
    pub light_grey: ColorInputState,
    pub red: ColorInputState,
    pub baby_pink: ColorInputState,
    pub pink: ColorInputState,
    pub line: ColorInputState,
    pub green: ColorInputState,
    pub vibrant_green: ColorInputState,
    pub nord_blue: ColorInputState,
    pub blue: ColorInputState,
    pub yellow: ColorInputState,
    pub sun: ColorInputState,
    pub purple: ColorInputState,
    pub dark_purple: ColorInputState,
    pub teal: ColorInputState,
    pub orange: ColorInputState,
    pub cyan: ColorInputState,
    pub statusline_bg: ColorInputState,
    pub lightbg: ColorInputState,
    pub pmenu_bg: ColorInputState,
    pub folder_bg: ColorInputState,
    pub base00: ColorInputState,
    pub base01: ColorInputState,
    pub base02: ColorInputState,
    pub base03: ColorInputState,
    pub base04: ColorInputState,
    pub base05: ColorInputState,
    pub base06: ColorInputState,
    pub base07: ColorInputState,
    pub base08: ColorInputState,
    pub base09: ColorInputState,
    pub base0A: ColorInputState,
    pub base0B: ColorInputState,
    pub base0C: ColorInputState,
    pub base0D: ColorInputState,
    pub base0E: ColorInputState,
    pub base0F: ColorInputState,
}

impl_has_focus!(
        white, darker_black, black, black2, one_bg, one_bg2, one_bg3, grey,
        grey_fg, grey_fg2, light_grey, red, baby_pink, pink, line, green,
        vibrant_green, nord_blue, blue, yellow, sun, purple, dark_purple,
        teal, orange, cyan, statusline_bg, lightbg, pmenu_bg, folder_bg,
        base00, base01, base02, base03, base04, base05, base06, base07,
        base08, base09, base0A, base0B, base0C, base0D, base0E, base0F
        for Base46);
impl_screen_cursor!(
    white, darker_black, black, black2, one_bg, one_bg2, one_bg3, grey,
    grey_fg, grey_fg2, light_grey, red, baby_pink, pink, line, green,
    vibrant_green, nord_blue, blue, yellow, sun, purple, dark_purple,
    teal, orange, cyan, statusline_bg, lightbg, pmenu_bg, folder_bg,
    base00, base01, base02, base03, base04, base05, base06, base07,
    base08, base09, base0A, base0B, base0C, base0D, base0E, base0F
    for Base46
);

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Base46,
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
        layout.widget(state.white.id(), L::Str("white"), W::Width(17));
        layout.widget(
            state.darker_black.id(),
            L::Str("darker_black"),
            W::Width(17),
        );
        layout.widget(state.black.id(), L::Str("black"), W::Width(17));
        layout.widget(state.black2.id(), L::Str("black2"), W::Width(17));
        layout.widget(state.one_bg.id(), L::Str("one_bg"), W::Width(17));
        layout.widget(state.one_bg2.id(), L::Str("one_bg2"), W::Width(17));
        layout.widget(state.one_bg3.id(), L::Str("one_bg3"), W::Width(17));
        layout.widget(state.grey.id(), L::Str("grey"), W::Width(17));
        layout.widget(state.grey_fg.id(), L::Str("grey_fg"), W::Width(17));
        layout.widget(state.grey_fg2.id(), L::Str("grey_fg2"), W::Width(17));
        layout.widget(state.light_grey.id(), L::Str("light_grey"), W::Width(17));
        layout.widget(state.red.id(), L::Str("red"), W::Width(17));
        layout.widget(state.baby_pink.id(), L::Str("baby_pink"), W::Width(17));
        layout.widget(state.pink.id(), L::Str("pink"), W::Width(17));
        layout.widget(state.line.id(), L::Str("line"), W::Width(17));
        layout.widget(state.green.id(), L::Str("green"), W::Width(17));
        layout.widget(
            state.vibrant_green.id(),
            L::Str("vibrant_green"),
            W::Width(17),
        );
        layout.widget(state.nord_blue.id(), L::Str("nord_blue"), W::Width(17));
        layout.widget(state.blue.id(), L::Str("blue"), W::Width(17));
        layout.widget(state.yellow.id(), L::Str("yellow"), W::Width(17));
        layout.widget(state.sun.id(), L::Str("sun"), W::Width(17));
        layout.widget(state.purple.id(), L::Str("purple"), W::Width(17));
        layout.widget(state.dark_purple.id(), L::Str("dark_purple"), W::Width(17));
        layout.widget(state.teal.id(), L::Str("teal"), W::Width(17));
        layout.widget(state.orange.id(), L::Str("orange"), W::Width(17));
        layout.widget(state.cyan.id(), L::Str("cyan"), W::Width(17));
        layout.widget(
            state.statusline_bg.id(),
            L::Str("statusline_bg"),
            W::Width(17),
        );
        layout.widget(state.lightbg.id(), L::Str("lightbg"), W::Width(17));
        layout.widget(state.pmenu_bg.id(), L::Str("pmenu_bg"), W::Width(17));
        layout.widget(state.folder_bg.id(), L::Str("folder_bg"), W::Width(17));
        layout.widget(state.base00.id(), L::Str("base00"), W::Width(17));
        layout.widget(state.base01.id(), L::Str("base01"), W::Width(17));
        layout.widget(state.base02.id(), L::Str("base02"), W::Width(17));
        layout.widget(state.base03.id(), L::Str("base03"), W::Width(17));
        layout.widget(state.base04.id(), L::Str("base04"), W::Width(17));
        layout.widget(state.base05.id(), L::Str("base05"), W::Width(17));
        layout.widget(state.base06.id(), L::Str("base06"), W::Width(17));
        layout.widget(state.base07.id(), L::Str("base07"), W::Width(17));
        layout.widget(state.base08.id(), L::Str("base08"), W::Width(17));
        layout.widget(state.base09.id(), L::Str("base09"), W::Width(17));
        layout.widget(state.base0A.id(), L::Str("base0A"), W::Width(17));
        layout.widget(state.base0B.id(), L::Str("base0B"), W::Width(17));
        layout.widget(state.base0C.id(), L::Str("base0C"), W::Width(17));
        layout.widget(state.base0D.id(), L::Str("base0D"), W::Width(17));
        layout.widget(state.base0E.id(), L::Str("base0E"), W::Width(17));
        layout.widget(state.base0F.id(), L::Str("base0F"), W::Width(17));
        form = form.layout(layout.build_endless(layout_size.width))
    }

    let mut form = form.into_buffer(area, &mut state.form);
    form.render(
        state.white.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.white,
    );
    form.render(
        state.darker_black.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.darker_black,
    );
    form.render(
        state.black.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.black,
    );
    form.render(
        state.black2.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.black2,
    );
    form.render(
        state.one_bg.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.one_bg,
    );
    form.render(
        state.one_bg2.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.one_bg2,
    );
    form.render(
        state.one_bg3.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.one_bg3,
    );
    form.render(
        state.grey.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.grey,
    );
    form.render(
        state.grey_fg.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.grey_fg,
    );
    form.render(
        state.grey_fg2.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.grey_fg2,
    );
    form.render(
        state.light_grey.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.light_grey,
    );
    form.render(
        state.red.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.red,
    );
    form.render(
        state.baby_pink.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.baby_pink,
    );
    form.render(
        state.pink.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.pink,
    );
    form.render(
        state.line.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.line,
    );
    form.render(
        state.green.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.green,
    );
    form.render(
        state.vibrant_green.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.vibrant_green,
    );
    form.render(
        state.nord_blue.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.nord_blue,
    );
    form.render(
        state.blue.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.blue,
    );
    form.render(
        state.yellow.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.yellow,
    );
    form.render(
        state.sun.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.sun,
    );
    form.render(
        state.purple.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.purple,
    );
    form.render(
        state.dark_purple.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.dark_purple,
    );
    form.render(
        state.teal.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.teal,
    );
    form.render(
        state.orange.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.orange,
    );
    form.render(
        state.cyan.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.cyan,
    );
    form.render(
        state.statusline_bg.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.statusline_bg,
    );
    form.render(
        state.lightbg.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.lightbg,
    );
    form.render(
        state.pmenu_bg.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.pmenu_bg,
    );
    form.render(
        state.folder_bg.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.folder_bg,
    );
    form.render(
        state.base00.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base00,
    );
    form.render(
        state.base01.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base01,
    );
    form.render(
        state.base02.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base02,
    );
    form.render(
        state.base03.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base03,
    );
    form.render(
        state.base04.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base04,
    );
    form.render(
        state.base05.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base05,
    );
    form.render(
        state.base06.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base06,
    );
    form.render(
        state.base07.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base07,
    );
    form.render(
        state.base08.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base08,
    );
    form.render(
        state.base09.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base09,
    );
    form.render(
        state.base0A.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base0A,
    );
    form.render(
        state.base0B.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base0B,
    );
    form.render(
        state.base0C.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base0C,
    );
    form.render(
        state.base0D.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base0D,
    );
    form.render(
        state.base0E.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base0E,
    );
    form.render(
        state.base0F.id(),
        || ColorInput::new().styles(ctx.theme.style(WidgetStyle::COLOR_INPUT)),
        &mut state.base0F,
    );
    form.finish(buf, &mut state.form);
    Ok(())
}

#[allow(unused_variables)]
pub fn event(
    event: &crossterm::event::Event,
    state: &mut Base46,
    _ctx: &mut Global,
) -> Result<Outcome, Error> {
    let mut mode_change = None;
    let r = 'f: {
        event_flow!(break 'f handle_color(event, &mut state.white, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.darker_black, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.black, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.black2, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.one_bg, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.one_bg2, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.one_bg3, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.grey, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.grey_fg, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.grey_fg2, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.light_grey, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.red, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.baby_pink, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.pink, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.line, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.green, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.vibrant_green, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.nord_blue, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.blue, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.yellow, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.sun, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.purple, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.dark_purple, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.teal, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.orange, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.cyan, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.statusline_bg, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.lightbg, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.pmenu_bg, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.folder_bg, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base00, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base01, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base02, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base03, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base04, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base05, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base06, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base07, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base08, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base09, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base0A, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base0B, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base0C, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base0D, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base0E, &mut mode_change)?);
        event_flow!(break 'f handle_color(event, &mut state.base0F, &mut mode_change)?);

        event_flow!(break 'f state.form.handle(event, Regular));
        Outcome::Continue
    };

    if let Some(mode_change) = mode_change {
        state.white.set_mode(mode_change);
        state.darker_black.set_mode(mode_change);
        state.black.set_mode(mode_change);
        state.black2.set_mode(mode_change);
        state.one_bg.set_mode(mode_change);
        state.one_bg2.set_mode(mode_change);
        state.one_bg3.set_mode(mode_change);
        state.grey.set_mode(mode_change);
        state.grey_fg.set_mode(mode_change);
        state.grey_fg2.set_mode(mode_change);
        state.light_grey.set_mode(mode_change);
        state.red.set_mode(mode_change);
        state.baby_pink.set_mode(mode_change);
        state.pink.set_mode(mode_change);
        state.line.set_mode(mode_change);
        state.green.set_mode(mode_change);
        state.vibrant_green.set_mode(mode_change);
        state.nord_blue.set_mode(mode_change);
        state.blue.set_mode(mode_change);
        state.yellow.set_mode(mode_change);
        state.sun.set_mode(mode_change);
        state.purple.set_mode(mode_change);
        state.dark_purple.set_mode(mode_change);
        state.teal.set_mode(mode_change);
        state.orange.set_mode(mode_change);
        state.cyan.set_mode(mode_change);
        state.statusline_bg.set_mode(mode_change);
        state.lightbg.set_mode(mode_change);
        state.pmenu_bg.set_mode(mode_change);
        state.folder_bg.set_mode(mode_change);
        state.base00.set_mode(mode_change);
        state.base01.set_mode(mode_change);
        state.base02.set_mode(mode_change);
        state.base03.set_mode(mode_change);
        state.base04.set_mode(mode_change);
        state.base05.set_mode(mode_change);
        state.base06.set_mode(mode_change);
        state.base07.set_mode(mode_change);
        state.base08.set_mode(mode_change);
        state.base09.set_mode(mode_change);
        state.base0A.set_mode(mode_change);
        state.base0B.set_mode(mode_change);
        state.base0C.set_mode(mode_change);
        state.base0D.set_mode(mode_change);
        state.base0E.set_mode(mode_change);
        state.base0F.set_mode(mode_change);
    }

    Ok(r)
}

fn handle_color(
    event: &crossterm::event::Event,
    color: &mut ColorInputState,
    mode_change: &mut Option<Mode>,
) -> Result<TextOutcome, Error> {
    let mode = color.mode();
    let r = color.handle(event, Regular);
    if color.mode() != mode {
        *mode_change = Some(color.mode());
    }
    Ok(r)
}

/*
white
darker_black
black
black2
one_bg
one_bg2
one_bg3
grey
grey_fg
grey_fg2
light_grey
red
baby_pink
pink
line
green
vibrant_green
nord_blue
blue
yellow
sun
purple
dark_purple
teal
orange
cyan
statusline_bg
lightbg
pmenu_bg
folder_bg
base00
base01
base02
base03
base04
base05
base06
base07
base08
base09
base0A
base0B
base0C
base0D
base0E
base0F
 */
