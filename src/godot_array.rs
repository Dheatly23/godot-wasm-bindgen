use crate::godot_value::{GodotValue, TypecastError, ValueType};

#[derive(Debug, Clone)]
pub struct GodotArray {
    value: GodotValue,
}

impl TryFrom<GodotValue> for GodotArray {
    type Error = TypecastError;

    fn try_from(value: GodotValue) -> Result<Self, TypecastError> {
        if value.is_array() {
            Ok(Self { value })
        } else {
            Err(TypecastError::new(ValueType::Array))
        }
    }
}

impl From<GodotValue> for Option<GodotArray> {
    fn from(value: GodotValue) -> Self {
        if value.is_array() {
            Some(GodotArray { value })
        } else {
            None
        }
    }
}

impl From<GodotArray> for GodotValue {
    fn from(value: GodotArray) -> Self {
        let GodotArray { value } = value;
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
    pub fn new() -> Self {
        Self {
            value: unsafe { GodotValue::from_raw(array_new()) },
        }
    }

    pub fn len(&self) -> usize {
        unsafe { array_len(self.value.to_raw()) as _ }
    }

    pub fn get(&self, ix: usize) -> GodotValue {
        unsafe { GodotValue::from_raw(array_get(self.value.to_raw(), ix as _)) }
    }

    pub fn set(&self, ix: usize, v: &GodotValue) {
        unsafe { array_set(self.value.to_raw(), ix as _, v.to_raw()) }
    }

    pub fn insert(&self, ix: usize, v: &GodotValue) {
        unsafe { array_insert(self.value.to_raw(), ix as _, v.to_raw()) }
    }

    pub fn remove(&self, ix: usize) {
        unsafe { array_remove(self.value.to_raw(), ix as _) }
    }

    pub fn erase(&self, v: &GodotValue) {
        unsafe { array_erase(self.value.to_raw(), v.to_raw()) }
    }

    pub fn resize(&self, ix: usize) {
        unsafe { array_resize(self.value.to_raw(), ix as _) }
    }

    pub fn push(&self, v: &GodotValue) {
        unsafe { array_push(self.value.to_raw(), v.to_raw()) }
    }

    pub fn pop(&self) -> GodotValue {
        unsafe { GodotValue::from_raw(array_pop(self.value.to_raw())) }
    }

    pub fn push_front(&self, v: &GodotValue) {
        unsafe { array_push_front(self.value.to_raw(), v.to_raw()) }
    }

    pub fn pop_front(&self) -> GodotValue {
        unsafe { GodotValue::from_raw(array_pop_front(self.value.to_raw())) }
    }

    pub fn count(&self, v: &GodotValue) -> usize {
        unsafe { array_count(self.value.to_raw(), v.to_raw()) as _ }
    }

    pub fn contains(&self, v: &GodotValue) -> bool {
        unsafe { array_contains(self.value.to_raw(), v.to_raw()) != 0 }
    }

    pub fn find(&self, v: &GodotValue, from: usize) -> Option<u32> {
        let ret = unsafe { array_find(self.value.to_raw(), v.to_raw(), from as _) };
        ret.try_into().ok()
    }

    pub fn rfind(&self, v: &GodotValue, from: usize) -> Option<u32> {
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
            }
        }
    }

    pub fn clear(&self) {
        unsafe { array_clear(self.value.to_raw()) }
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
}
