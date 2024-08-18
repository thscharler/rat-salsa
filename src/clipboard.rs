//!
//! There are too many clipboard crates.
//!

use crate::TextError;
use dyn_clone::DynClone;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct ClipboardError;

impl Display for ClipboardError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ClipboardError {}

impl From<ClipboardError> for TextError {
    fn from(_value: ClipboardError) -> Self {
        TextError::Clipboard
    }
}

/// Access some clipboard.
pub trait Clipboard: DynClone + Debug {
    /// Get text from the clipboard.
    fn get_string(&self) -> Result<String, ClipboardError>;

    /// Set text from the clipboard.
    fn set_string(&self, s: &str) -> Result<(), ClipboardError>;
}

/// Local clipboard.
/// A string in disguise.
#[derive(Debug, Default, Clone)]
pub struct LocalClipboard {
    text: Arc<Mutex<String>>,
}

impl LocalClipboard {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Clipboard for LocalClipboard {
    fn get_string(&self) -> Result<String, ClipboardError> {
        match self.text.lock() {
            Ok(v) => Ok(v.clone()),
            Err(_) => return Err(ClipboardError),
        }
    }

    fn set_string(&self, s: &str) -> Result<(), ClipboardError> {
        match self.text.lock() {
            Ok(mut v) => {
                *v = s.to_string();
                Ok(())
            }
            Err(_) => return Err(ClipboardError),
        }
    }
}
