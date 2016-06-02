use num::{Float, FromPrimitive};

pub trait BezCurve<F: Float + FromPrimitive> {
    type Point;
    type Vector;

    fn from_slice(&[Self::Point]) -> Self;

    fn interp(&self, t: F) -> Self::Point {
        ::check_t_bounds(t);
        self.interp_unbounded(t)
    }
    fn interp_unbounded(&self, t: F) -> Self::Point;

    fn slope(&self, t: F) -> Self::Vector {
        ::check_t_bounds(t);
        self.slope_unbounded(t)
    }
    fn slope_unbounded(&self, t: F) -> Self::Vector;
    fn order() -> usize;
}