use crossterm::event::Event;
use rat_event::{Dialog, Popup};
use rat_ftable::event::TableOutcome;
use rat_ftable::{Table, TableState, TableStyle};
use rat_menu::MenuStyle;
use rat_menu::event::MenuOutcome;
use rat_menu::menubar::{Menubar, MenubarState};
use rat_menu::menuline::{MenuLine, MenuLineState};
use rat_menu::popup_menu::{PopupMenu, PopupMenuState};
use rat_text::TextStyle;
use rat_text::color_input::{ColorInput, ColorInputState, ColorInputStyle};
use rat_text::date_input::{DateInput, DateInputState};
use rat_text::event::TextOutcome;
use rat_text::line_number::{LineNumberState, LineNumberStyle, LineNumbers};
use rat_text::number_input::{NumberInput, NumberInputState};
use rat_text::text_area::{TextArea, TextAreaState};
use rat_text::text_input::{TextInput, TextInputState};
use rat_text::text_input_mask::{MaskedInput, MaskedInputState};
use rat_widget::button::{Button, ButtonState, ButtonStyle};
use rat_widget::calendar::selection::{NoSelection, RangeSelection, SingleSelection};
use rat_widget::calendar::{CalendarState, CalendarStyle, Month, MonthState};
use rat_widget::checkbox::{Checkbox, CheckboxState, CheckboxStyle};
use rat_widget::choice::{Choice, ChoiceState, ChoiceStyle};
use rat_widget::clipper::{Clipper, ClipperBuffer, ClipperState, ClipperStyle};
use rat_widget::combobox::{Combobox, ComboboxState, ComboboxStyle};
use rat_widget::dialog_frame::{DialogFrame, DialogFrameState, DialogFrameStyle, DialogOutcome};
use rat_widget::event::{
    ButtonOutcome, CalOutcome, CheckOutcome, ChoiceOutcome, ComboboxOutcome, FormOutcome,
    RadioOutcome, SliderOutcome, TabbedOutcome,
};
use rat_widget::event::{ConsumedEvent, HandleEvent, MouseOnly, Outcome, Regular};
use rat_widget::file_dialog::{FileDialog, FileDialogState, FileDialogStyle};
use rat_widget::focus::HasFocus;
use rat_widget::form::{Form, FormBuffer, FormState, FormStyle};
use rat_widget::list::{List, ListState, ListStyle};
use rat_widget::msgdialog::{MsgDialog, MsgDialogState, MsgDialogStyle};
use rat_widget::paragraph::{Paragraph, ParagraphState, ParagraphStyle};
use rat_widget::radio::{Radio, RadioState, RadioStyle};
use rat_widget::reloc::RelocatableState;
use rat_widget::shadow::{Shadow, ShadowStyle};
use rat_widget::slider::{Slider, SliderState, SliderStyle};
use rat_widget::splitter::{Split, SplitState, SplitStyle};
use rat_widget::statusline::{StatusLine, StatusLineState, StatusLineStyle};
use rat_widget::statusline_stacked::StatusLineStacked;
use rat_widget::tabbed::{Tabbed, TabbedState, TabbedStyle};
use rat_widget::text::HasScreenCursor;
use rat_widget::view::{View, ViewBuffer, ViewState, ViewStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::StatefulWidget;
use std::fmt::Debug;

macro_rules! conform_widget {
    (CORE: $widget:ty, $state:ty, $style:ty) => {{
        let v = <$widget>::default();
        fn debug(_: &impl Debug) {}
        debug(&v);
        fn clone(_: &impl Clone) {}
        clone(&v);

        _ = v;
    }};
    (BASE: $widget:ty, $state:ty, $style:ty) => {{
        let mut v = <$widget>::default();

        v = v.block(Block::default());
        v = v.border_style(Style::default());
        v = v.title_style(Style::default());
        v = v.styles(<$style>::default());
        v = v.style(Style::default());

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
    }};
    (DUAL_POPUP: $widget:ty, $state:ty, $style:ty) => {{
        let v = <$widget>::default();
        let (w1, w2) = v.into_widgets();

        fn widget(_: &impl StatefulWidget<State = $state>) {}
        widget(&w1);
        widget(&w2);
    }};
    (POPUP: $widget:ty, $state:ty, $style:ty) => {{
        let _ = <$widget>::width;
        let _ = <$widget>::height;
    }};
    (VALUE: $widget:ty, $state:ty, $style:ty) => {{
        let _ = <$widget>::width;
        let _ = <$widget>::height;
    }};
    (CONTAINER: $widget:ty, $state:ty, $style:ty) => {{
        let _ = <$widget>::layout_size;
    }};
    (MULTI_CONTAINER: $widget:ty, $state:ty, $style:ty) => {{}};
    (VIEW: $widget:ty, $state:ty, $style:ty) => {{
        let _ = <$widget>::into_buffer;
    }};
}

macro_rules! conform_state {
    (CORE: $state:ty, $event:ident, $outcome:ty) => {{
        let v = <$state>::default();

        fn debug(_: &impl Debug) {}
        debug(&v);
        fn clone(_: &impl Clone) {}
        clone(&v);
        fn focus(_: &impl HasFocus) {}
        focus(&v);

        let _ = <$state>::new;
    }};
    (BASE: $state:ty, $event:ident, $outcome:ty) => {{
        let mut v = <$state>::default();

        fn relocatable(_: &impl RelocatableState) {}
        relocatable(&v);
        fn screen_cursor(_: &impl HasScreenCursor) {}
        screen_cursor(&v);

        let _: $outcome = v.handle(&Event::FocusGained, MouseOnly);
        let r: $outcome = v.handle(&Event::FocusGained, $event);

        fn consumed(_: &impl ConsumedEvent) {}
        consumed(&r);
        fn outcome(_: &impl Into<Outcome>) {}
        outcome(&r);

        _ = <$state>::new();
        _ = <$state>::named("some");

        _ = v.area;
        _ = v.non_exhaustive;
    }};
    (DUAL_POPUP: $state:ty, $event:ident, $outcome:ty) => {{
        let v = <$state>::default();

        let _ = v.inner;
    }};
    (POPUP: $state:ty, $event:ident, $outcome:ty) => {{
        let v = <$state>::default();

        let _ = <$state>::is_active;
        let _ = <$state>::set_active;
        let _ = <$state>::flip_active;

        let _ = v.inner;
    }};
    (VALUE: $state:ty, $event:ident, $outcome:ty) => {{
        let mut v = <$state>::default();

        let val = v.value();
        _ = v.set_value(val);
        v.clear();

        let _ = v.inner;
    }};
    (FALLIBLE_VALUE: $state:ty, $event:ident, $outcome:ty) => {{
        let mut v = <$state>::default();

        let val = v.value().unwrap_or_default();
        _ = v.set_value(val);
        v.clear();

        let _ = v.inner;
    }};
    (INFERRED_VALUE: $state:ty, $value_type:ty, $event:ident, $outcome:ty) => {{
        let mut v = <$state>::default();

        let val: $value_type = v.value();
        _ = v.set_value(val);
        v.clear();

        let _ = v.inner;
    }};
    (INFERRED_FALLIBLE_VALUE: $state:ty, $value_type:ty, $event:ident, $outcome:ty) => {{
        let mut v = <$state>::default();

        let val: $value_type = v.value().unwrap_or_default();
        _ = v.set_value(val);
        v.clear();

        let _ = v.inner;
    }};
    (CONTAINER: $state:ty, $event:ident, $outcome:ty) => {{
        let v = <$state>::default();

        _ = v.widget_area;
    }};
    (MULTI_CONTAINER: $state:ty, $event:ident, $outcome:ty) => {{
        let v = <$state>::default();

        _ = v.inner;
        _ = v.widget_areas;
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
        let _: $outcome = tmod::handle_events(&mut v, true, &Event::FocusGained);
        let _: $outcome = tmod::handle_mouse_events(&mut v, &Event::FocusGained);
    }};
}

macro_rules! conform_style {
    ($style:ty) => {{
        let v = <$style>::default();
        _ = v.clone();
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
    conform_state!(DUAL_POPUP : ChoiceState, Popup, ChoiceOutcome);
    conform_state!(VALUE : ChoiceState, Popup, ChoiceOutcome);
    conform_event_fn!(rat_widget::choice : ChoiceState, ChoiceOutcome);
    conform_widget!(CORE : Choice, ChoiceState, ChoiceStyle);
    conform_widget!(DUAL_POPUP : Choice, ChoiceState, ChoiceStyle);
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

    // color-input
    conform_style!(TextStyle);
    conform_state!(CORE : ColorInputState, Regular, TextOutcome);
    conform_state!(BASE : ColorInputState, Regular, TextOutcome);
    conform_state!(VALUE : ColorInputState, Regular, ColorInputOutcome);
    conform_event_fn!(rat_widget::color_input : ColorInputState, TextOutcome);
    conform_widget!(CORE : ColorInput, ColorInputState, ColorInputStyle);
    conform_widget!(BASE : ColorInput, ColorInputState, ColorInputStyle);
    conform_widget!(VALUE : ColorInput, ColorInputState, ColorInputStyle);

    // choice
    conform_style!(ComboboxStyle);
    conform_state!(CORE : ComboboxState, Popup, ComboboxOutcome);
    conform_state!(DUAL_POPUP : ComboboxState, Popup, ComboboxOutcome);
    conform_state!(VALUE : ComboboxState, Popup, ComboboxOutcome);
    conform_event_fn!(rat_widget::combobox : ComboboxState, ComboboxOutcome);
    conform_widget!(CORE : Combobox, ComboboxState, ComboboxStyle);
    conform_widget!(DUAL_POPUP : Combobox, ComboboxState, ComboboxStyle);
    conform_widget!(VALUE : Combobox, ComboboxState, ComboboxStyle);

    // date-input
    conform_style!(TextStyle);
    conform_state!(CORE : DateInputState, Regular, TextOutcome);
    conform_state!(BASE : DateInputState, Regular, TextOutcome);
    conform_state!(FALLIBLE_VALUE : DateInputState, Regular, DateInputOutcome);
    conform_event_fn!(rat_widget::date_input : DateInputState, TextOutcome);
    conform_widget!(CORE : DateInput, DateInputState, TextStyle);
    conform_widget!(BASE : DateInput, DateInputState, TextStyle);
    conform_widget!(VALUE : DateInput, DateInputState, TextStyle);

    // dialog-frame
    conform_style!(DialogFrameStyle);
    conform_state!(CORE: DialogFrameState, Dialog, DialogFrameOutcome);
    conform_state!(BASE: DialogFrameState, Dialog, DialogOutcome);
    conform_state!(CONTAINER: DialogFrameState, Dialog, DialogOutcome);
    conform_event_fn!(rat_widget::dialog_frame : DialogFrameState, DialogOutcome);
    conform_widget!(CORE: DialogFrame, DialogFrameState, DialogFrameStyle);
    conform_widget!(BASE: DialogFrame, DialogFrameState, DialogFrameStyle);
    conform_widget!(CONTAINER: DialogFrame, DialogFrameState, DialogFrameStyle);

    // file-dialog
    // TODO: use DialogFrame
    conform_style!(FileDialogStyle);
    conform_state!(CORE: FileDialogState, Dialog, Result<FileOutcome, io::Error>);
    // no point in that: conform_state!(BASE: FileDialogState, Dialog, Result<FileOutcome, io::Error>);
    // no mouse: conform_event_fn!(rat_widget::file_dialog : FileDialogState, Result<FileOutcome, io::Error>);
    conform_widget!(CORE: FileDialog, FileDialogState, FileDialogStyle);
    conform_widget!(BASE: FileDialog, FileDialogState, FileDialogStyle);

    // form
    conform_style!(FormStyle);
    conform_state!(CORE : FormState, Popup, FormOutcome);
    conform_state!(CONTAINER : FormState, Popup, FormOutcome);
    conform_event_fn!(rat_widget::form : FormState, FormOutcome);
    conform_widget!(CORE : Form, FormState, FormStyle);
    conform_widget!(CONTAINER : Form, FormState, FormStyle);
    conform_widget!(VIEW: Form, FormState, FormStyle);
    conform_view_buffer!(VIEW : FormBuffer<usize>, FormState, FormStyle);

    // hover
    // not yet: conform_state!(CORE : HoverState, Popup, FormOutcome);
    // not yet: conform_widget!(CORE : Hover, FormState, HoverStyle);

    // line-number
    conform_style!(LineNumberStyle);
    conform_state!(CORE: LineNumberState, Regular, Outcome);
    conform_state!(BASE: LineNumberState, Regular, Outcome);
    // no point?: conform_event_fn!(rat_widget::line_number : LineNumberState, Outcome);
    conform_widget!(CORE: LineNumbers, LineNumberState, LineNumberStyle);
    // not enough: conform_widget!(BASE: LineNumbers, LineNumberState, ButtonStyle);

    // list
    conform_style!(ListStyle);
    conform_state!(CORE : ListState, Regular, Outcome);
    conform_state!(BASE : ListState, Regular, Outcome);
    conform_event_fn!(rat_widget::list : ListState, Outcome);
    conform_widget!(CORE : List, ListState, ListStyle);
    conform_widget!(BASE : List, ListState, ListStyle);

    // todo: all of menu
    // button
    conform_style!(MenuStyle);
    conform_state!(CORE: MenubarState, Popup, MenuOutcome);
    conform_state!(BASE: MenubarState, Popup, MenuOutcome);
    conform_event_fn!(rat_widget::menu::menubar : MenubarState, MenuOutcome);
    conform_widget!(CORE: Menubar, MenubarState, MenuStyle);
    conform_widget!(DUAL_POPUP: Menubar, MenubarState, MenuStyle);

    conform_state!(CORE: MenuLineState, Regular, MenuOutcome);
    conform_state!(BASE: MenuLineState, Regular, MenuOutcome);
    conform_event_fn!(rat_widget::menu::menuline : MenuLineState, MenuOutcome);
    conform_widget!(CORE: MenuLine, MenuLineState, MenuStyle);
    conform_widget!(BASE: MenuLine, MenuLineState, MenuStyle);

    conform_state!(CORE: PopupMenuState, Popup, MenuOutcome);
    conform_state!(BASE: PopupMenuState, Popup, MenuOutcome);
    conform_event_fn!(rat_widget::menu::popup_menu : PopupMenuState, MenuOutcome);
    conform_widget!(CORE: PopupMenu, PopupMenuState, MenuStyle);
    conform_widget!(POPUP: PopupMenu, PopupMenuState, MenuStyle);

    // msgdialog
    // TODO: use DialogFrame
    conform_style!(MsgDialogStyle);
    conform_state!(CORE: MsgDialogState, Dialog, Result<FileOutcome, io::Error>);
    // no point in that: conform_state!(BASE: MsgDialogState, Dialog, Result<FileOutcome, io::Error>);
    // no mouse: conform_event_fn!(rat_widget::file_dialog : MsgDialogState, Result<FileOutcome, io::Error>);
    conform_widget!(CORE: MsgDialog, MsgDialogState, MsgDialogStyle);
    conform_widget!(BASE: MsgDialog, MsgDialogState, MsgDialogStyle);

    // number-input
    conform_style!(TextStyle);
    conform_state!(CORE : NumberInputState, Regular, TextOutcome);
    conform_state!(BASE : NumberInputState, Regular, TextOutcome);
    conform_state!(INFERRED_FALLIBLE_VALUE: NumberInputState, i32, Regular, NumberInputOutcome);
    conform_event_fn!(rat_widget::number_input : NumberInputState, TextOutcome);
    conform_widget!(CORE : NumberInput, NumberInputState, TextStyle);
    conform_widget!(BASE : NumberInput, NumberInputState, TextStyle);
    conform_widget!(VALUE : NumberInput, NumberInputState, TextStyle);

    // paired - more a utility than a widget ...

    // paragraph
    conform_style!(ParagraphStyle);
    conform_state!(CORE : ParagraphState, Regular, Outcome);
    conform_state!(BASE : ParagraphState, Regular, Outcome);
    conform_event_fn!(rat_widget::paragraph : ParagraphState, Outcome);
    conform_widget!(CORE : Paragraph, ParagraphState, ParagraphStyle);
    conform_widget!(BASE : Paragraph, ParagraphState, ParagraphStyle);

    // radio
    conform_style!(RadioStyle);
    conform_state!(CORE : RadioState, Regular, RadioOutcome);
    conform_state!(BASE : RadioState, Regular, RadioOutcome);
    conform_state!(VALUE : RadioState, Regular, RadioOutcome);
    conform_event_fn!(rat_widget::radio : RadioState, RadioOutcome);
    conform_widget!(CORE : Radio, RadioState, RadioStyle);
    conform_widget!(BASE : Radio, RadioState, RadioStyle);
    conform_widget!(VALUE : Radio, RadioState, RadioStyle);

    // shadow
    conform_style!(ShadowStyle);
    // conform_state!(CORE: (), Regular, Outcome);
    // conform_event_fn!(rat_widget::shadow : (), Outcome);
    conform_widget!(CORE: Shadow, (), ShadowStyle);

    // slider
    conform_style!(SliderStyle);
    conform_state!(CORE : SliderState, Regular, SliderOutcome);
    conform_state!(BASE : SliderState, Regular, SliderOutcome);
    conform_state!(VALUE : SliderState, Regular, SliderOutcome);
    conform_event_fn!(rat_widget::slider : SliderState, SliderOutcome);
    conform_widget!(CORE : Slider, SliderState, SliderStyle);
    conform_widget!(BASE : Slider, SliderState, SliderStyle);
    conform_widget!(VALUE : Slider, SliderState, SliderStyle);

    // splitter
    conform_style!(SplitStyle);
    conform_state!(CORE : SplitState, Popup, SplitterOutcome);
    conform_state!(MULTI_CONTAINER : SplitState, Popup, SplitterOutcome);
    conform_event_fn!(rat_widget::splitter : SplitState, Outcome);
    conform_widget!(CORE : Split, SplitterState, SplitStyle);
    conform_widget!(MULTI_CONTAINER : Split, SplitterState, SplitterStyle);

    // status-line
    conform_style!(StatusLineStyle);
    conform_state!(CORE: StatusLineState, Regular, Outcome);
    conform_state!(BASE: StatusLineState, Regular, Outcome);
    // no point: conform_event_fn!(rat_widget::statusline : StatusLineState, Outcome);
    conform_widget!(CORE: StatusLine, StatusLineState, StatusLineStyle);
    // no block, no regular style : conform_widget!(BASE: StatusLine, StatusLineState, StatusLineStyle);

    // status-line stacked
    conform_widget!(CORE: StatusLineStacked, (), ());

    // tabbed
    conform_style!(TabbedStyle);
    conform_state!(CORE : TabbedState, Popup, TabbedOutcome);
    conform_state!(CONTAINER : TabbedState, Popup, TabbedOutcome);
    conform_event_fn!(rat_widget::tabbed : TabbedState, TabbedOutcome);
    conform_widget!(CORE : Tabbed, TabbedState, TabbedStyle);
    conform_widget!(MULTI_CONTAINER : Tabbed, TabbedState, TabbedterStyle);

    // table
    conform_style!(TableStyle);
    conform_state!(CORE : TableState, Regular, TableOutcome);
    conform_state!(BASE : TableState, Regular, TableOutcome);
    // exist, can't check this way: conform_event_fn!(rat_widget::table : TableState, TableOutcome);
    // can't clone, rest ok: conform_widget!(CORE : Table, TableState, TableStyle);
    conform_widget!(BASE : Table, TableState, TableStyle);

    // text-input
    conform_style!(TextStyle);
    conform_state!(CORE : TextInputState, Regular, TextOutcome);
    conform_state!(BASE : TextInputState, Regular, TextOutcome);
    conform_state!(INFERRED_VALUE: TextInputState, String, Regular, TextInputOutcome);
    conform_event_fn!(rat_widget::text_input : TextInputState, TextOutcome);
    conform_widget!(CORE : TextInput, TextInputState, TextStyle);
    conform_widget!(BASE : TextInput, TextInputState, TextStyle);
    conform_widget!(VALUE : TextInput, TextInputState, TextStyle);

    // masked-input
    conform_style!(TextStyle);
    conform_state!(CORE : MaskedInputState, Regular, TextOutcome);
    conform_state!(BASE : MaskedInputState, Regular, TextOutcome);
    conform_state!(INFERRED_FALLIBLE_VALUE: MaskedInputState, String, Regular, MaskedInputOutcome);
    conform_event_fn!(rat_widget::text_input_mask : MaskedInputState, TextOutcome);
    conform_widget!(CORE : MaskedInput, MaskedInputState, TextStyle);
    conform_widget!(BASE : MaskedInput, MaskedInputState, TextStyle);
    conform_widget!(VALUE : MaskedInput, MaskedInputState, TextStyle);

    // text_area
    conform_style!(TextStyle);
    conform_state!(CORE : TextAreaState, Regular, TextOutcome);
    conform_state!(BASE : TextAreaState, Regular, TextOutcome);
    conform_state!(INFERRED_VALUE: TextAreaState, String, Regular, TextAreaOutcome);
    conform_event_fn!(rat_widget::textarea : TextAreaState, TextOutcome);
    conform_widget!(CORE : TextArea, TextAreaState, TextStyle);
    conform_widget!(BASE : TextArea, TextAreaState, TextStyle);
    conform_widget!(VALUE : TextArea, TextAreaState, TextStyle);

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
