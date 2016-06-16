extern crate num;

#[macro_use]
mod macros;
mod nbez;
pub mod traitdefs;
pub use nbez::*;
use traitdefs::Float;

#[inline]
fn lerp<F: Float>(a: F, b: F, factor: F) -> F {
    let fact1 = F::from_f32(1.0).unwrap() - factor;
    a * factor + b * fact1
}

// There are macros in place to make it easier to create new bezier structs, as they can be created
// with a very consistent pattern. However, those macros are also written in a very consistent pattern
// which unfortunately is significantly harder, if not impossible, to create with a traditional
// macro. So, the macro invocations are generated with the build script and then inserted here.
include!(concat!(env!("OUT_DIR"), "/macro_invocs.rs"));

impl<F: Float> Vector2d<F> {
    pub fn perp(self) -> Vector2d<F> {
        Vector2d {
            x: -self.y,
            y: self.x
        }
    }
}

pub struct BezIter<F: Float, B: BezCurve<F>> {
    points: *const B::Point,
    len: usize,
    order: usize
}

impl<F: Float, B: BezCurve<F>> Iterator for BezIter<F, B> {
    type Item = B;
    fn next(&mut self) -> Option<B> {
        use std::slice;

        if self.len <= self.order {
            None
        } else {unsafe{
            let slice = slice::from_raw_parts(self.points, self.order + 1);
            self.points = self.points.offset(self.order as isize);
            self.len -= self.order;
            B::from_slice(slice)
        }}
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = (self.len - (self.len - 1) % self.order) / self.order;
        (size, Some(size))
    }
}

impl<F: Float, B: BezCurve<F>> DoubleEndedIterator for BezIter<F, B> {
    fn next_back(&mut self) -> Option<B> {
        use std::slice;

        // If there are any control points in the iterator that can't be used to create a full curve,
        // ignore them.
        let end = self.len - (self.len - 1) % self.order;
        if end <= self.order {            
            None
        } else {unsafe{
            let slice = slice::from_raw_parts(self.points.offset((end-self.order-1) as isize), self.order + 1);
            self.len -= self.order;
            B::from_slice(slice)
        }}
    }
}

impl<F: Float, B:BezCurve<F>> ExactSizeIterator for BezIter<F, B> {}


pub trait BezCurve<F: Float> 
        where Self: Sized {
    type Point;
    type Vector;
    type Elevated;

    /// Attempt to create a curve from a slice. Fails if the slice's length does not match the
    /// curve's order + 1, or if it is being used to create an `NBez`/`NBezPoly`.
    fn from_slice(&[Self::Point]) -> Option<Self>;

    /// Perform interpolation on the curve for a value of `t` from `0.0` to `1.0` inclusive. Returns `None`
    /// if `t` is outside of that range.
    fn interp(&self, t: F) -> Option<Self::Point> {
        check_t_bounds!(t);
        Some(self.interp_unbounded(t))
    }
    /// Perform interpolation on the curve with no range bounds
    fn interp_unbounded(&self, t: F) -> Self::Point;

    /// Get the slope for the given `t`, bounded on `0.0` to `1.0` inclusive. Returns `None` if
    /// `t` is not within bounds.
    fn slope(&self, t: F) -> Option<Self::Vector> {
        check_t_bounds!(t);
        Some(self.slope_unbounded(t))
    }
    /// Get the slope for the given `t` with no range bounds
    fn slope_unbounded(&self, t: F) -> Self::Vector;

    fn elevate(&self) -> Self::Elevated;
    
    /// Gets the order of the curve
    fn order(&self) -> usize;
}

/// Gets the statically-known order of the curve
pub trait OrderStatic {
    fn order_static() -> usize;
}