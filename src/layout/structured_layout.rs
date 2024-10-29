use crate::pager::AreaHandle;
use ratatui::layout::{Position, Rect};
use std::cell::Cell;
use std::ops::{Index, IndexMut};

/// Container for the areas coming out of a layout function.
///
/// This is more or less a `Vec<Rect>`, but it takes a _stride_
/// as parameter and treats N Rects as one unit.
///
/// This way it can add some structure to the list and
/// express something like 'the label area for the 5th item'.
///
/// As a second feature it returns a handle for each item,
/// which can be used to retrieve the item later.
///
/// ```rust
/// # use rat_widget::layout::StructuredLayout;
/// # use ratatui::buffer::Buffer;
/// # use ratatui::layout::Rect;
/// # use ratatui::text::Span;
/// # use ratatui::widgets::Widget;
/// # use std::ops::Index;
/// pub enum LW {
///     Label,
///     Widget
/// }
/// #    impl Index<[Rect;2]> for LW {
/// #        type Output = Rect;
/// #
/// #        fn index(&self, index: [Rect; 2]) -> &Self::Output {
/// #            match self {
/// #                LW::Label => &index[0],
/// #                LW::Widget => &index[1]
/// #            }
/// #        }
/// #    }
/// #
/// #    impl LW {
/// #        pub fn count() -> usize {
/// #            2
/// #        }
/// #    }
/// #
/// # let mut buf = Buffer::default();
/// # let buf = &mut buf;
/// #
/// # use LW::*;
/// # use rat_widget::checkbox::Checkbox;
///
/// let mut l = StructuredLayout::new(LW::count());
///
/// // ... some layout calculations ...
/// let w0 = l.add(&[
///         Rect::new(0,0,5,1),
///         Rect::new(6,0,15,1)
/// ]);
///
/// // ... something entirely else ...
///
/// Span::from("label")
///     .render(l[w0][Label], buf);
///
/// Checkbox::new()
///     .text("Check this out")
///     .render(l[w0][Widget], buf);
///
/// ```
///
#[derive(Debug, Clone)]
pub struct StructuredLayout {
    // reference area.
    area: Rect,
    // bounding box for all areas
    bounds: Cell<Option<Rect>>,
    // stride within areas
    stride: usize,
    // list of areas
    areas: Vec<Rect>,
    // manual breaks
    row_breaks: Vec<u16>,
}

impl Default for StructuredLayout {
    fn default() -> Self {
        Self {
            area: Default::default(),
            bounds: Cell::new(None),
            stride: 1, // non standard
            areas: vec![],
            row_breaks: vec![],
        }
    }
}

impl StructuredLayout {
    /// New layout with the given stride.
    pub fn new(stride: usize) -> Self {
        Self {
            stride,
            ..Default::default()
        }
    }

    /// Original area for which this layout has been calculated.
    /// Can be used to invalidate a layout if the area changes.
    pub fn area(&self) -> Rect {
        self.area
    }

    /// Original area for which this layout has been calculated.
    /// Can be used to invalidate a layout if the area changes.
    pub fn set_area(&mut self, area: Rect) {
        self.area = area;
    }

    /// Change detection.
    pub fn width_change(&self, width: u16) -> bool {
        self.area.width != width
    }

    /// Change detection.
    pub fn height_change(&self, height: u16) -> bool {
        self.area.height != height
    }

    /// Change detection.
    pub fn pos_change(&self, pos: Position) -> bool {
        self.area.as_position() != pos
    }

    /// Add the areas for one item.
    ///
    /// You can refer
    /// Returns a handle to access the item later.
    /// You can always use a simple index too.
    pub fn add(&mut self, areay: &[Rect]) -> AreaHandle {
        assert_eq!(self.stride, areay.len());

        // invalidate
        self.bounds.set(None);

        let h = AreaHandle(self.areas.len());

        for a in areay {
            self.areas.push(*a);
        }

        h
    }

    /// Add a manual break after the given position.
    ///
    /// __See__
    /// [SinglePager](crate::pager::SinglePager) and
    /// [DualPager](crate::pager::DualPager) who can work with this.
    /// Other widgets may simply ignore this.
    pub fn break_after_row(&mut self, y: u16) {
        // invalidate
        self.bounds.set(None);
        self.row_breaks.push(y + 1);
    }

    /// Add a manual break before the given position.
    ///
    /// __See__
    /// [SinglePager](crate::pager::SinglePager) and
    /// [DualPager](crate::pager::DualPager) who can work with this.
    /// Other widgets may simply ignore this.
    pub fn break_before_row(&mut self, y: u16) {
        // invalidate
        self.bounds.set(None);
        self.row_breaks.push(y);
    }

    /// Return the manual page breaks.
    pub fn row_breaks(&self) -> &[u16] {
        self.row_breaks.as_slice()
    }

    /// Return the manual page breaks.
    pub fn row_breaks_mut(&mut self) -> &mut [u16] {
        self.row_breaks.as_mut_slice()
    }

    /// Sort and dedup the row-breaks.
    pub fn sort_row_breaks_desc(&mut self) {
        self.row_breaks.sort_by(|a, b| b.cmp(a));
        self.row_breaks.dedup();
    }

    /// Number of areas.
    pub fn len(&self) -> usize {
        self.areas.len()
    }

    /// Any areas?
    pub fn is_empty(&self) -> bool {
        self.areas.is_empty()
    }

    /// Stride per item.
    pub fn stride(&self) -> usize {
        self.stride
    }

    /// All areas. If you want to access a specific item you
    /// must use the stride to calculate the offset.
    pub fn as_slice(&self) -> &[Rect] {
        self.areas.as_slice()
    }

    /// All areas. If you want to access a specific item you
    /// must use the stride to calculate the offset.
    pub fn as_mut_slice(&mut self) -> &mut [Rect] {
        self.areas.as_mut_slice()
    }

    /// Iterator over all areas.
    pub fn iter(&self) -> impl Iterator<Item = &'_ Rect> {
        self.areas.iter()
    }

    /// Iterator over all areas chunked by stride.
    pub fn chunked(&self) -> impl Iterator<Item = &[Rect]> {
        self.areas.chunks(self.stride)
    }

    /// Calculate the bounding box for all areas.
    pub fn bounds(&self) -> Rect {
        if let Some(bounds) = self.bounds.get() {
            return bounds;
        }

        let mut bounds;
        'fine: {
            for v in &self.areas {
                if !v.is_empty() {
                    bounds = *v;
                    break 'fine;
                }
            }
            bounds = Rect::new(self.area.x, self.area.y, 0, 0);
        }

        for v in &self.areas {
            if !v.is_empty() {
                bounds = bounds.union(*v);
            }
        }

        self.bounds.set(Some(bounds));

        bounds
    }
}

impl Index<usize> for StructuredLayout {
    type Output = [Rect];

    fn index(&self, index: usize) -> &Self::Output {
        &self.areas[index * self.stride..(index + 1) * self.stride]
    }
}

impl IndexMut<usize> for StructuredLayout {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.areas[index * self.stride..(index + 1) * self.stride]
    }
}

impl Index<AreaHandle> for StructuredLayout {
    type Output = [Rect];

    fn index(&self, index: AreaHandle) -> &Self::Output {
        &self.areas[index.0 * self.stride..(index.0 + 1) * self.stride]
    }
}

impl IndexMut<AreaHandle> for StructuredLayout {
    fn index_mut(&mut self, index: AreaHandle) -> &mut Self::Output {
        &mut self.areas[index.0 * self.stride..(index.0 + 1) * self.stride]
    }
}
