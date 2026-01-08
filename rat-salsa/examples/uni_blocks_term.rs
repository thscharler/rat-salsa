use crate::glyph_info::{GlyphInfo, GlyphInfoState};
use crate::glyphs::{Glyphs, GlyphsState};
use crate::uni_blocks_data::BLOCKS;
use anyhow::Error;
use log::{debug, error};
use rat_event::{HandleEvent, Outcome, Popup, Regular, ct_event, event_flow};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_salsa::event::{QuitEvent, RenderedEvent};
use rat_salsa::poll::PollCrossterm;
use rat_salsa::timer::TimeOut;
use rat_salsa::{Control, SalsaAppContext, SalsaContext};
use rat_salsa::{RunConfig, run_tui};
use rat_theme4::theme::SalsaTheme;
use rat_theme4::{StyleName, WidgetStyle, create_salsa_theme, salsa_themes};
use rat_widget::checkbox::{Checkbox, CheckboxState};
use rat_widget::choice::{Choice, ChoiceState};
use rat_widget::event::{ChoiceOutcome, SliderOutcome, TextOutcome};
use rat_widget::paired::Paired;
use rat_widget::popup::Placement;
use rat_widget::scrolled::{Scroll, ScrollbarPolicy};
use rat_widget::slider::{Slider, SliderState};
use rat_widget::text::clipboard::cli::setup_cli_clipboard;
use rat_widget::text::{HasScreenCursor, TextStyle};
use rat_widget::text_input::{TextInput, TextInputState};
use rat_widget::text_input_mask::{MaskedInput, MaskedInputState};
use rat_widget::view::{View, ViewState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Layout, Rect};
use ratatui_core::style::Style;
use ratatui_core::text::{Line, Span};
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;
use std::fs;
use std::path::PathBuf;

mod uni_blocks_data;

// static SAMPLE_TEXT: &str = "ff fl <= >= ﬁ ﬂ";
// static SAMPLE_TEXT: &str = "Hello World! مرحبا بالعالم 0123456789000000000";
static SAMPLE_TEXT: &str = "مرحبا بالعالم";
static SAMPLE_SPAN: &str = "";

pub fn main() -> Result<(), Error> {
    setup_logging()?;
    setup_cli_clipboard();

    let config = Config::default();
    let theme = create_salsa_theme("EverForest Light");
    let mut global = Global::new(config, theme);
    let mut state = Minimal::new();

    run_tui(
        init, //
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::default()? //
            .poll(PollCrossterm),
    )?;

    Ok(())
}

/// Globally accessible data/state.
pub struct Global {
    // the salsa machinery
    ctx: SalsaAppContext<AppEvent, Error>,

    pub cfg: Config,
    pub theme: SalsaTheme,
    pub blocks: &'static [&'static str],
}

impl SalsaContext<AppEvent, Error> for Global {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<AppEvent, Error>) {
        self.ctx = app_ctx;
    }

    fn salsa_ctx(&self) -> &SalsaAppContext<AppEvent, Error> {
        &self.ctx
    }
}

impl Global {
    pub fn new(cfg: Config, theme: SalsaTheme) -> Self {
        Self {
            ctx: Default::default(),
            cfg,
            theme,
            blocks: BLOCKS,
        }
    }
}

/// Configuration.
#[derive(Debug, Default)]
pub struct Config {}

#[derive(Debug)]
pub enum AppEvent {
    NoOp,
    CtEvent(crossterm::event::Event),
    TimeOut(TimeOut),
    Quit,
    Rendered,
}

impl From<crossterm::event::Event> for AppEvent {
    fn from(value: crossterm::event::Event) -> Self {
        AppEvent::CtEvent(value)
    }
}

impl From<RenderedEvent> for AppEvent {
    fn from(_: RenderedEvent) -> Self {
        AppEvent::Rendered
    }
}

