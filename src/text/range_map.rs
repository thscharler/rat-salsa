use crate::text::textarea_core::{TextPosition, TextRange};
use iset::IntervalMap;
use std::ops::Range;
use std::{mem, slice};

/// Maps a range to a list of usize.
#[derive(Debug, Default, Clone)]
pub(crate) struct RangeMap {
    buf: Vec<(Range<TextPosition>, RangeMapEntry)>,
    map: IntervalMap<TextPosition, RangeMapEntry>,
}

/// Value as stored in the range map.
#[derive(Debug, Default, Clone)]
pub(crate) enum RangeMapEntry {
    #[default]
    None,
    One(usize),
    Many(Vec<usize>),
}

impl RangeMapEntry {
    pub(crate) fn iter(&self) -> RangeMapEntryIter<'_> {
        match self {
            RangeMapEntry::None => RangeMapEntryIter {
                single: None,
                iter: None,
            },
            RangeMapEntry::One(s) => RangeMapEntryIter {
                single: Some(*s),
                iter: None,
            },
            RangeMapEntry::Many(s) => RangeMapEntryIter {
                single: None,
                iter: Some(s.iter()),
            },
        }
    }
}

#[derive(Debug)]
pub(crate) struct RangeMapEntryIter<'a> {
    single: Option<usize>,
    iter: Option<slice::Iter<'a, usize>>,
}

impl<'a> Iterator for RangeMapEntryIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(single) = self.single {
            self.single = None;
            Some(single)
        } else if let Some(iter) = &mut self.iter {
            iter.next().copied()
        } else {
            None
        }
    }
}

impl RangeMap {
    /// Remove ranges.
    pub(crate) fn clear(&mut self) {
        self.buf.clear();
        self.map.clear();
    }

    /// Add a value to a range.
    ///
    /// The same range can be added again with a different value.
    pub(crate) fn add(&mut self, range: TextRange, value: usize) {
        if range.is_empty() {
            return;
        }
        self.map
            .entry(range.into())
            .and_modify(|v| {
                let new_v = match v {
                    RangeMapEntry::None => {
                        //
                        RangeMapEntry::One(value)
                    }
                    RangeMapEntry::One(w) => {
                        if *w != value {
                            let mut s_vec = vec![*w, value];
                            s_vec.sort();
                            RangeMapEntry::Many(s_vec)
                        } else {
                            RangeMapEntry::One(*w)
                        }
                    }
                    RangeMapEntry::Many(w) => {
                        let mut w = mem::take(w);
                        match w.binary_search(&value) {
                            Ok(_) => {}
                            Err(i) => w.insert(i, value),
                        }
                        RangeMapEntry::Many(w)
                    }
                };
                *v = new_v;
            })
            .or_insert_with(|| RangeMapEntry::One(value));
    }

    /// Remove a value for a range.
    ///
    /// This must match exactly in range and value to be removed.
    pub(crate) fn remove(&mut self, range: TextRange, value: usize) {
        if range.is_empty() {
            return;
        }
        let Some(v) = self.map.get_mut(range.into()) else {
            return;
        };

        let new_v = match v {
            RangeMapEntry::None => {
                //
                RangeMapEntry::None
            }
            RangeMapEntry::One(w) => {
                if *w == value {
                    RangeMapEntry::None
                } else {
                    RangeMapEntry::One(*w)
                }
            }
            RangeMapEntry::Many(w) => {
                let mut w = mem::take(w);
                w.retain(|s| *s != value);
                if w.len() > 1 {
                    RangeMapEntry::Many(w)
                } else {
                    RangeMapEntry::One(w[0])
                }
            }
        };
        let is_empty = matches!(new_v, RangeMapEntry::None);
        *v = new_v;

        if is_empty {
            self.map.remove(range.into());
        }
    }

    /// List of all values.
    pub(crate) fn values(&self) -> impl Iterator<Item = (TextRange, Vec<usize>)> + '_ {
        self.map.iter(..).map(|v| {
            (
                v.0.into(), //
                v.1.iter().collect::<Vec<_>>(),
            )
        })
    }

    /// Find all values that touch the given position.
    pub(crate) fn values_at(&self, pos: TextPosition, buf: &mut Vec<usize>) {
        for v in self
            .map
            .overlap(pos) //
            .map(|v| v.1)
        {
            match v {
                RangeMapEntry::None => {}
                RangeMapEntry::One(w) => {
                    buf.push(*w);
                }
                RangeMapEntry::Many(w) => {
                    buf.extend_from_slice(w);
                }
            }
        }
    }

    /// Map and rebuild the IntervalMap.
    #[inline]
    pub(crate) fn remap(
        &mut self,
        mut remap_fn: impl FnMut(TextRange, &RangeMapEntry) -> Option<TextRange>,
    ) {
        self.buf.clear();

        let map = mem::take(&mut self.map);
        for (r, s) in map.into_iter(..) {
            if let Some(r) = remap_fn(r.into(), &s) {
                self.buf.push((r.into(), s));
            }
        }
        self.map = IntervalMap::from_sorted(self.buf.drain(..));
    }
}
