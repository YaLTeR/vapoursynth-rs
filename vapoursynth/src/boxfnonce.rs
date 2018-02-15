use std::borrow::Cow;

use frame::Frame;
use node::Node;

// This is a collection of hacks to allow passing FnOnce as callbacks.
// Based on the boxfnonce crate.

pub trait NodeFnBox {
    fn call(self: Box<Self>, frame: Result<Frame, Cow<str>>, n: usize, node: Node);
}

impl<F> NodeFnBox for F
where
    F: FnOnce(Result<Frame, Cow<str>>, usize, Node) + Send + 'static,
{
    fn call(self: Box<Self>, frame: Result<Frame, Cow<str>>, n: usize, node: Node) {
        let this = *self;
        this(frame, n, node)
    }
}