impl From<QuitEvent> for AppEvent {
    fn from(_: QuitEvent) -> Self {
        AppEvent::Quit
    }
}

impl From<TimeOut> for AppEvent {
    fn from(value: TimeOut) -> Self {
        Self::TimeOut(value)
    }
}

#[derive(Debug)]
pub struct Minimal {
    pub blocks: ChoiceState<usize>,
    pub underline: CheckboxState,
    pub bold: CheckboxState,
    pub italic: CheckboxState,
    pub combining_base: MaskedInputState,
    pub free_text: TextInputState,

    pub view: ViewState,
    pub glyphs: GlyphsState,
    pub glyphinfo: GlyphInfoState,
}

impl Minimal {
    pub fn new() -> Self {
        Self {
            blocks: Default::default(),
            underline: Default::default(),
            bold: Default::default(),
            italic: Default::default(),
            combining_base: MaskedInputState::new().with_mask("_").expect("valid mask"),
            free_text: Default::default(),
            view: Default::default(),
            glyphs: Default::default(),
            glyphinfo: Default::default(),
        }
    }
}

impl HasFocus for Minimal {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.blocks);
        builder.widget(&self.underline);
        builder.widget(&self.bold);
        builder.widget(&self.italic);
        builder.widget(&self.combining_base);
        builder.widget(&self.free_text);
        builder.widget(&self.glyphs);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!()
    }

    fn area(&self) -> Rect {
        unimplemented!()
    }
}

