use traitdefs::{Float, Point, Vector};
use std::convert::{AsRef, AsMut, From};
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::fmt::{Debug, Formatter};
use std::ops::Range;


use super::{BezCurve, Point2d, Vector2d, lerp};

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


/// An n-order bezier curve
#[derive(Clone)]
pub struct NBez<F, P = Point2d<F>, V = Vector2d<F>, C = Vec<P>> 
        where F: Float,
              P: Point<F, V>,
              V: Vector<F>,
              C: AsRef<[P]> {
    points: C,
    factor_vec: RefCell<Vec<u64>>,
    factors: Cell<RangeSlice>,
    dfactors: Cell<RangeSlice>,
    phantom: PhantomData<(F, P, V)>
}

impl<F, P, V, C> From<C> for NBez<F, P, V, C>
        where F: Float,
              P: Point<F, V>,
              V: Vector<F>,
              C: AsRef<[P]> {
    fn from(container: C) -> NBez<F, P, V, C> {
        NBez::from_container(container)
    }
}

impl<F, P, V, C> NBez<F, P, V, C>
        where F: Float,
              P: Point<F, V>,
              V: Vector<F>,
              C: AsRef<[P]> {
    #[inline]
    pub fn from_container(points: C) -> NBez<F, P, V, C> {
        if points.as_ref().len() >= 22 {
            panic!("Cannot create Bézier polynomials with an order >= 21")
        }

        NBez {
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
}

impl<F, P, V, C> BezCurve<F> for NBez<F, P, V, C> 
        where F: Float,
              P: Point<F, V>,
              V: Vector<F>,
              C: AsRef<[P]> {
    type Point = P;
    type Vector = V;
    type Elevated = NBez<F, P, V, Vec<P>>;

    /// Currently non-functional; returns `None`
    fn from_slice(_: &[P]) -> Option<NBez<F, P, V, C>> {
        None
    }

    fn interp_unbounded(&self, t: F) -> P {
        let points = self.points.as_ref();
        update_factors(self.order(), &self.factors, &self.dfactors, &self.factor_vec);
        let factors = &self.factor_vec.borrow()[self.factors.get().as_range()];


        let t1 = F::from_f32(1.0).unwrap() - t;
        let order = factors.len() - 1;
        let mut acc = P::zero();
        let mut factor = 0;

        for point in points.iter() {
            acc = acc + *point * 
                        t.powi(factor as i32) *
                        t1.powi((order - factor) as i32) *
                        F::from_u64(factors[factor]).unwrap();
            factor += 1;
        }            
        acc
    }

    fn slope_unbounded(&self, t: F) -> V {
        let points = self.points.as_ref();
        update_factors(self.order(), &self.factors, &self.dfactors, &self.factor_vec);
        let dfactors = &self.factor_vec.borrow()[self.dfactors.get().as_range()];

        let t1 = F::from_f32(1.0).unwrap() - t;
        let order = dfactors.len() - 1;
        let mut acc = P::zero();
        let mut factor = 0;
        let mut point_last = points[0].clone();

        for point in points[1..].iter().map(|p| *p) {
            acc = acc + (point - point_last) *
                        t.powi(factor as i32) *
                        t1.powi((order-factor) as i32) *
                        F::from_u64(dfactors[factor] * (order + 1) as u64).unwrap();
            point_last = point;
            factor += 1;
        }            
        acc.into()
    }

    fn elevate(&self) -> NBez<F, P, V, Vec<P>> {        
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
        NBez::from_container(el_points)
    }

    /// Currently non-functional; returns `None`
    fn split(&self, _: F) -> Option<(NBez<F, P, V, C>, NBez<F, P, V, C>)> {
        None
    }

    fn order(&self) -> usize {
        self.points.as_ref().len()-1
    }
}

impl<F, P, V, C> AsRef<C> for NBez<F, P, V, C>
        where F: Float,
              P: Point<F, V>,
              V: Vector<F>,
              C: AsRef<[P]> {
    fn as_ref(&self) -> &C {
        &self.points
    }
}

impl<F, P, V, C> AsMut<C> for NBez<F, P, V, C>
        where F: Float,
              P: Point<F, V>,
              V: Vector<F>,
              C: AsRef<[P]> {
    fn as_mut(&mut self) -> &mut C {
        &mut self.points
    }
}

impl<F, P, V, C> Debug for NBez<F, P, V, C>
        where F: Float,
              P: Point<F, V>,
              V: Vector<F>,
              C: AsRef<[P]> + Debug {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        f.debug_tuple("NBez")
            .field(&self.points)
            .finish()
    }
}
