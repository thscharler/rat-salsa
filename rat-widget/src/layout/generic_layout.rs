use ratatui_core::layout::{Position, Rect, Size};
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::Hash;
use ratatui_widgets::block::Block;

/// Stores layout data resulting from some layout algorithm.
///
/// Widgets and labels are stored for some key that identifies
/// the widget. It is also possible to store the label text.
///
/// [Block]s can be added too. It is expected that blocks
/// will be rendered in order of addition.
///
/// There is a concept for pages too. The page-height defines
/// the pages. The page-width is not used to constrain
/// the pages and is just informational. It can be used
/// to find if the layout has to be rebuilt after a resize.
///
/// The page-count is available too, but there may be
/// areas that map beyond the page-count.
///
/// __See__
/// [LayoutForm](crate::layout::LayoutForm)
/// [layout_edit](crate::layout::layout_edit())
///
#[derive(Debug, Clone)]
pub struct GenericLayout<W>
where
    W: Eq + Hash + Clone,
{
    /// Area of the layout.
    area: Rect,
    /// Page size.
    page_size: Size,
    /// Pages.
    page_count: usize,

    /// Widget keys.
    widgets: HashMap<W, usize>,
    rwidgets: HashMap<usize, W>,
    /// Widget areas.
    widget_areas: Vec<Rect>,
    /// Widget labels.
    labels: Vec<Option<Cow<'static, str>>>,
    /// Label areas.
    label_areas: Vec<Rect>,

    /// Container areas.
    block_areas: Vec<Rect>,
    /// Container blocks.
    blocks: Vec<Option<Block<'static>>>,
}

impl<W> Default for GenericLayout<W>
where
    W: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self {
            area: Default::default(),
            page_size: Size::new(u16::MAX, u16::MAX),
            page_count: 1,
            widgets: Default::default(),
            rwidgets: Default::default(),
            widget_areas: Default::default(),
            labels: Default::default(),
            label_areas: Default::default(),
            block_areas: Default::default(),
            blocks: Default::default(),
        }
    }
}

