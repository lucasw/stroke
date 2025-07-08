use super::point::Point;
use super::LineSegment;
use super::QuadraticBezier;
use super::*;

/// A 2D  cubic Bezier curve defined by four points: the starting point, two successive control points and the ending point.
///
/// The curve is defined by equation
/// : ```∀ t ∈ [0..1],  P(t) = (1 - t)³ * start + 3 * (1 - t)² * t * ctrl1 + 3 * t² * (1 - t) * ctrl2 + t³ * end```
// TODO(lucasw) the vim syntax highlighting doesn't like the triple slashes followed by triple
// back-tick above, but putting the ':' on the same line avoids it
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CubicBezier<P, const PDIM: usize> {
    pub(crate) start: P,
    pub(crate) ctrl1: P,
    pub(crate) ctrl2: P,
    pub(crate) end: P,
}

// TODO(lucasw) can I avoid the redundant sizes?
// CubicBezier::<PointN<f64, 2>, 2>

//#[allow(dead_code)]
impl<P, const PDIM: usize> CubicBezier<P, PDIM>
where
    P: Point,
{
    pub fn new(start: P, ctrl1: P, ctrl2: P, end: P) -> Self {
        CubicBezier {
            start,
            ctrl1,
            ctrl2,
            end,
        }
    }

    /// Evaluate a CubicBezier curve at t by direct evaluation of the polynomial (not numerically stable)
    pub fn eval(&self, t: NativeFloat) -> P {
        self.start * ((-t + 1.0) * (-t + 1.0) * (-t + 1.0))
            + self.ctrl1 * (t * (-t + 1.0) * (-t + 1.0) * 3.0)
            + self.ctrl2 * (t * t * (-t + 1.0) * 3.0)
            + self.end * (t * t * t)
    }

    /// Evaluate a CubicBezier curve at t using the numerically stable De Casteljau algorithm
    pub fn eval_casteljau(&self, t: NativeFloat) -> P {
        // unrolled de casteljau algorithm
        // _1ab is the first iteration from first (a) to second (b) control point and so on
        let ctrl_1ab = self.start + (self.ctrl1 - self.start) * t;
        let ctrl_1bc = self.ctrl1 + (self.ctrl2 - self.ctrl1) * t;
        let ctrl_1cd = self.ctrl2 + (self.end - self.ctrl2) * t;
        // second iteration
        let ctrl_2ab = ctrl_1ab + (ctrl_1bc - ctrl_1ab) * t;
        let ctrl_2bc = ctrl_1bc + (ctrl_1cd - ctrl_1bc) * t;
        // third iteration, return final point on the curve ctrl_3ab
        ctrl_2ab + (ctrl_2bc - ctrl_2ab) * t
    }

    pub fn control_points(&self) -> [P; 4] {
        [self.start, self.ctrl1, self.ctrl2, self.end]
    }

    /// Returns the x coordinate of the curve evaluated at t
    /// Convenience shortcut for bezier.eval(t).x()
    pub fn axis(&self, t: NativeFloat, axis: usize) -> NativeFloat {
        let t2 = t * t;
        let t3 = t2 * t;
        let one_t = -t + 1.0;
        let one_t2 = one_t * one_t;
        let one_t3 = one_t2 * one_t;

        one_t3 * self.start.axis(axis)
            + one_t2 * t * self.ctrl1.axis(axis) * 3.0
            + one_t * t2 * self.ctrl2.axis(axis) * 3.0
            + t3 * self.end.axis(axis)
    }

    /// Approximates the arc length of the curve by flattening it with straight line segments.
    /// Remember arclen also works by linear approximation, not the integral, so we have to accept error!
    /// This approximation is unfeasable if desired accuracy is greater than 2 decimal places
    pub fn arclen(&self, nsteps: usize) -> NativeFloat {
        let stepsize = 1.0 / (nsteps as NativeFloat);
        let mut arclen = NativeFloat::from(0.0);
        for t in 1..nsteps {
            let t = (t as NativeFloat) * 1.0 / (nsteps as NativeFloat);
            let p1 = self.eval_casteljau(t);
            let p2 = self.eval_casteljau(t + stepsize);

            arclen += (p1 - p2).squared_length().sqrt();
        }
        arclen
    }

    pub fn split(&self, t: NativeFloat) -> (Self, Self) {
        // unrolled de casteljau algorithm
        // _1ab is the first iteration from first (a) to second (b) control point and so on
        let ctrl_1ab = self.start + (self.ctrl1 - self.start) * t;
        let ctrl_1bc = self.ctrl1 + (self.ctrl2 - self.ctrl1) * t;
        let ctrl_1cd = self.ctrl2 + (self.end - self.ctrl2) * t;
        // second iteration
        let ctrl_2ab = ctrl_1ab + (ctrl_1bc - ctrl_1ab) * t;
        let ctrl_2bc = ctrl_1bc + (ctrl_1cd - ctrl_1bc) * t;
        // third iteration, final point on the curve
        let ctrl_3ab = ctrl_2ab + (ctrl_2bc - ctrl_2ab) * t;

        (
            CubicBezier {
                start: self.start,
                ctrl1: ctrl_1ab,
                ctrl2: ctrl_2ab,
                end: ctrl_3ab,
            },
            CubicBezier {
                start: ctrl_3ab,
                ctrl1: ctrl_2bc,
                ctrl2: ctrl_1cd,
                end: self.end,
            },
        )
    }

    /// Return the derivative curve.
    /// The derivative is also a bezier curve but of degree n-1 (cubic->quadratic)
    /// Since it returns the derivative function, eval() needs to be called separately
    pub fn derivative(&self) -> QuadraticBezier<P, PDIM> {
        QuadraticBezier {
            start: (self.ctrl1 - self.start) * 3.0,
            ctrl: (self.ctrl2 - self.ctrl1) * 3.0,
            end: (self.end - self.ctrl2) * 3.0,
        }
    }

    /// Direct Derivative - Sample the axis coordinate at 'axis' of the curve's derivative at t
    /// without creating a new curve. This is a convenience function for .derivative().eval(t).axis(n)  
    /// Parameters:
    ///   t: the sampling parameter on the curve interval [0..1]
    ///   axis: the index of the coordinate axis [0..N]
    /// Returns:
    ///   Scalar value of the points own type type F  
    /// May be deprecated in the future.  
    /// This function can cause out of bounds panic when axis is larger than dimension of P
    pub fn dd(&self, t: NativeFloat, axis: usize) -> NativeFloat {
        let t2 = t * t;
        let c0 = t * -3.0 + t * 6.0 - 3.0;
        let c1 = t2 * 9.0 - t * 12.0 + 3.0;
        let c2 = t2 * -9.0 + t * 6.0;
        let c3 = t2 * 3.0;

        self.start.axis(axis) * c0
            + self.ctrl1.axis(axis) * c1
            + self.ctrl2.axis(axis) * c2
            + self.end.axis(axis) * c3
    }

    /// Return the tangent at position t
    pub fn tangent(&self, t: NativeFloat) -> P {
        let derivative = self.derivative().eval(t);
        let d_len = derivative.squared_length().sqrt();
        let scale = 1.0 / d_len;
        derivative * scale
    }

    // curvature is 1.0 / radius, or 0.0 when radius is close to zero
    pub fn curvature(&self, t: NativeFloat) -> NativeFloat {
        let d = self.derivative();
        let d_t = d.eval(t);
        let dx = d_t.axis(0);
        let dy = d_t.axis(1);

        let dd = d.derivative();
        let dd_t = dd.eval(t);
        let ddx = dd_t.axis(0);
        let ddy = dd_t.axis(1);

        let numerator = dx * ddy - dy * ddx;
        let denominator = (dx * dx + dy * dy).powf(1.5);
        // this is what graphite bezier solver does
        if denominator.abs() < 1e-3 {
            0.0
        } else {
            numerator / denominator
        }
    }

    // pub fn radius(&self, t: NativeFloat) -> F
    // where
    // F: NativeFloatloat,
    // P:  Sub<P, Output = P>
    //     + Add<P, Output = P>
    //     + Mul<F, Output = P>,
    // NativeFloat: Sub<F, Output = F>
    //     + Add<F, Output = F>
    //     + Mul<F, Output = F>
    //     + Float
    //     + Into
    // {
    //     return 1.0 / self.curvature(t)
    // }

    fn capture_closest(
        closest_point_on_curve: &mut P,
        tmin: &mut NativeFloat,
        dmin: &mut NativeFloat,
        candidate: &P,
        t: NativeFloat,
        point: &P,
    ) {
        let distance = (*candidate - *point).squared_length();
        if distance < *dmin {
            *tmin = t;
            *closest_point_on_curve = *candidate;
            *dmin = distance;
        }
    }

    /// Calculates the minimum distance between given 'point' and the curve, returns the closest point on
    /// the curve, the t-value, and the distance.
    /// Uses two passes with the same amount of steps in t:
    /// 1. coarse search over the whole curve
    /// 2. fine search around the minimum yielded by the coarse search
    pub fn closest_to_point(&self, point: P) -> (P, NativeFloat, NativeFloat) {
        let nsteps: usize = 64;
        let mut tmin: NativeFloat = 0.5;
        let mut dmin: NativeFloat = (point - self.start).squared_length();
        let mut closest_point_on_curve = self.start;

        // 1. coarse pass
        for i in 0..nsteps {
            // calculate next step value
            let t: NativeFloat =
                i as NativeFloat * 1.0 as NativeFloat / (nsteps as NativeFloat);
            // calculate distance to candidate
            let candidate = self.eval(t);
            Self::capture_closest(
                &mut closest_point_on_curve,
                &mut tmin,
                &mut dmin,
                &candidate,
                t,
                &point,
            );
        }
        // 2. fine pass
        for i in 0..nsteps {
            // calculate next step value ( a 64th of a 64th from first step)
            let t: NativeFloat =
                i as NativeFloat * 1.0 as NativeFloat / ((nsteps * nsteps) as NativeFloat);
            // calculate distance to candidate centered around tmin from before
            let candidate: P = self.eval(tmin + t - t * (nsteps as NativeFloat / 2.0));
            Self::capture_closest(
                &mut closest_point_on_curve,
                &mut tmin,
                &mut dmin,
                &candidate,
                t,
                &point,
            );
        }
        (closest_point_on_curve, tmin, dmin.sqrt())
    }

    pub fn distance_to_point(&self, point: P) -> NativeFloat {
        let (_, _, distance) = self.closest_to_point(point);
        distance
    }

    pub fn baseline(&self) -> LineSegment<P, PDIM> {
        LineSegment {
            start: self.start,
            end: self.end,
        }
    }

    pub fn is_linear(&self, tolerance: NativeFloat) -> bool {
        // if start and end are (nearly) the same
        if (self.start - self.end).squared_length() < EPSILON {
            return false;
        }
        // else check if ctrl points lie on baseline
        self.are_points_colinear(tolerance)
    }

    fn are_points_colinear(&self, tolerance: NativeFloat) -> bool {
        let line = self.baseline();
        line.distance_to_point(self.ctrl1) <= tolerance
            && line.distance_to_point(self.ctrl2) <= tolerance
    }

    // Returs if the whole set of control points can be considered one singular point
    // given some tolerance.
    // TODO use machine epsilon vs squared_length OK?
    pub fn is_a_point(&self, tolerance: NativeFloat) -> bool {
        let tolerance_squared = tolerance * tolerance;
        // Use <= so that tolerance can be zero.
        (self.start - self.end).squared_length() <= tolerance_squared
            && (self.start - self.ctrl1).squared_length() <= tolerance_squared
            && (self.end - self.ctrl2).squared_length() <= tolerance_squared
    }

    /// Compute the real roots of the cubic bezier function with
    /// parameters of the form a*t^3 + b*t^2 + c*t + d for each dimension
    /// using cardano's algorithm (code adapted from github.com/nical/lyon)
    /// returns an ArrayVec of the present roots (max 3)
    #[allow(clippy::many_single_char_names)] // this is math, get over it
    pub(crate) fn real_roots(
        &self,
        a: NativeFloat,
        b: NativeFloat,
        c: NativeFloat,
        d: NativeFloat,
    ) -> ArrayVec<[NativeFloat; 3]> {
        let mut result = ArrayVec::new();

        // check if can be handled below cubic order
        if a.abs() < EPSILON {
            if b.abs() < EPSILON {
                if c.abs() < EPSILON {
                    // no solutions
                    return result;
                }
                // is linear equation
                result.push(-d / c);
                return result;
            }
            // is quadratic equation
            let delta = c * c - b * d * 4.0;
            if delta > 0.0 {
                let sqrt_delta = delta.sqrt();
                result.push((-c - sqrt_delta) / (b * 2.0));
                result.push((-c + sqrt_delta) / (b * 2.0));
            } else if delta.abs() < EPSILON {
                result.push(-c / (b * 2.0));
            }
            return result;
        }

        // is cubic equation -> use cardano's algorithm
        let frac_1_3 = NativeFloat::from(1.0 / 3.0);

        let bn = b / a;
        let cn = c / a;
        let dn = d / a;

        let delta0: NativeFloat = (cn * 3.0 - bn * bn) / 9.0;
        let delta1: NativeFloat = (bn * cn * 9.0 - dn * 27.0 - bn * bn * bn * 2.0) / 54.0;
        let delta_01: NativeFloat = delta0 * delta0 * delta0 + delta1 * delta1;

        if delta_01 >= NativeFloat::from(0.0) {
            let delta_p_sqrt: NativeFloat = delta1 + delta_01.sqrt();
            let delta_m_sqrt: NativeFloat = delta1 - delta_01.sqrt();

            let s = delta_p_sqrt.signum() * delta_p_sqrt.abs().powf(frac_1_3);
            let t = delta_m_sqrt.signum() * delta_m_sqrt.abs().powf(frac_1_3);

            result.push(-bn * frac_1_3 + (s + t));

            // Don't add the repeated root when s + t == 0.
            if (s - t).abs() < EPSILON && (s + t).abs() >= EPSILON {
                result.push(-bn * frac_1_3 - (s + t) / 2.0);
            }
        } else {
            let theta = (delta1 / (-delta0 * delta0 * delta0).sqrt()).acos();
            let two_sqrt_delta0 = (-delta0).sqrt() * 2.0;
            result.push(two_sqrt_delta0 * Float::cos(theta * frac_1_3) - bn * frac_1_3);
            result
                .push(two_sqrt_delta0 * Float::cos((theta + 2.0 * PI) * frac_1_3) - bn * frac_1_3);
            result
                .push(two_sqrt_delta0 * Float::cos((theta + 4.0 * PI) * frac_1_3) - bn * frac_1_3);
        }

        result
    }

    /// Solves the cubic bezier function given a particular coordinate axis value
    /// by solving the roots for the axis functions
    /// Parameters:
    /// value: the coordinate value on the particular axis
    /// axis: the index of the axis
    /// Returns those roots of the function that are in the interval [0.0, 1.0].
    #[allow(dead_code)]
    fn solve_t_for_axis(&self, value: NativeFloat, axis: usize) -> ArrayVec<[NativeFloat; 3]> {
        let mut result = ArrayVec::new();
        // check if all points are the same or if the curve is really just a line
        if self.is_a_point(EPSILON)
            || (self.are_points_colinear(EPSILON)
                && (self.start - self.end).squared_length() < EPSILON)
        {
            return result;
        }
        let a = -self.start.axis(axis) + self.ctrl1.axis(axis) * 3.0 - self.ctrl2.axis(axis) * 3.0
            + self.end.axis(axis);
        let b =
            self.start.axis(axis) * 3.0 - self.ctrl1.axis(axis) * 6.0 + self.ctrl2.axis(axis) * 3.0;
        let c = -self.start.axis(axis) * 3.0 + self.ctrl1.axis(axis) * 3.0;
        let d = self.start.axis(axis) - value;

        let roots = self.real_roots(a, b, c, d);
        for root in roots {
            if root > 0.0 && root < 1.0 {
                result.push(root);
            }
        }

        result
    }

    /// Return the bounding box of the curve as an array of (min, max) tuples for each dimension (its index)
    pub fn bounding_box(&self) -> [(NativeFloat, NativeFloat); PDIM] {
        // calculate coefficients for the derivative: at^2 + bt + c
        // from the expansion of the cubic bezier curve: sum_i=0_to_3( binomial(3, i) * t^i * (1-t)^(n-i) )
        // yields coeffcients
        // po: [1, -2,  1]
        // p1: [0,  2, -2]
        // p2: [0,  0,  1]
        //      c   b   a
        let mut bounds = [(0.0, 0.0); PDIM];
        let derivative = self.derivative();
        // calculate coefficients for derivative
        let a: P = derivative.start + derivative.ctrl * -2.0 + derivative.end;
        let b: P = derivative.start * -2.0 + derivative.ctrl * 2.0;
        let c: P = derivative.start;

        // calculate roots for t over x axis and plug them into the bezier function
        //  to get x,y values (make vec 2 bigger for t=0,t=1 values)
        // loop over any of the points dimensions (they're all the same)
        for (dim, _) in a.into_iter().enumerate() {
            let mut extrema: ArrayVec<[NativeFloat; 4]> = ArrayVec::new();
            extrema.extend(
                derivative
                    .real_roots(a.axis(dim), b.axis(dim), c.axis(dim))
                    .into_iter(),
            );
            // only retain roots for which t is in [0..1]
            extrema.retain(|root| -> bool { root > &mut 0.0 && root < &mut 1.0 });
            // evaluates roots in original function
            for t in extrema.iter_mut() {
                *t = self.eval_casteljau(*t).axis(dim);
            }
            // add y-values for start and end point as candidates
            extrema.push(self.start.axis(dim));
            extrema.push(self.end.axis(dim));
            // sort to get min and max values for bounding box
            extrema.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

            // determine xmin, xmax, ymin, ymax, from the set {B(xroots), B(yroots), B(0), B(1)}
            // (Intermediate control points can't form a boundary)
            // .unwrap() is ok as it can never be empty as it always at least contains the endpoints
            bounds[dim] = (extrema[0], *extrema.last().unwrap());
        }
        bounds
    }
}

