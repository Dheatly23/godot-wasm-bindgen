use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Rect2 {
    pub position: Vector2,
    pub size: Vector2,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Transform2D {
    pub a: Vector2,
    pub b: Vector2,
    pub origin: Vector2,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Plane {
    pub normal: Vector3,
    pub d: f32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Aabb {
    pub position: Vector3,
    pub size: Vector3,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Basis {
    pub elements: [Vector3; 3],
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Transform {
    pub basis: Basis,
    pub origin: Vector3,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl fmt::Display for Vector2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { x, y } = self;
        write!(f, "Vector2({x:.3} {y:.3})")
    }
}

impl fmt::Display for Vector3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { x, y, z } = self;
        write!(f, "Vector3({x:.3} {y:.3} {z:.3})")
    }
}

impl fmt::Display for Quat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { x, y, z, w } = self;
        write!(f, "Quat({x:.3} {y:.3} {z:.3} {w:.3})")
    }
}

impl fmt::Display for Rect2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            position: Vector2 { x: x1, y: y1 },
            size: Vector2 { x: x2, y: y2 },
        } = self;
        write!(
            f,
            "Rect2(({x1:.3} {y1:.3})-({x2:.3} {y2:.3}))",
            x2 = x1 + x2,
            y2 = y1 + y2,
        )
    }
}

impl fmt::Display for Transform2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            a: Vector2 { x: v11, y: v21 },
            b: Vector2 { x: v12, y: v22 },
            origin: Vector2 { x: v13, y: v23 },
        } = self;
        writeln!(f, "Transform2D[")?;
        writeln!(f, "  {v11:.3} {v12:.3} {v13:.3}")?;
        writeln!(f, "  {v21:.3} {v22:.3} {v23:.3}")?;
        write!(f, "]")
    }
}

impl fmt::Display for Plane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            normal: Vector3 { x, y, z },
            d,
        } = self;
        write!(f, "Plane({x:.3} {y:.3} {z:.3} {d:.3})")
    }
}

impl fmt::Display for Aabb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            position:
                Vector3 {
                    x: x1,
                    y: y1,
                    z: z1,
                },
            size:
                Vector3 {
                    x: x2,
                    y: y2,
                    z: z2,
                },
        } = self;
        write!(
            f,
            "Aabb(({x1:.3} {y1:.3} {z1:.3})-({x2:.3} {y2:.3} {z2:.3}))",
            x2 = x1 + x2,
            y2 = y1 + y2,
            z2 = z1 + z2,
        )
    }
}

impl fmt::Display for Basis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            elements:
                [Vector3 {
                    x: v11,
                    y: v21,
                    z: v31,
                }, Vector3 {
                    x: v12,
                    y: v22,
                    z: v32,
                }, Vector3 {
                    x: v13,
                    y: v23,
                    z: v33,
                }],
        } = self;
        writeln!(f, "Basis[")?;
        writeln!(f, "  {v11:.3} {v12:.3} {v13:.3}")?;
        writeln!(f, "  {v21:.3} {v22:.3} {v23:.3}")?;
        writeln!(f, "  {v31:.3} {v32:.3} {v33:.3}")?;
        write!(f, "]")
    }
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            basis:
                Basis {
                    elements:
                        [Vector3 {
                            x: v11,
                            y: v21,
                            z: v31,
                        }, Vector3 {
                            x: v12,
                            y: v22,
                            z: v32,
                        }, Vector3 {
                            x: v13,
                            y: v23,
                            z: v33,
                        }],
                },
            origin:
                Vector3 {
                    x: v14,
                    y: v24,
                    z: v34,
                },
        } = self;
        writeln!(f, "Transform[")?;
        writeln!(f, "  {v11:.3} {v12:.3} {v13:.3} {v14:.3}")?;
        writeln!(f, "  {v21:.3} {v22:.3} {v23:.3} {v24:.3}")?;
        writeln!(f, "  {v31:.3} {v32:.3} {v33:.3} {v34:.3}")?;
        write!(f, "]")
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { r, g, b, a } = self;
        write!(f, "Quat({r:.3} {g:.3} {b:.3} {a:.3})")
    }
}

