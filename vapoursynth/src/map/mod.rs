//! VapourSynth maps.

use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ops::{Deref, DerefMut};
use std::{mem, ptr, result, slice};
use vapoursynth_sys as ffi;

use api::API;
use frame::{Frame, FrameRef};
use function::Function;
use node::Node;

mod errors;
pub use self::errors::{Error, InvalidKeyError, Result};

mod iterators;
pub use self::iterators::{Keys, ValueIter};

mod value;
pub use self::value::ValueType;

/// A VapourSynth map.
///
/// A map contains key-value pairs where the value is zero or more elements of a certain type.
// WARNING: use ONLY references to this type. The only thing this type is for is doing &ffi::VSMap
// and &mut ffi::VSMap without exposing the (unknown size) ffi type outside.
pub struct Map(ffi::VSMap);

#[doc(hidden)]
impl Deref for Map {
    type Target = ffi::VSMap;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(self) }
    }
}

#[doc(hidden)]
impl DerefMut for Map {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { mem::transmute(self) }
    }
}

/// An owned VapourSynth map.
///
/// A map contains key-value pairs where the value is zero or more elements of a certain type.
#[derive(Debug)]
pub struct OwnedMap {
    handle: *mut ffi::VSMap,
}

unsafe impl Send for Map {}
unsafe impl Sync for Map {}

impl Drop for OwnedMap {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            API::get_cached().free_map(&mut *self.handle);
        }
    }
}

impl Deref for OwnedMap {
    type Target = Map;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { Map::from_ptr(self.handle) }
    }
}

impl DerefMut for OwnedMap {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Map::from_mut_ptr(self.handle) }
    }
}

// impl Clone for Map {
//     fn clone(&self) -> Self {
//         let mut map = Map::new(self.api);
//
//         for i in 0..self.key_count() {
//             let key = self.key_raw(i);
//             let value = unsafe { self.values_raw_unchecked(key).unwrap() };
//
//             // TODO: this is stupid.
//             match value {
//                 ValueArray::Ints(xs) => unsafe {
//                     #[cfg_attr(feature = "cargo-clippy", allow(needless_borrow))]
//                     map.set_values_raw_unchecked(key, Values::IntArray(&xs));
//                 },
//                 ValueArray::Floats(xs) => unsafe {
//                     #[cfg_attr(feature = "cargo-clippy", allow(needless_borrow))]
//                     map.set_values_raw_unchecked(key, Values::FloatArray(&xs));
//                 },
//                 ValueArray::Data(xs) => unsafe {
//                     map.set_values_raw_unchecked(key, Values::Data(&mut xs.iter().map(|&x| x)));
//                 },
//                 ValueArray::Nodes(xs) => unsafe {
//                     map.set_values_raw_unchecked(key, Values::Nodes(&mut xs.iter()));
//                 },
//                 ValueArray::Frames(xs) => unsafe {
//                     map.set_values_raw_unchecked(key, Values::Frames(&mut xs.iter()));
//                 },
//                 ValueArray::Functions(xs) => unsafe {
//                     map.set_values_raw_unchecked(key, Values::Functions(&mut xs.iter()));
//                 },
//             }
//         }
//
//         map
//     }
// }

impl OwnedMap {
    /// Creates a new map.
    #[inline]
    pub fn new(api: API) -> Self {
        let handle = api.create_map();
        Self { handle }
    }
}

/// Turns a `prop_get_something()` error into a `Result`.
#[inline]
fn handle_get_prop_error(error: i32) -> Result<()> {
    if error == 0 {
        Ok(())
    } else {
        Err(match error {
            x if x == ffi::VSGetPropErrors::peUnset as i32 => Error::KeyNotFound,
            x if x == ffi::VSGetPropErrors::peType as i32 => Error::WrongValueType,
            x if x == ffi::VSGetPropErrors::peIndex as i32 => Error::IndexOutOfBounds,
            _ => unreachable!(),
        })
    }
}

