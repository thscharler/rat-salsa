use rat_focus::FocusFlag;
use rat_widget::button::{Button, ButtonState, ButtonStyle};
use rat_widget::calendar::selection::{NoSelection, RangeSelection, SingleSelection};
use rat_widget::calendar::{CalendarState, CalendarStyle, Month, MonthState};
use rat_widget::checkbox::{Checkbox, CheckboxState, CheckboxStyle};
use rat_widget::choice::{Choice, ChoiceState, ChoiceStyle};
use rat_widget::clipper::{Clipper, ClipperBuffer, ClipperState, ClipperStyle};
use rat_widget::event::{ButtonOutcome, CalOutcome, CheckOutcome, ChoiceOutcome, FormOutcome};
use rat_widget::event::{ConsumedEvent, HandleEvent, MouseOnly, Outcome, Popup, Regular};
use rat_widget::focus::HasFocus;
use rat_widget::form::{Form, FormBuffer, FormState, FormStyle};
use rat_widget::reloc::RelocatableState;
use rat_widget::scrolled::Scroll;
use rat_widget::text::HasScreenCursor;
use rat_widget::view::{View, ViewBuffer, ViewState, ViewStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget;
use std::borrow::Cow;

macro_rules! conform_widget {
    (CORE: $widget:ty, $state:ty, $style:ty) => {{
        let mut v = <$widget>::default();
        print!("{:?}", v);
        _ = v.clone();

        v = v.styles(<$style>::default());
        v = v.style(Style::default());
        v = v.block(Block::default());
        _ = v;
    }};
    (BASE: $widget:ty, $state:ty, $style:ty) => {{
        let v = <$widget>::default();

        fn stateful_widget(_: &impl StatefulWidget<State = $state>) {}
        stateful_widget(&v);

        let mut state = <$state>::default();
        let mut buf = Buffer::default();
        <$widget>::default().render(Rect::new(0, 0, 0, 0), &mut buf, &mut state);
        assert_eq!(state.area, Rect::new(0, 0, 0, 0));

        let mut state = <$state>::default();
        let mut buf = Buffer::empty(Rect::new(5, 5, 15, 15));
        <$widget>::default().render(Rect::new(5, 5, 15, 15), &mut buf, &mut state);
        assert_eq!(state.area, Rect::new(5, 5, 15, 15));
        assert_ne!(state.area, Rect::new(0, 0, 0, 0));
    }};
    (POPUP: $widget:ty, $state:ty, $style:ty) => {{
        let v = <$widget>::default();
        let (w1, w2) = v.into_widgets();

        fn widget(_: &impl StatefulWidget<State = $state>) {}
        widget(&w1);
        widget(&w2);
    }};
    (VALUE: $widget:ty, $state:ty, $style:ty) => {{
        let v = <$widget>::default();
        let _: u16 = v.width();
        let _: u16 = v.height();
    }};
    (CONTAINER: $widget:ty, $state:ty, $style:ty) => {{
        let _ = <$widget>::layout_size;
    }};
    (VIEW: $widget:ty, $state:ty, $style:ty) => {{
        let _ = <$widget>::into_buffer;
    }};
}

macro_rules! conform_state {
    (CORE: $state:ty, $event:ident, $outcome:ty) => {{
        let _ = <$state>::default();
        let _ = <$state>::new;
    }};
    (BASE: $state:ty, $event:ident, $outcome:ty) => {{
        let mut v = <$state>::default();
        print!("{:?}", v);
        _ = v.clone();
        fn focus(_: &impl HasFocus) {}
        focus(&v);
        fn relocatable(_: &impl RelocatableState) {}
        relocatable(&v);
        fn screen_cursor(_: &impl HasScreenCursor) {}
        screen_cursor(&v);

        let event = crossterm::event::Event::FocusGained;
        let _: $outcome = v.handle(&event, MouseOnly);
        let r: $outcome = v.handle(&event, $event);
        r.is_consumed();
        let _: Outcome = r.into();

        _ = <$state>::new();
        _ = <$state>::named("some");

        _ = v.area;
        _ = v.non_exhaustive;
    }};
    (POPUP: $state:ty, $event:ident, $outcome:ty) => {{
        let v = <$state>::default();

        let _ = v.inner;
        let _: FocusFlag = v.focus;
    }};
    (VALUE: $state:ty, $event:ident, $outcome:ty) => {{
        let mut v = <$state>::default();
        let val = v.value();
        v.set_value(val);
        v.clear();

        let _ = v.inner;
        let _: FocusFlag = v.focus;
    }};
    (CONTAINER: $state:ty, $event:ident, $outcome:ty) => {{
        let v = <$state>::default();
        _ = v.widget_area;
    }};
    (VIEW: $state:ty, $event:ident, $outcome:ty) => {{}};
}

macro_rules! conform_view_buffer {
    (VIEW: $widget:ty, $state:ty, $style:ty) => {{
        let _ = <$widget>::buffer;
    }};
}

macro_rules! conform_event_fn {
    ($module:path : $state:ty, $outcome:ty) => {{
        use $module as tmod;

        let mut v = <$state>::default();
        let _: $outcome = tmod::handle_events(&mut v, true, &crossterm::event::Event::FocusGained);
        let _: $outcome = tmod::handle_mouse_events(&mut v, &crossterm::event::Event::FocusGained);
    }};
}

macro_rules! conform_style {
    ($style:ty) => {{
        let v = <$style>::default();
        _ = v.non_exhaustive;
    }};
}

#[test]
fn conform() {
    // button
    conform_style!(ButtonStyle);
    conform_state!(CORE: ButtonState, Regular, ButtonOutcome);
    conform_state!(BASE: ButtonState, Regular, ButtonOutcome);
    conform_event_fn!(rat_widget::button : ButtonState, ButtonOutcome);
    conform_widget!(CORE: Button, ButtonState, ButtonStyle);
    conform_widget!(BASE: Button, ButtonState, ButtonStyle);

    // calendar
    conform_style!(CalendarStyle);
    conform_state!(BASE: CalendarState <3, SingleSelection>, Regular, CalOutcome);
    conform_widget!(BASE: Month<SingleSelection>, MonthState<SingleSelection>, CalendarStyle);
    conform_state!(BASE: MonthState<SingleSelection>, Regular, CalOutcome);

    conform_state!(BASE: CalendarState <3, RangeSelection>, Regular, CalOutcome);
    conform_widget!(BASE: Month<RangeSelection>, MonthState<RangeSelection>, CalendarStyle);
    conform_state!(BASE: MonthState<RangeSelection>, Regular, CalOutcome);

    conform_state!(BASE: CalendarState <3, NoSelection>, Regular, CalOutcome);
    conform_widget!(BASE: Month<NoSelection>, MonthState<NoSelection>, CalendarStyle);
    conform_state!(BASE: MonthState<NoSelection>, Regular, CalOutcome);

    // checkbox
    conform_style!(CheckboxStyle);
    conform_state!(CORE : CheckboxState, Regular, CheckOutcome);
    conform_state!(BASE : CheckboxState, Regular, CheckOutcome);
    conform_state!(VALUE : CheckboxState, Regular, CheckOutcome);
    conform_event_fn!(rat_widget::checkbox : CheckboxState, CheckOutcome);
    conform_widget!(CORE : Checkbox, CheckboxState, CheckboxStyle);
    conform_widget!(BASE : Checkbox, CheckboxState, CheckboxStyle);
    conform_widget!(VALUE : Checkbox, CheckboxState, CheckboxStyle);

    // choice
    conform_style!(ChoiceStyle);
    conform_state!(CORE : ChoiceState, Popup, ChoiceOutcome);
    conform_state!(POPUP : ChoiceState, Popup, ChoiceOutcome);
    conform_state!(VALUE : ChoiceState, Popup, ChoiceOutcome);
    conform_event_fn!(rat_widget::choice : ChoiceState, ChoiceOutcome);
    conform_widget!(CORE : Choice, ChoiceState, ChoiceStyle);
    conform_widget!(POPUP : Choice, ChoiceState, ChoiceStyle);
    conform_widget!(VALUE : Choice, ChoiceState, ChoiceStyle);

    // clipper
    conform_style!(ClipperStyle);
    conform_state!(CORE : ClipperState, Popup, ClipperOutcome);
    conform_state!(CONTAINER : ClipperState, Popup, ClipperOutcome);
    conform_event_fn!(rat_widget::clipper : ClipperState, Outcome);
    conform_widget!(CORE : Clipper, ClipperState, ClipperStyle);
    conform_widget!(CONTAINER : Clipper, ClipperState, ClipperStyle);
    conform_widget!(VIEW: Clipper, ClipperState, ClipperStyle);
    conform_view_buffer!(VIEW : ClipperBuffer<usize>, ClipperState, ClipperStyle);

    // form
    conform_style!(FormStyle);
    conform_state!(CORE : FormState, Popup, FormOutcome);
    conform_state!(CONTAINER : FormState, Popup, FormOutcome);
    conform_event_fn!(rat_widget::form : FormState, FormOutcome);
    conform_widget!(CORE : Form, FormState, FormStyle);
    conform_widget!(CONTAINER : Form, FormState, FormStyle);
    conform_widget!(VIEW: Form, FormState, FormStyle);
    conform_view_buffer!(VIEW : FormBuffer<usize>, FormState, FormStyle);

    // view
    conform_style!(ViewStyle);
    conform_state!(CORE : ViewState, Popup, ViewOutcome);
    conform_state!(CONTAINER : ViewState, Popup, ViewOutcome);
    conform_event_fn!(rat_widget::view : ViewState, Outcome);
    conform_widget!(CORE : View, ViewState, ViewStyle);
    conform_widget!(CONTAINER : View, ViewState, ViewStyle);
    conform_widget!(VIEW: View, ViewState, ViewStyle);
    conform_view_buffer!(VIEW : ViewBuffer, ViewState, ViewStyle);
}
