use std::fmt;
use std::marker::PhantomData;
use std::mem;

#[derive(Debug)]
#[repr(transparent)]
pub struct GodotValue {
    ptr: u32,
}

#[link(wasm_import_module = "godot_wasm")]
extern "C" {
    fn duplicate(ptr: u32) -> u32;
    fn delete(ptr: u32);
}

impl Clone for GodotValue {
    fn clone(&self) -> Self {
        let ptr = unsafe { duplicate(self.ptr) };
        debug_assert_ne!(self.ptr, ptr, "Duplicated pointer!");
        Self { ptr }
    }
}

impl Drop for GodotValue {
    fn drop(&mut self) {
        if self.ptr != 0 {
            unsafe {
                delete(self.ptr);
            }
        }
    }
}

macro_rules! typeis {
    ($(($vname:ident : $ifunc:ident => $iname:literal)),* $(,)?) => {
        #[link(wasm_import_module = "godot_wasm")]
        extern "C" {$(
            #[link_name = $iname]
            fn $ifunc(ptr: u32) -> u32;
        )*}

        impl GodotValue {$(
            pub fn $ifunc(&self) -> bool {
                if self.ptr == 0 {
                    false
                } else {
                    unsafe {
                        $ifunc(self.ptr) != 0
                    }
                }
            }
        )*}

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u8)]
        pub enum ValueType {
            Null = 0,
            $($vname),*
        }

        impl fmt::Display for ValueType {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", match self {
                    Self::Null => "Null",
                    $(Self::$vname => stringify!($vname)),*
                })
            }
        }

        impl<'a> From<&'a GodotValue> for ValueType {
            fn from(v: &'a GodotValue) -> Self {
                if v.is_null() { Self::Null }
                $( else if v.$ifunc() { Self::$vname } )*
                else { panic!("Unknown value!") }
            }
        }

        impl GodotValue {
            #[inline]
            pub fn value_type(&self) -> ValueType {
                ValueType::from(self)
            }
        }
    };
}

impl GodotValue {
    #[inline]
    pub unsafe fn to_raw(&self) -> u32 {
        self.ptr
    }

    #[inline]
    pub unsafe fn into_raw(self) -> u32 {
        let ret = self.ptr;
        mem::forget(self);
        ret
    }

    #[inline]
    pub unsafe fn from_raw(ptr: u32) -> Self {
        Self { ptr }
    }

    #[inline]
    pub const fn is_null(&self) -> bool {
        self.ptr == 0
    }

    #[inline]
    pub const fn is_nonnull(&self) -> bool {
        self.ptr != 0
    }
}

typeis!(
    (Bool: is_bool => "bool.is"),
    (I64: is_int => "int.is"),
    (F64: is_float => "float.is"),
    (GodotString: is_string => "string.is"),
    (Vector2: is_vector2 => "vector2.is"),
    (Rect2: is_rect2 => "rect2.is"),
    (Vector3: is_vector3 => "vector3.is"),
    (Transform2D: is_transform2d => "transform2d.is"),
    (Plane: is_plane => "plane.is"),
    (Quat: is_quat => "quat.is"),
    (Aabb: is_aabb => "aabb.is"),
    (Basis: is_basis => "basis.is"),
    (Transform: is_transform => "transform.is"),
    (Color: is_color => "color.is"),
    (Nodepath: is_nodepath => "nodepath.is"),
    (Rid: is_rid => "rid.is"),
    (Object: is_object => "object.is"),
    (Dictionary: is_dictionary => "dictionary.is"),
    (Array: is_array => "array.is"),
    (ByteArray: is_byte_array => "byte_array.is"),
    (IntArray: is_int_array => "int_array.is"),
    (FloatArray: is_float_array => "float_array.is"),
    (StringArray: is_string_array => "string_array.is"),
    (Vector2Array: is_vector2_array => "vector2_array.is"),
    (Vector3Array: is_vector3_array => "vector3_array.is"),
    (ColorArray: is_color_array => "color_array.is"),
);

pub struct NullValueError(PhantomData<&'static str>);

impl fmt::Display for NullValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Null pointer")
    }
}

impl fmt::Debug for NullValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

pub struct TypecastError {
    expect: ValueType,
}

impl TypecastError {
    pub(crate) fn new(expect: ValueType) -> Self {
        Self { expect }
    }
}

impl fmt::Display for TypecastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Casting error (expected {})", self.expect)
    }
}

impl fmt::Debug for TypecastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