macro_rules! impl_unop {
    (@type $a:ident $f:ident $arg:ident) => { $a.$arg.$f() };
    (@type $a:ident $f:ident $arg:ident $e:expr) => { $e };
    (@ret $a:ident $f:ident { $($arg:ident $(: $e:expr)?),* $(,)? }) => {
        Self::Output {$(
            $arg : impl_unop!(@type $a $f $arg $($e)?),
        )*}
    };
    (@ret $a:ident $f:ident $e:expr) => { $e };
    (<$f:ident ($a:ident) : $i:ident> : []) => {};
    (
        <$f:ident ($a:ident) : $i:ident $(,)?> :
        [[$t:ty] $args:tt $(,)?]
    ) => {
        impl $i for $t {
            type Output = Self;

            fn $f(self) -> Self {
                let $a = self;
                impl_unop!(@ret $a $f $args)
            }
        }

        impl<'a> $i for &'a $t {
            type Output = $t;

            fn $f(self) -> Self::Output {
                let $a = self;
                impl_unop!(@ret $a $f $args)
            }
        }
    };
    (
        <$f:ident ($a:ident) : $i:ident $(,)?> :
        [[$t:ty => $r:ty] $args:tt $(,)?]
    ) => {
        impl $i for $t {
            type Output = $r;

            fn $f(self) -> Self::Output {
                let $a = self;
                impl_unop!(@ret $a $f $args)
            }
        }

        impl<'a> $i for &'a $t {
            type Output = $r;

            fn $f(self) -> Self::Output {
                let $a = self;
                impl_unop!(@ret $a $f $args)
            }
        }
    };
    (
        <$f:ident ($a:ident) : $i:ident> :
        [
            [$($t0:tt)*] $args0:tt
            $(, [ $($t:tt)* ] $args:tt)+ $(,)?
        ]
    ) => {
        impl_unop!(<$f($a): $i>: [[$($t0)*] $args0]);
        $(impl_unop!(<$f($a): $i>: [[$($t)*] $args]);)+
    };
}

