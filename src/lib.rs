extern crate num;

#[macro_use]
mod macros;
mod nbez;
pub mod traitdefs;
pub mod traits;
pub use nbez::*;
use traitdefs::Float;
use traits::BezCurve;

use std::marker::PhantomData;

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

pub struct BezChain3o2d<F, C>
        where F: Float,
              C: AsRef<[Point2d<F>]>
{
    points: C,
    phantom: PhantomData<F>
}

impl<F, C> BezChain3o2d<F, C>
        where F: Float,
              C: AsRef<[Point2d<F>]>
{
    pub fn from_container(container: C) -> BezChain3o2d<F, C> {
        BezChain3o2d {
            points: container,
            phantom: PhantomData
        }
    }

    pub fn curve(&self, index: usize) -> Bez3o2d<F> {
        use traits::BezCurve;

        let curve_index = index * 3;
        Bez3o2d::from_slice(&self.points.as_ref()[curve_index..curve_index+4]).unwrap()
    }

    pub fn iter(&self) -> BezIter<F, Bez3o2d<F>> {
        BezIter {
            points: self.points.as_ref().as_ptr(),
            len: self.points.as_ref().len(),
            order: 3
        }
    }

    pub fn unwrap(self) -> C {
        self.points
    }
}

impl<F, C> AsRef<C> for BezChain3o2d<F, C> 
        where F: Float,
              C: AsRef<[Point2d<F>]>
{
    fn as_ref(&self) -> &C {
        &self.points
    }
}

impl<F, C> AsMut<C> for BezChain3o2d<F, C> 
        where F: Float,
              C: AsRef<[Point2d<F>]>
{
    fn as_mut(&mut self) -> &mut C {
        &mut self.points
    }
}