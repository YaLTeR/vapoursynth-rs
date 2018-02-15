// This is a collection of hacks to allow passing FnOnce as callbacks.
// Based on the boxfnonce crate.

pub trait NodeFnBox<Arguments, Result> {
    fn call(self: Box<Self>, args: Arguments, last_arg: Option<&str>) -> Result;
}

impl<A1, A2, A3, R, F> NodeFnBox<(A1, A2, A3), R> for F
where
    F: FnOnce(A1, A2, A3, Option<&str>) -> R + Send + 'static,
{
    fn call(self: Box<Self>, args: (A1, A2, A3), last_arg: Option<&str>) -> R {
        let this = *self;
        this(args.0, args.1, args.2, last_arg)
    }
}