pub fn init(state: &mut Minimal, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(state));
    ctx.focus().focus(&state.glyphs);

    state.free_text.set_value(SAMPLE_TEXT);

    Ok(())
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Minimal,
    ctx: &mut Global,
) -> Result<(), Error> {
    let vlayout = Layout::vertical([
        Constraint::Length(6), //
        Constraint::Fill(1),   //
    ])
    .split(area);
    let hlayout = Layout::horizontal([
        Constraint::Percentage(61), //
        Constraint::Percentage(39),
    ])
    .spacing(1)
    .split(vlayout[1]);

    let bg_style = ctx.theme.style_style(Style::CONTAINER_BASE);
    buf.set_style(area, bg_style);

    Span::from(" :: ").render(Rect::new(area.x, area.y, 4, 1), buf);

    let blocks_area = Rect::new(area.x + 6, area.y + 1, 40, 1);
    let (blocks, blocks_popup) = Choice::new()
        .items(ctx.blocks.iter().enumerate().map(|(n, v)| (n, *v)))
        .popup_len(4)
        .popup_placement(Placement::Right)
        .popup_y_offset(-2)
        .styles(ctx.theme.style(WidgetStyle::CHOICE))
        .into_widgets();
    blocks.render(blocks_area, buf, &mut state.blocks);

    let underline_area = Rect::new(area.x + 6, area.y + 2, 14, 1);
    Checkbox::new()
        .text("underline")
        .styles(ctx.theme.style(WidgetStyle::CHECKBOX))
        .render(underline_area, buf, &mut state.underline);

    let bold_area = Rect::new(area.x + 20, area.y + 2, 9, 1);
    Checkbox::new()
        .text("bold")
        .styles(ctx.theme.style(WidgetStyle::CHECKBOX))
        .render(bold_area, buf, &mut state.bold);

    let italic_area = Rect::new(area.x + 29, area.y + 2, 11, 1);
    Checkbox::new()
        .text("italic")
        .styles(ctx.theme.style(WidgetStyle::CHECKBOX))
        .render(italic_area, buf, &mut state.italic);

    let combining_area = Rect::new(area.x + 47, area.y + 2, 20, 1);
    Paired::new_labeled(
        "combining-base",
        MaskedInput::new().styles(ctx.theme.style(WidgetStyle::TEXT)),
    )
    .render(combining_area, buf, &mut state.combining_base);

    let free_text_area = Rect::new(area.x + 6, area.y + 4, 61, 1);
    let mut free_text_style = ctx.theme.style::<TextStyle>(WidgetStyle::TEXT);
    if state.underline.checked() {
        free_text_style.style = free_text_style.style.underlined();
    }
    if state.bold.checked() {
        free_text_style.style = free_text_style.style.bold();
    }
    if state.italic.checked() {
        free_text_style.style = free_text_style.style.italic();
    }
    TextInput::new().styles(free_text_style).render(
        free_text_area,
        buf,
        &mut state.free_text,
    );

    let sample_area = Rect::new(area.x + 6, area.y + 4, 55, 1);
    Span::from(SAMPLE_SPAN).render(sample_area, buf);

    let block = ctx.blocks[state.blocks.value()];
    let block = unic_ucd::BlockIter::new()
        .find(|v| v.name == block)
        .expect("block");

    let glyphs = Glyphs::new()
        .style(ctx.theme.style(Style::DOCUMENT_BASE))
        .codepoint_style(ctx.theme.style(Style::CONTAINER_BASE))
        .combining_base(state.combining_base.text())
        .focus_style(ctx.theme.style(Style::FOCUS))
        .underline(state.underline.value())
        .bold(state.bold.value())
        .italic(state.italic.value())
        .start(block.range.low)
        .end(block.range.high);

    let mut view_buf = View::new()
        .view_height(glyphs.height())
        .view_width(glyphs.width())
        .block(Block::bordered().border_type(BorderType::Rounded))
        .vscroll(Scroll::new().policy(ScrollbarPolicy::Always))
        .hscroll(Scroll::new())
        .styles(ctx.theme.style(WidgetStyle::VIEW))
        .into_buffer(hlayout[0], &mut state.view);

    let glyphs_area = Rect::new(0, 0, glyphs.width(), glyphs.height());
    view_buf.render(glyphs, glyphs_area, &mut state.glyphs);

    view_buf.finish(buf, &mut state.view);

    if let Some(cc) = state.glyphs.codepoint.get(state.glyphs.selected) {
        let info_area = Rect::new(
            hlayout[1].x,
            hlayout[1].y + 1,
            hlayout[1].width,
            hlayout[1].height,
        );
        GlyphInfo::new()
            .style(ctx.theme.style(Style::CONTAINER_BASE))
            .cc(*cc)
            .combining_base(state.combining_base.text())
            .render(info_area, buf, &mut state.glyphinfo);
    }

    // popup
    blocks_popup.render(blocks_area, buf, &mut state.blocks);

    ctx.set_screen_cursor(
        state
            .free_text
            .screen_cursor()
            .or(state.combining_base.screen_cursor())
            .or(state.glyphs.screen_cursor()),
    );

    Ok(())
}

