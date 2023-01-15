#![doc(hidden)]

use std::marker::PhantomData;

pub use crate::godot_value::GodotValue;

#[repr(u8)]
enum DataTypeEnum {
    U8 = 1,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    GodotValue,
}

pub struct DataTypeValue<T>(PhantomData<T>);

impl DataTypeValue<u8> {
    pub const fn value() -> u8 {
        DataTypeEnum::U8 as _
    }
}

impl DataTypeValue<i8> {
    pub const fn value() -> u8 {
        DataTypeEnum::I8 as _
    }
}

impl DataTypeValue<u16> {
    pub const fn value() -> u8 {
        DataTypeEnum::U16 as _
    }
}

impl DataTypeValue<i16> {
    pub const fn value() -> u8 {
        DataTypeEnum::I16 as _
    }
}

impl DataTypeValue<u32> {
    pub const fn value() -> u8 {
        DataTypeEnum::U32 as _
    }
}

impl DataTypeValue<i32> {
    pub const fn value() -> u8 {
        DataTypeEnum::I32 as _
    }
}

impl DataTypeValue<u64> {
    pub const fn value() -> u8 {
        DataTypeEnum::U64 as _
    }
}

impl DataTypeValue<i64> {
    pub const fn value() -> u8 {
        DataTypeEnum::I64 as _
    }
}

impl DataTypeValue<f32> {
    pub const fn value() -> u8 {
        DataTypeEnum::F32 as _
    }
}

impl DataTypeValue<f64> {
    pub const fn value() -> u8 {
        DataTypeEnum::F64 as _
    }
}

impl DataTypeValue<GodotValue> {
    pub const fn value() -> u8 {
        DataTypeEnum::GodotValue as _
    }
}
