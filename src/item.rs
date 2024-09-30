use crate::Separator;
use std::ops::Range;

/// A menu-item with the following separator.
#[derive(Debug, Default, Clone)]
pub(crate) struct Item<'a> {
    pub(crate) item: &'a str,
    pub(crate) highlight: Option<Range<usize>>,
    pub(crate) navchar: Option<char>,
    pub(crate) right: Option<&'a str>,
    pub(crate) sep: Separator,
}

impl<'a> Item<'a> {
    pub(crate) fn width(&self) -> u16 {
        (self.item.len() + self.right.map(|v| v.len()).unwrap_or_default()) as u16
    }

    pub(crate) fn height(&self) -> u16 {
        if self.sep == Separator::None {
            1
        } else {
            2
        }
    }
}