pub fn event(
    event: &AppEvent,
    state: &mut Minimal,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    if let AppEvent::CtEvent(event) = event {
        ctx.set_focus(FocusBuilder::rebuild_for(state, ctx.take_focus()));
        ctx.handle_focus(event);

        match event {
            ct_event!(resized) => event_flow!({ Control::Changed }),
            ct_event!(key press CONTROL-'q') => event_flow!(Control::Quit),

            ct_event!(keycode press F(3)) => event_flow!({ next_block(state, ctx)? }),
            ct_event!(keycode press SHIFT-F(3)) => event_flow!({ prev_block(state, ctx)? }),

            ct_event!(keycode press F(4)) => event_flow!({
                state.underline.flip_checked();
                Control::Changed
            }),
            ct_event!(keycode press F(5)) => event_flow!({
                state.bold.flip_checked();
                Control::Changed
            }),
            ct_event!(keycode press F(6)) => event_flow!({
                state.italic.flip_checked();
                Control::Changed
            }),

            ct_event!(keycode press F(8)) => event_flow!({
                let themes = salsa_themes();
                let pos = themes
                    .iter()
                    .position(|v| *v == ctx.theme.name())
                    .unwrap_or(0);
                let pos = (pos + 1) % themes.len();
                ctx.theme = create_salsa_theme(&themes[pos]);
                Control::Changed
            }),
            ct_event!(keycode press SHIFT-F(8)) => event_flow!({
                let themes = salsa_themes();
                let pos = themes
                    .iter()
                    .position(|v| *v == ctx.theme.name())
                    .unwrap_or(0);
                let pos = pos.saturating_sub(1);
                ctx.theme = create_salsa_theme(&themes[pos]);
                Control::Changed
            }),

            _ => {}
        }

        event_flow!(state.blocks.handle(event, Popup));

        event_flow!(state.underline.handle(event, Regular));
        event_flow!(state.bold.handle(event, Regular));
        event_flow!(state.italic.handle(event, Regular));
        event_flow!(state.combining_base.handle(event, Regular));
        event_flow!(match state.free_text.handle(event, Regular) {
            TextOutcome::Changed => {
                debug!("free text changed");
                Control::Changed
            }
            TextOutcome::TextChanged => {
                debug!("free text changed");
                Control::Changed
            }
            r => r.into(),
        });
        event_flow!(state.view.handle(event, Regular));
        event_flow!(match state.glyphs.handle(event, Regular) {
            Outcome::Changed => {
                state.view.show_area(state.glyphs.selected_view());
                Control::Changed
            }
            r => r.into(),
        });

        if state.glyphs.is_focused() {
            match event {
                ct_event!(keycode press Home) => event_flow!({
                    state.blocks.set_value(0);
                    Control::Changed
                }),
                ct_event!(keycode press End) => event_flow!({
                    state.blocks.set_value(ctx.blocks.len().saturating_sub(1));
                    Control::Changed
                }),
                ct_event!(keycode press PageDown) => event_flow!({ next_block(state, ctx)? }),
                ct_event!(keycode press PageUp) => event_flow!({ prev_block(state, ctx)? }),
                _ => {}
            }
        }
    }

    Ok(Control::Continue)
}

fn next_block(state: &mut Minimal, ctx: &mut Global) -> Result<Control<AppEvent>, Error> {
    let v = state.blocks.value();
    if v + 1 < ctx.blocks.len() {
        state.blocks.set_value(v + 1);
        state.glyphs.selected = 0;
        state.view.set_vertical_offset(0);
        state.view.set_horizontal_offset(0);
    }
    Ok(Control::Changed)
}

fn prev_block(state: &mut Minimal, _ctx: &mut Global) -> Result<Control<AppEvent>, Error> {
    let v = state.blocks.value();
    if v > 0 {
        state.blocks.set_value(v - 1);
        state.glyphs.selected = 0;
        state.view.set_vertical_offset(0);
        state.view.set_horizontal_offset(0);
    }
    Ok(Control::Changed)
}

pub fn error(
    event: Error,
    _state: &mut Minimal,
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    error!("{:?}", event);
    Ok(Control::Changed)
}

