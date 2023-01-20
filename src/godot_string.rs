use std::iter::FusedIterator;
use std::ops::{Bound, Range};
use std::{fmt, ops::RangeBounds};

use crate::godot_value::{GodotValue, TypecastError, ValueType};

#[derive(Debug, Clone)]
pub struct GodotString {
    value: GodotValue,

    len_: usize,
}

#[link(wasm_import_module = "godot_wasm")]
extern "C" {
    #[link_name = "string.len"]
    fn string_len(id: u32) -> u32;
    #[link_name = "string.read"]
    fn read_string(id: u32, ptr: *mut u8) -> u32;
    #[link_name = "string.write"]
    fn write_string(ptr: *const u8, len: u32) -> u32;
}

impl GodotString {
    #[inline]
    fn update_len(&mut self) {
        self.len_ = unsafe { string_len(self.value.to_raw()) as _ }
    }

    pub fn len(&self) -> usize {
        self.len_
    }

    pub fn extend_string(&self, s: &mut String) {
        let len = self.len();

        s.reserve(len);

        unsafe {
            // SAFETY: We need a vector for set_len()
            let v = s.as_mut_vec();
            // Assert we have enough space
            debug_assert!((v.capacity() - v.len()) >= len);

            // Write into the end of the string
            // SAFETY: Appending a valid string with valid string is always valid.
            let p = v.as_mut_ptr_range().end;
            let r = read_string(self.value.to_raw(), p);
            debug_assert_ne!(r, 0, "Read operation failed");

            // SAFETY: Successfully appended string.
            v.set_len(v.len() + len);
        }
    }

    pub fn write_string(&self, s: &mut String) {
        s.clear();
        self.extend_string(s);
    }

    pub fn extend_vector(&self, v: &mut Vec<u8>) {
        let len = self.len();

        v.reserve(len);

        unsafe {
            // Write into the end of the vector
            // SAFETY: We have enough space available.
            let p = v.as_mut_ptr_range().end;
            let r = read_string(self.value.to_raw(), p);
            debug_assert_ne!(r, 0, "Read operation failed");

            // SAFETY: Successfully extended vector.
            v.set_len(v.len() + len);
        }
    }

    pub fn to_vector(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        self.extend_vector(&mut ret);
        ret
    }
}

impl TryFrom<GodotValue> for GodotString {
    type Error = TypecastError;

    fn try_from(v: GodotValue) -> Result<Self, Self::Error> {
        if !v.is_string() {
            Err(TypecastError::new(ValueType::GodotString))
        } else {
            let mut ret = Self { value: v, len_: 0 };
            ret.update_len();
            Ok(ret)
        }
    }
}

impl From<GodotValue> for Option<GodotString> {
    #[inline]
    fn from(v: GodotValue) -> Self {
        v.try_into().ok()
    }
}

impl<T> From<T> for GodotString
where
    T: AsRef<str>,
{
    fn from(v: T) -> Self {
        let v = v.as_ref();
        unsafe {
            Self {
                value: GodotValue::from_raw(write_string(v.as_ptr(), v.len() as _)),
                len_: v.len(),
            }
        }
    }
}

impl From<GodotString> for GodotValue {
    fn from(v: GodotString) -> Self {
        v.value
    }
}

impl From<&'_ GodotString> for String {
    fn from(v: &GodotString) -> Self {
        let ptr = unsafe { v.value.to_raw() };

        // SAFETY: Length is the byte length to be allocated
        let len = unsafe { string_len(ptr) as _ };
        let mut ret = vec![0u8; len];

        // SAFETY: Read into allocated array
        let v = unsafe { read_string(ptr, ret.as_mut_ptr()) };
        debug_assert_ne!(v, 0, "Read operation failed");

        // SAFETY: Returned string is assumed to be UTF-8
        unsafe { String::from_utf8_unchecked(ret) }
    }
}

impl fmt::Display for GodotString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = self.into();
        write!(f, "{}", s)
    }
}

#[link(wasm_import_module = "godot_wasm")]
extern "C" {
    #[link_name = "string_array.len"]
    fn string_array_len(id: u32) -> u32;
    #[link_name = "string_array.get"]
    fn get_string_array(id: u32, i: u32) -> u32;
    #[link_name = "string_array.get_many"]
    fn get_many_string_array(id: u32, i: u32, start: *mut u32, end: *mut u32) -> u32;
    #[link_name = "string_array.build"]
    fn build_string_array(start: *const u32, end: *const u32) -> u32;
}

#[derive(Debug, Clone)]
pub struct StringArray {
    value: GodotValue,

    len_: usize,
}