macro_rules! impl_binop {
    (@type $a:ident $b:ident $f:ident $arg:ident) => { $a.$arg.$f($b.$arg) };
    (@type $a:ident $b:ident $f:ident $arg:ident $e:expr) => { $e };
    (@ret $a:ident $b:ident $f:ident { $($arg:ident $(: $e:expr)?),* $(,)? }) => {
        Self::Output {$(
            $arg : impl_binop!(@type $a $b $f $arg $($e)?),
        )*}
    };
    (@ret $a:ident $b:ident $f:ident $e:expr) => { $e };
    (<$f:ident ($a:ident, $b:ident) : $i:ident> : []) => {};
    (
        <$f:ident ($a:ident, $b:ident) : $i:ident $(,)?> :
        [[$t:ty] $args:tt $(,)?]
    ) => {
        impl $i<$t> for $t {
            type Output = Self;

            fn $f(self, other: Self) -> Self {
                let $a = self;
                let $b = other;
                impl_binop!(@ret $a $b $f $args)
            }
        }

        impl<'a> $i<&'a $t> for $t {
            type Output = Self;

            fn $f(self, other: &'a Self) -> Self {
                let $a = self;
                let $b = other;
                impl_binop!(@ret $a $b $f $args)
            }
        }

        impl<'a> $i<$t> for &'a $t {
            type Output = $t;

            fn $f(self, other: $t) -> Self::Output {
                let $a = self;
                let $b = other;
                impl_binop!(@ret $a $b $f $args)
            }
        }

        impl<'a, 'b> $i<&'a $t> for &'b $t {
            type Output = $t;

            fn $f(self, other: &'a $t) -> Self::Output {
                let $a = self;
                let $b = other;
                impl_binop!(@ret $a $b $f $args)
            }
        }
    };
    (
        <$f:ident ($a:ident, $b:ident) : $i:ident $(,)?> :
        [[$t:ty => $r:ty] $args:tt $(,)?]
    ) => {
        impl $i<$t> for $t {
            type Output = $r;

            fn $f(self, other: Self) -> Self::Output {
                let $a = self;
                let $b = other;
                impl_binop!(@ret $a $b $f $args)
            }
        }

        impl<'a> $i<&'a $t> for $t {
            type Output = $r;

            fn $f(self, other: &'a Self) -> Self::Output {
                let $a = self;
                let $b = other;
                impl_binop!(@ret $a $b $f $args)
            }
        }

        impl<'a> $i<$t> for &'a $t {
            type Output = $r;

            fn $f(self, other: $t) -> Self::Output {
                let $a = self;
                let $b = other;
                impl_binop!(@ret $a $b $f $args)
            }
        }

        impl<'a, 'b> $i<&'a $t> for &'b $t {
            type Output = $r;

            fn $f(self, other: &'a $t) -> Self::Output {
                let $a = self;
                let $b = other;
                impl_binop!(@ret $a $b $f $args)
            }
        }
    };
    (
        <$f:ident ($a:ident, $b:ident) : $i:ident $(,)?> :
        [[$t:ty, $t2:ty => $r:ty] $args:tt $(,)?]
    ) => {
        impl $i<$t2> for $t {
            type Output = $r;

            fn $f(self, other: $t2) -> Self::Output {
                let $a = self;
                let $b = other;
                impl_binop!(@ret $a $b $f $args)
            }
        }

        impl<'a> $i<&'a $t2> for $t {
            type Output = $r;

            fn $f(self, other: &'a $t2) -> Self::Output {
                let $a = self;
                let $b = other;
                impl_binop!(@ret $a $b $f $args)
            }
        }

        impl<'a> $i<$t2> for &'a $t {
            type Output = $r;

            fn $f(self, other: $t2) -> Self::Output {
                let $a = self;
                let $b = other;
                impl_binop!(@ret $a $b $f $args)
            }
        }

        impl<'a, 'b> $i<&'a $t2> for &'b $t {
            type Output = $r;

            fn $f(self, other: &'a $t2) -> Self::Output {
                let $a = self;
                let $b = other;
                impl_binop!(@ret $a $b $f $args)
            }
        }
    };
    (
        <$f:ident ($a:ident, $b:ident) : $i:ident> :
        [
            [$($t0:tt)*] $args0:tt
            $(, [ $($t:tt)* ] $args:tt)+ $(,)?
        ]
    ) => {
        impl_binop!(<$f($a, $b): $i>: [[$($t0)*] $args0]);
        $(impl_binop!(<$f($a, $b): $i>: [[$($t)*] $args]);)+
    };
}

impl_unop!(
    <neg(a): Neg> : [
        [Vector2] { x, y },
        [Vector3] { x, y, z },
        [Rect2] {
            position: -(a.position + a.size),
            size: a.size,
        },
        [Aabb] {
            position: -(a.position + a.size),
            size: a.size,
        },
        [Plane] { normal, d },
    ]
);