fn setup_logging() -> Result<(), Error> {
    let log_path = PathBuf::from("");
    let log_file = log_path.join("uni_blocks.log");
    _ = fs::remove_file(&log_file);
    fern::Dispatch::new()
        .format(|out, message, record| {
            if record.target() == "rat_salsa_wgpu::framework" {
                out.finish(format_args!("{}", message)) //
            }
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(&log_file)?)
        .apply()?;
    Ok(())
}

mod glyph_info {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;
    use ratatui_core::style::Style;
    use ratatui_core::text::Text;
    use ratatui_core::widgets::{StatefulWidget, Widget};
    use std::fmt::{Debug, Formatter};
    use std::marker::PhantomData;
    use unic_ucd::{CanonicalCombiningClass, Name};
    use unicode_script::UnicodeScript;
    use unicode_width::UnicodeWidthChar;

    pub struct GlyphInfo<'a> {
        cc: char,
        combining_base: &'a str,
        style: Style,
        _phantom: PhantomData<&'a ()>,
    }

    #[derive(Default)]
    pub struct GlyphInfoState {
        pub area: Rect,
    }

    impl Debug for GlyphInfoState {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("GlyphInfoState").finish()
        }
    }

    impl<'a> GlyphInfo<'a> {
        pub fn new() -> Self {
            Self {
                cc: 'A',
                combining_base: " ",
                style: Default::default(),
                _phantom: Default::default(),
            }
        }

        pub fn style(mut self, style: Style) -> Self {
            self.style = style;
            self
        }

        pub fn combining_base(mut self, cc: &'a str) -> Self {
            self.combining_base = cc;
            self
        }

        pub fn cc(mut self, cc: char) -> Self {
            self.cc = cc;
            self
        }
    }

    impl<'a> StatefulWidget for GlyphInfo<'a> {
        type State = GlyphInfoState;

        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            state.area = area;

            let mut txt = Text::default();
            txt.push_line(format!("codepoint {:05x}", self.cc as u32));
            if let Some(name) = Name::of(self.cc) {
                txt.push_line(format!("name {}", name));
            }
            txt.push_line(format!("width {:?}", self.cc.width().unwrap_or_default()));
            txt.push_line(format!("script {:?}", self.cc.script()));
            txt.push_line(format!("script-ext {:?}", self.cc.script_extension()));
            txt.push_line(format!(
                "combining {}",
                CanonicalCombiningClass::of(self.cc).is_reordered()
            ));
            txt.push_line("");

            buf.set_style(area, self.style);
            txt.render(area, buf);
        }
    }

    impl GlyphInfoState {
        pub fn new() -> Self {
            Self {
                area: Default::default(),
            }
        }
    }
}

mod glyphs {
    use log::debug;
    use rat_event::{FromBool, HandleEvent, Outcome, Regular, ct_event, event_flow};
    use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
    use rat_widget::reloc::{RelocatableState, relocate_area, relocate_pos_tuple_opt};
    use rat_widget::text::HasScreenCursor;
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;
    use ratatui_core::style::Style;
    use ratatui_core::text::Span;
    use ratatui_core::widgets::{StatefulWidget, Widget};
    use ratatui_crossterm::crossterm::event::Event;
    use std::marker::PhantomData;
    use unic_ucd::CanonicalCombiningClass;
    use unicode_width::UnicodeWidthChar;

    const CLUSTER: u32 = 16;

