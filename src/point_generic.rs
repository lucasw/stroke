use core::iter::IntoIterator;
use core::slice;

use super::*;
// use super::Point;

/// Point with dimensions of constant generic size N and of type NatiiveFloat
///
/// (Implemented as Newtype Pattern on an array
/// see book or https://www.worthe-it.co.za/blog/2020-10-31-newtype-pattern-in-rust.html)
/// This type only interacts with the library through
/// the point trait, so you are free to use your own
/// Point/Coord/Vec structures instead by implementing the (small) trait
#[derive(Debug, Copy, Clone)]
pub struct PointN<const N: usize>([NativeFloat; N]);

impl<const N: usize> PointN<N> {
    pub fn new(array: [NativeFloat; N]) -> Self {
        PointN(array)
    }
}

/// Initialize with the Default value for the underlying type
impl<const N: usize> Default for PointN<N> {
    fn default() -> Self {
        PointN([NativeFloat::default(); N])
    }
}

impl<const N: usize> PartialEq for PointN<N>
{
    fn eq(&self, other: &Self) -> bool {
        for i in 0..N {
            if self.0[i] != other.0[i] {
                return false;
            }
        }
        true
    }
}

impl<const N: usize> Add for PointN<N>
{
    type Output = Self;

    fn add(self, other: PointN<N>) -> PointN<N> {
        let mut res = self;
        for i in 0..N {
            res.0[i] = self.0[i] + other.0[i];
        }
        res
    }
}

/// This is not required by the Point trait or library but
/// convenient if you want to use the type externally
impl<const N: usize> Add<NativeFloat> for PointN<N>
{
    type Output = Self;

    fn add(self, _rhs: NativeFloat) -> PointN<N> {
        let mut res = self;
        for i in 0..N {
            res.0[i] = self.0[i] + _rhs;
        }
        res
    }
}

impl<const N: usize> Sub for PointN<N>
{
    type Output = Self;

    fn sub(self, other: PointN<N>) -> PointN<N> {
        let mut res = self;
        for i in 0..N {
            res.0[i] = self.0[i] - other.0[i];
        }
        res
    }
}

/// This is not required by the Point trait or library but
/// convenient if you want to use the type externally
impl<const N: usize> Sub<NativeFloat> for PointN<N>
{
    type Output = Self;

    fn sub(self, _rhs: NativeFloat) -> PointN<N> {
        let mut res = self;
        for i in 0..N {
            res.0[i] = self.0[i] - _rhs;
        }
        res
    }
}

impl<const N: usize> Mul<NativeFloat> for PointN<N>
where
    // The mulitplication is done by mulitpling T * U => T, this
    // trait bound for T will specify this requirement as the mul operator is
    // translated to using the first operand as self and the second as rhs.
{
    type Output = PointN<N>;

    fn mul(self, rhs: NativeFloat) -> PointN<N> {
        let mut res = self;
        for i in 0..res.0.len() {
            res.0[i] *= rhs;
        }
        res
    }
}

impl<const N: usize> IntoIterator for PointN<N> {
    type Item = NativeFloat;
    type IntoIter = core::array::IntoIter<Self::Item, N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(self.0)
    }
}

impl<'a, const N: usize> IntoIterator for &'a mut PointN<N> {
    type Item = &'a mut NativeFloat;
    type IntoIter = slice::IterMut<'a, NativeFloat>;

    fn into_iter(self) -> slice::IterMut<'a, NativeFloat> {
        self.0.iter_mut()
    }
}

impl<const N: usize> Point for PointN<N>
where
{
    // type Scalar = NativeFloat;
    const DIM: usize = { N };

    fn axis(&self, index: usize) -> NativeFloat {
        assert!(index <= N);
        self.0[index]
    }

    fn squared_length(&self) -> NativeFloat {
        let mut sqr_dist: NativeFloat = 0.0;
        for i in 0..N {
            sqr_dist += self.0[i] * self.0[i];
        }
        sqr_dist
    }
}
