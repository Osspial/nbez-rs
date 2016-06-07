use traitdefs::{Float, Point, Vector};
use std::convert::{AsRef, AsMut, From};
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;

use super::traits::BezCurve;
use super::lerp;

/// A struct that contains range information for slicing, used for slicing into the global factor
/// vector. The reason this is used instead of stdlib's `Range` struct is that `Range` does not
/// implement Copy, which means we have to use `RefCell`s instead of `Cell`s for interior mutability.
#[derive(Copy, Clone)]
struct RangeSlice {
    start: usize,
    end: usize
}

impl RangeSlice {
    #[inline]
    fn new(start: usize, end: usize) -> RangeSlice {
        RangeSlice {
            start: start,
            end: end
        }
    }

    fn len(&self) -> usize {
        self.end - self.start
    }
}

fn combination(n: u64, k: u64) -> u64 {
    factorial(n) / (factorial(k) * factorial(n - k))
}

fn factorial(mut n: u64) -> u64 {
    let mut accumulator: u64 = 1;
    while n > 0 {
        accumulator = accumulator.checked_mul(n).expect("Attempted to create Bézier curve with factors that overflow u64; decrease curve order");
        n -= 1;
    }
    accumulator
}

/// Gets the index of the bezier factors inside of FACTORS global.
fn order_index(order: usize) -> usize {
    (order*order+order)/2
}

thread_local!{
    static FACTORS: RefCell<(isize, Vec<u64>)> = RefCell::new((-1, Vec::with_capacity(order_index(20+1))))
}

/// Returns a RangeSlice for FACTORS with the appropriate factors for the given order. Calculates
/// factors if necessary.
fn factors(order: usize) -> RangeSlice {
    FACTORS.with(|f| {
        let max_order = f.borrow().0;
        if order as isize > max_order {
            let mut f = f.borrow_mut();
            f.0 = order as isize;

            // Because max_order defines the maximum current order, we need to increment it in order to avoid
            // re-pushing the current max order. Also, we increment the upper bound, `order`, so that the
            // calculations include `order`.
            for n in (max_order+1) as usize..order+1 {
                for k in 0..n+1 {
                    f.1.push(combination(n as u64, k as u64));
                }
            }
        }

        let order_index = order_index(order);
        RangeSlice::new(order_index, order_index+order+1)
    })
}


/// An n-order bezier polynomial
#[derive(Clone)]
pub struct NBezPoly<F, C = Vec<F>> 
        where F: Float,
              C: AsRef<[F]> {
    points: C,
    factors: Cell<RangeSlice>,
    dfactors: Cell<RangeSlice>,
    phantom: PhantomData<F>
}

impl<F, C> From<C> for NBezPoly<F, C>
        where F: Float,
              C: AsRef<[F]> {
    fn from(container: C) -> NBezPoly<F, C> {
        NBezPoly::from_container(container)
    }
}

impl<F, C> NBezPoly<F, C>
        where F: Float,
              C: AsRef<[F]> {
    #[inline]
    pub fn from_container(points: C) -> NBezPoly<F, C> {
        NBezPoly {
            points: points,
            factors: Cell::new(RangeSlice::new(0, 0)),
            dfactors: Cell::new(RangeSlice::new(0, 0)),
            phantom: PhantomData
        }
    }

    #[inline]
    pub fn unwrap(self) -> C {
        self.points
    }

    unsafe fn interp_unchecked<I: Iterator<Item=F>>(t: F, factors: RangeSlice, iter: I) -> F {
        let t1 = F::from_f32(1.0).unwrap() - t;
        let order = factors.len() - 1;
        let mut acc = F::from_f32(0.0).unwrap();
        let mut factor = 0;

        FACTORS.with(|fs| {
            let fs = fs.borrow();
            for point in iter {
                acc = acc + t.powi(factor as i32) *
                            t1.powi((order-factor) as i32) *
                            point *
                            F::from_u64(fs.1[factors.start + factor]).unwrap();
                factor += 1;
            }            
        });
        acc        
    }

    unsafe fn slope_unchecked<I: Iterator<Item=F>>(t: F, dfactors: RangeSlice, mut iter: I) -> F {
        let t1 = F::from_f32(1.0).unwrap() - t;
        let order = dfactors.len() - 1;
        let mut acc = F::from_f32(0.0).unwrap();
        let mut factor = 0;
        let mut point_next = iter.next().unwrap();

        FACTORS.with(|fs| {
            let fs = fs.borrow();
            for point in iter {
                acc = acc + t.powi(factor as i32) *
                            t1.powi((order-factor) as i32) *
                            (point_next - point) *
                            F::from_u64(fs.1[dfactors.start + factor] * (order + 1) as u64).unwrap();
                point_next = point;
                factor += 1;
            }            
        });
        acc
    }
}

