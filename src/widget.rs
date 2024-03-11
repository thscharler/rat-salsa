use crate::ControlUI;
use ratatui::layout::Rect;
use ratatui::Frame;

/// This trait capture event-handling. It's intended to be implemented
/// on some ui-state struct. It returns a ControlUI state.
#[deprecated]
pub trait HandleEvent<Action, Err> {
    /// Event handling.
    fn handle(&mut self, evt: &crossterm::event::Event) -> ControlUI<Action, Err>;
}

/// Executes the actions defined by a widget.
///
/// A widget can define a set of standard actions to manipulate its state.
/// The event-handler maps input-events to actions, and executes them using [`Actionable::perform`]
pub trait Actionable<R> {
    /// Type of actions for a widget.
    type WidgetAction;

    /// Perform the given action.
    fn perform(&mut self, action: Self::WidgetAction) -> R;
}

/// Marker struct. Used by HandleCrossterm to differentiate between key-mappings.
#[derive(Debug)]
pub struct DefaultKeys;

/// Marker struct like [DefaultKeys]. This one selects an event-handler that processes only
/// mouse events. Useful when creating your own key-bindings but not wanting to touch
/// the mouse interactions.
#[derive(Debug)]
pub struct MouseOnly;

/// Handle events received by crossterm.
///
/// Implementations translate from input-events to widget-actions.
/// The actions are immediately executed by calling [Actionable::perform()] on self.
///
/// There is one extra type parameter K which is used to implement more than one event-handler
/// for the same widget. It's recommended to use [DefaultKeys] for the baseline implementation.
///
/// Users of a widget can easily define their own keymap for existing widgets this way.
/// [DefaultKeys] has no data of its own, but nothing prevents a user of the trait to provide
/// a method to create a configurable keymap.
///
/// Another recommendation is to split the event-handler between keyboard and mouse-events by
/// using [MouseOnly] for the latter. [DefaultKeys] forwards any unprocessed event to the `MouseOnly`
/// handler.
pub trait HandleCrossterm<R, K = DefaultKeys>
where
    Self: Actionable<R>,
{
    fn handle_crossterm(&mut self, event: &crossterm::event::Event, keymap: K) -> R;
}

/// Extra rendering which passes on the frame to a [FrameWidget].
/// This allows setting the cursor inside a widget.
pub trait RenderFrameWidget {
    fn render_frame_widget<W: FrameWidget>(&mut self, widget: W, area: Rect, state: &mut W::State);
}

impl<'a> RenderFrameWidget for Frame<'a> {
    fn render_frame_widget<W: FrameWidget>(&mut self, widget: W, area: Rect, state: &mut W::State) {
        widget.render(self, area, state)
    }
}

/// Another kind of widget that takes a frame instead of a buffer.
/// Allows to set the cursor while rendering.
///
/// This also always takes a state, just use () if not needed.
pub trait FrameWidget {
    /// Type of the corresponding state struct.
    type State: ?Sized;

    /// Do render.
    fn render(self, frame: &mut Frame<'_>, area: Rect, state: &mut Self::State);
}
