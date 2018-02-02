use frame::Frame;
use function::Function;
use node::Node;

/// A value that can be stored in a `Map`.
#[derive(Debug, Clone)]
pub enum Value<'a> {
    Int(i64),
    Float(f64),
    Data(&'a [u8]),
    Node(Node),
    Frame(Frame),
    Function(Function),
}

/// An array of values.
#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone)]
pub enum ValueArray<'a> {
    // API 3.1 introduced more optimized getters for some value types.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    Ints(&'a [i64]),
    #[cfg(feature = "gte-vapoursynth-api-31")]
    Floats(&'a [f64]),

    #[cfg(not(feature = "gte-vapoursynth-api-31"))]
    Ints(Vec<i64>),
    #[cfg(not(feature = "gte-vapoursynth-api-31"))]
    Floats(Vec<f64>),

    Data(Vec<&'a [u8]>),
    Nodes(Vec<Node>),
    Frames(Vec<Frame>),
    Functions(Vec<Function>),
}
