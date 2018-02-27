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
