use crate::Global;
use anyhow::Error;
use crossterm::event::Event;
use rat_theme4::WidgetStyle;
use rat_theme4::theme::SalsaTheme;
use rat_widget::dialog_frame::{DialogFrame, DialogFrameState, DialogOutcome};
use rat_widget::event::{Dialog, HandleEvent, Outcome, Popup, Regular, event_flow};
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_widget::form::{Form, FormState};
use rat_widget::layout::LayoutForm;
use rat_widget::list::{List, ListState};
use rat_widget::menu::{Menubar, MenubarState, StaticMenu};
use rat_widget::popup::Placement;
use rat_widget::reloc::RelocatableState;
use rat_widget::scrolled::Scroll;
use rat_widget::splitter::{Split, SplitState, SplitType};
use rat_widget::statusline::{StatusLine, StatusLineState};
use rat_widget::table::textdata::{Cell, Row};
use rat_widget::table::{Table, TableState};
use rat_widget::text::HasScreenCursor;
use rat_widget::text_input::{TextInput, TextInputState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::prelude::StatefulWidget;
use ratatui::widgets::Block;
use std::cmp::min;

#[derive(Debug)]
pub struct SampleOther {
    pub form: FormState<usize>,
    pub dialog: DialogSampleState,
    pub split: SplitState,
    pub list: ListState,
    pub table: TableState,
    pub menu: MenubarState,
    pub status: StatusLineState,
}

impl HasFocus for SampleOther {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.menu);
        builder.widget(&self.form);
        builder.widget(&self.list);
        builder.widget(&self.table);
        builder.widget(&self.split);
        builder.widget(&self.dialog);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not available")
    }

    fn area(&self) -> Rect {
        unimplemented!("not available")
    }
}

impl HasScreenCursor for SampleOther {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.dialog.screen_cursor()
    }
}

impl Default for SampleOther {
    fn default() -> Self {
        let mut z = Self {
            form: FormState::named("form"),
            dialog: DialogSampleState::default(),
            split: SplitState::named("split"),
            list: ListState::named("list"),
            table: TableState::named("table"),
            menu: MenubarState::named("menubar"),
            status: StatusLineState::named("status"),
        };
        z.status.status(0, "... something ...");
        z.status.status(1, "[join]");
        z.status.status(2, "[conn]");
        z.status.status(3, "[sync]");
        z
    }
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut SampleOther,
    ctx: &mut Global,
) -> Result<(), Error> {
    let l0 = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .spacing(1)
    .split(area);

    let mut form = Form::new().styles(ctx.show_theme.style(WidgetStyle::FORM));

    let layout_size = form.layout_size(area);
    if !state.form.valid_layout(layout_size) {
        use rat_widget::layout::{FormLabel as L, FormWidget as W};
        let mut layout = LayoutForm::<usize>::new().flex(Flex::Legacy);
        layout.widget(state.list.id(), L::Str("List"), W::WideStretchXY(20, 4));
        layout.page_break();
        layout.widget(state.table.id(), L::Str("Table"), W::WideStretchXY(20, 4));
        layout.page_break();
        layout.widget(state.split.id(), L::Str("Split"), W::WideStretchXY(20, 4));
        layout.page_break();
        layout.widget(state.dialog.id(), L::Str("Dialog"), W::WideStretchXY(20, 4));
        form = form.layout(layout.build_paged(layout_size));
    }
    let mut form = form.into_buffer(l0[1], buf, &mut state.form);

    form.render(
        state.list.id(),
        || {
            List::new([
                "Backpacks: A portable bag with straps for carrying personal items, commonly used for school or travel.",
                "Books: Written or printed works consisting of pages bound together along one side, used for reading and learning.",
                "Bicycles: Human-powered vehicles with two wheels, used for transportation and recreation.",
                "Coffee Makers: Appliances designed to brew coffee from ground beans, commonly found in homes and offices.",
                "Smartphones: Portable devices combining a mobile phone with advanced computing capabilities, including internet access and apps.",
                "Gardens: Plots of land cultivated for growing plants, flowers, or vegetables, often for aesthetic or practical purposes.",
                "Music Boxes: Mechanical devices that play music through a rotating cylinder with pins, often used as decorative items.",
                "Pens: Writing instruments that dispense ink, used for writing or drawing.",
                "Laptops: Portable computers with integrated screen, keyboard, and battery, designed for mobile computing.",
                "Dogs: Domesticated mammals commonly kept as pets, known for loyalty and companionship."
            ])
                .scroll(Scroll::new())
                .styles(ctx.show_theme.style(WidgetStyle::LIST))
        },
        &mut state.list,
    );
    form.render(
        state.table.id(),
        || {
            Table::new_ratatui(
                [
                    Row::new([
                        Cell::new("1"),
                        Cell::new("67.9"),
                        Cell::new("Female"),
                        Cell::new("236.4"),
                        Cell::new("129.8"),
                        Cell::new("26.4"),
                        Cell::new("Yes"),
                        Cell::new("High"),
                    ]),
                    Row::new([
                        Cell::new("2"),
                        Cell::new("54.8"),
                        Cell::new("Female"),
                        Cell::new("256.3"),
                        Cell::new("133.4"),
                        Cell::new("28.4"),
                        Cell::new("No"),
                        Cell::new("Medium"),
                    ]),
                    Row::new([
                        Cell::new("3"),
                        Cell::new("68.4"),
                        Cell::new("Male"),
                        Cell::new("198.7"),
                        Cell::new("158.5"),
                        Cell::new("24.1"),
                        Cell::new("Yes"),
                        Cell::new("High"),
                    ]),
                    Row::new([
                        Cell::new("4"),
                        Cell::new("67.9"),
                        Cell::new("Male"),
                        Cell::new("205.0"),
                        Cell::new("136.0"),
                        Cell::new("19.9"),
                        Cell::new("No"),
                        Cell::new("Low"),
                    ]),
                    Row::new([
                        Cell::new("5"),
                        Cell::new("60.9"),
                        Cell::new("Male"),
                        Cell::new("207.7"),
                        Cell::new("145.4"),
                        Cell::new("26.7"),
                        Cell::new("No"),
                        Cell::new("Medium"),
                    ]),
                    Row::new([
                        Cell::new("6"),
                        Cell::new("44.9"),
                        Cell::new("Female"),
                        Cell::new("222.5"),
                        Cell::new("130.6"),
                        Cell::new("30.6"),
                        Cell::new("Noe"),
                        Cell::new("Low"),
                    ]),
                ],
                [
                    Constraint::Length(1),
                    Constraint::Length(4),
                    Constraint::Length(6),
                    Constraint::Length(11),
                    Constraint::Length(10),
                    Constraint::Length(5),
                    Constraint::Length(7),
                    Constraint::Length(9),
                ],
            )
            .scroll(Scroll::new())
            .column_spacing(1)
            .header(Row::new([
                Cell::new("#"),
                Cell::new("Age"),
                Cell::new("Gender"),
                Cell::new("Cholesterol"),
                Cell::new("SystolicBP"),
                Cell::new("BMI"),
                Cell::new("Smoking"),
                Cell::new("Education"),
            ]))
            .styles(ctx.show_theme.style(WidgetStyle::TABLE))
            .layout_column_widths()
        },
        &mut state.table,
    );
    let split_overlay = form.render2(
        state.split.id(),
        || {
            Split::new()
                .direction(Direction::Horizontal)
                .split_type(SplitType::FullPlain)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                ])
                .styles(ctx.show_theme.style(WidgetStyle::SPLIT))
                .into_widgets()
        },
        &mut state.split,
    );
    form.render_popup(state.split.id(), || split_overlay, &mut state.split);
    form.render(
        state.dialog.id(),
        || DialogSample::new(&ctx.show_theme),
        &mut state.dialog,
    );

    let (menu, menu_popup) = Menubar::new(&StaticMenu {
        menu: &[
            ("_File", &["_Open", "_Save", "\\___", "_Quit"]),
            ("_Help|F1", &["No Help"]),
        ],
    })
    .styles(ctx.show_theme.style(WidgetStyle::MENU))
    .popup_placement(Placement::Below)
    .into_widgets();
    menu.render(l0[0], buf, &mut state.menu);

    StatusLine::new()
        .layout([
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
        ])
        .styles_ext(ctx.show_theme.style(WidgetStyle::STATUSLINE))
        .render(l0[2], buf, &mut state.status);

    menu_popup.render(l0[0], buf, &mut state.menu);

    Ok(())
}