impl_binop!(
    <add(a, b): Add> : [
        [Vector2] { x, y },
        [Vector2, f32 => Vector2] {
            x: a.x + b,
            y: a.y + b,
        },
        [Vector2, Rect2 => Rect2] {
            position: b.position + a,
            size: b.size + a,
        },
        [Vector3] { x, y, z },
        [Vector3, f32 => Vector3] {
            x: a.x + b,
            y: a.y + b,
            z: a.z + b,
        },
        [Vector3, Aabb => Aabb] {
            position: b.position + a,
            size: b.size + a,
        },
        [Vector3, Plane => Plane] {
            normal: b.normal,
            d: a.dot(b.normal) + b.d,
        },
        [Rect2] { position, size },
        [Rect2, f32 => Rect2] {
            position: a.position + b,
            size: a.size + b,
        },
        [Rect2, Vector2 => Rect2] {
            position: a.position + b,
            size: a.size + b,
        },
        [Aabb] { position, size },
        [Aabb, f32 => Aabb] {
            position: a.position + b,
            size: a.size + b,
        },
        [Aabb, Vector3 => Aabb] {
            position: a.position + b,
            size: a.size + b,
        },
        [Plane, Vector3 => Plane] {
            normal: a.normal,
            d: b.dot(a.normal) + a.d,
        },
    ]
);

impl_binop!(
    <sub(a, b): Sub> : [
        [Vector2] { x, y },
        [Vector2, f32 => Vector2] {
            x: a.x - b,
            y: a.y - b,
        },
        [Vector2, Rect2 => Rect2] {
            position: b.position - a,
            size: b.size - a,
        },
        [Vector3] { x, y, z },
        [Vector3, f32 => Vector3] {
            x: a.x - b,
            y: a.y - b,
            z: a.z - b,
        },
        [Vector3, Aabb => Aabb] {
            position: b.position - a,
            size: b.size - a,
        },
        [Vector3, Plane => Plane] {
            normal: -b.normal,
            d: -(a.dot(b.normal) + b.d),
        },
        [Rect2] { position, size },
        [Rect2, f32 => Rect2] {
            position: a.position - b,
            size: a.size - b,
        },
        [Rect2, Vector2 => Rect2] {
            position: a.position - b,
            size: a.size - b,
        },
        [Aabb] { position, size },
        [Aabb, f32 => Aabb] {
            position: a.position - b,
            size: a.size - b,
        },
        [Aabb, Vector3 => Aabb] {
            position: a.position - b,
            size: a.size - b,
        },
        [Plane, Vector3 => Plane] {
            normal: a.normal,
            d: a.d - b.dot(a.normal),
        },
    ]
);

