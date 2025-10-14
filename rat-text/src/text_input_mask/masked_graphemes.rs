use crate::core::SkipLine;
use crate::grapheme::StrGraphemes;
use crate::text_input_mask::mask_token::{Mask, MaskToken};
use crate::{Grapheme, TextError};
use std::borrow::Cow;
use std::slice;

#[derive(Debug, Clone)]
pub(crate) struct MaskedGraphemes<'a> {
    pub iter_str: StrGraphemes<'a>,
    pub iter_mask: slice::Iter<'a, MaskToken>,

    pub compact: bool,
    pub sym_neg: String,
    pub sym_dec: String,
    pub sym_grp: String,
    pub sym_pos: String,

    pub byte_pos: usize,
}

impl<'a> Iterator for MaskedGraphemes<'a> {
    type Item = Grapheme<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let g = self.iter_str.next();
            let t = self.iter_mask.next();

            let (Some(g), Some(t)) = (g, t) else {
                return None;
            };

            self.byte_pos = g.text_bytes().end;

            let r = match (self.compact, &t.right, g.grapheme()) {
                (true, Mask::Numeric(_), "-") => Some(Grapheme::new(
                    Cow::Owned(self.sym_neg.clone()),
                    g.text_bytes(),
                )),
                (true, Mask::DecimalSep, ".") => Some(Grapheme::new(
                    Cow::Owned(self.sym_dec.clone()),
                    g.text_bytes(),
                )),
                (true, Mask::GroupingSep, ",") => Some(Grapheme::new(
                    Cow::Owned(self.sym_grp.clone()),
                    g.text_bytes(),
                )),
                (true, Mask::GroupingSep, "-") => Some(Grapheme::new(
                    Cow::Owned(self.sym_neg.clone()),
                    g.text_bytes(),
                )),
                (true, Mask::Sign, "-") => Some(Grapheme::new(
                    Cow::Owned(self.sym_neg.clone()),
                    g.text_bytes(),
                )),

                (true, Mask::Numeric(_), " ") => None,
                (true, Mask::Digit(_), " ") => None,
                (true, Mask::DecimalSep, " ") => None,
                (true, Mask::GroupingSep, " ") => None,
                (true, Mask::Sign, _) => {
                    if self.sym_pos != " " {
                        Some(Grapheme::new(
                            Cow::Owned(self.sym_pos.clone()),
                            g.text_bytes(),
                        ))
                    } else {
                        None
                    }
                }
                (true, Mask::Hex, " ") => None,
                (true, Mask::Oct, " ") => None,
                (true, Mask::Dec, " ") => None,

                (false, Mask::Numeric(_), "-") => Some(Grapheme::new(
                    Cow::Owned(self.sym_neg.clone()),
                    g.text_bytes(),
                )),
                (false, Mask::DecimalSep, ".") => Some(Grapheme::new(
                    Cow::Owned(self.sym_dec.clone()),
                    g.text_bytes(),
                )),
                (false, Mask::GroupingSep, ",") => Some(Grapheme::new(
                    Cow::Owned(self.sym_grp.clone()),
                    g.text_bytes(),
                )),
                (false, Mask::GroupingSep, "-") => Some(Grapheme::new(
                    Cow::Owned(self.sym_neg.clone()),
                    g.text_bytes(),
                )),
                (false, Mask::Sign, "-") => Some(Grapheme::new(
                    Cow::Owned(self.sym_neg.clone()),
                    g.text_bytes(),
                )),
                (false, Mask::Sign, _) => Some(Grapheme::new(
                    Cow::Owned(self.sym_pos.clone()),
                    g.text_bytes(),
                )),

                (_, _, _) => Some(g),
            };
            if r.is_some() {
                break r;
            }
        }
    }
}

impl<'a> SkipLine for MaskedGraphemes<'a> {
    fn skip_line(&mut self) -> Result<(), TextError> {
        // all in one line, eat the rest.
        for _ in self.iter_str.by_ref() {}
        for _ in self.iter_mask.by_ref() {}
        Ok(())
    }

    fn skip_to(&mut self, byte_pos: usize) -> Result<(), TextError> {
        if byte_pos > self.byte_pos {
            Err(TextError::ByteIndexOutOfBounds(byte_pos, self.byte_pos))
        } else if byte_pos == self.byte_pos {
            Ok(())
        } else {
            for g in self.iter_str.by_ref() {
                _ = self.iter_mask.next();

                if byte_pos == g.text_bytes().end {
                    return Ok(());
                } else if byte_pos < g.text_bytes().end {
                    return Err(TextError::ByteIndexNotCharBoundary(byte_pos));
                }
            }
            Err(TextError::ByteIndexOutOfBounds(byte_pos, byte_pos))
        }
    }
}