#[cfg(test)]
mod tests {
    use super::PointN;
    use super::*;
    use core::f64::consts::PI;
    #[test]
    fn circle_approximation_error() {
        // define closure for unit circle
        let circle =
            |p: PointN<NativeFloat, 2>| -> NativeFloat { p.into_iter().map(|x| x * x).sum::<NativeFloat>().sqrt() - 1.0 };

        // define control points for 4 bezier segments
        // control points are chosen for minimum radial distance error
        // according to: http://spencermortensen.com/articles/bezier-circle/
        // TODO don't hardcode values
        let c = 0.551915024494;
        let max_drift_perc = 0.019608; // radial drift percent
        let max_error = max_drift_perc * 0.011; // absolute max radial error

        const PDIM: usize = 2;
        let bezier_quadrant_1 = CubicBezier::<_, PDIM> {
            start: PointN::new([0.0, 1.0]),
            ctrl1: PointN::new([c, 1.0]),
            ctrl2: PointN::new([1.0, c]),
            end: PointN::new([1.0, 0.0]),
        };
        let bezier_quadrant_2 = CubicBezier::<_, PDIM> {
            start: PointN::new([1.0, 0.0]),
            ctrl1: PointN::new([1.0, -c]),
            ctrl2: PointN::new([c, -1.0]),
            end: PointN::new([0.0, -1.0]),
        };
        let bezier_quadrant_3 = CubicBezier::<_, PDIM> {
            start: PointN::new([0.0, -1.0]),
            ctrl1: PointN::new([-c, -1.0]),
            ctrl2: PointN::new([-1.0, -c]),
            end: PointN::new([-1.0, 0.0]),
        };
        let bezier_quadrant_4 = CubicBezier::<_, PDIM> {
            start: PointN::new([-1.0, 0.0]),
            ctrl1: PointN::new([-1.0, c]),
            ctrl2: PointN::new([-c, 1.0]),
            end: PointN::new([0.0, 1.0]),
        };
        let nsteps = 1000;
        for t in 0..=nsteps {
            let t = t as NativeFloat * 1.0 / (nsteps as NativeFloat);

            let point = bezier_quadrant_1.eval(t);
            let contour = circle(point);
            assert!(contour.abs() <= max_error, "{} <= {max_error}", contour.abs());

            let point = bezier_quadrant_2.eval(t);
            let contour = circle(point);
            assert!(contour.abs() <= max_error, "{} <= {max_error}", contour.abs());

            let point = bezier_quadrant_3.eval(t);
            let contour = circle(point);
            assert!(contour.abs() <= max_error, "{} <= {max_error}", contour.abs());

            let point = bezier_quadrant_4.eval(t);
            let contour = circle(point);
            assert!(contour.abs() <= max_error, "{} <= {max_error}", contour.abs());
        }
    }

