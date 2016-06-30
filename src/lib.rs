extern crate num;

#[macro_use]
mod macros;
pub mod traitdefs;

mod nbez;
pub use nbez::*;

use traitdefs::Float;
use std::fmt::{Debug, Formatter};
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

/// Iterator over bezier curve chains
pub struct BezIter<'a, F, B>
        where F: Float,
              B: BezCurve<F> + OrderStatic {
    points: *const B::Point,
    len: usize,
    lifetime: PhantomData<&'a ()>
}

impl<'a, F, B> Iterator for BezIter<'a, F, B>
        where F: Float,
              B: BezCurve<F> + OrderStatic {
    type Item = B;
    fn next(&mut self) -> Option<B> {
        use std::slice;

        let order = B::order_static();

        if self.len <= order {
            None
        } else {unsafe{
            let slice = slice::from_raw_parts(self.points, order + 1);
            self.points = self.points.offset(order as isize);
            self.len -= order;
            B::from_slice(slice)
        }}
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let order = B::order_static();
        let size = (self.len - (self.len - 1) % order) / order;
        (size, Some(size))
    }
}

impl<'a, F, B> DoubleEndedIterator for BezIter<'a, F, B>
        where F: Float,
              B: BezCurve<F> + OrderStatic {
    fn next_back(&mut self) -> Option<B> {
        use std::slice;

        let order = B::order_static();

        // If there are any control points in the iterator that can't be used to create a full curve,
        // ignore them.
        let end = self.len - (self.len - 1) % order;
        if end <= order {            
            None
        } else {unsafe{
            let slice = slice::from_raw_parts(self.points.offset((end-order-1) as isize), order + 1);
            self.len -= order;
            B::from_slice(slice)
        }}
    }
}

impl<'a, F, B> ExactSizeIterator for BezIter<'a, F, B> 
        where F: Float,
              B: BezCurve<F> + OrderStatic {}


/// Bezier curve trait
pub trait BezCurve<F: Float> 
        where Self: Sized {
    type Point;
    type Vector;
    type Elevated: BezCurve<F, Point = Self::Point, Vector = Self::Vector>;

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
    /// Get the slope for the given `t` with no range bounds.
    fn slope_unbounded(&self, t: F) -> Self::Vector;

    /// Elevate the curve order, getting a curve that is one order higher but gives the same results
    /// upon interpolation
    fn elevate(&self) -> Self::Elevated;
    
    /// Gets the order of the curve
    fn order(&self) -> usize;
}

/// Gets the statically-known order of the curve
pub trait OrderStatic {
    fn order_static() -> usize;
}


/// A chain on bezier curves, with the last point of each curve being the first point of the next.
#[derive(Clone, Copy)]
pub struct BezChain<F, B, C>
        where F: Float,
              B: BezCurve<F> + OrderStatic,
              C: AsRef<[B::Point]> {
    points: C,
    phantom: PhantomData<(F, B)>
}

impl<F, B, C> BezChain<F, B, C>
        where F: Float,
              B: BezCurve<F> + OrderStatic,
              C: AsRef<[B::Point]> {
    /// Create a new BezChain by wrapping around a container.
    #[inline]
    pub fn from_container(container: C) -> BezChain<F, B, C> {
        BezChain {
            points: container,
            phantom: PhantomData
        }
    }

    /// Get the bezier curve that is `index` curves away from the start. Returns `None` if not enough
    /// points exist for the given curve index.
    pub fn get(&self, index: usize) -> Option<B> {
        let len = self.points.as_ref().len();
        let order = B::order_static();
        let curve_index = index * order;
        let curve_end_index = index * order + order + 1;

        if curve_end_index > len - (len - 1) % order {
            None
        } else {
            Some(B::from_slice(&self.points.as_ref()[curve_index..curve_end_index]).unwrap())
        }
    }

    /// Get an iterator over all curves in the chain.
    #[inline]
    pub fn iter(&self) -> BezIter<F, B> {
        BezIter {
            points: self.points.as_ref().as_ptr(),
            len: self.points.as_ref().len(),
            lifetime: PhantomData
        }
    }

    /// Get the order of the chain's curves. Identical to order_static().
    #[inline]
    pub fn order(&self) -> usize {
        B::order_static()
    }

    /// Unwrap the chain, returning the underlying container.
    #[inline]
    pub fn unwrap(self) -> C {
        self.points
    }
}

impl<F, B, C> OrderStatic for BezChain<F, B, C>
        where F: Float,
              B: BezCurve<F> + OrderStatic,
              C: AsRef<[B::Point]> {
    #[inline]
    fn order_static() -> usize {
        B::order_static()
    }
}

impl<F, B, C> AsRef<C> for BezChain<F, B, C>
        where F: Float,
              B: BezCurve<F> + OrderStatic,
              C: AsRef<[B::Point]> {
    fn as_ref(&self) -> &C {
        &self.points
    }
}

impl<F, B, C> AsMut<C> for BezChain<F, B, C>
        where F: Float,
              B: BezCurve<F> + OrderStatic,
              C: AsRef<[B::Point]> {
    fn as_mut(&mut self) -> &mut C {
        &mut self.points
    }
}

