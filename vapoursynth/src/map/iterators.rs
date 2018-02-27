use std::borrow::Cow;

use super::*;

/// An iterator over the keys of a map.
#[derive(Clone, Copy)]
pub struct Keys<'a> {
    map: &'a Map,
    count: usize,
    index: usize,
}

impl<'a> Keys<'a> {
    #[inline]
    pub(crate) fn new(map: &'a Map) -> Self {
        Self {
            map,
            count: map.key_count(),
            index: 0,
        }
    }
}

impl<'a> Iterator for Keys<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.count {
            return None;
        }

        let key = self.map.key(self.index);
        self.index += 1;
        Some(key)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.count - self.index;
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for Keys<'a> {}

/// An iterator over the values associated with a certain key of a map.
pub struct ValueIter<'map, 'key, T> {
    map: &'map Map,
    key: Cow<'key, CStr>, // Just using this as an enum { owned, borrowed }.
    count: i32,
    index: i32,
    _variance: PhantomData<fn() -> T>,
}

macro_rules! impl_value_iter {
    ($value_type:path, $type:ty, $func:ident) => (
        impl<'map, 'key> ValueIter<'map, 'key, $type> {
            /// Creates a `ValueIter` from the given `map` and `key`.
            ///
            /// # Safety
            /// The caller must ensure `key` is valid.
            pub(crate) unsafe fn new(map: &'map Map, key: Cow<'key, CStr>) -> Result<Self> {
                // Check if the value type is correct.
                match map.value_type_raw_unchecked(&key)? {
                    $value_type => {},
                    _ => return Err(Error::WrongValueType)
                };

                let count = map.value_count_raw_unchecked(&key)? as i32;
                Ok(Self {
                    map,
                    key,
                    count,
                    index: 0,
                    _variance: PhantomData,
                })
            }
        }

        impl<'map, 'key> Iterator for ValueIter<'map, 'key, $type> {
            type Item = $type;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                if self.index == self.count {
                    return None;
                }

                let value = unsafe { self.map.$func(&self.key, self.index).unwrap() };
                self.index += 1;

                Some(value)
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = (self.count - self.index) as usize;
                (len, Some(len))
            }
        }

        impl<'map, 'key> ExactSizeIterator for ValueIter<'map, 'key, $type> {}
    )
}

impl_value_iter!(ValueType::Int, i64, get_int_raw_unchecked);
impl_value_iter!(ValueType::Float, f64, get_float_raw_unchecked);
impl_value_iter!(ValueType::Data, &'map [u8], get_data_raw_unchecked);
impl_value_iter!(ValueType::Node, Node, get_node_raw_unchecked);
impl_value_iter!(ValueType::Frame, Frame, get_frame_raw_unchecked);
impl_value_iter!(ValueType::Function, Function, get_function_raw_unchecked);
