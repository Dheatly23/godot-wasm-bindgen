use std::fmt;
use std::marker::PhantomData;
use std::mem;

#[derive(Debug)]
#[repr(transparent)]
pub struct GodotValue {
    ptr: u32,
    phantom: PhantomData<*const u32>,
}

assert_not_impl_any!(GodotValue: Send, Sync);

#[link(wasm_import_module = "godot_wasm")]
extern "C" {
    fn duplicate(ptr: u32) -> u32;
    fn delete(ptr: u32);
}

impl Clone for GodotValue {
    fn clone(&self) -> Self {
        unsafe {
            if self.ptr != 0 {
                let ptr = duplicate(self.ptr);
                debug_assert_ne!(self.ptr, ptr, "Duplicated pointer!");
                Self::from_raw(ptr)
            } else {
                Self::from_raw(0)
            }
        }
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
    ($(($vname:ident = $vnum:literal : $ifunc:ident => $iname:literal)),* $(,)?) => {
        #[link(wasm_import_module = "godot_wasm")]
        extern "C" {
            $(
            #[link_name = $iname]
            fn $ifunc(ptr: u32) -> u32;
            )*
            #[link_name = "variant_type"]
            fn variant_type(ptr: u32) -> u32;
        }

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
            $($vname = $vnum),*
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
                if v.is_null() {
                    return Self::Null;
                }

                let v = unsafe { variant_type(v.ptr) };
                match v {
                    0 => Self::Null,
                    $( $vnum => Self::$vname, )*
                    _ => unreachable!("Invalid number"),
                }
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
        Self {
            ptr,
            phantom: PhantomData,
        }
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
    (Bool = 1: is_bool => "bool.is"),
    (I64 = 2: is_int => "int.is"),
    (F64 = 3: is_float => "float.is"),
    (GodotString = 4: is_string => "string.is"),
    (Vector2 = 5: is_vector2 => "vector2.is"),
    (Rect2 = 6: is_rect2 => "rect2.is"),
    (Vector3 = 7: is_vector3 => "vector3.is"),
    (Transform2D = 8: is_transform2d => "transform2d.is"),
    (Plane = 9: is_plane => "plane.is"),
    (Quat = 10: is_quat => "quat.is"),
    (Aabb = 11: is_aabb => "aabb.is"),
    (Basis = 12: is_basis => "basis.is"),
    (Transform = 13: is_transform => "transform.is"),
    (Color = 14: is_color => "color.is"),
    (Nodepath = 15: is_nodepath => "nodepath.is"),
    (Rid = 16: is_rid => "rid.is"),
    (Object = 17: is_object => "object.is"),
    (Dictionary = 18: is_dictionary => "dictionary.is"),
    (Array = 19: is_array => "array.is"),
    (ByteArray = 20: is_byte_array => "byte_array.is"),
    (IntArray = 21: is_int_array => "int_array.is"),
    (FloatArray = 22: is_float_array => "float_array.is"),
    (StringArray = 23: is_string_array => "string_array.is"),
    (Vector2Array = 24: is_vector2_array => "vector2_array.is"),
    (Vector3Array = 25: is_vector3_array => "vector3_array.is"),
    (ColorArray = 26: is_color_array => "color_array.is"),
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
    got: ValueType,
}

impl TypecastError {
    pub(crate) fn new(expect: ValueType, got: ValueType) -> Self {
        Self { expect, got }
    }
}

impl fmt::Display for TypecastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Casting error (expected {}, got {})",
            self.expect, self.got
        )
    }
}

impl fmt::Debug for TypecastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

pub struct TypecastErrorOwned {
    pub original: GodotValue,

    expect: ValueType,
    got: ValueType,
}

impl TypecastErrorOwned {
    pub(crate) fn new(original: GodotValue, expect: ValueType, got: ValueType) -> Self {
        Self {
            original,
            expect,
            got,
        }
    }

    pub(crate) fn from_typecast_error(original: GodotValue, error: TypecastError) -> Self {
        let TypecastError { expect, got } = error;
        Self {
            original,
            expect,
            got,
        }
    }
}

impl fmt::Display for TypecastErrorOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Casting error (expected {}, got {})",
            self.expect, self.got
        )
    }
}

