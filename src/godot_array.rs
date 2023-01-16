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

    pub fn to_vec(&self) -> Vec<GodotValue> {
        let mut ret = Vec::new();
        for i in 0..self.len() {
            ret.push(self.get(i))
        }
        ret
    }
}
