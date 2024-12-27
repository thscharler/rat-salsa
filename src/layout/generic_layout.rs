use ratatui::layout::{Rect, Size};
use ratatui::widgets::Block;
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::Hash;

/// Stores layout data.
///
/// This can store three types of layout areas:
/// * widget area
/// * label associated with a widget
/// * container/[Block] areas.
///
/// This layout has a simple concept of pages too.
/// The y-coordinate of each area divided by the
/// page height.
///
/// There may be layout generators that don't set
/// the page size. This will give a division by zero
/// when calling page related functions.
///
#[derive(Debug, Clone)]
pub struct GenericLayout<W, C = ()>
where
    W: Eq + Hash + Clone,
    C: Eq,
{
    /// Reference area.
    area: Rect,

    /// Page size.
    page_size: Size,
    /// Pages.
    page_count: usize,

    /// Widget keys.
    widgets: HashMap<W, usize>,
    rwidgets: HashMap<usize, W>,
    /// Widget areas.
    areas: Vec<Rect>,
    /// Widget labels.
    labels: Vec<Option<Cow<'static, str>>>,
    /// Label areas.
    label_areas: Vec<Rect>,

    /// Container keys.
    containers: Vec<C>,
    /// Container areas.
    container_areas: Vec<Rect>,
    /// Container blocks.
    container_blocks: Vec<Option<Block<'static>>>,
}

impl<W, C> Default for GenericLayout<W, C>
where
    W: Eq + Hash + Clone,
    C: Eq,
{
    fn default() -> Self {
        Self {
            area: Default::default(),
            page_size: Default::default(),
            page_count: 1,
            widgets: Default::default(),
            rwidgets: Default::default(),
            areas: Default::default(),
            labels: Default::default(),
            label_areas: Default::default(),
            containers: Default::default(),
            container_areas: Default::default(),
            container_blocks: Default::default(),
        }
    }
}

impl<W, C> GenericLayout<W, C>
where
    W: Eq + Hash + Clone,
    C: Eq,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(w: usize, c: usize) -> Self {
        Self {
            area: Default::default(),
            page_size: Default::default(),
            page_count: Default::default(),
            widgets: HashMap::with_capacity(w),
            rwidgets: HashMap::with_capacity(w),
            areas: Vec::with_capacity(w),
            labels: Vec::with_capacity(w),
            label_areas: Vec::with_capacity(w),
            containers: Vec::with_capacity(c),
            container_areas: Vec::with_capacity(c),
            container_blocks: Vec::with_capacity(c),
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
    pub fn area_changed(&self, area: Rect) -> bool {
        self.area != area
    }

    /// Set the page-size for this layout.
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

    /// Add a widget + label areas.
    pub fn add(
        &mut self, //
        key: W,
        area: Rect,
        label: Option<Cow<'static, str>>,
        label_area: Rect,
    ) {
        let idx = self.areas.len();
        self.widgets.insert(key.clone(), idx);
        self.rwidgets.insert(idx, key);
        self.areas.push(area);
        self.labels.push(label);
        self.label_areas.push(label_area);
    }

    /// Add a container + block.
    pub fn add_container(
        &mut self, //
        key: C,
        area: Rect,
        block: Option<Block<'static>>,
    ) {
        self.containers.push(key);
        self.container_areas.push(area);
        self.container_blocks.push(block);
    }

    /// First widget on the given page.
    pub fn first(&self, page: usize) -> Option<&W> {
        for (idx, area) in self.areas.iter().enumerate() {
            let test = (area.y / self.page_size.height) as usize;
            if page == test {
                return self.rwidgets.get(&idx);
                // return Some(&self.widgets[idx]);
            }
        }
        None
    }

    /// Calculates the page of the widget.
    pub fn page_of(&self, widget: &W) -> Option<usize> {
        let Some(idx) = self.widget_idx(widget) else {
            return None;
        };

        Some((self.areas[idx].y / self.page_size.height) as usize)
    }

    /// Number of widgets/labels.
    #[inline]
    pub fn widget_len(&self) -> usize {
        self.widgets.len()
    }

    /// Returns the index for this widget.
    pub fn widget_idx(&self, widget: &W) -> Option<usize> {
        self.widgets.get(widget).copied()
        // self.widgets
        //     .iter()
        //     .enumerate()
        //     .find_map(|(idx, w)| if w == widget { Some(idx) } else { None })
    }

    /// Access widget key.
    #[inline]
    pub fn widget_key(&self, idx: usize) -> &W {
        &self.rwidgets.get(&idx).expect("valid_idx")
    }

    /// Access widget keys
    #[inline]
    pub fn widget_keys(&self) -> impl Iterator<Item = &W> {
        self.widgets.keys()
    }

    /// Access label area.
    #[inline]
    pub fn label(&self, idx: usize) -> Rect {
        self.label_areas[idx]
    }

    /// Set the label area.
    #[inline]
    pub fn set_label(&mut self, idx: usize, area: Rect) {
        self.label_areas[idx] = area;
    }

    /// Access widget area.
    #[inline]
    pub fn widget(&self, idx: usize) -> Rect {
        self.areas[idx]
    }

    /// Change the widget area.
    #[inline]
    pub fn set_widget(&mut self, idx: usize, area: Rect) {
        self.areas[idx] = area;
    }

    /// Access label string.
    #[inline]
    pub fn label_str(&self, idx: usize) -> &Option<Cow<'static, str>> {
        &self.labels[idx]
    }

    /// Set the label string.
    #[inline]
    pub fn set_label_str(&mut self, idx: usize, str: Option<Cow<'static, str>>) {
        self.labels[idx] = str;
    }

    /// Container count.
    #[inline]
    pub fn container_len(&self) -> usize {
        self.containers.len()
    }

    /// Access container key.
    #[inline]
    pub fn container_key(&self, idx: usize) -> &C {
        &self.containers[idx]
    }

    /// Access container keys.
    #[inline]
    pub fn container_keys(&self) -> impl Iterator<Item = &C> {
        self.containers.iter()
    }

    /// Access container area.
    #[inline]
    pub fn container(&self, idx: usize) -> Rect {
        self.container_areas[idx]
    }

    /// Set the container area.
    #[inline]
    pub fn set_container(&mut self, idx: usize, area: Rect) {
        self.container_areas[idx] = area;
    }

    /// Iterate container areas.
    #[inline]
    pub fn containers(&self) -> impl Iterator<Item = &Rect> {
        self.container_areas.iter()
    }

    /// Access container block.
    #[inline]
    pub fn container_block(&self, idx: usize) -> &Option<Block<'static>> {
        &self.container_blocks[idx]
    }

    /// Set the container block.
    #[inline]
    pub fn set_container_block(&mut self, idx: usize, block: Option<Block<'static>>) {
        self.container_blocks[idx] = block;
    }
}
