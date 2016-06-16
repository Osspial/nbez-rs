extern crate num;

#[macro_use]
mod macros;
mod nbez;
pub mod traitdefs;
pub mod traits;
pub use nbez::*;
use traitdefs::Float;
use traits::BezCurve;

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