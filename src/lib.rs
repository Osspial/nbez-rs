extern crate num;

#[macro_use]
mod macros;

use std::convert::{Into, AsRef};
use std::ops::{Mul, Div, Neg};
use std::marker::PhantomData;
use num::{Float, FromPrimitive};

impl_npoint!{2; Point2d<F: Float> {
    x: F,
    y: F
}, Vector2d<F>}

impl_npoint!{2; Vector2d<F: Float> {
    x: F,
    y: F
}, Point2d<F>}

impl<F: Float + FromPrimitive> Vector2d<F> {
    pub fn len(self) -> F {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn perp(self) -> Vector2d<F> {
        Vector2d {
            x: -self.y,
            y: self.x
        }
    }

    pub fn normalize(self) -> Vector2d<F> {
        self / self.len()
    }
}

impl<F: Float> Mul<F> for Vector2d<F> {
    type Output = Vector2d<F>;

    fn mul(self, rhs: F) -> Vector2d<F> {
        Vector2d {
            x: self.x * rhs,
            y: self.y * rhs
        }
    }
}

impl<F: Float> Div<F> for Vector2d<F> {
    type Output = Vector2d<F>;

    fn div(self, rhs: F) -> Vector2d<F> {
        Vector2d {
            x: self.x / rhs,
            y: self.y / rhs
        }
    }
}

impl<F: Float> Neg for Vector2d<F> {
    type Output = Vector2d<F>;

    fn neg(self) -> Vector2d<F> {
        Vector2d {
            x: -self.x,
            y: -self.y
        }
    }
}


n_bezier!{BezPoly3o {
    start: 1,
    ctrl0: 3,
    ctrl1: 3,
    end:   1
} derived {
    ctrl0 - start: 1,
    ctrl1 - ctrl0: 2,
    end   - ctrl1: 1
}}


#[derive(Debug, Clone, Copy)]
pub enum BezNode<F: Float> {
    Point2d {
        x: F,
        y: F
    },

    Control {
        x: F,
        y: F
    }
}

impl<F: Float> BezNode<F> {
    pub fn new_point(x: F, y: F) -> BezNode<F> {
        BezNode::Point2d {
            x: x,
            y: y
        }
    }

    pub fn new_control(x: F, y: F) -> BezNode<F> {
        BezNode::Control {
            x: x,
            y: y
        }
    }

    pub fn x(self) -> F {
        <BezNode<F> as Into<(F, F)>>::into(self).0
    }

    pub fn y(self) -> F {
        <BezNode<F> as Into<(F, F)>>::into(self).1
    }

    pub fn is_point(self) -> bool {
        use self::BezNode::*;
        match self {
            Point2d{..} => true,
            Control{..} => false
        }
    }

    pub fn is_control(self) -> bool {
        use self::BezNode::*;
        match self {
            Point2d{..} => false,
            Control{..} => true
        }
    }
}

impl<F: Float> Into<[F; 2]> for BezNode<F> {
    fn into(self) -> [F; 2] {
        use self::BezNode::*;

        match self {
            Point2d{x, y}    |
            Control{x, y} => [x, y]
        }
    }
}

impl<F: Float> Into<(F, F)> for BezNode<F> {
    fn into(self) -> (F, F) {
        use self::BezNode::*;

        match self {
            Point2d{x, y}    |
            Control{x, y} => (x, y)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bez3o2d<F: Float + FromPrimitive> {
    pub x: BezPoly3o<F>,
    pub y: BezPoly3o<F>
}

impl<F: Float + FromPrimitive> Bez3o2d<F> {
    pub fn interp(&self, t: F) -> Point2d<F> {
        Point2d {
            x: self.x.interp(t),
            y: self.y.interp_unbounded(t) // The interp is already checked when we call x.interp, so we don't have to do it again here
        }
    }

    pub fn derivative(&self, t: F) -> Vector2d<F> {
        Vector2d {
            x: self.x.derivative(t),
            y: self.y.derivative_unbounded(t)
        }
    }
}

#[derive(Debug, Clone)]
pub struct BezChain3o2d<C, F> 
        where C: AsRef<[BezNode<F>]>,
              F: Float {
    container: C,
    float_type: PhantomData<F>
}

impl<C, F> BezChain3o2d<C, F> 
        where C: AsRef<[BezNode<F>]>,
              F: Float {
    pub fn from_container(c: C) -> Result<BezChain3o2d<C, F>, BevError> {
        {
            let cslice = c.as_ref();
            if cslice.len() % 3 != 1 {
                return Err(BevError::InvalidLength)
            }

            for i in 0..cslice.len()/3 {
                let curve = &cslice[i*3..(i+1)*3+1];
                if !(curve[0].is_point()   &&
                     curve[1].is_control() &&
                     curve[2].is_control() &&
                     curve[3].is_point()) {
                    return Err(BevError::BadNodePattern)
                }
            }
        }

        Ok(BezChain3o2d {
            container: c,
            float_type: PhantomData
        })
    }

    pub unsafe fn from_container_unchecked(c: C) -> BezChain3o2d<C, F> {
        BezChain3o2d {
            container: c,
            float_type: PhantomData
        }
    }

    pub fn unwrap(self) -> C {
        self.container
    }
}

impl<C, F> AsRef<C> for BezChain3o2d<C, F> 
        where C: AsRef<[BezNode<F>]>,
              F: Float {

    fn as_ref(&self) -> &C {
        &self.container
    }
}

#[derive(Debug)]
pub enum BevError {
    BadNodePattern,
    InvalidLength
}