    pub struct Glyphs<'a> {
        style: Style,
        codepoint_style: Style,
        focus_style: Style,
        start: char,
        end: char,
        underline: bool,
        bold: bool,
        italic: bool,
        combining_base: &'a str,
        _phantom: PhantomData<&'a ()>,
    }

    #[derive(Debug, Default)]
    pub struct GlyphsState {
        pub area: Rect,

        // selected. may not correspond with the vec's below.
        pub selected: usize,

        // codepoints displayed
        pub codepoint: Vec<char>,
        // areas for each codepoint in display coord.
        pub areas: Vec<Rect>,
        // areas for each codepoint in rendered coord.
        pub rendered: Vec<Rect>,
        // screen-cursor
        screen_cursor: Option<(u16, u16)>,

        pub focus: FocusFlag,
    }

    impl<'a> Glyphs<'a> {
        pub fn new() -> Self {
            Self {
                style: Default::default(),
                codepoint_style: Default::default(),
                focus_style: Default::default(),
                start: '\u{0000}',
                end: '\u{0000}',
                underline: Default::default(),
                bold: Default::default(),
                italic: Default::default(),
                combining_base: " ",
                _phantom: Default::default(),
            }
        }

        pub fn style(mut self, style: Style) -> Self {
            self.style = style;
            self
        }

        pub fn combining_base(mut self, cc: &'a str) -> Self {
            self.combining_base = cc;
            self
        }

        pub fn codepoint_style(mut self, style: Style) -> Self {
            self.codepoint_style = style;
            self
        }

        #[allow(dead_code)]
        pub fn focus_style(mut self, style: Style) -> Self {
            self.focus_style = style;
            self
        }

        pub fn start(mut self, cc: char) -> Self {
            self.start = cc;
            self
        }

        pub fn end(mut self, cc: char) -> Self {
            self.end = cc;
            self
        }

        pub fn underline(mut self, underline: bool) -> Self {
            self.underline = underline;
            self
        }

        pub fn bold(mut self, bold: bool) -> Self {
            self.bold = bold;
            self
        }

        pub fn italic(mut self, italic: bool) -> Self {
            self.italic = italic;
            self
        }

        pub fn width(&self) -> u16 {
            9 + CLUSTER as u16 * 2
        }

        pub fn height(&self) -> u16 {
            let rows = (self.end as u32 - self.start as u32) / CLUSTER + 1;
            rows as u16 * 2
        }
    }

    impl RelocatableState for GlyphsState {
        fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
            self.area.relocate(shift, clip);
            self.screen_cursor = relocate_pos_tuple_opt(self.screen_cursor, shift, clip);
            for (rendered, area) in self.rendered.iter().zip(self.areas.iter_mut()) {
                *area = relocate_area(*rendered, shift, clip);
            }
        }
    }

    impl HasFocus for GlyphsState {
        fn build(&self, builder: &mut FocusBuilder) {
            builder.leaf_widget(self);
        }

        fn focus(&self) -> FocusFlag {
            self.focus.clone()
        }

        fn area(&self) -> Rect {
            self.area
        }
    }

    impl HasScreenCursor for GlyphsState {
        fn screen_cursor(&self) -> Option<(u16, u16)> {
            self.screen_cursor
        }
    }

    impl<'a> StatefulWidget for Glyphs<'a> {
        type State = GlyphsState;

        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            state.area = area;
            state.screen_cursor = None;

            state.codepoint.clear();
            state.areas.clear();
            state.rendered.clear();

            buf.set_style(area, self.style);

            let mut tmp = String::new();
            for cc in self.start..=self.end {
                let off = cc as u32 - self.start as u32;

                let row = off / CLUSTER;
                let col = off % CLUSTER;

                if col == 0 {
                    let byte_span = format!("{:#06x} ", self.start as u32 + off,);
                    let head_area = Rect::new(area.x, area.y + 2 * row as u16, 14, 1);
                    Span::from(byte_span).render(head_area, buf);
                }

                let mut glyph_style = if state.is_focused() && state.selected == off as usize {
                    self.codepoint_style.patch(self.focus_style)
                } else {
                    self.codepoint_style
                };
                glyph_style = if self.underline {
                    glyph_style.underlined()
                } else {
                    glyph_style
                };
                glyph_style = if self.bold {
                    glyph_style.bold()
                } else {
                    glyph_style
                };
                glyph_style = if self.italic {
                    glyph_style.italic()
                } else {
                    glyph_style
                };

                let mut cp_area = Rect::new(
                    area.x + 9 + 2 * col as u16, //
                    area.y + 2 * row as u16,
                    1,
                    1,
                );
                if state.is_focused() && state.selected == off as usize {
                    state.screen_cursor = Some((cp_area.x, cp_area.y));
                }

                if let Some(cell) = buf.cell_mut(cp_area.as_position()) {
                    cell.set_style(glyph_style);

                    if cc as u32 >= 32 && cc as u32 != 127 {
                        cp_area.width = cc.width().unwrap_or(1).max(1) as u16;

                        tmp.clear();
                        if CanonicalCombiningClass::of(cc).is_reordered() {
                            tmp.push_str(self.combining_base);
                        } else if cc.width() == Some(0) {
                            // would combine with the previous cell. probably.
                            tmp.push(' ');
                        }
                        tmp.push(cc);
                        cell.set_symbol(&tmp);
                    } else {
                        cell.set_symbol("?");
                    }
                }

                // need to skip. span et al. do this automatically.
                if let Some(cell) = buf.cell_mut((cp_area.x + 1, cp_area.y)) {
                    cell.skip = cc.width().unwrap_or(1) > 1;
                }

                state.codepoint.push(cc);
                state.rendered.push(cp_area.intersection(area));
                state.areas.push(cp_area.intersection(area));
            }
        }
    }

    impl GlyphsState {
        pub fn new() -> Self {
            Self {
                area: Default::default(),
                selected: 0,
                codepoint: Default::default(),
                areas: Default::default(),
                rendered: Default::default(),
                screen_cursor: None,
                focus: Default::default(),
            }
        }

        /// Returns the area for the codepoint in view-coords (rendered coords).
        pub fn selected_view(&self) -> Rect {
            if self.selected < self.codepoint.len() {
                self.rendered[self.selected]
            } else {
                Rect::default()
            }
        }

        pub fn first(&mut self) -> bool {
            let old_idx = self.selected;
            self.selected = 0;
            debug!("first {}", self.selected != old_idx);
            self.selected != old_idx
        }

        pub fn last(&mut self) -> bool {
            let old_idx = self.selected;
            self.selected = self.codepoint.len().saturating_sub(1);
            debug!("last {}", self.selected != old_idx);
            self.selected != old_idx
        }

        pub fn next(&mut self) -> bool {
            let old_idx = self.selected;
            if self.selected + 1 < self.codepoint.len() {
                self.selected += 1;
            } else {
                self.selected = self.codepoint.len().saturating_sub(1);
            }

            self.selected != old_idx
        }

        pub fn prev(&mut self) -> bool {
            let old_idx = self.selected;
            if self.selected > 0 {
                if self.selected < self.codepoint.len() {
                    self.selected -= 1;
                } else {
                    self.selected = self.codepoint.len().saturating_sub(1);
                }
            }

            self.selected != old_idx
        }

        pub fn up(&mut self) -> bool {
            let old_idx = self.selected;
            if self.selected >= CLUSTER as usize {
                if self.selected < self.codepoint.len() {
                    self.selected -= CLUSTER as usize;
                } else {
                    self.selected = self.codepoint.len().saturating_sub(1);
                }
            }

            self.selected != old_idx
        }

        pub fn down(&mut self) -> bool {
            let old_idx = self.selected;
            if (self.selected + CLUSTER as usize) < self.codepoint.len() {
                self.selected += CLUSTER as usize;
            } else if self.selected >= self.codepoint.len() {
                self.selected = self.codepoint.len().saturating_sub(1);
            }

            self.selected != old_idx
        }
    }

    impl HandleEvent<Event, Regular, Outcome> for GlyphsState {
        fn handle(&mut self, event: &Event, _qualifier: Regular) -> Outcome {
            if self.is_focused() {
                event_flow!(
                    return match event {
                        ct_event!(keycode press Home) => self.first().as_changed_continue(),
                        ct_event!(keycode press End) => self.last().as_changed_continue(),
                        ct_event!(keycode press Left) => self.prev().into(),
                        ct_event!(keycode press Right) => self.next().into(),
                        ct_event!(keycode press Up) => self.up().into(),
                        ct_event!(keycode press Down) => self.down().into(),
                        _ => Outcome::Continue,
                    }
                );
            }

            match event {
                ct_event!(mouse down Left for x,y) => event_flow!(
                    return {
                        if let Some(idx) = rat_event::util::item_at(&self.areas, *x, *y) {
                            self.selected = idx;
                            Outcome::Changed
                        } else {
                            Outcome::Continue
                        }
                    }
                ),
                _ => {}
            }

            Outcome::Continue
        }
    }
}
