extern crate num;

#[macro_use]
mod macros;

use std::marker::PhantomData;
use num::{Float, FromPrimitive};

#[inline]
pub fn check_t_bounds<F: Float + FromPrimitive>(t: F) {
    let zero = F::from_f32(0.0).unwrap();
    let one  = F::from_f32(1.0).unwrap();
    assert!(zero <= t && t <= one);
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