impl<F, C> BezCurve<F> for NBezPoly<F, C> 
        where F: Float,
              C: AsRef<[F]> {
    type Point = F;
    type Vector = F;
    type Elevated = NBezPoly<F>;

    fn from_slice(_: &[F]) -> Option<NBezPoly<F, C>> {
        None
    }

    fn interp_unbounded(&self, t: F) -> F {
        let points = self.points.as_ref();
        if self.factors.get().len() != self.order() {
            self.factors.set(factors(self.order()))
        }

        unsafe{ NBezPoly::<_, &[_]>::interp_unchecked(t, self.factors.get(), points.iter().map(|f| *f)) }
    }

    fn slope_unbounded(&self, t: F) -> F {
        let points = self.points.as_ref();
        let order = self.order() - 1;
        if self.dfactors.get().len() != order {
            self.dfactors.set(factors(order))
        }

        unsafe{ NBezPoly::<_, &[_]>::slope_unchecked(t, self.dfactors.get(), points.iter().map(|f| *f)) }
    }

    fn elevate(&self) -> NBezPoly<F> {
        let points = self.points.as_ref();
        let order = self.order() + 1;
        let order_f = F::from_usize(order).unwrap();
        
        // Elevated points
        let mut el_points = Vec::with_capacity(order + 1);
        el_points.push(points[0]);

        let mut prev_p = points[0];
        for (i, p) in points.iter().map(|p| *p).enumerate().skip(1) {
            el_points.push(lerp(prev_p, p, F::from_usize(i).unwrap()/order_f));

            prev_p = p;
        }

        el_points.push(points[self.order()]);
        NBezPoly::from_container(el_points)
    }

    fn order(&self) -> usize {
        self.points.as_ref().len()-1
    }

    fn order_static() -> Option<usize> {
        None
    }
}

impl<F, C> AsRef<C> for NBezPoly<F, C>
        where F: Float,
              C: AsRef<[F]> {
    fn as_ref(&self) -> &C {
        &self.points
    }
}

impl<F, C> AsMut<C> for NBezPoly<F, C>
        where F: Float,
              C: AsRef<[F]> {
    fn as_mut(&mut self) -> &mut C {
        &mut self.points
    }
}

#[derive(Clone)]
pub struct NBez<P, V, F, C>
        where F: Float,
              C: AsRef<[P]>,
              P: Point<F>,
              V: Vector<F, P> {
    points: C,
    factors: Cell<RangeSlice>,
    dfactors: Cell<RangeSlice>,

    float_phantom: PhantomData<F>,
    point_phantom: PhantomData<P>,
    vector_phantom: PhantomData<V>
}

impl<P, V, F, C> From<C> for NBez<P, V, F, C>
        where F: Float,
              C: AsRef<[P]>,
              P: Point<F>,
              V: Vector<F, P> {
    fn from(container: C) -> NBez<P, V, F, C> {
        NBez::from_container(container)
    }
}

