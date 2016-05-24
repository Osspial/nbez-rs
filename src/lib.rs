extern crate num;

#[macro_use]
mod macros;

use std::convert::{Into, AsRef};
use std::ops::{Mul, Div, Neg};
use std::marker::PhantomData;
use num::{Float, FromPrimitive};

#[inline]
pub fn check_t_bounds<F: Float + FromPrimitive>(t: F) {
    let zero = F::from_f32(0.0).unwrap();
    let one  = F::from_f32(1.0).unwrap();
    assert!(zero <= t && t <= one);
}

n_pointvector!{2; Point2d, Vector2d {
    x,
    y
}}

impl<F: Float> Vector2d<F> {
    pub fn perp(self) -> Vector2d<F> {
        Vector2d {
            x: -self.y,
            y: self.x
        }
    }
}


n_bezier!{BezPoly2o {
    start: 1,
    ctrl : 2,
    end  : 1
} derived {
    ctrl - start: 1,
    end  - ctrl:  1
}}

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

n_bezier!{BezPoly4o {
    start: 1,
    ctrl0: 4,
    ctrl1: 6,
    ctrl2: 4,
    end:   1
} derived {
    ctrl0 - start: 1,
    ctrl1 - ctrl0: 3,
    ctrl2 - ctrl1: 3,
    end   - ctrl1: 1
}}

n_bezier!{BezPoly5o {
    start: 1,
    ctrl0: 5,
    ctrl1: 10,
    ctrl2: 10,
    ctrl3: 5,
    end:   1
} derived {
    ctrl0 - start: 1,
    ctrl1 - ctrl0: 4,
    ctrl2 - ctrl1: 6,
    ctrl3 - ctrl2: 4,
    end   - ctrl3: 1
}}

n_bezier!{BezPoly6o {
    start: 1,
    ctrl0: 6,
    ctrl1: 15,
    ctrl2: 20,
    ctrl3: 15,
    ctrl4: 6,
    end:   1
} derived {
    ctrl0 - start: 1,
    ctrl1 - ctrl0: 5,
    ctrl2 - ctrl1: 10,
    ctrl3 - ctrl2: 10,
    ctrl4 - ctrl3: 5,
    end   - ctrl4: 1
}}

bez_composite!{ Bez3o2d<BezPoly3o> {
    x,
    y
} -> <Point2d; Vector2d>}


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