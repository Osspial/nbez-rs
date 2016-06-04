use num::{Float, FromPrimitive};

pub trait BezCurve<F: Float + FromPrimitive> 
        where Self: Sized {
    type Point;
    type Vector;

    fn from_slice(&[Self::Point]) -> Option<Self>;

    fn interp(&self, t: F) -> Option<Self::Point> {
        check_t_bounds!(t);
        Some(self.interp_unbounded(t))
    }
    fn interp_unbounded(&self, t: F) -> Self::Point;

    fn slope(&self, t: F) -> Option<Self::Vector> {
        check_t_bounds!(t);
        Some(self.slope_unbounded(t))
    }
    fn slope_unbounded(&self, t: F) -> Self::Vector;
    
    fn order(&self) -> usize {
        Self::order_static().unwrap()
    }
    fn order_static() -> Option<usize>;
}