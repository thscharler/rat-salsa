use crate::Repaint;
use ratatui::layout::Rect;
use ratatui::Frame;

/// Execute an abstract input-action defined by a widget.
///
/// A widget can define a set of standard actions that manipulate its state. The event-handler
/// translates the concrete events to these actions and executes them calling [`Input::perform`].
/// This requires that both `Input` and `HandleCrossterm` are implemented for the widget-state.
/// As it's not obligatory to use this extra layer this is not enforced at type-level.
pub trait Input<R> {
    /// Type of action request for a widget.
    type Request;

    /// Perform the given action.
    fn perform(&mut self, req: Self::Request) -> R;
}

/// Marker struct. Used by [HandleCrossterm] to differentiate between key-mappings.
#[derive(Debug)]
pub struct DefaultKeys;

/// Marker struct like [DefaultKeys]. This one selects an event-handler that processes only
/// mouse events. Useful when creating your own key-bindings but not wanting to touch
/// the mouse interactions. If this separation exists for a widget it should be called
/// automatically by the [DefaultKeys] handler.
#[derive(Debug)]
pub struct MouseOnly;

/// Handle events received by crossterm.
///
/// This one should be implemented for the state struct of a widget and can do whatever. And it
/// can return whatever extra outcome is needed. Common usage would return a
/// [ControlUI](crate::ControlUI) flag as a result.
///
/// There is an extra parameter `KeyMap` which can be used to define more than one mapping for
/// a widget. This can be useful when overriding the default behaviour for a widget. Two
/// keymaps for common usage are defined in this library: [DefaultKeys] and [MouseOnly].
///
/// ```rust ignore
///     check_break!(uistate.page1.table1.handle(evt, DefaultKeys));
/// ```
///
/// _Remark_
///
/// There is only HandleCrossterm for now, as that is what I needed. But there is no problem
/// adding a HandleTermion, HandleTermwiz or whatever. One could add a second type parameter
/// for this differentiation, but I think that would complicate usage unnecessarily. And
/// any application will probably decide to use one or the other and not all of them.
/// A widget library can easily support all of them with this scheme without some added layer
/// of indirection and use a feature flag to select between them.
pub trait HandleCrossterm<R, KeyMap = DefaultKeys> {
    fn handle(&mut self, event: &crossterm::event::Event, keymap: KeyMap) -> R;
}

/// A specialized version of [HandleCrossterm] which needs [Repaint] to trigger an extra repaint.
/// Used by [Focus](crate::Focus) as it doesn't want to consume any events, but still needs a repaint.
pub trait HandleCrosstermRepaint<R, KeyMap = DefaultKeys> {
    fn handle_with_repaint(
        &mut self,
        event: &crossterm::event::Event,
        repaint: &Repaint,
        keymap: KeyMap,
    ) -> R;
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
