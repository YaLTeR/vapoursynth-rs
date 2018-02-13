use std::borrow::Cow;

use super::*;

/// An iterator over the keys of a `VSMap`.
#[derive(Debug, Clone, Copy)]
pub struct Keys<'a, T: 'a> {
    map: &'a T,
    count: usize,
    index: usize,
}

impl<'a, T: 'a> Keys<'a, T>
where
    T: VSMap,
{
    #[inline]
    pub(crate) fn new(map: &'a T) -> Self {
        Self {
            map,
            count: map.key_count(),
            index: 0,
        }
    }
}

impl<'a, T: 'a> Iterator for Keys<'a, T>
where
    T: VSMap,
{
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

impl<'a, T: 'a> ExactSizeIterator for Keys<'a, T>
where
    T: VSMap,
{
}

/// An iterator over the entries of a `VSMap`.
#[derive(Debug, Clone, Copy)]
pub struct Iter<'a, T: 'a> {
    map: &'a T,
    count: usize,
    index: usize,
}

impl<'a, T: 'a> Iter<'a, T>
where
    T: VSMap,
{
    #[inline]
    pub(crate) fn new(map: &'a T) -> Self {
        Self {
            map,
            count: map.key_count(),
            index: 0,
        }
    }
}

impl<'a, T: 'a> Iterator for Iter<'a, T>
where
    T: VSMap,
{
    type Item = (&'a str, ValueArray<'a>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.count {
            return None;
        }

        let key = self.map.key(self.index);
        let raw_key = self.map.key_raw(self.index);
        let values = unsafe { self.map.values_raw_unchecked(raw_key).unwrap() };

        self.index += 1;
        Some((key, values))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.count - self.index;
        (len, Some(len))
    }
}

impl<'a, T: 'a> ExactSizeIterator for Iter<'a, T>
where
    T: VSMap,
{
}

/// An iterator over the map's values.
pub struct ValueIter<'map, 'key, T> {
    map: MapRef<'map>,
    key: Cow<'key, CStr>, // Just using this as an enum { owned, borrowed }.
    count: i32,
    index: i32,
    _variance: PhantomData<fn() -> T>,
}

impl<'map, 'key, T> ValueIter<'map, 'key, T> {
    /// Creates a `ValueIter` from the given `map` and `key`.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    pub(crate) unsafe fn new(map: MapRef<'map>, key: Cow<'key, CStr>) -> Result<Self> {
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

macro_rules! impl_value_iter {
    ($type:ty, $func:ident, $process:expr) => (
        impl<'map, 'key> Iterator for ValueIter<'map, 'key, $type> {
            type Item = $type;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                if self.index == self.count {
                    return None;
                }

                let mut error = 0;
                let value = unsafe { self.map.$func(&self.key, self.index, &mut error) };
                debug_assert!(error == 0);
                let value = $process(&*self, value);
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

impl_value_iter!(i64, get_int_raw_unchecked, |_, x| x);
impl_value_iter!(f64, get_float_raw_unchecked, |_, x| x);
impl_value_iter!(
    &'map [u8],
    get_data_raw_unchecked,
    |v: &ValueIter<&'map [u8]>, x| {
        let mut error = 0;
        let size = unsafe {
            v.map
                .get_data_size_raw_unchecked(&v.key, v.index, &mut error)
        };
        debug_assert!(error == 0);
        debug_assert!(size >= 0);
        unsafe { slice::from_raw_parts(x as *const u8, size as usize) }
    }
);
impl_value_iter!(
    Node,
    get_node_raw_unchecked,
    |v: &ValueIter<Node>, x| unsafe { Node::from_ptr(v.map.api(), x) }
);
impl_value_iter!(
    Frame,
    get_frame_raw_unchecked,
    |v: &ValueIter<Frame>, x| unsafe { Frame::from_ptr(v.map.api(), x) }
);
impl_value_iter!(
    Function,
    get_function_raw_unchecked,
    |v: &ValueIter<Function>, x| unsafe { Function::from_ptr(v.map.api(), x) }
);