macro_rules! typecast {
    (@typefrom $t:ty) => {$t};
    (@typeto $t:ty) => {$t};
    (@typefrom $t:ty, $t2:tt) => {$t};
    (@typeto $t:ty, $t2:tt) => {$t2};
    ($(($vname:ident => $ifunc:ident, $rfunc:ident => $rname:literal, $wfunc:ident => $wname:literal) => [$($t:tt)*]),* $(,)?) => {
        #[link(wasm_import_module = "godot_wasm")]
        extern "C" {$(
            #[link_name = $rname]
            fn $rfunc(id: u32, ptr: *mut typecast!(@typeto $($t)*)) -> u32;
            #[link_name = $wname]
            fn $wfunc(ptr: *const typecast!(@typeto $($t)*)) -> u32;
        )*}

        $(
            impl TryFrom<&'_ GodotValue> for typecast!(@typefrom $($t)*) {
                type Error = TypecastError;

                fn try_from(v: &GodotValue) -> Result<Self, Self::Error> {
                    if v.ptr == 0 {
                        Err(TypecastError::new(ValueType::$vname))
                    } else if !v.$ifunc() {
                        Err(TypecastError::new(ValueType::$vname))
                    } else {
                        let mut ret = <typecast!(@typeto $($t)*)>::default();

                        let v = unsafe { $rfunc(v.ptr, &mut ret as _) };
                        debug_assert_ne!(v, 0, "Read operation failed");

                        match v {
                            0 => Err(TypecastError::new(ValueType::$vname)),
                            _ => Ok(ret.into()),
                        }
                    }
                }
            }

            impl TryFrom<GodotValue> for typecast!(@typefrom $($t)*) {
                type Error = TypecastError;

                #[inline]
                fn try_from(v: GodotValue) -> Result<Self, Self::Error> {
                    Self::try_from(&v)
                }
            }

            impl From<&'_ GodotValue> for Option<typecast!(@typefrom $($t)*)> {
                #[inline]
                fn from(v: &GodotValue) -> Self {
                    v.try_into().ok()
                }
            }

            impl From<GodotValue> for Option<typecast!(@typefrom $($t)*)> {
                #[inline]
                fn from(v: GodotValue) -> Self {
                    v.try_into().ok()
                }
            }

            impl From<typecast!(@typefrom $($t)*)> for GodotValue {
                fn from(v: typecast!(@typefrom $($t)*)) -> Self {
                    Self {
                        ptr: unsafe { $wfunc(&v.into() as _) },
                    }
                }
            }
        )*
    };
}

macro_rules! typecast_proxy {
    ($($from:ty => $to:ty),* $(,)?) => {$(
        impl TryFrom<&'_ GodotValue> for $to {
            type Error = TypecastError;

            fn try_from(v: &GodotValue) -> Result<Self, Self::Error> {
                Ok(<$from>::try_from(v)? as _)
            }
        }

        impl TryFrom<GodotValue> for $to {
            type Error = TypecastError;

            fn try_from(v: GodotValue) -> Result<Self, Self::Error> {
                Self::try_from(&v)
            }
        }

        impl From<&'_ GodotValue> for Option<$to> {
            #[inline]
            fn from(v: &GodotValue) -> Self {
                v.try_into().ok()
            }
        }

        impl From<GodotValue> for Option<$to> {
            #[inline]
            fn from(v: GodotValue) -> Self {
                v.try_into().ok()
            }
        }

        impl From<$to> for GodotValue {
            fn from(v: $to) -> Self {
                Self::from(v as $from)
            }
        }
    )*};
}

#[derive(Default)]
#[repr(transparent)]
struct BoolWrapper(u32);

impl From<bool> for BoolWrapper {
    fn from(value: bool) -> Self {
        Self(if value { 1 } else { 0 })
    }
}

impl Into<bool> for BoolWrapper {
    fn into(self) -> bool {
        self.0 != 0
    }
}

typecast!(
    (Bool => is_bool, read_bool => "bool.read", write_bool => "bool.write") => [bool, BoolWrapper],
    (I64 => is_int, read_int => "int.read", write_int => "int.write") => [i64],
    (F64 => is_float, read_float => "float.read", write_float => "float.write") => [f64],
    (Vector2 => is_vector2, read_vector2 => "vector2.read", write_vector2 => "vector2.write") => [crate::primitive::Vector2],
    (Vector3 => is_vector3, read_vector3 => "vector3.read", write_vector3 => "vector3.write") => [crate::primitive::Vector3],
    (Rect2 => is_rect2, read_rect2 => "rect2.read", write_rect2 => "rect2.write") => [crate::primitive::Rect2],
    (Transform2D => is_transform2d, read_transform2d => "transform2d.read", write_transform2d => "transform2d.write") => [crate::primitive::Transform2D],
    (Plane => is_plane, read_plane => "plane.read", write_plane => "plane.write") => [crate::primitive::Plane],
    (Quat => is_quat, read_quat => "quat.read", write_quat => "quat.write") => [crate::primitive::Quat],
    (Aabb => is_aabb, read_aabb => "aabb.read", write_aabb => "aabb.write") => [crate::primitive::Aabb],
    (Basis => is_basis, read_basis => "basis.read", write_basis => "basis.write") => [crate::primitive::Basis],
    (Transform => is_transform, read_transform => "transform.read", write_transform => "transform.write") => [crate::primitive::Transform],
    (Color => is_color, read_color => "color.read", write_color => "color.write") => [crate::primitive::Color],
);

typecast_proxy!(
    i64 => u8,
    i64 => i8,
    i64 => u16,
    i64 => i16,
    i64 => u32,
    i64 => i32,
    f64 => f32,
);