impl<W> GenericLayout<W>
where
    W: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize with a certain capacity.
    pub fn with_capacity(num_widgets: usize, num_blocks: usize) -> Self {
        Self {
            area: Default::default(),
            page_size: Size::new(u16::MAX, u16::MAX),
            page_count: Default::default(),
            widgets: HashMap::with_capacity(num_widgets),
            rwidgets: HashMap::with_capacity(num_widgets),
            widget_areas: Vec::with_capacity(num_widgets),
            labels: Vec::with_capacity(num_widgets),
            label_areas: Vec::with_capacity(num_widgets),
            block_areas: Vec::with_capacity(num_blocks),
            blocks: Vec::with_capacity(num_blocks),
        }
    }

    /// Clear all data.
    pub fn clear(&mut self) {
        self.area = Rect::default();
        self.page_size = Size::default();
        self.page_count = 0;
        self.widgets.clear();
        self.rwidgets.clear();
        self.widget_areas.clear();
        self.labels.clear();
        self.label_areas.clear();
        self.block_areas.clear();
        self.blocks.clear();
    }

    /// Set the area used for this layout.
    /// The area may or may not have anything to do with the page-size.
    pub fn set_area(&mut self, area: Rect) {
        self.area = area;
    }

    /// The area used for this layout.
    /// The area may or may not have anything to do with the page-size.
    pub fn area(&self) -> Rect {
        self.area
    }

    /// Area differs from stored area?
    pub fn area_changed(&self, area: Rect) -> bool {
        self.area != area
    }

    /// Set the page-size for this layout.
    ///
    /// Defaults to (u16::MAX, u16::MAX).
    pub fn set_page_size(&mut self, size: Size) {
        self.page_size = size;
    }

    /// Get the page-size for this layout.
    pub fn page_size(&self) -> Size {
        self.page_size
    }

    /// Page-size changed.
    pub fn size_changed(&self, size: Size) -> bool {
        self.page_size != size
    }

    /// Number of pages
    pub fn set_page_count(&mut self, page_count: usize) {
        self.page_count = page_count;
    }

    /// Number of pages
    pub fn page_count(&self) -> usize {
        self.page_count
    }

    /// Add widget + label areas.
    pub fn add(
        &mut self, //
        key: W,
        area: Rect,
        label: Option<Cow<'static, str>>,
        label_area: Rect,
    ) {
        let idx = self.widget_areas.len();
        self.widgets.insert(key.clone(), idx);
        self.rwidgets.insert(idx, key);
        self.widget_areas.push(area);
        self.labels.push(label);
        self.label_areas.push(label_area);
    }

    /// Add a block.
    pub fn add_block(
        &mut self, //
        area: Rect,
        block: Option<Block<'static>>,
    ) {
        self.block_areas.push(area);
        self.blocks.push(block);
    }

    /// Places the layout at the given position.
    /// This shifts all area right by the given offset.
    ///
    /// Most layout functions create a layout that starts at (0,0).
    /// That is ok, as the widgets __using__ such a layout
    /// associate their top/left position with (0,0) and start
    /// from there.
    ///
    /// If you want to use the layout without such a widget,
    /// this one is nice.
    #[inline]
    pub fn place(mut self, pos: Position) -> Self {
        for v in self.widget_areas.iter_mut() {
            *v = place(*v, pos);
        }
        for v in self.label_areas.iter_mut() {
            *v = place(*v, pos);
        }
        for v in self.block_areas.iter_mut() {
            *v = place(*v, pos);
        }
        self
    }

    /// First widget on the given page.
    pub fn first(&self, page: usize) -> Option<W> {
        for (idx, area) in self.widget_areas.iter().enumerate() {
            let test = (area.y / self.page_size.height) as usize;
            if page == test {
                return self.rwidgets.get(&idx).cloned();
            }
        }
        None
    }

    /// Calculates the page of the widget.
    #[allow(clippy::question_mark)]
    pub fn page_of(&self, widget: W) -> Option<usize> {
        let Some(idx) = self.try_index_of(widget) else {
            return None;
        };

        Some((self.widget_areas[idx].y / self.page_size.height) as usize)
    }

    /// Any widgets/blocks?
    pub fn is_empty(&self) -> bool {
        self.widget_areas.is_empty() && self.block_areas.is_empty()
    }

    /// Is this as layout with height = u16::MAX?
    pub fn is_endless(&self) -> bool {
        self.area.height == u16::MAX
    }

    /// Number of widgets/labels.
    #[inline]
    pub fn widget_len(&self) -> usize {
        self.widgets.len()
    }

    /// Returns the index for this widget.
    pub fn try_index_of(&self, widget: W) -> Option<usize> {
        self.widgets.get(&widget).copied()
    }

    /// Returns the index for this widget.
    ///
    /// __Panic__
    /// Panics if there is no widget for the key.
    pub fn index_of(&self, widget: W) -> usize {
        self.widgets.get(&widget).copied().expect("widget")
    }

    /// Access widget key.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    #[inline]
    pub fn widget_key(&self, idx: usize) -> W {
        self.rwidgets.get(&idx).cloned().expect("valid_idx")
    }

    /// Access widget keys
    #[inline]
    pub fn widget_keys(&self) -> impl Iterator<Item = &W> {
        self.widgets.keys()
    }

    /// Access the label area by key.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    /// Panics if the key doesn't exist.
    #[inline]
    pub fn label_for(&self, widget: W) -> Rect {
        self.label_areas[self.index_of(widget)]
    }

    /// Access label area.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    #[inline]
    pub fn label(&self, idx: usize) -> Rect {
        self.label_areas[idx]
    }

    /// Set the label area.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    #[inline]
    pub fn set_label(&mut self, idx: usize, area: Rect) {
        self.label_areas[idx] = area;
    }

    /// Access the widget area by key.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    /// Panics if the key doesn't exist.
    #[inline]
    pub fn widget_for(&self, widget: W) -> Rect {
        self.widget_areas[self.index_of(widget)]
    }

    /// Access widget area.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    #[inline]
    pub fn widget(&self, idx: usize) -> Rect {
        self.widget_areas[idx]
    }

    /// Change the widget area.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    #[inline]
    pub fn set_widget(&mut self, idx: usize, area: Rect) {
        self.widget_areas[idx] = area;
    }

    /// Access the label string by key.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    /// Panics if the key doesn't exist.
    #[inline]
    pub fn try_label_str_for(&self, widget: W) -> &Option<Cow<'static, str>> {
        &self.labels[self.index_of(widget)]
    }

    /// Access label string.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    #[inline]
    pub fn try_label_str(&self, idx: usize) -> &Option<Cow<'static, str>> {
        &self.labels[idx]
    }

    /// Access the label string by key.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    /// Panics if the key doesn't exist.
    #[inline]
    pub fn label_str_for(&self, widget: W) -> &str {
        self.labels[self.index_of(widget)]
            .as_ref()
            .map(|v| v.as_ref())
            .unwrap_or("")
    }

    /// Access label string.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    #[inline]
    pub fn label_str(&self, idx: usize) -> &str {
        self.labels[idx]
            .as_ref() //
            .map(|v| v.as_ref())
            .unwrap_or("")
    }

    /// Set the label string.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    #[inline]
    pub fn set_label_str(&mut self, idx: usize, str: Option<Cow<'static, str>>) {
        self.labels[idx] = str;
    }

    /// Container count.
    #[inline]
    pub fn block_len(&self) -> usize {
        self.blocks.len()
    }

    /// Access block area.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    #[inline]
    pub fn block_area(&self, idx: usize) -> Rect {
        self.block_areas[idx]
    }

    /// Set the block area.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    #[inline]
    pub fn set_block_area(&mut self, idx: usize, area: Rect) {
        self.block_areas[idx] = area;
    }

    /// Iterate block areas.
    #[inline]
    pub fn block_area_iter(&self) -> impl Iterator<Item = &Rect> {
        self.block_areas.iter()
    }

    /// Access container block.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    #[inline]
    pub fn block(&self, idx: usize) -> &Option<Block<'static>> {
        &self.blocks[idx]
    }

    /// Set the container block.
    ///
    /// __Panic__
    /// Panics on out of bounds.
    #[inline]
    pub fn set_block(&mut self, idx: usize, block: Option<Block<'static>>) {
        self.blocks[idx] = block;
    }

    /// Append another layout.
    ///
    /// Shifts the second layout to the given starting position and
    /// adds everything.
    ///
    /// If the given layout uses the same widget-identifiers, they
    /// will simply overwrite existing ones.
    pub fn append(&mut self, place: Position, mut layout: GenericLayout<W>) {
        layout = layout.place(place);

        for (w, idx) in layout.widgets {
            let new_idx = self.widget_areas.len();
            self.widget_areas.push(layout.widget_areas[idx]);
            self.labels.push(layout.labels[idx].take());
            self.label_areas.push(layout.label_areas[idx]);
            self.widgets.insert(w.clone(), new_idx);
            self.rwidgets.insert(new_idx, w);
        }
        self.block_areas.extend(layout.block_areas);
        self.blocks.extend(layout.blocks);
    }
}

#[inline]
fn place(mut area: Rect, pos: Position) -> Rect {
    area.x += pos.x;
    area.y += pos.y;
    area
}
