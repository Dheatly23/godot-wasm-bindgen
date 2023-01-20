use std::iter::FusedIterator;
use std::ops::Range;

use crate::godot_value::{GodotValue, TypecastError, ValueType};

#[derive(Debug, Clone)]
pub struct GodotArray {
    value: GodotValue,

    len_: usize,
}

impl TryFrom<GodotValue> for GodotArray {
    type Error = TypecastError;

    fn try_from(value: GodotValue) -> Result<Self, TypecastError> {
        if value.is_array() {
            let mut ret = Self { value, len_: 0 };
            ret.update_len();
            Ok(ret)
        } else {
            Err(TypecastError::new(ValueType::Array))
        }
    }
}

impl From<GodotValue> for Option<GodotArray> {
    fn from(value: GodotValue) -> Self {
        value.try_into().ok()
    }
}

impl From<GodotArray> for GodotValue {
    fn from(value: GodotArray) -> Self {
        let GodotArray { value, .. } = value;
        value
    }
}

#[link(wasm_import_module = "godot_wasm")]
extern "C" {
    #[link_name = "array.new"]
    fn array_new() -> u32;
    #[link_name = "array.len"]
    fn array_len(ptr: u32) -> i32;
    #[link_name = "array.get"]
    fn array_get(ptr: u32, i: u32) -> u32;
    #[link_name = "array.set"]
    fn array_set(ptr: u32, i: u32, x: u32);
    #[link_name = "array.remove"]
    fn array_remove(ptr: u32, i: u32);
    #[link_name = "array.erase"]
    fn array_erase(ptr: u32, x: u32);
    #[link_name = "array.resize"]
    fn array_resize(ptr: u32, i: u32);
    #[link_name = "array.push"]
    fn array_push(ptr: u32, v: u32);
    #[link_name = "array.pop"]
    fn array_pop(ptr: u32) -> u32;
    #[link_name = "array.push_front"]
    fn array_push_front(ptr: u32, v: u32);
    #[link_name = "array.pop_front"]
    fn array_pop_front(ptr: u32) -> u32;
    #[link_name = "array.insert"]
    fn array_insert(ptr: u32, i: u32, x: u32);
    #[link_name = "array.count"]
    fn array_count(ptr: u32, x: u32) -> u32;
    #[link_name = "array.contains"]
    fn array_contains(ptr: u32, x: u32) -> u32;
    #[link_name = "array.find"]
    fn array_find(ptr: u32, x: u32, from: u32) -> i32;
    #[link_name = "array.rfind"]
    fn array_rfind(ptr: u32, x: u32, from: u32) -> i32;
    #[link_name = "array.find_last"]
    fn array_find_last(ptr: u32, x: u32) -> i32;
    #[link_name = "array.duplicate"]
    fn array_duplicate(ptr: u32) -> u32;
    #[link_name = "array.clear"]
    fn array_clear(ptr: u32);
    #[link_name = "array.sort"]
    fn array_sort(ptr: u32);
}

impl GodotArray {
    #[inline]
    fn update_len(&mut self) {
        self.len_ = unsafe { array_len(self.value.to_raw()) as _ }
    }

    #[inline]
    fn check_len(&self, ix: usize) {
        if ix >= self.len_ {
            panic!("Index out of bounds! ({} >= {})", ix, self.len_);
        }
    }

