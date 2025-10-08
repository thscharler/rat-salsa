use futures_util::Stream;
use std::cell::Cell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

// Token stream for the state-machine.
#[derive(Debug, Default, Clone)]
pub struct TokenStream {
    pub token: Rc<Cell<Option<char>>>,
}

impl TokenStream {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push next token.
    pub fn push_next(&mut self, token: char) {
        self.token.set(Some(token));
    }
}

impl Stream for TokenStream {
    type Item = char;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(token) = self.token.take() {
            Poll::Ready(Some(token))
        } else {
            Poll::Pending
        }
    }
}
