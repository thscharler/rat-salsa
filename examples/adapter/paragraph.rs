use crate::adapter::_private::NonExhaustive;
use rat_event::{flow, HandleEvent, MouseOnly, Outcome};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget, Wrap};

#[derive(Debug, Default)]
pub struct ParagraphS<'a> {
    widget: Paragraph<'a>,
    is_wrap: bool,
    block: Option<Block<'a>>,
    vscroll: Option<Scroll<'a>>,
    hscroll: Option<Scroll<'a>>,
}

#[derive(Debug)]
pub struct ParagraphSState {
    pub area: Rect,
    pub inner: Rect,

    pub len: usize,
    pub vscroll: ScrollState,
    pub width: usize,
    pub hscroll: ScrollState,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> ParagraphS<'a> {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        let t = text.into();
        Self {
            widget: Paragraph::new(t),
            ..Self::default()
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn scroll(mut self, scroll: Scroll<'a>) -> Self {
        self.hscroll = Some(scroll.clone().override_horizontal());
        self.vscroll = Some(scroll.override_vertical());
        self
    }

    pub fn hscroll(mut self, scroll: Scroll<'a>) -> Self {
        self.hscroll = Some(scroll.override_horizontal());
        self
    }

    pub fn vscroll(mut self, scroll: Scroll<'a>) -> Self {
        self.vscroll = Some(scroll.override_vertical());
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.widget = self.widget.style(style);
        self
    }

    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.is_wrap = true;
        self.widget = self.widget.wrap(wrap);
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.widget = self.widget.alignment(alignment);
        self
    }

    pub fn left_aligned(mut self) -> Self {
        self.widget = self.widget.left_aligned();
        self
    }

    pub fn centered(mut self) -> Self {
        self.widget = self.widget.centered();
        self
    }

    pub fn right_aligned(mut self) -> Self {
        self.widget = self.widget.right_aligned();
        self
    }
}

impl<'a> StatefulWidget for ParagraphS<'a> {
    type State = ParagraphSState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;

        let scroll = ScrollArea::new()
            .block(self.block.as_ref())
            .h_scroll(self.hscroll.as_ref())
            .v_scroll(self.vscroll.as_ref());
        state.inner = scroll.inner(area, Some(&state.hscroll), Some(&state.vscroll));

        state.hscroll.set_max_offset(if self.is_wrap {
            0
        } else {
            self.widget
                .line_width()
                .saturating_sub(state.inner.width as usize)
        });
        state.hscroll.set_page_len(state.inner.width as usize);

        let lines = self.widget.line_count(area.width) + 4; // 4 is an estimate. line_count seems not very accurate.
        state
            .vscroll
            .set_max_offset(lines.saturating_sub(state.inner.height as usize));
        state.vscroll.set_page_len(state.inner.height as usize);

        scroll.render(
            area,
            buf,
            &mut ScrollAreaState::new()
                .h_scroll(&mut state.hscroll)
                .v_scroll(&mut state.vscroll),
        );

        self.widget
            .clone() // TODO: not optimal
            .scroll((state.vscroll.offset() as u16, state.hscroll.offset() as u16))
            .render(state.inner, buf);
    }
}

impl Default for ParagraphSState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            len: 0,
            vscroll: Default::default(),
            width: 0,
            hscroll: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ParagraphSState {
    pub fn vertical_offset(&self) -> usize {
        self.vscroll.offset()
    }

    pub fn set_vertical_offset(&mut self, offset: usize) -> bool {
        self.vscroll.set_offset(offset)
    }

    pub fn horizontal_offset(&self) -> usize {
        self.hscroll.offset()
    }

    pub fn set_horizontal_offset(&mut self, offset: usize) -> bool {
        self.hscroll.set_offset(offset)
    }

    pub fn hscroll(&mut self, n: isize) -> bool {
        self.hscroll.set_offset(
            self.hscroll
                .clamp_offset(self.hscroll.offset() as isize + n),
        )
    }

    pub fn vscroll(&mut self, n: isize) -> bool {
        self.vscroll.set_offset(
            self.vscroll
                .clamp_offset(self.vscroll.offset() as isize + n),
        )
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ParagraphSState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        flow!(match self.hscroll.handle(event, MouseOnly) {
            ScrollOutcome::HPos(v) => {
                self.set_horizontal_offset(v).into()
            }
            r => Outcome::from(r),
        });
        flow!(match self.vscroll.handle(event, MouseOnly) {
            ScrollOutcome::VPos(v) => {
                self.set_vertical_offset(v).into()
            }
            r => Outcome::from(r),
        });
        flow!({
            let mut sas = ScrollAreaState::new()
                .area(self.inner)
                .h_scroll(&mut self.hscroll)
                .v_scroll(&mut self.vscroll);
            match sas.handle(event, MouseOnly) {
                ScrollOutcome::Up(v) => {
                    if self.vscroll(-(v as isize)) {
                        Outcome::Changed
                    } else {
                        Outcome::Continue
                    }
                }
                ScrollOutcome::Down(v) => {
                    if self.vscroll(v as isize) {
                        Outcome::Changed
                    } else {
                        Outcome::Continue
                    }
                }
                ScrollOutcome::Left(v) => {
                    if self.hscroll(-(v as isize)) {
                        Outcome::Changed
                    } else {
                        Outcome::Continue
                    }
                }
                ScrollOutcome::Right(v) => {
                    if self.hscroll(v as isize) {
                        Outcome::Changed
                    } else {
                        Outcome::Continue
                    }
                }
                r => Outcome::from(r),
            }
        });

        Outcome::Continue
    }
}
