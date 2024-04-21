use ratatui::layout::Rect;
use ratatui::Frame;

/// Execute an abstract input-action defined by a widget.
///
/// A widget can define a set of standard actions that manipulate its state. The event-handler
/// translates the concrete events to these actions and executes them calling [`Input::perform`].
/// This requires that both `Input` and `HandleCrossterm` are implemented for the widget-state.
/// As it's not obligatory to use this extra layer this is not enforced at type-level.
// todo: remove this
pub trait Input<R> {
    /// Type of action request for a widget.
    type Request;

    /// Perform the given action.
    fn perform(&mut self, req: Self::Request) -> R;
}

/// Add-on to ratatui.
///
/// Adds a `render_frame_widget()` to Frame for use with [FrameWidget].
pub trait RenderFrameWidget {
    fn render_frame_widget<W: FrameWidget>(&mut self, widget: W, area: Rect, state: &mut W::State);
}

impl<'a> RenderFrameWidget for Frame<'a> {
    fn render_frame_widget<W: FrameWidget>(&mut self, widget: W, area: Rect, state: &mut W::State) {
        widget.render(self, area, state)
    }
}

/// Add-on to ratatui.
///
/// Another kind of widget that takes a frame instead of a buffer.
/// This allows the widget to call set_cursor(), that's all.
pub trait FrameWidget {
    /// Type of the corresponding state struct.
    type State: ?Sized;

    /// Do render.
    fn render(self, frame: &mut Frame<'_>, area: Rect, state: &mut Self::State);
}
