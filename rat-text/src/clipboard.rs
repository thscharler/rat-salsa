//!
//! There are too many clipboard crates.
//!
//! Provides the Clipboard trait to connect all text-widgets
//! with the clipboard crate of your choice.
//!
//! There is a default implementation that allows copying
//! within the application.
//!

use crate::TextError;
use dyn_clone::{clone_box, DynClone};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::{Arc, Mutex, OnceLock};

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

    /// Set text to the clipboard.
    fn set_string(&self, s: &str) -> Result<(), ClipboardError>;
}

static GLOBAL_CLIPBOARD: OnceLock<StaticClipboard> = OnceLock::new();

/// Get a Clone of the global default clipboard.
pub fn global_clipboard() -> Box<dyn Clipboard> {
    let c = GLOBAL_CLIPBOARD.get_or_init(StaticClipboard::default);
    Box::new(c.clone())
}

/// Change the global default clipboard.
pub fn set_global_clipboard(clipboard: impl Clipboard + Send + 'static) {
    let c = GLOBAL_CLIPBOARD.get_or_init(StaticClipboard::default);
    c.replace(clipboard);
}

/// Clipboard that can be set as a static.
/// It can replace the actual clipboard implementation at a later time.
/// Initializes with a LocalClipboard.
#[derive(Debug, Clone)]
struct StaticClipboard {
    clip: Arc<Mutex<Box<dyn Clipboard + Send>>>,
}

impl Default for StaticClipboard {
    fn default() -> Self {
        Self {
            clip: Arc::new(Mutex::new(Box::new(LocalClipboard::new()))),
        }
    }
}

impl StaticClipboard {
    /// Replace the static clipboard with the given one.
    fn replace(&self, clipboard: impl Clipboard + Send + 'static) {
        let mut clip = self.clip.lock().expect("clipboard-lock");
        *clip = Box::new(clipboard);
    }
}

impl Clipboard for StaticClipboard {
    fn get_string(&self) -> Result<String, ClipboardError> {
        self.clip.lock().expect("clipboard-lock").get_string()
    }

    fn set_string(&self, s: &str) -> Result<(), ClipboardError> {
        self.clip.lock().expect("clipboard-lock").set_string(s)
    }
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
            Err(_) => Err(ClipboardError),
        }
    }

    fn set_string(&self, s: &str) -> Result<(), ClipboardError> {
        match self.text.lock() {
            Ok(mut v) => {
                *v = s.to_string();
                Ok(())
            }
            Err(_) => Err(ClipboardError),
        }
    }
}

impl Clone for Box<dyn Clipboard> {
    fn clone(&self) -> Self {
        clone_box(self.as_ref())
    }
}

impl Clipboard for Box<dyn Clipboard> {
    fn get_string(&self) -> Result<String, ClipboardError> {
        self.as_ref().get_string()
    }

    fn set_string(&self, s: &str) -> Result<(), ClipboardError> {
        self.as_ref().set_string(s)
    }
}