impl StringArray {
    fn update_len(&mut self) {
        self.len_ = unsafe { string_array_len(self.value.to_raw()) as _ };
    }

    pub fn len(&self) -> usize {
        self.len_
    }

    pub fn get(&self, ix: usize) -> Option<GodotString> {
        if ix >= self.len() {
            None
        } else {
            // SAFETY: Get string array is always a string type
            let ret =
                unsafe { GodotValue::from_raw(get_string_array(self.value.to_raw(), ix as _)) };
            ret.into()
        }
    }

    pub fn slice<T>(&self, range: T) -> Option<Vec<GodotString>>
    where
        T: RangeBounds<usize>,
    {
        let range = Range {
            start: match range.start_bound() {
                Bound::Included(&v) => v,
                Bound::Excluded(&v) => v + 1,
                Bound::Unbounded => 0,
            },
            end: match range.end_bound() {
                Bound::Included(&v) => v + 1,
                Bound::Excluded(&v) => v,
                Bound::Unbounded => self.len(),
            },
        };
        if range.end > self.len() {
            None
        } else {
            let mut ret = vec![0u32; range.len()];

            let r = unsafe {
                let pr = ret.as_mut_ptr_range();
                get_many_string_array(self.value.to_raw(), range.start as _, pr.start, pr.end)
            };

            debug_assert_eq!(r, range.len() as _);

            Some(
                ret.into_iter()
                    .map(|v| unsafe { GodotValue::from_raw(v).try_into().unwrap() })
                    .collect(),
            )
        }
    }

    pub fn slice_iter<T>(&self, range: T) -> Iter<'_>
    where
        T: RangeBounds<usize>,
    {
        let range = Range {
            start: match range.start_bound() {
                Bound::Included(&v) => v,
                Bound::Excluded(&v) => v + 1,
                Bound::Unbounded => 0,
            },
            end: match range.end_bound() {
                Bound::Included(&v) => v + 1,
                Bound::Excluded(&v) => v,
                Bound::Unbounded => self.len(),
            },
        };
        if range.end > self.len() {
            panic!("Index out of bound! ({} >= {})", range.end, self.len());
        }

        Iter { arr: self, range }
    }
}

impl TryFrom<GodotValue> for StringArray {
    type Error = TypecastError;

    fn try_from(v: GodotValue) -> Result<Self, Self::Error> {
        if !v.is_string_array() {
            Err(TypecastError::new(ValueType::StringArray))
        } else {
            let mut ret = Self { value: v, len_: 0 };
            ret.update_len();
            Ok(ret)
        }
    }
}

impl From<StringArray> for GodotValue {
    fn from(v: StringArray) -> Self {
        v.value
    }
}

impl<'a> FromIterator<&'a GodotString> for StringArray {
    fn from_iter<T: IntoIterator<Item = &'a GodotString>>(it: T) -> Self {
        // SAFETY: We will drop all value later.
        let v: Vec<_> = it
            .into_iter()
            .map(|v| unsafe { v.value.to_raw() })
            .collect();

        let ret = unsafe {
            let r = v.as_ptr_range();
            // SAFETY: It is always a string array
            Self {
                value: GodotValue::from_raw(build_string_array(r.start, r.end)),
                len_: v.len(),
            }
        };

        ret
    }
}

impl FromIterator<GodotString> for StringArray {
    fn from_iter<T: IntoIterator<Item = GodotString>>(it: T) -> Self {
        // SAFETY: We will drop all value later.
        let v: Vec<_> = it
            .into_iter()
            .map(|v| unsafe { v.value.into_raw() })
            .collect();

        let ret = unsafe {
            let r = v.as_ptr_range();
            // SAFETY: It is always a string array
            Self {
                value: GodotValue::from_raw(build_string_array(r.start, r.end)),
                len_: v.len(),
            }
        };

        // Drop value in vector
        // SAFETY: Always a valid raw value.
        for i in v.into_iter() {
            unsafe { drop(GodotValue::from_raw(i)) }
        }

        ret
    }
}

pub struct Iter<'a> {
    arr: &'a StringArray,
    range: Range<usize>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = GodotString;

    fn next(&mut self) -> Option<Self::Item> {
        match self.range.next() {
            Some(v) => self.arr.get(v),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.range.next_back() {
            Some(v) => self.arr.get(v),
            None => None,
        }
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl<'a> FusedIterator for Iter<'a> {}

impl<'a> IntoIterator for &'a StringArray {
    type IntoIter = Iter<'a>;
    type Item = GodotString;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            range: 0..self.len(),
            arr: self,
        }
    }
}
