use frame::FrameRef;
use function::Function;
use map::{Map, Result, ValueIter};
use node::Node;

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

/// A trait for values which can be stored in a map.
pub trait Value<'map, 'elem: 'map>: Sized {
    /// Retrieves the value from the map.
    fn get_from_map(map: &'map Map<'elem>, key: &str) -> Result<Self>;

    /// Retrieves an iterator over the values from the map.
    fn get_iter_from_map<'key>(
        map: &'map Map<'elem>,
        key: &str,
    ) -> Result<ValueIter<'map, 'elem, 'key, Self>>;

    /// Sets the property value in the map.
    fn store_in_map(map: &'map mut Map<'elem>, key: &str, x: &Self) -> Result<()>;

    /// Appends the value to the map.
    fn append_to_map(map: &'map mut Map<'elem>, key: &str, x: &Self) -> Result<()>;
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for i64 {
    fn get_from_map(map: &Map, key: &str) -> Result<Self> {
        map.get_int(key)
    }

    fn get_iter_from_map<'key>(
        map: &'map Map<'elem>,
        key: &str,
    ) -> Result<ValueIter<'map, 'elem, 'key, Self>> {
        map.get_int_iter(key)
    }

    fn store_in_map(map: &mut Map, key: &str, x: &Self) -> Result<()> {
        map.set_int(key, *x)
    }

    fn append_to_map(map: &mut Map, key: &str, x: &Self) -> Result<()> {
        map.append_int(key, *x)
    }
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for f64 {
    fn get_from_map(map: &Map, key: &str) -> Result<Self> {
        map.get_float(key)
    }

    fn get_iter_from_map<'key>(
        map: &'map Map<'elem>,
        key: &str,
    ) -> Result<ValueIter<'map, 'elem, 'key, Self>> {
        map.get_float_iter(key)
    }

    fn store_in_map(map: &mut Map, key: &str, x: &Self) -> Result<()> {
        map.set_float(key, *x)
    }

    fn append_to_map(map: &mut Map, key: &str, x: &Self) -> Result<()> {
        map.append_float(key, *x)
    }
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for &'map [u8] {
    fn get_from_map(map: &'map Map, key: &str) -> Result<Self> {
        map.get_data(key)
    }

    fn get_iter_from_map<'key>(
        map: &'map Map<'elem>,
        key: &str,
    ) -> Result<ValueIter<'map, 'elem, 'key, Self>> {
        map.get_data_iter(key)
    }

    fn store_in_map(map: &'map mut Map, key: &str, x: &Self) -> Result<()> {
        map.set_data(key, x)
    }

    fn append_to_map(map: &'map mut Map, key: &str, x: &Self) -> Result<()> {
        map.append_data(key, x)
    }
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for Node<'elem> {
    fn get_from_map(map: &Map<'elem>, key: &str) -> Result<Self> {
        map.get_node(key)
    }

    fn get_iter_from_map<'key>(
        map: &'map Map<'elem>,
        key: &str,
    ) -> Result<ValueIter<'map, 'elem, 'key, Self>> {
        map.get_node_iter(key)
    }

    fn store_in_map(map: &mut Map<'elem>, key: &str, x: &Self) -> Result<()> {
        map.set_node(key, x)
    }

    fn append_to_map(map: &mut Map<'elem>, key: &str, x: &Self) -> Result<()> {
        map.append_node(key, x)
    }
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for FrameRef<'elem> {
    fn get_from_map(map: &Map<'elem>, key: &str) -> Result<Self> {
        map.get_frame(key)
    }

    fn get_iter_from_map<'key>(
        map: &'map Map<'elem>,
        key: &str,
    ) -> Result<ValueIter<'map, 'elem, 'key, Self>> {
        map.get_frame_iter(key)
    }

    fn store_in_map(map: &mut Map<'elem>, key: &str, x: &Self) -> Result<()> {
        map.set_frame(key, x)
    }

    fn append_to_map(map: &mut Map<'elem>, key: &str, x: &Self) -> Result<()> {
        map.append_frame(key, x)
    }
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for Function<'elem> {
    fn get_from_map(map: &Map<'elem>, key: &str) -> Result<Self> {
        map.get_function(key)
    }

    fn get_iter_from_map<'key>(
        map: &'map Map<'elem>,
        key: &str,
    ) -> Result<ValueIter<'map, 'elem, 'key, Self>> {
        map.get_function_iter(key)
    }

    fn store_in_map(map: &mut Map<'elem>, key: &str, x: &Self) -> Result<()> {
        map.set_function(key, x)
    }

    fn append_to_map(map: &mut Map<'elem>, key: &str, x: &Self) -> Result<()> {
        map.append_function(key, x)
    }
}
