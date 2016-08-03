extern crate num_traits;

#[macro_use]
mod macros;

mod markers;
pub use markers::*;

mod nbez;
pub use nbez::*;

use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

#[inline] 
fn lerp<PV: PVOps<F>, F: Float>(a: PV, b: PV, factor: F) -> PV { 
    let fact1 = F::from_f32(1.0).unwrap() - factor; 
    a * fact1 + b * factor 
}

// There are macros in place to make it easier to create new bezier structs, as they can be created
// with a very consistent pattern. However, those macros are also written in a very consistent pattern
// which unfortunately is significantly harder, if not impossible, to create with a traditional
// macro. So, the macro invocations are generated with the build script and then inserted here.
include!(concat!(env!("OUT_DIR"), "/macro_invocs.rs"));

impl<F: Float> Vector2d<F> {
    /// Returns a vector perpendicular to `self`. Currently only implemented for 2-dimensional vectors.
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

/// An iterator over a bezier curve's interpolated points
pub struct InterpIter<'a, F, B>
        where F: Float,
              B: 'a + BezCurve<F> {
    curve: &'a B,
    /// Instead of storing t dirctly, we store t as an integer that is then divided by `samples` in
    /// order to get the actual t. This is done to avoid floating-point precision errors. Initialized
    /// to `0`.
    t_nodiv: u32,
    /// Like `t_nodiv`, but used as the upper bound for next_back. Initialized to `samples`'s integer
    /// representation.
    t_back_nodiv: u32,
    /// The number of samples we are taking from the interpolated curve.
    samples: F
}

impl<'a, F, B> Iterator for InterpIter<'a, F, B>
        where F: Float,
              B: BezCurve<F> {
    type Item = B::Point;
    fn next(&mut self) -> Option<B::Point> {
        if self.t_nodiv <= self.t_back_nodiv {
            let point = self.curve.interp(F::from_u32(self.t_nodiv).unwrap() / self.samples);
            self.t_nodiv += 1;
            point
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = if self.t_back_nodiv < self.t_nodiv {
            0
        } else {
            self.t_back_nodiv - self.t_nodiv + 1
        } as usize;

        (size, Some(size))
    }
}

impl<'a, F, B> DoubleEndedIterator for InterpIter<'a, F, B>
        where F: Float,
              B: BezCurve<F> {
    fn next_back(&mut self) -> Option<B::Point> {
        if self.t_nodiv <= self.t_back_nodiv {
            let point = self.curve.interp(F::from_u32(self.t_back_nodiv).unwrap() / self.samples);

            // Because `t_back_nodiv` is unsigned, we can't let it go below zero. So, this checks if
            // `t_back_nodiv` is zero, and if it is set `t_nodiv` to 1, which causes any future calls
            // to the iterator to properly return `None`
            if 0 == self.t_back_nodiv {
                self.t_nodiv = 1;
            } else {
                self.t_back_nodiv -= 1;
            }

            point
        } else {
            None
        }
    }
}

impl<'a, F, B> ExactSizeIterator for InterpIter<'a, F, B>
        where F: Float,
              B: BezCurve<F> {}

/// Bezier curve trait
pub trait BezCurve<F: Float>: AsRef<[<Self as BezCurve<F>>::Point]> + AsMut<[<Self as BezCurve<F>>::Point]>
        where Self: Sized {
    type Point: Point<F>;
    type Elevated: BezCurve<F, Point = Self::Point>;

    /// Attempt to create a curve from a slice. Fails if the slice's length does not match the
    /// curve's order + 1.
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
    fn slope(&self, t: F) -> Option<<Self::Point as Point<F>>::Vector> {
        check_t_bounds!(t);
        Some(self.slope_unbounded(t))
    }
    /// Get the slope for the given `t` with no range bounds
    fn slope_unbounded(&self, t: F) -> <Self::Point as Point<F>>::Vector;

    /// Elevate the curve order, getting a curve that is one order higher but gives the same results
    /// upon interpolation
    fn elevate(&self) -> Self::Elevated;

    /// Split the curve at the given `t`, bounded on `0.0` to `1.0` inclusive. Returns `None` if `t` is
    /// not within bounds.
    fn split(&self, t: F) -> Option<(Self, Self)> {
        check_t_bounds!(t);
        Some(self.split_unbounded(t))
    }

    /// Split the curve with no range bounds
    fn split_unbounded(&self, t: F) -> (Self, Self);
    
    /// Gets the order of the curve
    fn order(&self) -> usize;

    fn interp_iter<'a>(&'a self, samples: u32) -> InterpIter<'a, F, Self> {
        InterpIter {
            curve: self,
            t_nodiv: 0,
            t_back_nodiv: samples,
            samples: F::from_u32(samples).unwrap()
        }
    }
}