    #[cfg(debug_assertions)]
    #[inline]
    fn debug_len(&self) {
        let len = unsafe { array_len(self.value.to_raw()) as _ };
        debug_assert_eq!(
            self.len_, len,
            "Length mismatch! ({} != {})",
            self.len_, len
        );
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    fn debug_len(&self) {}

    pub fn new() -> Self {
        Self {
            value: unsafe { GodotValue::from_raw(array_new()) },
            len_: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len_
    }

    pub fn get(&self, ix: usize) -> GodotValue {
        self.check_len(ix);
        unsafe { GodotValue::from_raw(array_get(self.value.to_raw(), ix as _)) }
    }

    pub fn set(&mut self, ix: usize, v: &GodotValue) {
        self.check_len(ix);
        unsafe { array_set(self.value.to_raw(), ix as _, v.to_raw()) }
    }

    pub fn insert(&mut self, ix: usize, v: &GodotValue) {
        self.check_len(ix);
        unsafe { array_insert(self.value.to_raw(), ix as _, v.to_raw()) }
        self.len_ += 1;
        self.debug_len();
    }

    pub fn remove(&mut self, ix: usize) {
        self.check_len(ix);
        unsafe { array_remove(self.value.to_raw(), ix as _) }
        self.len_ -= 1;
        self.debug_len();
    }

    pub fn erase(&mut self, v: &GodotValue) {
        unsafe { array_erase(self.value.to_raw(), v.to_raw()) }
        self.update_len();
    }

    pub fn resize(&mut self, ix: usize) {
        unsafe { array_resize(self.value.to_raw(), ix as _) }
        self.len_ = ix;
        self.debug_len();
    }

    pub fn push(&mut self, v: &GodotValue) {
        unsafe { array_push(self.value.to_raw(), v.to_raw()) }
        self.len_ += 1;
        self.debug_len();
    }

    pub fn pop(&mut self) -> GodotValue {
        self.check_len(0);
        let ret = unsafe { GodotValue::from_raw(array_pop(self.value.to_raw())) };
        self.len_ -= 1;
        self.debug_len();
        ret
    }

    pub fn push_front(&mut self, v: &GodotValue) {
        unsafe { array_push_front(self.value.to_raw(), v.to_raw()) }
        self.len_ += 1;
        self.debug_len();
    }

    pub fn pop_front(&mut self) -> GodotValue {
        self.check_len(0);
        let ret = unsafe { GodotValue::from_raw(array_pop_front(self.value.to_raw())) };
        self.len_ -= 1;
        self.debug_len();
        ret
    }

    pub fn count(&self, v: &GodotValue) -> usize {
        unsafe { array_count(self.value.to_raw(), v.to_raw()) as _ }
    }

    pub fn contains(&self, v: &GodotValue) -> bool {
        unsafe { array_contains(self.value.to_raw(), v.to_raw()) != 0 }
    }

    pub fn find(&self, v: &GodotValue, from: usize) -> Option<u32> {
        self.check_len(from);
        let ret = unsafe { array_find(self.value.to_raw(), v.to_raw(), from as _) };
        ret.try_into().ok()
    }

    pub fn rfind(&self, v: &GodotValue, from: usize) -> Option<u32> {
        self.check_len(from);
        let ret = unsafe { array_rfind(self.value.to_raw(), v.to_raw(), from as _) };
        ret.try_into().ok()
    }

    pub fn find_last(&self, v: &GodotValue) -> Option<u32> {
        let ret = unsafe { array_find_last(self.value.to_raw(), v.to_raw()) };
        ret.try_into().ok()
    }

    pub fn duplicate(&self) -> GodotArray {
        unsafe {
            Self {
                value: GodotValue::from_raw(array_duplicate(self.value.to_raw())),
                len_: self.len_,
            }
        }
    }

    pub fn clear(&mut self) {
        unsafe { array_clear(self.value.to_raw()) }
        self.len_ = 0;
        self.debug_len();
    }

    pub fn sort(&self) {
        unsafe { array_sort(self.value.to_raw()) }
    }

    pub fn to_vec(&self) -> Vec<GodotValue> {
        let l = self.len();
        let mut ret = Vec::with_capacity(l);
        for i in 0..l {
            ret.push(self.get(i))
        }
        ret
    }

    pub fn from_slice(s: &[GodotValue]) -> Self {
        let mut ret = Self::new();
        for i in s {
            ret.push(i);
        }
        ret
    }

    #[inline]
    pub fn slice_iter(&self, start: usize, end: usize) -> Iter<'_> {
        Iter {
            array: self,
            range: start..end,
        }
    }
}

pub struct Iter<'a> {
    array: &'a GodotArray,
    range: Range<usize>,
}

impl<'a> IntoIterator for &'a GodotArray {
    type Item = GodotValue;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            array: self,
            range: 0..self.len(),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = GodotValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self.range.next() {
            Some(i) => Some(self.array.get(i)),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.range.next_back() {
            Some(i) => Some(self.array.get(i)),
            None => None,
        }
    }
}

impl<'a> FusedIterator for Iter<'a> {}

impl Extend<GodotValue> for GodotArray {
    fn extend<T: IntoIterator<Item = GodotValue>>(&mut self, iter: T) {
        for i in iter {
            self.push(&i);
        }
    }
}

impl<'a> Extend<&'a GodotValue> for GodotArray {
    fn extend<T: IntoIterator<Item = &'a GodotValue>>(&mut self, iter: T) {
        for i in iter {
            self.push(i);
        }
    }
}

impl FromIterator<GodotValue> for GodotArray {
    fn from_iter<T: IntoIterator<Item = GodotValue>>(iter: T) -> Self {
        let mut ret = Self::new();
        ret.extend(iter);
        ret
    }
}

impl<'a> FromIterator<&'a GodotValue> for GodotArray {
    fn from_iter<T: IntoIterator<Item = &'a GodotValue>>(iter: T) -> Self {
        let mut ret = Self::new();
        ret.extend(iter);
        ret
    }
}