impl_binop!(
    <mul(a, b): Mul> : [
        [Vector2] { x, y },
        [Vector2, f32 => Vector2] {
            x: a.x * b,
            y: a.y * b,
        },
        [Vector2, Transform2D => Vector2] {
            x: a.x * b.a.x + a.y * b.b.x + b.origin.x,
            y: a.x * b.a.y + a.y * b.b.y + b.origin.y,
        },
        [Vector3, Quat => Vector3] {
            let i = b.conjugate();
            let x = b.w * a.x + b.y * a.z - b.z * a.y;
            let y = b.w * a.y - b.x * a.z + b.z * a.x;
            let z = b.w * a.z + b.x * a.y - b.y * a.x;
            let w = -(b.x * a.x + b.y * a.y + b.z * a.z);

            Vector3 {
                x: w * i.x + x * i.w + y * i.z - z * i.y,
                y: w * i.y - x * i.z + y * i.w + z * i.x,
                z: w * i.z + x * i.y - y * i.x + z * i.w,
            }
        },
        [Vector3] { x, y, z },
        [Vector3, f32 => Vector3] {
            x: a.x * b,
            y: a.y * b,
            z: a.z * b,
        },
        [Vector3, Basis => Vector3] {
            x: a.dot(b.elements[0]),
            y: a.dot(b.elements[1]),
            z: a.dot(b.elements[2]),
        },
        [Vector3, Transform => Vector3] {
            x: b.basis.elements[0].x * a.x + b.basis.elements[1].x * a.y + b.basis.elements[2].x * a.z + b.origin.x,
            y: b.basis.elements[0].y * a.x + b.basis.elements[1].y * a.y + b.basis.elements[2].y * a.z + b.origin.y,
            z: b.basis.elements[0].z * a.x + b.basis.elements[1].z * a.y + b.basis.elements[2].z * a.z + b.origin.z,
        },
        [Rect2, f32 => Rect2] {
            position: a.position * b,
            size: a.size * b,
        },
        [Aabb, f32 => Aabb] {
            position: a.position * b,
            size: a.size * b,
        },
        [Transform2D] {
            a: Vector2 {
                x: a.a.x * b.a.x + a.b.x * b.a.y,
                y: a.a.y * b.a.x + a.b.y * b.a.y,
            },
            b: Vector2 {
                x: a.a.x * b.a.x + a.b.x * b.b.y,
                y: a.a.y * b.a.x + a.b.y * b.b.y,
            },
            origin: Vector2 {
                x: a.a.x * b.origin.x + a.b.x * b.origin.y + a.origin.x,
                y: a.a.y * b.origin.x + a.b.y * b.origin.y + a.origin.y,
            },
        },
        [Transform2D, Vector2 => Vector2] {
            x: b.x * a.a.x + b.y * a.b.x + a.origin.x,
            y: b.x * a.a.y + b.y * a.b.y + a.origin.y,
        },
        [Quat] {
            x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
            y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
            z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
            w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
        },
        [Quat, Vector3 => Vector3] {
            let i = a.conjugate();
            let x = a.w * b.x + a.y * b.z - a.z * b.y;
            let y = a.w * b.y - a.x * b.z + a.z * b.x;
            let z = a.w * b.z + a.x * b.y - a.y * b.x;
            let w = -(a.x * b.x + a.y * b.y + a.z * b.z);

            Vector3 {
                x: w * i.x + x * i.w + y * i.z - z * i.y,
                y: w * i.y - x * i.z + y * i.w + z * i.x,
                z: w * i.z + x * i.y - y * i.x + z * i.w,
            }
        },
        [Basis] {
            elements: [
                a * b.elements[0],
                a * b.elements[1],
                a * b.elements[2],
            ],
        },
        [Basis, Vector3 => Vector3] {
            x: a.elements[0].x * b.x + a.elements[1].x * b.y + a.elements[2].x * b.z,
            y: a.elements[0].y * b.x + a.elements[1].y * b.y + a.elements[2].y * b.z,
            z: a.elements[0].z * b.x + a.elements[1].z * b.y + a.elements[2].z * b.z,
        },
        [Basis, Transform => Transform] {
            basis: a * b.basis,
            origin: a * b.origin,
        },
        [Transform] {
            basis: a.basis * b.basis,
            origin: a.basis * b.origin + a.origin,
        },
        [Transform, Basis => Transform] {
            basis: a.basis * b,
            origin: a.origin,
        },
        [Transform, Vector3 => Vector3] {
            x: a.basis.elements[0].x * b.x + a.basis.elements[1].x * b.y + a.basis.elements[2].x * b.z + a.origin.x,
            y: a.basis.elements[0].y * b.x + a.basis.elements[1].y * b.y + a.basis.elements[2].y * b.z + a.origin.y,
            z: a.basis.elements[0].z * b.x + a.basis.elements[1].z * b.y + a.basis.elements[2].z * b.z + a.origin.z,
        },
    ]
);

impl_binop!(
    <div(a, b): Div> : [
        [Vector2] { x, y },
        [Vector2, f32 => Vector2] {
            x: a.x / b,
            y: a.y / b,
        },
        [Vector3] { x, y, z },
        [Vector3, f32 => Vector3] {
            x: a.x / b,
            y: a.y / b,
            z: a.z / b,
        },
        [Vector3, Quat => Vector3] {
            let i = b.conjugate();
            let x = i.w * a.x + i.y * a.z - i.z * a.y;
            let y = i.w * a.y - i.x * a.z + i.z * a.x;
            let z = i.w * a.z + i.x * a.y - i.y * a.x;
            let w = -(i.x * a.x + i.y * a.y + i.z * a.z);

            Vector3 {
                x: w * b.x + x * b.w + y * b.z - z * b.y,
                y: w * b.y - x * b.z + y * b.w + z * b.x,
                z: w * b.z + x * b.y - y * b.x + z * b.w,
            }
        },
        [Rect2, f32 => Rect2] {
            position: a.position / b,
            size: a.size / b,
        },
        [Aabb, f32 => Aabb] {
            position: a.position / b,
            size: a.size / b,
        },
        [Quat] {
            a * b.conjugate()
        },
    ]
);

