use ratatui::layout::{Rect, Size};
use ratatui::widgets::Block;
use std::borrow::Cow;

/// Stores layout data.
#[derive(Debug, Clone)]
pub struct GenericLayout<W, C = ()>
where
    W: Eq,
    C: Eq,
{
    /// Reference area.
    pub area: Rect,

    /// Pages.
    pub page_count: usize,

    /// Widget keys.
    pub widgets: Vec<W>,
    /// Widget areas.
    pub areas: Vec<Rect>,
    /// Widget labels.
    pub labels: Vec<Option<Cow<'static, str>>>,
    /// Label areas.
    pub label_areas: Vec<Rect>,

    /// Container keys.
    pub containers: Vec<C>,
    /// Container areas.
    pub container_areas: Vec<Rect>,
    /// Container blocks.
    pub container_blocks: Vec<Option<Block<'static>>>,
}

impl<W, C> Default for GenericLayout<W, C>
where
    W: Eq,
    C: Eq,
{
    fn default() -> Self {
        Self {
            area: Default::default(),
            page_count: 1,
            widgets: Default::default(),
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
    W: Eq,
    C: Eq,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(w: usize, c: usize) -> Self {
        Self {
            area: Default::default(),
            page_count: Default::default(),
            widgets: Vec::with_capacity(w),
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
    pub fn size_changed(&self, size: Size) -> bool {
        self.area.as_size() != size
    }

    /// Add a widget + label.
    pub fn add(
        &mut self, //
        key: W,
        area: Rect,
        label: Option<Cow<'static, str>>,
        label_area: Rect,
    ) {
        self.widgets.push(key);
        self.areas.push(area);
        self.labels.push(label);
        self.label_areas.push(label_area);
    }

    /// Add a container/block.
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
            let test = (area.y / self.area.height) as usize;
            if page == test {
                return Some(&self.widgets[idx]);
            }
        }
        None
    }

    /// Calculates the page of the widget.
    pub fn page_of(&self, widget: &W) -> Option<usize> {
        let Some(idx) = self.widget_idx(widget) else {
            return None;
        };

        Some((self.areas[idx].y / self.area.height) as usize)
    }

    /// Returns the __first__ index for this widget.
    ///
    /// Todo: It's not a good thing to have multiple areas for a widget.
    pub fn widget_idx(&self, widget: &W) -> Option<usize> {
        self.widgets
            .iter()
            .enumerate()
            .find_map(|(idx, w)| if w == widget { Some(idx) } else { None })
    }
}