/// Turns a `prop_set_something(paAppend)` error into a `Result`.
#[inline]
fn handle_append_prop_error(error: i32) -> Result<()> {
    if error != 0 {
        debug_assert!(error == 1);
        Err(Error::WrongValueType)
    } else {
        Ok(())
    }
}

impl Map {
    /// Converts a pointer to a map to a reference.
    ///
    /// # Safety
    /// The caller needs to ensure the pointer is valid, the lifetime is valid and there are no
    /// active mutable references to the map during the lifetime.
    #[inline]
    pub(crate) unsafe fn from_ptr<'a>(handle: *const ffi::VSMap) -> &'a Map {
        #[cfg_attr(feature = "cargo-clippy", allow(transmute_ptr_to_ref))]
        unsafe { mem::transmute(handle) }
    }

    /// Converts a mutable pointer to a map to a reference.
    ///
    /// # Safety
    /// The caller needs to ensure the pointer is valid, the lifetime is valid and there are no
    /// active references to the map during the lifetime.
    #[inline]
    pub(crate) unsafe fn from_mut_ptr<'a>(handle: *mut ffi::VSMap) -> &'a mut Map {
        #[cfg_attr(feature = "cargo-clippy", allow(transmute_ptr_to_ref))]
        unsafe { mem::transmute(handle) }
    }

    /// Checks if the key is valid. Valid keys start with an alphabetic character or an underscore,
    /// and contain only alphanumeric characters and underscores.
    pub fn is_key_valid(key: &str) -> result::Result<(), InvalidKeyError> {
        if key.is_empty() {
            return Err(InvalidKeyError::EmptyKey);
        }

        let mut chars = key.chars();

        let first = chars.next().unwrap();
        if !first.is_ascii_alphabetic() && first != '_' {
            return Err(InvalidKeyError::InvalidCharacter(0));
        }

        for (i, c) in chars.enumerate() {
            if !c.is_ascii_alphanumeric() && c != '_' {
                return Err(InvalidKeyError::InvalidCharacter(i + 1));
            }
        }

        Ok(())
    }

    /// Checks if the key is valid and makes it a `CString`.
    #[inline]
    pub(crate) fn make_raw_key(key: &str) -> Result<CString> {
        Map::is_key_valid(key)?;
        Ok(CString::new(key).unwrap())
    }