impl Vector2 {
    pub const ZERO: Self = Self { x: 0., y: 0. };
    pub const ONE: Self = Self { x: 1., y: 1. };
    pub const LEFT: Self = Self { x: -1., y: 0. };
    pub const RIGHT: Self = Self { x: 1., y: 0. };
    pub const UP: Self = Self { x: 0., y: -1. };
    pub const DOWN: Self = Self { x: 0., y: 1. };

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }

    pub fn length_squared(self) -> f32 {
        self.x.powi(2) + self.y.powi(2)
    }

    pub fn length(self) -> f32 {
        self.x.hypot(self.y)
    }

    pub fn normalize(self) -> Self {
        self / self.length()
    }

    pub fn project(self, onto: Self) -> Self {
        onto * (self.dot(onto) / onto.length_squared())
    }
}

impl Vector3 {
    pub const ZERO: Self = Self {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    pub const ONE: Self = Self {
        x: 1.,
        y: 1.,
        z: 1.,
    };
    pub const LEFT: Self = Self {
        x: -1.,
        y: 0.,
        z: 0.,
    };
    pub const RIGHT: Self = Self {
        x: 1.,
        y: 0.,
        z: 0.,
    };
    pub const UP: Self = Self {
        x: 0.,
        y: -1.,
        z: 0.,
    };
    pub const DOWN: Self = Self {
        x: 0.,
        y: 1.,
        z: 0.,
    };
    pub const FORWARD: Self = Self {
        x: 0.,
        y: 0.,
        z: -1.,
    };
    pub const BACK: Self = Self {
        x: 0.,
        y: 0.,
        z: 1.,
    };

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn length_squared(self) -> f32 {
        self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalize(self) -> Self {
        self / self.length()
    }

    pub fn project(self, onto: Self) -> Self {
        onto * (self.dot(onto) / onto.length_squared())
    }
}

impl Transform2D {
    pub const IDENTITY: Self = Self {
        a: Vector2 { x: 1., y: 0. },
        b: Vector2 { x: 0., y: 1. },
        origin: Vector2::ZERO,
    };

    pub fn determinant(self) -> f32 {
        let Self {
            a: Vector2 { x: ax, y: ay },
            b: Vector2 { x: bx, y: by },
            ..
        } = self;
        ax * by - bx * ay
    }

    pub fn inverse(self) -> Self {
        let Self {
            a: Vector2 { x: ax, y: ay },
            b: Vector2 { x: bx, y: by },
            origin: Vector2 { x: ox, y: oy },
        } = self;
        let det_inv = (ax * by - bx * ay).recip();

        Self {
            a: Vector2 {
                x: det_inv * by,
                y: det_inv * -ay,
            },
            b: Vector2 {
                x: det_inv * -bx,
                y: det_inv * ax,
            },
            origin: Vector2 {
                x: det_inv * (ox * by + oy * -bx),
                y: det_inv * (ox * -ay + oy * ax),
            },
        }
    }
}

impl Quat {
    pub const IDENTITY: Self = Self {
        x: 0.,
        y: 0.,
        z: 0.,
        w: 1.,
    };

    pub fn axis_angle(axis: Vector3, angle: f32) -> Self {
        let Vector3 { x, y, z } = axis;
        let (s, c) = (angle * 0.5).sin_cos();
        Self {
            x: x * s,
            y: y * s,
            z: z * s,
            w: c,
        }
    }

    pub fn norm(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2) + self.w.powi(2)).sqrt()
    }

