use traitdefs::{Float, Point, Vector};
use std::convert::{AsRef, AsMut};
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;

use super::traits::BezCurve;

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

fn combination(n: usize, k: usize) -> usize {
    factorial(n) / (factorial(k) * factorial(n - k))
}

fn factorial(n: usize) -> usize {
    match n {
        0 => 1,
        _ => n * factorial(n-1)
    }
}

fn order_index(order: usize) -> usize {
    (order*order+order)/2
}

thread_local!{
    static FACTORS: RefCell<(usize, Vec<usize>)> = RefCell::new((0, Vec::with_capacity(order_index(16+1))))
}

fn factors(order: usize) -> RangeSlice {
    FACTORS.with(|f| {
        let max_order = f.borrow().0;
        if order > max_order {
            let mut f = f.borrow_mut();
            f.0 = order;
            for n in max_order..order+1 {
                for k in 0..n+1 {
                    f.1.push(combination(n, k));
                }
            }
        }

        let order_index = order_index(order);
        RangeSlice::new(order_index, order_index+order+1)
    })
}


#[derive(Clone)]
pub struct NBezPoly<F, C = Vec<F>> 
        where F: Float,
              C: AsRef<[F]> {
    points: C,
    factors: Cell<RangeSlice>,
    dfactors: Cell<RangeSlice>,
    phantom: PhantomData<F>
}

impl<F, C> NBezPoly<F, C>
        where F: Float,
              C: AsRef<[F]> {
    #[inline]
    fn new(points: C, factors: RangeSlice, dfactors: RangeSlice) -> NBezPoly<F, C> {
        NBezPoly {
            points: points,
            factors: Cell::new(factors),
            dfactors: Cell::new(dfactors),
            phantom: PhantomData
        }
    }

    #[inline]
    pub fn from_container(points: C) -> NBezPoly<F, C> {
        NBezPoly::new(points, RangeSlice::new(0, 0), RangeSlice::new(0, 0))
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
                            F::from_usize(fs.1[factors.start + factor]).unwrap();
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
                            F::from_usize(fs.1[dfactors.start + factor] * (order + 1) as usize).unwrap();
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

pub struct NBez<F, C, P, V>
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

impl<F, C, P, V> NBez<F, C, P, V>
        where F: Float,
              C: AsRef<[P]>,
              P: Point<F>,
              V: Vector<F, P> {
    pub fn from_container(container: C) -> NBez<F, C, P, V> {
        NBez {
            points: container,
            factors: Cell::new(RangeSlice::new(0, 0)),
            dfactors: Cell::new(RangeSlice::new(0, 0)),

            float_phantom: PhantomData,
            point_phantom: PhantomData,
            vector_phantom: PhantomData
        }
    }
}

impl<F, C, P, V> BezCurve<F> for NBez<F, C, P, V>
        where F: Float,
              C: AsRef<[P]>,
              P: Point<F>,
              V: Vector<F, P> {
    type Point = P;
    type Vector = V;

    fn from_slice(_: &[P]) -> Option<NBez<F, C, P, V>> {
        None
    }

    fn interp_unbounded(&self, t: F) -> P {
        let points = self.points.as_ref();
        let mut point = points[0].clone();

        if self.factors.get().len() != self.order() {
            self.factors.set(factors(self.order()))
        }

        for (i, f) in point.as_mut().iter_mut().enumerate() {
            let iter = points.iter().map(|p| p.as_ref()[i]);
            *f = unsafe{ NBezPoly::<_, &[_]>::interp_unchecked(t, self.factors.get(), iter) };
        }

        point
    }

    fn slope_unbounded(&self, t: F) -> V {
        let points = self.points.as_ref();
        let mut vector: V = points[0].clone().into();

        let order = self.order() - 1;
        if self.dfactors.get().len() != order {
            self.dfactors.set(factors(order))
        }

        for (i, f) in vector.as_mut().iter_mut().enumerate() {
            let iter = points.iter().map(|p| p.as_ref()[i]);
            *f = unsafe{ NBezPoly::<_, &[_]>::slope_unchecked(t, self.dfactors.get(), iter) };
        }

        vector
    }

    fn order(&self) -> usize {
        self.points.as_ref().len() - 1
    }

    fn order_static() -> Option<usize> {
        None
    }
}

impl<F, C, P, V> AsRef<C> for NBez<F, C, P, V>
        where F: Float,
              C: AsRef<[P]>,
              P: Point<F>,
              V: Vector<F, P> {
    fn as_ref(&self) -> &C {
        &self.points
    }
}

impl<F, C, P, V> AsMut<C> for NBez<F, C, P, V>
        where F: Float,
              C: AsRef<[P]>,
              P: Point<F>,
              V: Vector<F, P> {
    fn as_mut(&mut self) -> &mut C {
        &mut self.points
    }
}