    #[test]
    fn circle_circumference_approximation() {
        // define control points for 4 cubic bezier segments to best approximate a unit circle
        // control points are chosen for minimum radial distance error, see circle_approximation_error() in this file
        // given this, the circumference will also be close to 2*pi
        // (remember arclen also works by linear approximation, not the true integral, so we have to accept error)!
        // This approximation is unfeasable if desired accuracy is greater than ~2 decimal places (at 1000 steps)
        // TODO don't hardcode values, solve for them
        let c = 0.551915024494 as NativeFloat;
        let max_error = 1e-2 as NativeFloat;
        let nsteps = 1e3 as usize;
        let tau = 2. * PI as NativeFloat;

        let bezier_quadrant_1 = CubicBezier::<_, 2> {
            start: PointN::new([0.0, 1.0]),
            ctrl1: PointN::new([c, 1.0]),
            ctrl2: PointN::new([1.0, c]),
            end: PointN::new([1.0, 0.0]),
        };
        let bezier_quadrant_2 = CubicBezier::<_, 2> {
            start: PointN::new([1.0, 0.0]),
            ctrl1: PointN::new([1.0, -c]),
            ctrl2: PointN::new([c, -1.0]),
            end: PointN::new([0.0, -1.0]),
        };
        let bezier_quadrant_3 = CubicBezier::<_, 2> {
            start: PointN::new([0.0, -1.0]),
            ctrl1: PointN::new([-c, -1.0]),
            ctrl2: PointN::new([-1.0, -c]),
            end: PointN::new([-1.0, 0.0]),
        };
        let bezier_quadrant_4 = CubicBezier::<_, 2> {
            start: PointN::new([-1.0, 0.0]),
            ctrl1: PointN::new([-1.0, c]),
            ctrl2: PointN::new([-c, 1.0]),
            end: PointN::new([0.0, 1.0]),
        };
        let circumference = (bezier_quadrant_1.arclen(nsteps)
            + bezier_quadrant_2.arclen(nsteps)
            + bezier_quadrant_3.arclen(nsteps)
            + bezier_quadrant_4.arclen(nsteps)) as NativeFloat;
        //dbg!(circumference);
        //dbg!(tau);
        assert!(((tau + max_error) > circumference) && ((tau - max_error) < circumference));
    }

