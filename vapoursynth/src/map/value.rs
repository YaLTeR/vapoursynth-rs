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