impl fmt::Debug for TypecastErrorOwned {
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
                    match v.into() {
                        ValueType::$vname => {
                            let mut ret = <typecast!(@typeto $($t)*)>::default();

                            let v = unsafe { $rfunc(v.ptr, &mut ret as _) };
                            debug_assert_ne!(v, 0, "Read operation failed");

                            Ok(ret.into())
                        },
                        v => Err(TypecastError::new(ValueType::$vname, v)),
                    }
                }
            }

            impl TryFrom<GodotValue> for typecast!(@typefrom $($t)*) {
                type Error = TypecastErrorOwned;

                #[inline]
                fn try_from(v: GodotValue) -> Result<Self, Self::Error> {
                    Self::try_from(&v).map_err(|e| Self::Error::from_typecast_error(v, e))
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
                    unsafe { Self::from_raw($wfunc(&v.into() as _)) }
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
            type Error = TypecastErrorOwned;

            fn try_from(v: GodotValue) -> Result<Self, Self::Error> {
                <$from>::try_from(v).map(|v| v as _)
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

macro_rules! typecast_pool {
    ($($t:ty : $l:literal => [
        $vname:ident => $ifunc:ident,
        $lfunc:ident => $lname:literal,
        $rfunc:ident => $rname:literal,
        $wfunc:ident => $wname:literal
    ]),* $(,)?) => {
        #[link(wasm_import_module = "godot_wasm")]
        extern "C" {$(
            #[link_name = $lname]
            fn $lfunc(id: u32) -> u32;
            #[link_name = $rname]
            fn $rfunc(id: u32, ptr: *mut $t) -> u32;
            #[link_name = $wname]
            fn $wfunc(ptr: *const $t, n: u32) -> u32;
        )*}

        $(
            assert_eq_size!($t, [u8; $l]);

            impl TryFrom<&'_ GodotValue> for Vec<$t> {
                type Error = TypecastError;

                fn try_from(v: &GodotValue) -> Result<Self, Self::Error> {
                    match v.into() {
                        ValueType::$vname => {
                            let len = unsafe { $lfunc(v.ptr) as _};
                            let mut ret = vec![<$t>::default(); len];

                            let v = unsafe { $rfunc(v.ptr, ret.as_mut_ptr()) };
                            debug_assert_ne!(v, 0, "Read operation failed");

                            Ok(ret.into())
                        },
                        v => Err(TypecastError::new(ValueType::$vname, v)),
                    }
                }
            }

            impl TryFrom<GodotValue> for Vec<$t> {
                type Error = TypecastErrorOwned;

                #[inline]
                fn try_from(v: GodotValue) -> Result<Self, Self::Error> {
                    Self::try_from(&v).map_err(|e| Self::Error::from_typecast_error(v, e))
                }
            }

            impl From<&'_ GodotValue> for Option<Vec<$t>> {
                #[inline]
                fn from(v: &GodotValue) -> Self {
                    v.try_into().ok()
                }
            }

            impl From<GodotValue> for Option<Vec<$t>> {
                #[inline]
                fn from(v: GodotValue) -> Self {
                    v.try_into().ok()
                }
            }

            impl From<Vec<$t>> for GodotValue {
                fn from(v: Vec<$t>) -> Self {
                    unsafe { Self::from_raw($wfunc(v.as_ptr(), v.len() as _)) }
                }
            }

            impl From<&[$t]> for GodotValue {
                fn from(v: &[$t]) -> Self {
                    unsafe { Self::from_raw($wfunc(v.as_ptr(), v.len() as _)) }
                }
            }
        )*
    };
}

typecast_pool!(
    u8: 1 => [
        ByteArray => is_byte_array,
        len_byte_array => "byte_array.len",
        read_byte_array => "byte_array.read",
        write_byte_array => "byte_array.write"
    ],
    u32: 4 => [
        IntArray => is_int_array,
        len_int_array => "int_array.len",
        read_int_array => "int_array.read",
        write_int_array => "int_array.write"
    ],
    f32: 4 => [
        FloatArray => is_float_array,
        len_float_array => "float_array.len",
        read_float_array => "float_array.read",
        write_float_array => "float_array.write"
    ],
    crate::primitive::Vector2: 8 => [
        Vector2Array => is_vector2_array,
        len_vector2_array => "vector2_array.len",
        read_vector2_array => "vector2_array.read",
        write_vector2_array => "vector2_array.write"
    ],
    crate::primitive::Vector3: 12 => [
        Vector3Array => is_vector3_array,
        len_vector3_array => "vector3_array.len",
        read_vector3_array => "vector3_array.read",
        write_vector3_array => "vector3_array.write"
    ],
    crate::primitive::Color: 16 => [
        ColorArray => is_color_array,
        len_color_array => "color_array.len",
        read_color_array => "color_array.read",
        write_color_array => "color_array.write"
    ],
);