    #[test]
    fn eval_equivalence_casteljau() {
        // all eval methods should be approximately equivalent for well defined test cases
        // and not equivalent where numerical stability becomes an issue for normal eval
        let bezier = CubicBezier::<_, 2>::new(
            PointN::new([0.0, 1.77]),
            PointN::new([1.1, -1.0]),
            PointN::new([4.3, 3.0]),
            PointN::new([3.2, -4.0]),
        );

        let nsteps: usize = 1000;
        for t in 0..=nsteps {
            let t = t as NativeFloat * 1.0 / (nsteps as NativeFloat);
            let p1 = bezier.eval(t);
            let p2 = bezier.eval_casteljau(t);
            let err = p2 - p1;
            assert!(err.squared_length() < EPSILON);
        }
    }

    #[test]
    fn split_equivalence() {
        // chose some arbitrary control points and construct a cubic bezier
        let bezier = CubicBezier::<_, 2> {
            start: PointN::new([0.0, 1.77]),
            ctrl1: PointN::new([2.9, 0.0]),
            ctrl2: PointN::new([4.3, 3.0]),
            end: PointN::new([3.2, -4.0]),
        };
        // split it at an arbitrary point
        let at = 0.5;
        let (left, right) = bezier.split(at);
        // compare left and right subcurves with parent curve
        // take the difference of the two points which must not exceed the absolute error
        let nsteps: usize = 1000;
        for t in 0..=nsteps {
            let t = t as NativeFloat * 1.0 / (nsteps as NativeFloat);
            // left
            let mut err = bezier.eval(t / 2.0) - left.eval(t);
            assert!(err.squared_length() < EPSILON);
            // right
            err = bezier.eval((t * 0.5) + 0.5) - right.eval(t);
            assert!(err.squared_length() < EPSILON);
        }
    }

