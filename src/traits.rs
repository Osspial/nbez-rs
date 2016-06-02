use num::{Float, FromPrimitive};

pub trait BezCurve<F: Float + FromPrimitive> {
    type Interp;
    type Slope;

    fn interp(&self, t: F) -> Self::Interp {
        ::check_t_bounds(t);
        self.interp_unbounded(t)
    }
    fn interp_unbounded(&self, t: F) -> Self::Interp;

    fn slope(&self, t: F) -> Self::Slope {
        ::check_t_bounds(t);
        self.slope_unbounded(t)
    }
    fn slope_unbounded(&self, t: F) -> Self::Slope;
    fn order() -> usize;
}