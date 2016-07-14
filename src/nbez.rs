use traitdefs::{Float, Point, Vector};
use std::convert::{AsRef, AsMut, From};
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::fmt::{Debug, Formatter};
use std::ops::Range;


use super::{BezCurve, lerp};

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

    fn as_range(&self) -> Range<usize> {
        self.start..self.end
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
        accumulator = accumulator.checked_mul(n).expect("Attempted to create Bézier curve with combination that overflow u64; decrease curve order");
        n -= 1;
    }
    accumulator
}

/// Given the `order` and references to the `factors`, `dfactors`, and `vec` cells, update the
/// cells to contain accurate information about the factors of the order. 
fn update_factors(order: usize, factors: &Cell<RangeSlice>, dfactors: &Cell<RangeSlice>, vec: &RefCell<Vec<u64>>) {
    if factors.get().len() != order + 1 {
        let mut vec = vec.borrow_mut();
        // Remove everything from the vector without freeing memory
        unsafe{ vec.set_len(0) };

        // The vector stores both the factors of the order and the order's derivative, and this is the
        // length necessary to contain those factors.
        let new_len = (order + 1) * 2 - 1;
        if vec.capacity() < new_len {
            let reserve_amount = new_len - vec.capacity();
            vec.reserve(reserve_amount);
        }

        {
            let order = order as u64;

            for k in 0..order + 1 {
                vec.push(combination(order, k));
            }

            for k in 0..order {
                vec.push(combination(order - 1, k));
            }
        }

        factors.set(RangeSlice::new(0, order + 1));
        dfactors.set(RangeSlice::new(order + 1, vec.len()));
    }
}


/// An n-order bezier polynomial
#[derive(Clone)]
pub struct NBezPoly<F, C = Vec<F>> 
        where F: Float,
              C: AsRef<[F]> {
    points: C,
    factor_vec: RefCell<Vec<u64>>,
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
        if points.as_ref().len() >= 22 {
            panic!("Cannot create Bézier polynomials with an order >= 21")
        }

        NBezPoly {
            points: points,
            factor_vec: RefCell::new(Vec::new()),
            factors: Cell::new(RangeSlice::new(0, 0)),
            dfactors: Cell::new(RangeSlice::new(0, 0)),
            phantom: PhantomData
        }
    }

    #[inline]
    pub fn unwrap(self) -> C {
        self.points
    }

    unsafe fn interp_unchecked<I: Iterator<Item=F>>(t: F, factors: &[u64], iter: I) -> F {
        let t1 = F::from_f32(1.0).unwrap() - t;
        let order = factors.len() - 1;
        let mut acc = F::from_f32(0.0).unwrap();
        let mut factor = 0;

        for point in iter {
            acc = acc + t.powi(factor as i32) *
                        t1.powi((order-factor) as i32) *
                        point *
                        F::from_u64(factors[factor]).unwrap();
            factor += 1;
        }            
        acc        
    }

    unsafe fn slope_unchecked<I: Iterator<Item=F>>(t: F, dfactors: &[u64], mut iter: I) -> F {
        let t1 = F::from_f32(1.0).unwrap() - t;
        let order = dfactors.len() - 1;
        let mut acc = F::from_f32(0.0).unwrap();
        let mut factor = 0;
        let mut point_next = iter.next().unwrap();

        for point in iter {
            acc = acc + t.powi(factor as i32) *
                        t1.powi((order-factor) as i32) *
                        (point - point_next) *
                        F::from_u64(dfactors[factor] * (order + 1) as u64).unwrap();
            point_next = point;
            factor += 1;
        }            
        acc
    }
}