pub fn event(
    event: &crossterm::event::Event,
    state: &mut SampleOther,
    _ctx: &mut Global,
) -> Result<Outcome, Error> {
    event_flow!(state.menu.handle(event, Popup));

    event_flow!(state.list.handle(event, Regular));
    event_flow!(state.table.handle(event, Regular));
    event_flow!(state.split.handle(event, Regular));
    event_flow!(match state.dialog.handle(event, Dialog) {
        DialogOutcome::Unchanged => {
            // ignore this result!!
            DialogOutcome::Continue
        }
        r => r,
    });

    event_flow!(state.form.handle(event, Regular));

    Ok(Outcome::Continue)
}

pub struct DialogSample<'a> {
    theme: &'a SalsaTheme,
}

#[derive(Debug, Default)]
pub struct DialogSampleState {
    pub frame: DialogFrameState,
    pub input: TextInputState,
    pub container: FocusFlag,
}

impl<'a> DialogSample<'a> {
    pub fn new(theme: &'a SalsaTheme) -> Self {
        Self { theme }
    }
}

impl<'a> StatefulWidget for DialogSample<'a> {
    type State = DialogSampleState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        DialogFrame::new()
            .left(Constraint::Length(3))
            .right(Constraint::Length(3))
            .top(Constraint::Length(3))
            .bottom(Constraint::Length(3))
            .block(Block::bordered().title("Dialog"))
            .styles(self.theme.style(WidgetStyle::DIALOG_FRAME))
            .render(area, buf, &mut state.frame);

        let txt_area = Rect::new(
            state.frame.widget_area.x + 1,
            state.frame.widget_area.y + 1,
            min(state.frame.widget_area.width.saturating_sub(1), 20),
            1,
        );

        TextInput::new()
            .styles(self.theme.style(WidgetStyle::TEXT))
            .render(txt_area, buf, &mut state.input);
    }
}

impl HasFocus for DialogSampleState {
    fn build(&self, builder: &mut FocusBuilder) {
        let tag = builder.start(self);
        builder.widget(&self.input);
        builder.widget(&self.frame);
        builder.end(tag);
    }

    fn focus(&self) -> FocusFlag {
        self.container.focus()
    }

    fn area(&self) -> Rect {
        self.frame.area
    }
}

impl HasScreenCursor for DialogSampleState {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.input.screen_cursor()
    }
}

impl RelocatableState for DialogSampleState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.input.relocate(shift, clip);
        self.frame.relocate(shift, clip);
    }
}

impl HandleEvent<Event, Dialog, DialogOutcome> for DialogSampleState {
    fn handle(&mut self, event: &Event, _qualifier: Dialog) -> DialogOutcome {
        event_flow!(return Outcome::from(self.input.handle(event, Regular)));
        event_flow!(return self.frame.handle(event, Dialog));
        DialogOutcome::Continue
    }
}
