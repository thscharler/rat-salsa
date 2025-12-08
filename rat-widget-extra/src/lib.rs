#[cfg(feature = "color-input")]
pub mod color_input;
#[cfg(feature = "iban")]
pub mod iban;

#[allow(dead_code)]
mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