impl<F, C> BezCurve<F> for NBezPoly<F, C> 
        where F: Float,
              C: AsRef<[F]> {
    type Point = F;
    type Vector = F;
    type Elevated = NBezPoly<F>;

    /// Currently non-functional; returns `None`
    fn from_slice(_: &[F]) -> Option<NBezPoly<F, C>> {
        None
    }

    fn interp_unbounded(&self, t: F) -> F {
        let points = self.points.as_ref();
        update_factors(self.order(), &self.factors, &self.dfactors, &self.factor_vec);

        unsafe{ NBezPoly::<_, &[_]>::interp_unchecked(t, &self.factor_vec.borrow()[self.factors.get().as_range()], points.iter().map(|f| *f)) }
    }

    fn slope_unbounded(&self, t: F) -> F {
        let points = self.points.as_ref();
        update_factors(self.order(), &self.factors, &self.dfactors, &self.factor_vec);

        unsafe{ NBezPoly::<_, &[_]>::slope_unchecked(t, &self.factor_vec.borrow()[self.dfactors.get().as_range()], points.iter().map(|f| *f)) }
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
            el_points.push(lerp(p, prev_p, F::from_usize(i).unwrap()/order_f));

            prev_p = p;
        }

        el_points.push(points[self.order()]);
        NBezPoly::from_container(el_points)
    }

    /// Currently non-functional; returns `None`
    fn split(&self, _: F) -> Option<(NBezPoly<F, C>, NBezPoly<F, C>)> {
        None
    }

    fn order(&self) -> usize {
        self.points.as_ref().len()-1
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

impl<F, C> Debug for NBezPoly<F, C>
        where F: Float,
              C: AsRef<[F]> + Debug {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        f.debug_tuple("NBezPoly")
            .field(&self.points)
            .finish()
    }
}


#[derive(Clone)]
pub struct NBez<P, V, F, C>
        where F: Float,
              C: AsRef<[P]>,
              P: Point<F>,
              V: Vector<F, P> {
    points: C,
    factor_vec: RefCell<Vec<u64>>,
    factors: Cell<RangeSlice>,
    dfactors: Cell<RangeSlice>,

    phantom: PhantomData<(F, P, V)>
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
        if container.as_ref().len() >= 22 {
            panic!("Cannot create Bézier curves with an order >= 21")
        }

        NBez {
            points: container,
            factor_vec: RefCell::new(Vec::new()),
            factors: Cell::new(RangeSlice::new(0, 0)),
            dfactors: Cell::new(RangeSlice::new(0, 0)),

            phantom: PhantomData,
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
              P: Point<F>,
              V: Vector<F, P> {
    type Point = P;
    type Vector = V;
    type Elevated = NBez<P, V, F, Vec<P>>;

    /// Currently non-functional; returns `None`
    fn from_slice(_: &[P]) -> Option<NBez<P, V, F, C>> {
        None
    }

    fn interp_unbounded(&self, t: F) -> P {
        let points = self.points.as_ref();

        // Initialize a point by cloning one from the point list
        let mut point = points[0].clone();

        update_factors(self.order(), &self.factors, &self.dfactors, &self.factor_vec);

        // Iterate over all elements of the point and set them to the interpolated value
        for (i, f) in point.as_mut().iter_mut().enumerate() {
            let iter = points.iter().map(|p| p.as_ref()[i]);
            *f = unsafe{ NBezPoly::<_, &[_]>::interp_unchecked(t, &self.factor_vec.borrow()[self.factors.get().as_range()], iter) };
        }

        point
    }

    fn slope_unbounded(&self, t: F) -> V {
        let points = self.points.as_ref();

        // Initialize a vector by cloning it from the point list and converting it into a vector
        let mut vector: V = points[0].clone().into();

        update_factors(self.order(), &self.factors, &self.dfactors, &self.factor_vec);

        // Iterate over all elements of the vector and set them to the interpolated value
        for (i, f) in vector.as_mut().iter_mut().enumerate() {
            let iter = points.iter().map(|p| p.as_ref()[i]);
            *f = unsafe{ NBezPoly::<_, &[_]>::slope_unchecked(t, &self.factor_vec.borrow()[self.dfactors.get().as_range()], iter) };
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
                el_points[i].as_mut()[d] = lerp(c, r, F::from_usize(i).unwrap()/order);
            }
            prev_p = p.clone();
        }
        el_points.push(points[points.len()-1].clone());
        NBez::from_container(el_points)
    }

    /// Currently non-functional; returns `None`
    fn split(&self, _: F) -> Option<(NBez<P, V, F, C>, NBez<P, V, F, C>)> {
        None
    }

    fn order(&self) -> usize {
        self.points.as_ref().len() - 1
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

impl<P, V, F, C> Debug for NBez<P, V, F, C>
        where F: Float,
              C: AsRef<[P]> + Debug,
              P: Point<F>,
              V: Vector<F, P> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        f.debug_tuple("NBez")
            .field(&self.points)
            .finish()
    }
}