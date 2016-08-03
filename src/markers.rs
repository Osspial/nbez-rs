use num_traits::float;
use num_traits::identities::Zero;
use num_traits::cast::FromPrimitive;
use std::fmt::Debug;
use std::ops::{Add, Sub, Mul, Div};

/// A helper trait to simplify float generics
pub trait Float: float::Float + FromPrimitive + Debug {}
impl<F> Float for F where F: float::Float + FromPrimitive + Debug {}

/// A trait that specifies the necessary operators needed to have a point which `nbez` can properly
/// perform operations on
pub trait PVOps<F>:		
		Add<Self, Output = Self> +
		Sub<Self, Output = Self> +
		Mul<F, Output = Self> +
		Div<F, Output = Self> +
		Zero

		where Self: Sized,
			  F: Float {}

/// Specifies the needed traits to have a `nbez` point, as well as the vector type that this
/// corresponds to
pub trait Point<F>: 
		Into<<Self as Point<F>>::Vector> +
		Clone +
		Copy + 
		PVOps<F>

		where Self: Sized,
			  F: Float {
	/// The vector that is associatded with this point
	type Vector: Vector<F>;
}

/// A vector. Gets associated with any number of points
pub trait Vector<F: Float>: 
		Clone +
		Copy +
		PVOps<F> {}

impl PVOps<f32> for f32 {}
impl Point<f32> for f32 {
	type Vector = f32;
}
impl Vector<f32> for f32 {}

impl PVOps<f64> for f64 {}
impl Point<f64> for f64 {
	type Vector = f64;
}
impl Vector<f64> for f64 {}