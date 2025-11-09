use rat_widget::button::{Button, ButtonState, ButtonStyle};
use rat_widget::calendar::selection::{NoSelection, RangeSelection, SingleSelection};
use rat_widget::calendar::{CalendarState, CalendarStyle, Month, MonthState};
use rat_widget::event::{ButtonOutcome, CalOutcome};
use rat_widget::event::{ConsumedEvent, HandleEvent, MouseOnly, Outcome, Regular};
use rat_widget::focus::HasFocus;
use rat_widget::reloc::RelocatableState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::StatefulWidget;

macro_rules! conform_widget {
    ($widget:ty,
        $state:ty,
        $styles:ty) => {{
        let mut v = <$widget>::default();
        print!("{:?}", v);
        _ = v.clone();

        fn stateful_widget(_: &impl StatefulWidget<State = $state>) {}
        stateful_widget(&v);

        v = v.styles(<$styles>::default());
        _ = v.style(Style::default());

        let mut state = <$state>::default();
        let mut buf = Buffer::default();
        <$widget>::default().render(Rect::new(0, 0, 0, 0), &mut buf, &mut state);
        assert_eq!(state.area, Rect::new(0, 0, 0, 0));

        let mut state = <$state>::default();
        let mut buf = Buffer::empty(Rect::new(5, 5, 15, 15));
        <$widget>::default().render(Rect::new(5, 5, 15, 15), &mut buf, &mut state);
        assert_eq!(state.area, Rect::new(5, 5, 15, 15));
        assert_ne!(state.widget_area, Rect::new(0, 0, 0, 0));

        // todo: width/height???
        // Button
        // width+height
    }};
}

macro_rules! conform_style {
    ($style:ident) => {{
        let v = <$style>::default();
        _ = v.non_exhaustive;
    }};
}

macro_rules! conform_state {
    ($state:ty, $event:ident, $outcome:ty) => {{
        let mut v = <$state>::default();
        print!("{:?}", v);
        _ = v.clone();
        fn focus(_: &impl HasFocus) {}
        focus(&v);
        fn relocatable(_: &impl RelocatableState) {}
        relocatable(&v);

        let event = crossterm::event::Event::FocusGained;
        let _: $outcome = v.handle(&event, MouseOnly);
        let r: $outcome = v.handle(&event, $event);
        r.is_consumed();
        let _: Outcome = r.into();

        _ = <$state>::new();
        _ = <$state>::named("some");

        _ = v.area;
        _ = v.widget_area;
        _ = v.non_exhaustive;

        let _: Option<(u16, u16)> = v.screen_cursor();
    }};
}

macro_rules! conform_state_value {
    ($state:ty, $event:ident, $outcome:ty) => {{
        let mut v = <$state>::default();

        v.clear();
    }};
}

#[test]
fn test_conform() {
    conform_style!(ButtonStyle);
    conform_state!(ButtonState, Regular, ButtonOutcome);
    conform_widget!(Button, ButtonState, ButtonStyle);

    conform_state ! (CalendarState < 3, SingleSelection >, Regular, CalOutcome);
    conform_state ! (CalendarState <3, RangeSelection >, Regular, CalOutcome);
    conform_state ! (CalendarState <3, NoSelection >, Regular, CalOutcome);

    conform_style!(CalendarStyle);
    conform_state!(MonthState, Regular, CalOutcome);
    conform_widget!(
        Month<SingleSelection>,
        MonthState<SingleSelection>,
        CalendarStyle
    );
    conform_widget!(
        Month<RangeSelection>,
        MonthState<RangeSelection>,
        CalendarStyle
    );
    conform_widget!(Month<NoSelection>, MonthState<NoSelection>, CalendarStyle);
}