    /// Clears the map.
    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            API::get_cached().clear_map(self);
        }
    }

    /// Returns the error message contained in the map, if any.
    #[inline]
    pub fn error(&self) -> Option<Cow<str>> {
        let error_message = unsafe { API::get_cached().get_error(self) };
        if error_message.is_null() {
            return None;
        }

        let error_message = unsafe { CStr::from_ptr(error_message) };
        Some(error_message.to_string_lossy())
    }

    /// Adds an error message to a map. The map is cleared first.
    #[inline]
    pub fn set_error(&mut self, error_message: &str) -> Result<()> {
        let error_message = CString::new(error_message)?;
        unsafe {
            API::get_cached().set_error(self, error_message.as_ptr());
        }
        Ok(())
    }

    /// Returns the number of keys contained in a map.
    #[inline]
    pub fn key_count(&self) -> usize {
        let count = unsafe { API::get_cached().prop_num_keys(self) };
        debug_assert!(count >= 0);
        count as usize
    }

    /// Returns a key from a map.
    ///
    /// # Panics
    /// Panics if `index >= self.key_count()`.
    #[inline]
    pub(crate) fn key_raw(&self, index: usize) -> &CStr {
        assert!(index < self.key_count());
        let index = index as i32;

        unsafe { CStr::from_ptr(API::get_cached().prop_get_key(self, index)) }
    }

    /// Returns a key from a map.
    ///
    /// # Panics
    /// Panics if `index >= self.key_count()`.
    #[inline]
    pub fn key(&self, index: usize) -> &str {
        self.key_raw(index).to_str().unwrap()
    }

    /// Returns an iterator over all keys in a map.
    #[inline]
    pub fn keys(&self) -> Keys {
        Keys::new(self)
    }

    /// Returns the number of elements associated with a key in a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn value_count_raw_unchecked(&self, key: &CStr) -> Result<usize> {
        let rv = API::get_cached().prop_num_elements(self, key.as_ptr());
        if rv == -1 {
            Err(Error::KeyNotFound)
        } else {
            debug_assert!(rv >= 0);
            Ok(rv as usize)
        }
    }

    /// Returns the number of elements associated with a key in a map.
    #[inline]
    pub fn value_count(&self, key: &str) -> Result<usize> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.value_count_raw_unchecked(&key) }
    }

    /// Retrieves a value type from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn value_type_raw_unchecked(&self, key: &CStr) -> Result<ValueType> {
        match API::get_cached().prop_get_type(self, key.as_ptr()) {
            x if x == ffi::VSPropTypes::ptUnset as c_char => Err(Error::KeyNotFound),
            x if x == ffi::VSPropTypes::ptInt as c_char => Ok(ValueType::Int),
            x if x == ffi::VSPropTypes::ptFloat as c_char => Ok(ValueType::Float),
            x if x == ffi::VSPropTypes::ptData as c_char => Ok(ValueType::Data),
            x if x == ffi::VSPropTypes::ptNode as c_char => Ok(ValueType::Node),
            x if x == ffi::VSPropTypes::ptFrame as c_char => Ok(ValueType::Frame),
            x if x == ffi::VSPropTypes::ptFunction as c_char => Ok(ValueType::Function),
            _ => unreachable!(),
        }
    }

    /// Retrieves a value type from a map.
    #[inline]
    pub fn value_type(&self, key: &str) -> Result<ValueType> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.value_type_raw_unchecked(&key) }
    }

    /// Retrieves an integer from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_int(&self, key: &str) -> Result<i64> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_int_raw_unchecked(&key, 0) }
    }

    /// Retrieves integers from a map.
    #[inline]
    pub fn get_int_iter(&self, key: &str) -> Result<ValueIter<i64>> {
        let key = Map::make_raw_key(key)?;
        unsafe { ValueIter::<i64>::new(self, Cow::Owned(key)) }
    }

    /// Retrieves an array of integers from a map.
    ///
    /// This is faster than iterating over a `get_int_iter()`.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub fn get_int_array(&self, key: &str) -> Result<&[i64]> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_int_array_raw_unchecked(&key) }
    }

    /// Retrieves a floating point number from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_float(&self, key: &str) -> Result<f64> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_float_raw_unchecked(&key, 0) }
    }

    /// Retrieves an array of floating point numbers from a map.
    ///
    /// This is faster than iterating over a `get_float_iter()`.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub fn get_float_array(&self, key: &str) -> Result<&[f64]> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_float_array_raw_unchecked(&key) }
    }

    /// Retrieves floating point numbers from a map.
    #[inline]
    pub fn get_float_iter(&self, key: &str) -> Result<ValueIter<f64>> {
        let key = Map::make_raw_key(key)?;
        unsafe { ValueIter::<f64>::new(self, Cow::Owned(key)) }
    }

    /// Retrieves data from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_data(&self, key: &str) -> Result<&[u8]> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_data_raw_unchecked(&key, 0) }
    }

    /// Retrieves data from a map.
    #[inline]
    pub fn get_data_iter(&self, key: &str) -> Result<ValueIter<&[u8]>> {
        let key = Map::make_raw_key(key)?;
        unsafe { ValueIter::<&[u8]>::new(self, Cow::Owned(key)) }
    }

    /// Retrieves a node from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_node(&self, key: &str) -> Result<Node> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_node_raw_unchecked(&key, 0) }
    }

    /// Retrieves nodes from a map.
    #[inline]
    pub fn get_node_iter(&self, key: &str) -> Result<ValueIter<Node>> {
        let key = Map::make_raw_key(key)?;
        unsafe { ValueIter::<Node>::new(self, Cow::Owned(key)) }
    }

    /// Retrieves a frame from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_frame(&self, key: &str) -> Result<FrameRef> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_frame_raw_unchecked(&key, 0) }
    }

    /// Retrieves frames from a map.
    #[inline]
    pub fn get_frame_iter(&self, key: &str) -> Result<ValueIter<FrameRef>> {
        let key = Map::make_raw_key(key)?;
        unsafe { ValueIter::<FrameRef>::new(self, Cow::Owned(key)) }
    }

    /// Retrieves a function from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_function(&self, key: &str) -> Result<Function> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_function_raw_unchecked(&key, 0) }
    }

    /// Retrieves functions from a map.
    #[inline]
    pub fn get_function_iter(&self, key: &str) -> Result<ValueIter<Function>> {
        let key = Map::make_raw_key(key)?;
        unsafe { ValueIter::<Function>::new(self, Cow::Owned(key)) }
    }

    /// Retrieves an integer from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_int_raw_unchecked(&self, key: &CStr, index: i32) -> Result<i64> {
        let mut error = 0;
        let value = API::get_cached().prop_get_int(self, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        Ok(value)
    }

    /// Retrieves an array of integers from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub(crate) unsafe fn get_int_array_raw_unchecked(&self, key: &CStr) -> Result<&[i64]> {
        let mut error = 0;
        let value = API::get_cached().prop_get_int_array(self, key.as_ptr(), &mut error);
        handle_get_prop_error(error)?;

        let length = self.value_count_raw_unchecked(key).unwrap();
        Ok(slice::from_raw_parts(value, length))
    }

    /// Retrieves a floating point number from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_float_raw_unchecked(&self, key: &CStr, index: i32) -> Result<f64> {
        let mut error = 0;
        let value = API::get_cached().prop_get_float(self, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        Ok(value)
    }

    /// Retrieves an array of floating point numbers from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub(crate) unsafe fn get_float_array_raw_unchecked(&self, key: &CStr) -> Result<&[f64]> {
        let mut error = 0;
        let value = API::get_cached().prop_get_float_array(self, key.as_ptr(), &mut error);
        handle_get_prop_error(error)?;

        let length = self.value_count_raw_unchecked(key).unwrap();
        Ok(slice::from_raw_parts(value, length))
    }

    /// Retrieves data from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_data_raw_unchecked(&self, key: &CStr, index: i32) -> Result<&[u8]> {
        let mut error = 0;
        let value = API::get_cached().prop_get_data(self, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        let mut error = 0;
        let length = API::get_cached().prop_get_data_size(self, key.as_ptr(), index, &mut error);
        debug_assert!(error == 0);
        debug_assert!(length >= 0);

        Ok(slice::from_raw_parts(value as *const u8, length as usize))
    }

    /// Retrieves a node from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_node_raw_unchecked(&self, key: &CStr, index: i32) -> Result<Node> {
        let mut error = 0;
        let value = API::get_cached().prop_get_node(self, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        Ok(Node::from_ptr(value))
    }

    /// Retrieves a frame from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_frame_raw_unchecked(
        &self,
        key: &CStr,
        index: i32,
    ) -> Result<FrameRef> {
        let mut error = 0;
        let value = API::get_cached().prop_get_frame(self, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        Ok(FrameRef::from_ptr(value))
    }

    /// Retrieves a function from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_function_raw_unchecked(
        &self,
        key: &CStr,
        index: i32,
    ) -> Result<Function> {
        let mut error = 0;
        let value = API::get_cached().prop_get_func(self, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        Ok(Function::from_ptr(value))
    }

    /// Deletes the given key.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn delete_key_raw_unchecked(&mut self, key: &CStr) -> Result<()> {
        let result = API::get_cached().prop_delete_key(self, key.as_ptr());
        if result == 0 {
            Err(Error::KeyNotFound)
        } else {
            debug_assert!(result == 1);
            Ok(())
        }
    }

    /// Deletes the given key.
    #[inline]
    pub fn delete_key(&mut self, key: &str) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.delete_key_raw_unchecked(&key) }
    }

    /// Touches the key. That is, if the key exists, nothing happens, otherwise a key is created
    /// with no values associated.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    pub(crate) unsafe fn touch_raw_unchecked(&mut self, key: &CStr, value_type: ValueType) {
        macro_rules! touch_value {
            ($func:ident, $value:expr) => ({
                let result = API::get_cached().$func(
                    self,
                    key.as_ptr(),
                    $value,
                    ffi::VSPropAppendMode::paTouch
                );
                debug_assert!(result == 0);
            })
        }

        match value_type {
            ValueType::Int => touch_value!(prop_set_int, 0),
            ValueType::Float => touch_value!(prop_set_float, 0f64),
            ValueType::Data => touch_value!(prop_set_data, &[]),
            ValueType::Node => touch_value!(prop_set_node, ptr::null_mut()),
            ValueType::Frame => touch_value!(prop_set_frame, ptr::null()),
            ValueType::Function => touch_value!(prop_set_func, ptr::null_mut()),
        }
    }

    /// Touches the key. That is, if the key exists, nothing happens, otherwise a key is created
    /// with no values associated.
    #[inline]
    pub fn touch(&mut self, key: &str, value_type: ValueType) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.touch_raw_unchecked(&key, value_type);
        }
        Ok(())
    }

    /// Appends an integer to a map.
    #[inline]
    pub fn append_int(&mut self, key: &str, x: i64) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.append_int_raw_unchecked(&key, x) }
    }

    /// Appends a floating point number to a map.
    #[inline]
    pub fn append_float(&mut self, key: &str, x: f64) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.append_float_raw_unchecked(&key, x) }
    }

    /// Appends data to a map.
    #[inline]
    pub fn append_data(&mut self, key: &str, x: &[u8]) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.append_data_raw_unchecked(&key, x) }
    }

    /// Appends a node to a map.
    #[inline]
    pub fn append_node(&mut self, key: &str, x: &Node) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.append_node_raw_unchecked(&key, x) }
    }

    /// Appends a frame to a map.
    #[inline]
    pub fn append_frame(&mut self, key: &str, x: &Frame) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.append_frame_raw_unchecked(&key, x) }
    }

    /// Appends a function to a map.
    #[inline]
    pub fn append_function(&mut self, key: &str, x: &Function) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.append_function_raw_unchecked(&key, x) }
    }

    /// Appends an integer to a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn append_int_raw_unchecked(&mut self, key: &CStr, x: i64) -> Result<()> {
        let error =
            API::get_cached().prop_set_int(self, key.as_ptr(), x, ffi::VSPropAppendMode::paAppend);

        handle_append_prop_error(error)
    }

    /// Appends a floating point number to a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn append_float_raw_unchecked(&mut self, key: &CStr, x: f64) -> Result<()> {
        let error = API::get_cached().prop_set_float(
            self,
            key.as_ptr(),
            x,
            ffi::VSPropAppendMode::paAppend,
        );

        handle_append_prop_error(error)
    }

    /// Appends data to a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn append_data_raw_unchecked(&mut self, key: &CStr, x: &[u8]) -> Result<()> {
        let error =
            API::get_cached().prop_set_data(self, key.as_ptr(), x, ffi::VSPropAppendMode::paAppend);

        handle_append_prop_error(error)
    }

    /// Appends a node to a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn append_node_raw_unchecked(&mut self, key: &CStr, x: &Node) -> Result<()> {
        let error = API::get_cached().prop_set_node(
            self,
            key.as_ptr(),
            x.ptr(),
            ffi::VSPropAppendMode::paAppend,
        );

        handle_append_prop_error(error)
    }

    /// Appends a frame to a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn append_frame_raw_unchecked(
        &mut self,
        key: &CStr,
        x: &Frame,
    ) -> Result<()> {
        let error = API::get_cached().prop_set_frame(
            self,
            key.as_ptr(),
            x.ptr(),
            ffi::VSPropAppendMode::paAppend,
        );

        handle_append_prop_error(error)
    }

    /// Appends a function to a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn append_function_raw_unchecked(
        &mut self,
        key: &CStr,
        x: &Function,
    ) -> Result<()> {
        let error = API::get_cached().prop_set_func(
            self,
            key.as_ptr(),
            x.ptr(),
            ffi::VSPropAppendMode::paAppend,
        );

        handle_append_prop_error(error)
    }

    /// Sets a property value to an integer.
    #[inline]
    pub fn set_int(&mut self, key: &str, x: i64) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_int_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to an integer array.
    ///
    /// This is faster than calling `append_int()` in a loop.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub fn set_int_array(&mut self, key: &str, x: &[i64]) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_int_array_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to a floating point number.
    #[inline]
    pub fn set_float(&mut self, key: &str, x: f64) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_float_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to a floating point number array.
    ///
    /// This is faster than calling `append_float()` in a loop.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub fn set_float_array(&mut self, key: &str, x: &[f64]) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_float_array_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to data.
    #[inline]
    pub fn set_data(&mut self, key: &str, x: &[u8]) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_data_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to a node.
    #[inline]
    pub fn set_node(&mut self, key: &str, x: &Node) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_node_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to a frame.
    #[inline]
    pub fn set_frame(&mut self, key: &str, x: &Frame) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_frame_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to a function.
    #[inline]
    pub fn set_function(&mut self, key: &str, x: &Function) -> Result<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_function_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to an integer.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_int_raw_unchecked(&mut self, key: &CStr, x: i64) {
        let error =
            API::get_cached().prop_set_int(self, key.as_ptr(), x, ffi::VSPropAppendMode::paReplace);

        debug_assert!(error == 0);
    }

    /// Sets a property value to an integer array.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    ///
    /// # Panics
    /// Panics if `x.len()` can't fit in an `i32`.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub(crate) unsafe fn set_int_array_raw_unchecked(&mut self, key: &CStr, x: &[i64]) {
        let error = API::get_cached().prop_set_int_array(self, key.as_ptr(), x);

        debug_assert!(error == 0);
    }

    /// Sets a property value to a floating point number.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_float_raw_unchecked(&mut self, key: &CStr, x: f64) {
        let error = API::get_cached().prop_set_float(
            self,
            key.as_ptr(),
            x,
            ffi::VSPropAppendMode::paReplace,
        );

        debug_assert!(error == 0);
    }

    /// Sets a property value to a floating point number array.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    ///
    /// # Panics
    /// Panics if `x.len()` can't fit in an `i32`.
    #[cfg(feature = "gte-vapoursynth-api-31")]
    #[inline]
    pub(crate) unsafe fn set_float_array_raw_unchecked(&mut self, key: &CStr, x: &[f64]) {
        let error = API::get_cached().prop_set_float_array(self, key.as_ptr(), x);

        debug_assert!(error == 0);
    }

    /// Sets a property value to data.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_data_raw_unchecked(&mut self, key: &CStr, x: &[u8]) {
        let error = API::get_cached().prop_set_data(
            self,
            key.as_ptr(),
            x,
            ffi::VSPropAppendMode::paReplace,
        );

        debug_assert!(error == 0);
    }

    /// Sets a property value to a node.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_node_raw_unchecked(&mut self, key: &CStr, x: &Node) {
        let error = API::get_cached().prop_set_node(
            self,
            key.as_ptr(),
            x.ptr(),
            ffi::VSPropAppendMode::paReplace,
        );

        debug_assert!(error == 0);
    }

    /// Sets a property value to a frame.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_frame_raw_unchecked(&mut self, key: &CStr, x: &Frame) {
        let error = API::get_cached().prop_set_frame(
            self,
            key.as_ptr(),
            x.ptr(),
            ffi::VSPropAppendMode::paReplace,
        );

        debug_assert!(error == 0);
    }

    /// Sets a property value to a function.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_function_raw_unchecked(&mut self, key: &CStr, x: &Function) {
        let error = API::get_cached().prop_set_func(
            self,
            key.as_ptr(),
            x.ptr(),
            ffi::VSPropAppendMode::paReplace,
        );

        debug_assert!(error == 0);
    }
}