impl<P, V, F, C> NBez<P, V, F, C>
        where F: Float,
              C: AsRef<[P]>,
              P: Point<F>,
              V: Vector<F, P> {
    /// Create a new `NBez` curve from a container
    pub fn from_container(container: C) -> NBez<P, V, F, C> {
        NBez {
            points: container,
            factors: Cell::new(RangeSlice::new(0, 0)),
            dfactors: Cell::new(RangeSlice::new(0, 0)),

            float_phantom: PhantomData,
            point_phantom: PhantomData,
            vector_phantom: PhantomData
        }
    }

    /// Get the wrapped container, destroying the `NBez` curve
    pub fn unwrap(self) -> C {
        self.points
    }
}

impl<P, V, F, C> BezCurve<F> for NBez<P, V, F, C>
        where F: Float,
              C: AsRef<[P]>,
              P: Point<F> + ::std::fmt::Debug,
              V: Vector<F, P> {
    type Point = P;
    type Vector = V;
    type Elevated = NBez<P, V, F, Vec<P>>;

    fn from_slice(_: &[P]) -> Option<NBez<P, V, F, C>> {
        None
    }

    fn interp_unbounded(&self, t: F) -> P {
        let points = self.points.as_ref();

        // Initialize a point by cloning one from the point list
        let mut point = points[0].clone();

        // If the factors aren't correct for the current order, recompute factors
        if self.factors.get().len() != self.order() {
            self.factors.set(factors(self.order()))
        }

        // Iterate over all elements of the point and set them to the interpolated value
        for (i, f) in point.as_mut().iter_mut().enumerate() {
            let iter = points.iter().map(|p| p.as_ref()[i]);
            *f = unsafe{ NBezPoly::<_, &[_]>::interp_unchecked(t, self.factors.get(), iter) };
        }

        point
    }

    fn slope_unbounded(&self, t: F) -> V {
        let points = self.points.as_ref();

        // Initialize a vector by cloning it from the point list and converting it into a vector
        let mut vector: V = points[0].clone().into();
        let order = self.order() - 1;

        if self.dfactors.get().len() != order {
            self.dfactors.set(factors(order))
        }

        // Iterate over all elements of the vector and set them to the interpolated value
        for (i, f) in vector.as_mut().iter_mut().enumerate() {
            let iter = points.iter().map(|p| p.as_ref()[i]);
            *f = unsafe{ NBezPoly::<_, &[_]>::slope_unchecked(t, self.dfactors.get(), iter) };
        }

        vector
    }

    fn elevate(&self) -> NBez<P, V, F, Vec<P>> {
        let points = self.points.as_ref();
        let order = F::from_usize(self.order() + 1).unwrap();

        let mut el_points = Vec::with_capacity(points.len() + 1);
        el_points.push(points[0].clone());

        let mut prev_p = points[0].clone();
        for (i, p) in points.iter().enumerate().skip(1) {
            // Push an arbitrary point onto the vector for modification
            el_points.push(p.clone());

            // Iterate over each dimension of the current (c) and previous (r) point and interpolate
            // between them
            for (d, (c, r)) in p.as_ref().iter().zip(prev_p.as_ref().iter()).map(|(c, r)| (*c, *r)).enumerate() {
                el_points[i].as_mut()[d] = lerp(r, c, F::from_usize(i).unwrap()/order);
            }
            prev_p = p.clone();
        }
        el_points.push(points[points.len()-1].clone());
        NBez::from_container(el_points)
    }

    fn order(&self) -> usize {
        self.points.as_ref().len() - 1
    }

    fn order_static() -> Option<usize> {
        None
    }
}

impl<P, V, F, C> AsRef<C> for NBez<P, V, F, C>
        where F: Float,
              C: AsRef<[P]>,
              P: Point<F>,
              V: Vector<F, P> {
    fn as_ref(&self) -> &C {
        &self.points
    }
}

impl<P, V, F, C> AsMut<C> for NBez<P, V, F, C>
        where F: Float,
              C: AsRef<[P]>,
              P: Point<F>,
              V: Vector<F, P> {
    fn as_mut(&mut self) -> &mut C {
        &mut self.points
    }
}