use crate::text::textarea_core::{TextPosition, TextRange};
use iset::IntervalMap;
use std::cell::{Cell, RefCell};
use std::ops::Range;

/// Maps a range to a list of usize.
#[derive(Debug, Default, Clone)]
pub(crate) struct RangeMap {
    buf: Vec<(Range<TextPosition>, usize)>,
    map: IntervalMap<TextPosition, usize>,

    // cache for page-render
    page: Cell<TextRange>,
    page_map: RefCell<IntervalMap<TextPosition, usize>>,
}

impl RangeMap {
    /// Remove ranges.
    pub(crate) fn clear(&mut self) {
        self.buf.clear();
        self.map.clear();
        self.page.set(TextRange::default());
        self.page_map.borrow_mut().clear();
    }

    /// Add a list of values to a range.
    /// Attention:
    /// Doesn't check for duplicate values, just inserts them.
    /// Empty ranges are ignored.
    pub(crate) fn set(&mut self, styles: impl Iterator<Item = (TextRange, usize)>) {
        self.map.clear();
        for (r, v) in styles {
            if !r.is_empty() {
                self.map.force_insert(r.into(), v);
            }
        }
    }

    /// Add a value to a range.
    ///
    /// The same range can be added again with a different value.
    /// Duplicate values are ignored.
    pub(crate) fn add(&mut self, range: TextRange, value: usize) {
        if range.is_empty() {
            return;
        }
        if self
            .map
            .values_at(range.into())
            .find(|v| **v == value)
            .is_none()
        {
            self.map.force_insert(range.into(), value);
        }
    }

    /// Remove a value for a range.
    ///
    /// This must match exactly in range and value to be removed.
    pub(crate) fn remove(&mut self, range: TextRange, value: usize) {
        if range.is_empty() {
            return;
        }
        self.map.remove_where(range.into(), |v| *v == value);
    }

    /// List of all values.
    pub(crate) fn values(&self) -> impl Iterator<Item = (TextRange, usize)> + '_ {
        self.map.iter(..).map(|(r, v)| (r.into(), *v))
    }

    /// Clear page cache.
    #[inline]
    pub(crate) fn clear_page_values(&self) {
        self.page.set(TextRange::default());
        self.page_map.borrow_mut().clear();
    }

    /// Find all values for the page that touch the given position.
    pub(crate) fn values_at_page(&self, range: TextRange, pos: TextPosition, buf: &mut Vec<usize>) {
        let mut page_map = self.page_map.borrow_mut();
        if self.page.get() != range {
            self.page.set(range);
            page_map.clear();
            for (r, v) in self.map.iter(Range::from(range)) {
                page_map.force_insert(r, *v);
            }
        }
        for v in page_map.overlap(pos).map(|v| v.1) {
            buf.push(*v);
        }
    }

    /// Find all values that touch the given position.
    pub(crate) fn values_at(&self, pos: TextPosition, buf: &mut Vec<usize>) {
        for v in self.map.overlap(pos).map(|v| v.1) {
            buf.push(*v);
        }
    }

    /// Check if a given value exists for the position and return the range.
    pub(crate) fn value_match(&self, pos: TextPosition, value: usize) -> Option<TextRange> {
        for (r, s) in self.map.overlap(pos.into()) {
            if value == *s {
                return Some(r.into());
            }
        }
        None
    }

    /// Map and rebuild the IntervalMap.
    #[inline]
    pub(crate) fn remap(
        &mut self,
        mut remap_fn: impl FnMut(TextRange, usize) -> Option<TextRange>,
    ) {
        self.buf.clear();

        let mut change = false;
        for (range, value) in self.map.iter(..) {
            let range = TextRange::from(range);
            if let Some(new_range) = remap_fn(range, *value) {
                if range != new_range {
                    change = true;
                }
                self.buf.push((new_range.into(), *value));
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
                    self.map.force_insert(r.into(), v);
                }
            }
        }
    }
}
