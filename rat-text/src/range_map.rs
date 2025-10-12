use iset::IntervalMap;
use std::cell::RefCell;
use std::ops::Range;

/// Maps byte ranges to a style index.
#[derive(Debug, Default, Clone)]
pub(crate) struct RangeMap {
    buf: Vec<(Range<usize>, usize)>,
    map: IntervalMap<usize, usize>,

    // cache for page-render
    cache: RangeMapCache,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct RangeMapCache {
    page: RefCell<Range<usize>>,
    page_map: RefCell<IntervalMap<usize, usize>>,
}

impl RangeMapCache {
    fn clear(&self) {
        *self.page.borrow_mut() = Default::default();
        self.page_map.borrow_mut().clear();
    }
}

impl RangeMap {
    /// Remove ranges.
    pub(crate) fn clear(&mut self) {
        self.buf.clear();
        self.map.clear();
        self.cache = Default::default();
    }

    /// Sets a list of byte-range/style.
    ///
    /// __Attention:__
    /// Doesn't check for duplicate values, just inserts them.
    /// Empty ranges are ignored.
    pub(crate) fn set(&mut self, styles: impl Iterator<Item = (Range<usize>, usize)>) {
        self.map.clear();
        self.cache.clear();
        for (r, v) in styles {
            if !r.is_empty() {
                self.map.force_insert(r, v);
            }
        }
    }

    /// Add a value to a range.
    ///
    /// The same range can be added again with a different value.
    /// Duplicate values are ignored.
    pub(crate) fn add(&mut self, range: Range<usize>, value: usize) {
        if range.is_empty() {
            return;
        }
        if !self.map.values_at(range.clone()).any(|v| *v == value) {
            self.map.force_insert(range, value);
        }
        self.cache.clear();
    }

    /// Remove a value for a range.
    ///
    /// This must match exactly in range and value to be removed.
    pub(crate) fn remove(&mut self, range: Range<usize>, value: usize) {
        if range.is_empty() {
            return;
        }
        self.map.remove_where(range, |v| *v == value);
        self.cache.clear();
    }

    /// List of all values.
    pub(crate) fn values(&self) -> impl Iterator<Item = (Range<usize>, usize)> + '_ {
        self.map.iter(..).map(|(r, v)| (r, *v))
    }

    /// Find all values for the page that touch the given position.
    pub(crate) fn values_at_page(&self, pos: usize, range: Range<usize>, buf: &mut Vec<usize>) {
        if range != *self.cache.page.borrow() {
            *self.cache.page.borrow_mut() = range.clone();

            let mut page_map = self.cache.page_map.borrow_mut();
            page_map.clear();
            if !range.is_empty() {
                for (r, v) in self.map.iter(range) {
                    page_map.force_insert(r, *v);
                }
            }
        }

        buf.clear();
        for v in self.cache.page_map.borrow().overlap(pos).map(|v| v.1) {
            buf.push(*v);
        }
    }

    /// Find everything that touches the given range.
    pub(crate) fn values_in(&self, range: Range<usize>, buf: &mut Vec<(Range<usize>, usize)>) {
        if range.is_empty() {
            return;
        }
        for (r, v) in self.map.iter(range) {
            buf.push((r, *v));
        }
    }

    /// Find everything that touches the given range.
    pub(crate) fn values_in_match(
        &self,
        range: Range<usize>,
        style: usize,
        buf: &mut Vec<(Range<usize>, usize)>,
    ) {
        if range.is_empty() {
            return;
        }
        for (r, v) in self.map.iter(range).filter(|(_, v)| **v == style) {
            buf.push((r, *v));
        }
    }

    /// Find all values that touch the given position.
    pub(crate) fn values_at(&self, pos: usize, buf: &mut Vec<(Range<usize>, usize)>) {
        for (r, v) in self.map.overlap(pos) {
            buf.push((r, *v));
        }
    }

    /// Check if a given value exists for the position and return the range.
    pub(crate) fn value_match(&self, pos: usize, value: usize) -> Option<Range<usize>> {
        for (r, s) in self.map.overlap(pos) {
            if value == *s {
                return Some(r);
            }
        }
        None
    }

    /// Map and rebuild the IntervalMap.
    #[inline]
    pub(crate) fn remap(
        &mut self,
        mut remap_fn: impl FnMut(Range<usize>, usize) -> Option<Range<usize>>,
    ) {
        self.buf.clear();

        let mut change = false;
        for (range, value) in self.map.iter(..) {
            if let Some(new_range) = remap_fn(range.clone(), *value) {
                if range != new_range {
                    change = true;
                }
                self.buf.push((new_range, *value));
            } else {
                change = true;
            }
        }
        // TODO: faster but doesn't allow duplicates.
        // if change {
        //     self.map = IntervalMap::from_sorted(self.buf.drain(..));
        // }
        if change {
            self.map.clear();
            for (r, v) in self.buf.drain(..) {
                if !r.is_empty() {
                    self.map.force_insert(r, v);
                }
            }
        }
        self.cache.clear();
    }
}

/// Ranges intersect
pub(crate) fn ranges_intersect(first: Range<usize>, second: Range<usize>) -> bool {
    first.start <= second.end && first.end >= second.start
}

/// Text range insertion.
pub(crate) fn expand_range_by(expand: Range<usize>, range: Range<usize>) -> Range<usize> {
    expand_by(expand.clone(), range.start)..expand_by(expand, range.end)
}

/// Text range insertion.
pub(crate) fn expand_by(expand: Range<usize>, pos: usize) -> usize {
    if pos < expand.start {
        pos
    } else {
        pos + (expand.end - expand.start)
    }
}

/// Text range removal.
pub(crate) fn shrink_range_by(shrink: Range<usize>, range: Range<usize>) -> Range<usize> {
    shrink_by(shrink.clone(), range.start)..shrink_by(shrink, range.end)
}

/// Text range removal.
pub(crate) fn shrink_by(shrink: Range<usize>, pos: usize) -> usize {
    if pos < shrink.start {
        pos
    } else if pos >= shrink.start && pos < shrink.end {
        shrink.start
    } else {
        pos - (shrink.end - shrink.start)
    }
}
