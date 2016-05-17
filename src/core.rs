use num::{Float, FromPrimitive};

/// A one-dimensional cubic bezier polynomial
#[derive(Debug, Clone)]
pub struct BezCubePoly<F> where F: Float + FromPrimitive {
    pub start: F,
    pub ctrl0: F,
    pub ctrl1: F,
    pub end: F
}

impl<F> BezCubePoly<F> where F: Float + FromPrimitive {
    pub fn new(start: F, ctrl0: F, ctrl1: F, end: F) -> BezCubePoly<F> {
        BezCubePoly {
            start: start,
            ctrl0: ctrl0,
            ctrl1: ctrl1,
            end: end 
        }
    }

    pub fn interp(&self, t: F) -> F {
        let zero = F::from_f32(0.0).unwrap();
        let one  = F::from_f32(1.0).unwrap();
        assert!(zero <= t && t <= one);
        self.interp_unbounded(t)
    }

    pub fn interp_unbounded(&self, t: F) -> F {
        let t1 = F::from_f32(1.0).unwrap() - t;
        t1.powi(3)             * self.start                             + 
        t1.powi(2) * t         * self.ctrl0 * F::from_f32(3.0).unwrap() + 
        t1         * t.powi(2) * self.ctrl1 * F::from_f32(3.0).unwrap() + 
                     t.powi(3) * self.end
    }
}