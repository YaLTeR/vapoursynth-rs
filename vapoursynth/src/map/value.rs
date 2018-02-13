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

impl<'a> Value<'a> {
    /// Returns a `ValueRef` to this `Value`.
    #[inline]
    pub fn get_ref(&self) -> ValueRef {
        match *self {
            Value::Int(x) => ValueRef::Int(x),
            Value::Float(x) => ValueRef::Float(x),
            Value::Data(x) => ValueRef::Data(x),
            Value::Node(ref x) => ValueRef::Node(x),
            Value::Frame(ref x) => ValueRef::Frame(x),
            Value::Function(ref x) => ValueRef::Function(x),
        }
    }
}

// TODO: is it possible to get rid of all of this Ref stuff?
// Not only it's extra types, but for instance making a function for taking Values out of
// ValueArray seems plain impossible.
/// A non-owned value that can be stored in a `Map`.
#[derive(Debug, Clone, Copy)]
pub enum ValueRef<'a> {
    Int(i64),
    Float(f64),
    Data(&'a [u8]),
    Node(&'a Node),
    Frame(&'a Frame),
    Function(&'a Function),
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

/// A number of non-owned values.
#[cfg_attr(rustfmt, rustfmt_skip)]
pub enum Values<'a, 'b: 'a> {
    Ints(&'a mut Iterator<Item = i64>),
    IntArray(&'a [i64]),
    Floats(&'a mut Iterator<Item = f64>),
    FloatArray(&'a [f64]),
    Data(&'a mut Iterator<Item = &'b [u8]>),
    Nodes(&'a mut Iterator<Item = &'b Node>),
    Frames(&'a mut Iterator<Item = &'b Frame>),
    Functions(&'a mut Iterator<Item = &'b Function>),
}

/// An enumeration of all possible value types.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ValueType {
    Int,
    Float,
    Data,
    Node,
    Frame,
    Function,
}
