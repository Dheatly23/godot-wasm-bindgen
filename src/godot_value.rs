use std::fmt;
use std::marker::PhantomData;

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
        Self {
            ptr: unsafe { duplicate(self.ptr) },
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
    ($($fname:ident),* $(,)?) => {
        #[link(wasm_import_module = "godot_wasm")]
        extern "C" {$(
            fn $fname(ptr: u32) -> u32;
        )*}

        impl GodotValue {$(
            pub fn $fname(&self) -> bool {
                if self.ptr == 0 {
                    false
                } else {
                    unsafe {
                        $fname(self.ptr) != 0
                    }
                }
            }
        )*}
    };
}

impl GodotValue {
    pub fn is_null(&self) -> bool {
        self.ptr == 0
    }

    pub fn is_nonnull(&self) -> bool {
        self.ptr != 0
    }
}

typeis!(
    is_bool,
    is_int,
    is_float,
    is_string,
    is_vector2,
    is_rect2,
    is_vector3,
    is_transform2d,
    is_plane,
    is_quat,
    is_aabb,
    is_basis,
    is_transform,
    is_color,
    is_nodepath,
    is_rid,
    is_object,
    is_dictionary,
    is_array,
    is_byte_array,
    is_int_array,
    is_float_array,
    is_string_array,
    is_vector2_array,
    is_vector3_array,
    is_color_array,
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

macro_rules! typecast {
    (@typefrom $t:ty) => {$t};
    (@typeto $t:ty) => {$t};
    (@typefrom $t:ty, $t2:tt) => {$t};
    (@typeto $t:ty, $t2:tt) => {$t2};
    ($(($rfunc:ident => $rname:literal, $wfunc:ident => $wname:literal) => [$($t:tt)*]),* $(,)?) => {
        #[link(wasm_import_module = "godot_wasm")]
        extern "C" {$(
            #[link_name = $rname]
            fn $rfunc(id: u32, ptr: *mut typecast!(@typeto $($t)*)) -> u32;
            #[link_name = $wname]
            fn $wfunc(ptr: *const typecast!(@typeto $($t)*)) -> u32;
        )*}

        $(
            impl TryFrom<&'_ GodotValue> for typecast!(@typefrom $($t)*) {
                type Error = NullValueError;

                fn try_from(v: &GodotValue) -> Result<Self, Self::Error> {
                    if v.ptr == 0 {
                        Err(NullValueError(PhantomData))
                    } else {
                        let mut ret = <typecast!(@typeto $($t)*)>::default();

                        let v = unsafe { $rfunc(v.ptr, &mut ret as _) };
                        debug_assert_ne!(v, 0, "Read operation failed");

                        match v {
                            0 => Err(NullValueError(PhantomData)),
                            _ => Ok(ret.into()),
                        }
                    }
                }
            }

            impl TryFrom<GodotValue> for typecast!(@typefrom $($t)*) {
                type Error = NullValueError;

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
            type Error = NullValueError;

            fn try_from(v: &GodotValue) -> Result<Self, Self::Error> {
                Ok(<$from>::try_from(v)? as _)
            }
        }

        impl TryFrom<GodotValue> for $to {
            type Error = NullValueError;

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
    (read_bool => "bool.read", write_bool => "bool.write") => [bool, BoolWrapper],
    (read_int => "int.read", write_int => "int.write") => [i64],
    (read_float => "float.read", write_float => "float.write") => [f64],
    (read_vector2 => "vector2.read", write_vector2 => "vector2.write") => [crate::primitive::Vector2],
    (read_vector3 => "vector3.read", write_vector3 => "vector3.write") => [crate::primitive::Vector3],
    (read_rect2 => "rect2.read", write_rect2 => "rect2.write") => [crate::primitive::Rect2],
    (read_transform2d => "transform2d.read", write_transform2d => "transform2d.write") => [crate::primitive::Transform2D],
    (read_plane => "plane.read", write_plane => "plane.write") => [crate::primitive::Plane],
    (read_quat => "quat.read", write_quat => "quat.write") => [crate::primitive::Quat],
    (read_aabb => "aabb.read", write_aabb => "aabb.write") => [crate::primitive::Aabb],
    (read_basis => "basis.read", write_basis => "basis.write") => [crate::primitive::Basis],
    (read_transform => "transform.read", write_transform => "transform.write") => [crate::primitive::Transform],
    (read_color => "color.read", write_color => "color.write") => [crate::primitive::Color],
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
