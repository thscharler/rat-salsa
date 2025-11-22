use crate::Global;
use anyhow::Error;
use rat_theme4::WidgetStyle;
use rat_theme4::palette::{ColorIdx, Colors};
use rat_widget::checkbox::{Checkbox, CheckboxState};
use rat_widget::choice::{Choice, ChoiceState};
use rat_widget::event::{HandleEvent, Outcome, Popup, Regular, event_flow};
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_widget::paragraph::{Paragraph, ParagraphState};
use rat_widget::scrolled::Scroll;
use rat_widget::text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{StatefulWidget, Wrap};

#[derive(Debug)]
pub struct SampleReadability {
    pub colors: ChoiceState<ColorIdx>,
    pub high_contrast: CheckboxState,
    pub para: ParagraphState,
}

impl SampleReadability {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for SampleReadability {
    fn default() -> Self {
        let mut z = Self {
            colors: Default::default(),
            high_contrast: Default::default(),
            para: Default::default(),
        };
        z.colors.set_value(ColorIdx(Colors::Gray, 0));
        z
    }
}

impl HasFocus for SampleReadability {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.colors);
        builder.widget(&self.high_contrast);
        builder.widget(&self.para);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not available")
    }

    fn area(&self) -> Rect {
        unimplemented!("not available")
    }
}

impl HasScreenCursor for SampleReadability {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        None
    }
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut SampleReadability,
    ctx: &mut Global,
) -> Result<(), Error> {
    let l0 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .split(area);
    let l1 = Layout::horizontal([
        Constraint::Fill(1), //
        Constraint::Fill(1),
    ])
    .spacing(1)
    .split(l0[1]);

    let pal_choice = crate::pal_choice(ctx.show_theme.p.clone());
    let (colors, colors_popup) = Choice::new()
        .items(pal_choice)
        .select_marker('*')
        .styles(ctx.show_theme.style(WidgetStyle::CHOICE))
        .into_widgets();
    colors.render(l1[0], buf, &mut state.colors);

    Checkbox::new()
        .styles(ctx.show_theme.style(WidgetStyle::CHECKBOX))
        .text("+Contrast")
        .render(l1[1], buf, &mut state.high_contrast);

    let sel_color = state.colors.value();
    let high_contrast = state.high_contrast.value();
    let text_style = if high_contrast {
        ctx.show_theme.p.high_style(sel_color.0, sel_color.1)
    } else {
        ctx.show_theme.p.style(sel_color.0, sel_color.1)
    };

    Paragraph::new(
            "
The __Paris Peace Accords__, officially the Agreement on Ending the War and Restoring Peace in Viet Nam, was a peace agreement signed on January 27, 1973, to establish peace in Vietnam and end the Vietnam War. The agreement was signed by the governments of the Democratic Republic of Vietnam (North Vietnam), the Republic of Vietnam (South Vietnam), the United States, and the Provisional Revolutionary Government of the Republic of South Vietnam (representing South Vietnamese communists).

The Paris Peace Accords removed the remaining United States forces, and fighting between the three remaining powers  temporarily stopped. The agreement's provisions were immediately and frequently broken by both North and South Vietnamese forces with no official response from the United States. Open fighting broke out in March 1973, and North Vietnamese offensives enlarged their territory by the end of the year. The war continued until the fall of Saigon to North Vietnamese forces in 1975. This photograph shows William P. Rogers, United States Secretary of State, signing the accords in Paris.
",
        )
            .vscroll(Scroll::new())
            .styles(ctx.show_theme.style(WidgetStyle::PARAGRAPH))
            .style(text_style)
            .wrap(Wrap { trim: false })
            .render(l0[3], buf, &mut state.para);

    // don't forget the popup ...
    colors_popup.render(l1[0], buf, &mut state.colors);

    Ok(())
}

pub fn event(
    event: &crossterm::event::Event,
    state: &mut SampleReadability,
    _ctx: &mut Global,
) -> Result<Outcome, Error> {
    event_flow!(state.colors.handle(event, Popup));
    event_flow!(state.high_contrast.handle(event, Regular));
    event_flow!(state.para.handle(event, Regular));
    Ok(Outcome::Continue)
}
