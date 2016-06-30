extern crate num;

#[macro_use]
mod macros;
pub mod traitdefs;

mod nbez;
pub use nbez::*;

use traitdefs::Float;
use std::marker::PhantomData;

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

pub struct BezIter<'a, F, B>
        where F: Float,
              B: BezCurve<F> + OrderStatic {
    points: *const B::Point,
    len: usize,
    order: usize,
    lifetime: PhantomData<&'a ()>
}

impl<'a, F, B> Iterator for BezIter<'a, F, B>
        where F: Float,
              B: BezCurve<F> + OrderStatic {
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

impl<'a, F, B> DoubleEndedIterator for BezIter<'a, F, B>
        where F: Float,
              B: BezCurve<F> + OrderStatic {
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

impl<'a, F, B> ExactSizeIterator for BezIter<'a, F, B> 
        where F: Float,
              B: BezCurve<F> + OrderStatic {}


pub trait BezCurve<F: Float> 
        where Self: Sized
{
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


#[derive(Clone, Copy)]
pub struct BezChain<F, B, C>
        where F: Float,
              B: BezCurve<F> + OrderStatic,
              C: AsRef<[B::Point]>
{
    points: C,
    phantom: PhantomData<(F, B)>
}

impl<F, B, C> BezChain<F, B, C>
        where F: Float,
              B: BezCurve<F> + OrderStatic,
              C: AsRef<[B::Point]>
{    
    pub fn from_container(container: C) -> BezChain<F, B, C> {
        BezChain {
            points: container,
            phantom: PhantomData
        }
    }

    pub fn get(&self, index: usize) -> B {
        let order = B::order_static();
        let curve_index = index * order;
        B::from_slice(&self.points.as_ref()[curve_index..curve_index + order + 1]).unwrap()
    }

    pub fn iter(&self) -> BezIter<F, B> {
        BezIter {
            points: self.points.as_ref().as_ptr(),
            len: self.points.as_ref().len(),
            order: B::order_static(),
            lifetime: PhantomData
        }
    }

    pub fn order(&self) -> usize {
        B::order_static()
    }

    pub fn unwrap(self) -> C {
        self.points
    }
}

impl<F, B, C> OrderStatic for BezChain<F, B, C>
        where F: Float,
              B: BezCurve<F> + OrderStatic,
              C: AsRef<[B::Point]>
{
    fn order_static() -> usize {
        B::order_static()
    }
}

impl<F, B, C> AsRef<C> for BezChain<F, B, C>
        where F: Float,
              B: BezCurve<F> + OrderStatic,
              C: AsRef<[B::Point]>
{
    fn as_ref(&self) -> &C {
        &self.points
    }
}

impl<F, B, C> AsMut<C> for BezChain<F, B, C>
        where F: Float,
              B: BezCurve<F> + OrderStatic,
              C: AsRef<[B::Point]>
{
    fn as_mut(&mut self) -> &mut C {
        &mut self.points
    }
}

// pub trait BezChain<F, C>: AsRef<C> + AsMut<C>
//         where F: Float,
//               C: AsRef<[<Self::Curve as BezCurve<F>>::Point]>,
//               Self: Sized
// {
//     type Curve: BezCurve<F>;

//     fn from_container(C) -> Self;
//     fn get(&self, usize) -> Self::Curve;
//     fn iter(&self) -> BezIter<F, Self::Curve>;
//     fn order(&self) -> usize;
//     fn unwrap(self) -> C;
// }