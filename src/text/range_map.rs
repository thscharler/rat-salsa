use crate::text::textarea_core::{TextPosition, TextRange};
use iset::IntervalMap;
use std::ops::Range;

/// Maps a range to a list of usize.
#[derive(Debug, Default, Clone)]
pub(crate) struct RangeMap {
    buf: Vec<(Range<TextPosition>, usize)>,
    map: IntervalMap<TextPosition, usize>,
}

impl RangeMap {
    /// Remove ranges.
    pub(crate) fn clear(&mut self) {
        self.buf.clear();
        self.map.clear();
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

    /// Find all values that touch the given position.
    pub(crate) fn values_at(&self, pos: TextPosition, buf: &mut Vec<usize>) {
        for v in self
            .map
            .overlap(pos) //
            .map(|v| v.1)
        {
            buf.push(*v);
        }
    }

    /// Map and rebuild the IntervalMap.
    #[inline]
    pub(crate) fn remap(
        &mut self,
        mut remap_fn: impl FnMut(TextRange, usize) -> Option<TextRange>,
    ) {
        self.buf.clear();

        for (r, v) in self.map.iter(..) {
            if let Some(r) = remap_fn(r.into(), *v) {
                self.buf.push((r.into(), *v));
            }
        }
        self.map.clear();
        for (r, v) in self.buf.drain(..) {
            if !r.is_empty() {
                self.map.force_insert(r.into(), v);
            }
        }
    }
}
