use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::widgets::StatefulWidget;

/// Render a hover.
///
/// The state contains a buffer which can be used to render the hover
/// wherever needed.
///
/// At the very end of rendering the Hover widget itself is rendered
/// and uses the stored buffer.
///
/// **unstable**
#[derive(Debug, Default)]
pub struct Hover {}

#[derive(Debug, Default)]
pub struct HoverState {
    pub hover_buf: Option<Buffer>,
}

impl Hover {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StatefulWidget for Hover {
    type State = HoverState;

    fn render(self, _area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(hover_buf) = state.hover_buf.take() {
            let buf_area = hover_buf.area;
            for y in 0..buf_area.height {
                for x in 0..buf_area.width {
                    let pos = Position::new(buf_area.x + x, buf_area.y + y);
                    let src_cell = hover_buf.cell(pos).expect("src-cell");
                    let tgt_cell = buf.cell_mut(pos).expect("tgt_cell");
                    *tgt_cell = src_cell.clone();
                }
            }
        }
    }
}

impl HoverState {
    pub fn new() -> Self {
        Self { hover_buf: None }
    }

    pub fn buffer_mut(&mut self, area: Rect) -> &mut Buffer {
        self.hover_buf = match self.hover_buf.take() {
            None => Some(Buffer::empty(area)),
            Some(mut buf) => {
                buf.resize(area);
                Some(buf)
            }
        };
        self.hover_buf.as_mut().expect("buffer")
    }
}