/// Trait to mark curves that have order known at compiletime.
pub trait OrderStatic {
    /// Gets the compiletime-known curve order.
    fn order_static() -> usize;
}


/// A chain of bezier curves, with the last point of each curve being the first point of the next.
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
            where B1: BezCurve<f64, Point = f64>,
                  B2: BezCurve<f64, Point = f64> {
        let mut t = 0.0;
        while t <= 1.0 {
            // Ditto.
            assert!((nbez_poly.slope(t).unwrap() - bez_poly.slope(t).unwrap()).abs() <= 0.000000001);
            t += 1.0/30.0;
        }
    }

    #[test]
    fn bez_poly_equiviliance() {
        let mut nbez_poly: NBez<f64, f64, Vec<f64>> = NBez::from_container(Vec::with_capacity(7));
        (nbez_poly.as_mut() as &mut Vec<f64>).push(0.0);
        (nbez_poly.as_mut() as &mut Vec<f64>).push(1.0);

        let bez1o = Bez1o::new(0.0, 1.0);
        test_poly_eq(&nbez_poly, &bez1o);
        test_poly_slope_eq(&nbez_poly, &bez1o);

        (nbez_poly.as_mut() as &mut Vec<f64>).push(-1.0);
        let bez2o = Bez2o::new(0.0, 1.0, -1.0);
        test_poly_eq(&nbez_poly, &bez2o);
        test_poly_slope_eq(&nbez_poly, &bez2o);
        
        (nbez_poly.as_mut() as &mut Vec<f64>).push(2.0);
        let bez3o = Bez3o::new(0.0, 1.0, -1.0, 2.0);
        test_poly_eq(&nbez_poly, &bez3o);
        test_poly_slope_eq(&nbez_poly, &bez3o);
        
        (nbez_poly.as_mut() as &mut Vec<f64>).push(-2.0);
        let bez4o = Bez4o::new(0.0, 1.0, -1.0, 2.0, -2.0);
        test_poly_eq(&nbez_poly, &bez4o);
        test_poly_slope_eq(&nbez_poly, &bez4o);
        
        (nbez_poly.as_mut() as &mut Vec<f64>).push(3.0);
        let bez5o = Bez5o::new(0.0, 1.0, -1.0, 2.0, -2.0, 3.0);
        test_poly_eq(&nbez_poly, &bez5o);
        test_poly_slope_eq(&nbez_poly, &bez5o);
        
        (nbez_poly.as_mut() as &mut Vec<f64>).push(-3.0);
        let bez6o = Bez6o::new(0.0, 1.0, -1.0, 2.0, -2.0, 3.0, -3.0);
        test_poly_eq(&nbez_poly, &bez6o);
        test_poly_slope_eq(&nbez_poly, &bez6o);
    }

    fn test_bez_elevation<B>(curve: &B)
            where B: BezCurve<f64, Point = f64> {
        
        let elevated = curve.elevate();
        test_poly_eq(curve, &elevated);
        test_poly_slope_eq(curve, &elevated);
    }

    #[test]
    fn bez_elevation() {
        test_bez_elevation(&Bez1o::new(0.0, 1.0));
        test_bez_elevation(&Bez2o::new(0.0, 1.0, -1.0));
        test_bez_elevation(&Bez3o::new(0.0, 1.0, -1.0, 2.0));
        test_bez_elevation(&Bez4o::new(0.0, 1.0, -1.0, 2.0, -2.0));
        test_bez_elevation(&Bez5o::new(0.0, 1.0, -1.0, 2.0, -2.0, 3.0));
        test_bez_elevation(&Bez6o::new(0.0, 1.0, -1.0, 2.0, -2.0, 3.0, -3.0));
    }

    #[test]
    fn nbez_elevation() {
        let mut nbez_poly = NBez::from_container(Vec::with_capacity(7));
        (nbez_poly.as_mut() as &mut Vec<f64>).push(0.0);
        (nbez_poly.as_mut() as &mut Vec<f64>).push(1.0);

        test_bez_elevation(&nbez_poly);

        (nbez_poly.as_mut() as &mut Vec<f64>).push(-1.0);
        test_bez_elevation(&nbez_poly);
        
        (nbez_poly.as_mut() as &mut Vec<f64>).push(2.0);
        test_bez_elevation(&nbez_poly);
        
        (nbez_poly.as_mut() as &mut Vec<f64>).push(-2.0);
        test_bez_elevation(&nbez_poly);
        
        (nbez_poly.as_mut() as &mut Vec<f64>).push(3.0);
        test_bez_elevation(&nbez_poly);
        
        (nbez_poly.as_mut() as &mut Vec<f64>).push(-3.0);
        test_bez_elevation(&nbez_poly);
    }

    fn test_bez_split<B>(curve: &B)
            where B: BezCurve<f64, Point = f64> {

        // Sanity check bool to make sure the assertions are actually running
        let mut has_run = false;

        let mut s = 10.0/30.0;
        while s < 1.0 {
            let (left, right) = curve.split(s).unwrap();

            let mut t = 0.0;
            while t < 1.0 {
                if t < s {
                    println!("l {} {}", curve.interp(t).unwrap(), left.interp(t/s).unwrap());
                    assert!((curve.interp(t).unwrap() - left.interp(t/s).unwrap()).abs() <= 0.000000001);
                    has_run = true;

                } else {
                    println!("r {} {}", curve.interp(t).unwrap(), right.interp((t-s)/(1.0 - s)).unwrap());
                    assert!((curve.interp(t).unwrap() - right.interp((t-s)/(1.0 - s)).unwrap()).abs() <= 0.000000001);
                    has_run = true;
                }

                t += 1.0/30.0;
            }

            s += 1.0/30.0;
        }

        assert!(has_run);
        println!("");
    }

    #[test]
    fn bez_split() {
        let bez1o = Bez1o::new(0.0, 1.0);
        test_bez_split(&bez1o);

        let bez2o = Bez2o::new(0.0, 1.0, -1.0);
        test_bez_split(&bez2o);
        
        let bez3o = Bez3o::new(0.0, 1.0, -1.0, 2.0);
        test_bez_split(&bez3o);
        
        let bez4o = Bez4o::new(0.0, 1.0, -1.0, 2.0, -2.0);
        test_bez_split(&bez4o);
        
        let bez5o = Bez5o::new(0.0, 1.0, -1.0, 2.0, -2.0, 3.0);
        test_bez_split(&bez5o);
        
        let bez6o = Bez6o::new(0.0, 1.0, -1.0, 2.0, -2.0, 3.0, -3.0);
        test_bez_split(&bez6o);
    }

    fn test_interp_iter<B>(curve: &B)
            where B: BezCurve<f64, Point = f64> {

        const SAMPLES: u32 = 30;
        let mut iter = curve.interp_iter(SAMPLES);

        for i in 0..SAMPLES + 1{
            assert_eq!(curve.interp(i as f64 / SAMPLES as f64), iter.next());
        }
        assert_eq!(None, iter.next());

        let mut iter_back = curve.interp_iter(SAMPLES);
        for i in (0..SAMPLES + 1).into_iter().map(|i| SAMPLES - i) {
            assert_eq!(curve.interp(i as f64 / SAMPLES as f64), iter_back.next_back());
        }
        assert_eq!(None, iter_back.next_back());
    }

    #[test]
    fn interp_iter() {
        let bez1o = Bez1o::new(0.0, 1.0);
        test_interp_iter(&bez1o);

        let bez2o = Bez2o::new(0.0, 1.0, -1.0);
        test_interp_iter(&bez2o);
        
        let bez3o = Bez3o::new(0.0, 1.0, -1.0, 2.0);
        test_interp_iter(&bez3o);
        
        let bez4o = Bez4o::new(0.0, 1.0, -1.0, 2.0, -2.0);
        test_interp_iter(&bez4o);
        
        let bez5o = Bez5o::new(0.0, 1.0, -1.0, 2.0, -2.0, 3.0);
        test_interp_iter(&bez5o);
        
        let bez6o = Bez6o::new(0.0, 1.0, -1.0, 2.0, -2.0, 3.0, -3.0);
        test_interp_iter(&bez6o);
    }
}