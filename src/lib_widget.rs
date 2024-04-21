use ratatui::layout::Rect;
use ratatui::Frame;

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
