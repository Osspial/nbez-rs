use num::{Float, FromPrimitive};

pub trait BezCurve<F: Float + FromPrimitive> 
        where Self: Sized {
    type Point;
    type Vector;
    type Elevated;

    /// Attempt to create a curve from a slice. Fails if the slice's length does not match the
    /// curve's order + 1, or if it is being used to create an `NBez`/`NBezPoly`.
    fn from_slice(&[Self::Point]) -> Option<Self>;

    /// Perform interpolation on the curve for a value of `t` from `0.0` to `1.0` inclusive. Returns `None`
    /// if `t` is outside of that range.
    fn interp(&self, t: F) -> Option<Self::Point> {
        check_t_bounds!(t);
        Some(self.interp_unbounded(t))
    }
    /// Perform interpolation on the curve with no range bounds
    fn interp_unbounded(&self, t: F) -> Self::Point;

    /// Get the slope for the given `t`, bounded on `0.0` to `1.0` inclusive. Returns `None` if
    /// `t` is not within bounds.
    fn slope(&self, t: F) -> Option<Self::Vector> {
        check_t_bounds!(t);
        Some(self.slope_unbounded(t))
    }
    /// Get the slope for the given `t` with no range bounds
    fn slope_unbounded(&self, t: F) -> Self::Vector;

    fn elevate(&self) -> Self::Elevated;
    
    /// Gets the order of the curve
    fn order(&self) -> usize {
        Self::order_static().unwrap()
    }
    /// If the order of the curve can be statically determined at compile time, return `Some(order)`.
    /// Otherwise, return `None`
    fn order_static() -> Option<usize>;
}