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