impl<F, B, C> Debug for BezChain<F, B, C>
        where F: Float,
              B: BezCurve<F> + OrderStatic,
              C: AsRef<[B::Point]> + Debug {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        f.debug_tuple("BezChain")
            .field(&self.points)
            .finish()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn test_poly_eq<B1, B2>(nbez_poly: &B1, bez_poly: &B2)
            where B1: BezCurve<f64, Point = f64>,
                  B2: BezCurve<f64, Point = f64> {
        let mut t = 0.0;
        while t <= 1.0 {
            // Assert that values are equal within a tolerance.
            assert!((nbez_poly.interp(t).unwrap() - bez_poly.interp(t).unwrap()).abs() <= 0.000000001);
            t += 1.0/30.0;
        }
    }

    fn test_poly_slope_eq<B1, B2>(nbez_poly: &B1, bez_poly: &B2) 
            where B1: BezCurve<f64, Vector = f64>, 
                  B2: BezCurve<f64, Vector = f64> {
        let mut t = 0.0;
        while t <= 1.0 {
            // Ditto.
            assert!((nbez_poly.slope(t).unwrap() - bez_poly.slope(t).unwrap()).abs() <= 0.000000001);
            t += 1.0/30.0;
        }
    }

    #[test]
    fn bez_poly_equiviliance() {
        let mut nbez_poly = NBezPoly::from_container(Vec::with_capacity(7));
        nbez_poly.as_mut().push(0.0);
        nbez_poly.as_mut().push(1.0);

        let bezpoly1o = BezPoly1o::new(0.0, 1.0);
        test_poly_eq(&nbez_poly, &bezpoly1o);
        test_poly_slope_eq(&nbez_poly, &bezpoly1o);

        nbez_poly.as_mut().push(-1.0);
        let bezpoly2o = BezPoly2o::new(0.0, 1.0, -1.0);
        test_poly_eq(&nbez_poly, &bezpoly2o);
        test_poly_slope_eq(&nbez_poly, &bezpoly2o);
        
        nbez_poly.as_mut().push(2.0);
        let bezpoly3o = BezPoly3o::new(0.0, 1.0, -1.0, 2.0);
        test_poly_eq(&nbez_poly, &bezpoly3o);
        test_poly_slope_eq(&nbez_poly, &bezpoly3o);
        
        nbez_poly.as_mut().push(-2.0);
        let bezpoly4o = BezPoly4o::new(0.0, 1.0, -1.0, 2.0, -2.0);
        test_poly_eq(&nbez_poly, &bezpoly4o);
        test_poly_slope_eq(&nbez_poly, &bezpoly4o);
        
        nbez_poly.as_mut().push(3.0);
        let bezpoly5o = BezPoly5o::new(0.0, 1.0, -1.0, 2.0, -2.0, 3.0);
        test_poly_eq(&nbez_poly, &bezpoly5o);
        test_poly_slope_eq(&nbez_poly, &bezpoly5o);
        
        nbez_poly.as_mut().push(-3.0);
        let bezpoly6o = BezPoly6o::new(0.0, 1.0, -1.0, 2.0, -2.0, 3.0, -3.0);
        test_poly_eq(&nbez_poly, &bezpoly6o);
        test_poly_slope_eq(&nbez_poly, &bezpoly6o);
    }

    fn test_bez_elevation<B>(curve: &B)
            where B: BezCurve<f64, Point = f64, Vector = f64> {
        
        let elevated = curve.elevate();
        test_poly_eq(curve, &elevated);
        test_poly_slope_eq(curve, &elevated);
    }

    #[test]
    fn bez_elevation() {
        test_bez_elevation(&BezPoly1o::new(0.0, 1.0));
        test_bez_elevation(&BezPoly2o::new(0.0, 1.0, -1.0));
        test_bez_elevation(&BezPoly3o::new(0.0, 1.0, -1.0, 2.0));
        test_bez_elevation(&BezPoly4o::new(0.0, 1.0, -1.0, 2.0, -2.0));
        test_bez_elevation(&BezPoly5o::new(0.0, 1.0, -1.0, 2.0, -2.0, 3.0));
        test_bez_elevation(&BezPoly6o::new(0.0, 1.0, -1.0, 2.0, -2.0, 3.0, -3.0));
    }

    #[test]
    fn nbez_elevation() {
        let mut nbez_poly = NBezPoly::from_container(Vec::with_capacity(7));
        nbez_poly.as_mut().push(0.0);
        nbez_poly.as_mut().push(1.0);

        test_bez_elevation(&nbez_poly);

        nbez_poly.as_mut().push(-1.0);
        test_bez_elevation(&nbez_poly);
        
        nbez_poly.as_mut().push(2.0);
        test_bez_elevation(&nbez_poly);
        
        nbez_poly.as_mut().push(-2.0);
        test_bez_elevation(&nbez_poly);
        
        nbez_poly.as_mut().push(3.0);
        test_bez_elevation(&nbez_poly);
        
        nbez_poly.as_mut().push(-3.0);
        test_bez_elevation(&nbez_poly);
    }
}