    pub fn conjugate(self) -> Self {
        let l = (self.x.powi(2) + self.y.powi(2) + self.z.powi(2) + self.w.powi(2)).recip();
        Self {
            x: -self.x * l,
            y: -self.y * l,
            z: -self.z * l,
            w: self.w * l,
        }
    }

    pub fn normalize(self) -> Self {
        let l = self.norm().recip();
        Self {
            x: self.x * l,
            y: self.y * l,
            z: self.z * l,
            w: self.w * l,
        }
    }
}

impl Basis {
    pub const IDENTITY: Self = Self {
        elements: [
            Vector3 {
                x: 1.,
                y: 0.,
                z: 0.,
            },
            Vector3 {
                x: 0.,
                y: 1.,
                z: 0.,
            },
            Vector3 {
                x: 0.,
                y: 0.,
                z: 1.,
            },
        ],
    };

    pub fn determinant(self) -> f32 {
        let Self {
            elements:
                [Vector3 { x: a, y: b, z: c }, Vector3 { x: d, y: e, z: f }, Vector3 { x: g, y: h, z: i }],
        } = self;
        a * e * i + b * f * g + c * d * h - a * f * h - b * d * i - c * e * g
    }

    pub fn inverse(self) -> Self {
        let Self {
            elements:
                [Vector3 { x: a, y: b, z: c }, Vector3 { x: d, y: e, z: f }, Vector3 { x: g, y: h, z: i }],
        } = self;
        let det_inv =
            (a * e * i + b * f * g + c * d * h - a * f * h - b * d * i - c * e * g).recip();

        Self {
            elements: [
                Vector3 {
                    x: det_inv * (e * i - f * h),
                    y: det_inv * (f * g - d * i),
                    z: det_inv * (d * h - e * g),
                },
                Vector3 {
                    x: det_inv * (c * h - b * i),
                    y: det_inv * (a * i - c * g),
                    z: det_inv * (b * g - a * h),
                },
                Vector3 {
                    x: det_inv * (b * f - c * e),
                    y: det_inv * (c * d - a * f),
                    z: det_inv * (a * e - b * d),
                },
            ],
        }
    }
}

impl Transform {
    pub const IDENTITY: Self = Self {
        basis: Basis::IDENTITY,
        origin: Vector3::ZERO,
    };

    pub fn determinant(self) -> f32 {
        self.basis.determinant()
    }

    pub fn inverse(self) -> Self {
        let Self { mut basis, origin } = self;
        basis = basis.inverse();
        Self {
            basis,
            origin: basis * origin,
        }
    }
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 1.,
    };
    pub const RED: Self = Self {
        r: 1.,
        g: 0.,
        b: 0.,
        a: 1.,
    };
    pub const YELLOW: Self = Self {
        r: 1.,
        g: 1.,
        b: 0.,
        a: 1.,
    };
    pub const GREEN: Self = Self {
        r: 0.,
        g: 1.,
        b: 0.,
        a: 1.,
    };
    pub const AQUA: Self = Self {
        r: 0.,
        g: 1.,
        b: 1.,
        a: 1.,
    };
    pub const BLUE: Self = Self {
        r: 0.,
        g: 0.,
        b: 1.,
        a: 1.,
    };
    pub const MAGENTA: Self = Self {
        r: 1.,
        g: 0.,
        b: 1.,
        a: 1.,
    };
    pub const WHITE: Self = Self {
        r: 1.,
        g: 1.,
        b: 1.,
        a: 1.,
    };

    pub fn mix(self, other: Self) -> Self {
        let ia = 1. - other.a;
        let a = other.a * self.a * ia;
        let r = a.recip();
        Self {
            r: (other.r * other.a + self.r * self.a * ia) / r,
            g: (other.g * other.a + self.g * self.a * ia) / r,
            b: (other.b * other.a + self.b * self.a * ia) / r,
            a,
        }
    }
}
