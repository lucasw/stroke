#![no_std]
#![forbid(unsafe_code)]
#![allow(incomplete_features)]
// this is needed to use expressions in const generics such as N-1 (see curve derivatives)
//#![feature(generic_const_exprs)]

// this feature was needed for tinyvec < 2.0 to compile for const generic arrays like ArrayVec<[f32;N]>
//#![feature(min_const_generics)]

// NO LONGER NECESSARY (stabilized in 1.79)
// removes the need for generics with associated types to specify the
// associated type like P:Point instead of P: Point<Scalar=f64>
//#![feature(associated_type_bounds)]

// make splines usable as Fn in trait bounds
//#![feature(fn_traits)]

use core::ops::{Add, Mul, Sub};

extern crate num_traits;
use num_traits::float::Float;

extern crate tinyvec;
use tinyvec::ArrayVec;
// export common types at crate root

pub mod f32 {
    use super::*;

    pub type NativeFloat = f32;
    const EPSILON: core::primitive::f32 = core::primitive::f32::EPSILON;
    use core::f32::consts::PI;

    // abstraction types
    // pub mod bezier_segment;
    // specialized types
    pub mod cubic_bezier {
        include!("cubic_bezier.rs");
    }
    pub mod line {
        include!("line.rs");
    }
    pub mod quadratic_bezier {
        include!("quadratic_bezier.rs");
    }
    // generic types
    // pub mod bezier;
    // pub mod bspline;
    pub mod point_generic {
        include!("point_generic.rs");
    }

    // Traits
    pub mod point {
        include!("point.rs");
    }
    pub mod spline {
        include!("spline.rs");
    }

    mod roots {
        include!("roots.rs");
    }

    // pub use bezier::Bezier;
    // pub use bspline::BSpline;
    pub use cubic_bezier::CubicBezier;
    pub use line::LineSegment;
    pub use point::Point;
    pub use point_generic::PointN;
    pub use quadratic_bezier::QuadraticBezier;
    pub use spline::Spline;
}

pub mod f64 {
    use super::*;

    pub type NativeFloat = f64;
    const EPSILON: core::primitive::f64 = core::primitive::f64::EPSILON;
    use core::f64::consts::PI;

    // abstraction types
    // pub mod bezier_segment;
    // specialized types
    pub mod cubic_bezier {
        include!("cubic_bezier.rs");
    }
    pub mod line {
        include!("line.rs");
    }
    pub mod quadratic_bezier {
        include!("quadratic_bezier.rs");
    }
    // generic types
    // pub mod bezier;
    // pub mod bspline;
    pub mod point_generic {
        include!("point_generic.rs");
    }

    // Traits
    pub mod point {
        include!("point.rs");
    }
    pub mod spline {
        include!("spline.rs");
    }

    mod roots {
        include!("roots.rs");
    }

    // pub use bezier::Bezier;
    // pub use bspline::BSpline;
    pub use cubic_bezier::CubicBezier;
    pub use line::LineSegment;
    pub use point::Point;
    pub use point_generic::PointN;
    pub use quadratic_bezier::QuadraticBezier;
    pub use spline::Spline;
}
