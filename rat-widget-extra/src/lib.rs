#![doc = include_str!("../readme.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg_attr(docsrs, doc(cfg(feature = "color-input")))]
#[cfg(feature = "color-input")]
pub mod color_input;
#[cfg_attr(docsrs, doc(cfg(feature = "iban")))]
#[cfg(feature = "iban")]
pub mod iban;

#[allow(dead_code)]
mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