    #[test]
    fn bounding_box_contains() {
        // check if bounding box for a curve contains all points (with some approximation error)
        let bezier = CubicBezier::<_, 2> {
            start: PointN::new([0.0, 1.77]),
            ctrl1: PointN::new([2.9, 0.0]),
            ctrl2: PointN::new([4.3, -3.0]),
            end: PointN::new([3.2, 4.0]),
        };

        let bounds = bezier.bounding_box();

        let max_err = 1e-2;

        let nsteps: usize = 100;
        for t in 0..=nsteps {
            let t = t as NativeFloat * 1.0 / (nsteps as NativeFloat);
            let p = bezier.eval_casteljau(t);
            //dbg!(t);
            //dbg!(p);
            //dbg!(xmin-max_err, ymin-max_err, xmax+max_err, ymax+max_err);
            for (idx, axis) in p.into_iter().enumerate() {
                assert!((axis >= (bounds[idx].0 - max_err)) && (axis <= (bounds[idx].1 + max_err)))
            }
        }
    }

    #[test]
    fn distance_to_point() {
        // degree 3, 4 control points => 4+3+1=8 knots
        let curve = CubicBezier::<_, 2> {
            start: PointN::new([0.0, 1.77]),
            ctrl1: PointN::new([1.1, -1.0]),
            ctrl2: PointN::new([4.3, 3.0]),
            end: PointN::new([3.2, -4.0]),
        };
        assert!(
            curve.distance_to_point(PointN::new([-5.1, -5.6]))
                > curve.distance_to_point(PointN::new([5.1, 5.6]))
        );
    }
}
