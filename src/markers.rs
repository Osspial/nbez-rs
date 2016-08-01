use num_traits::float;
use num_traits::identities::Zero;
use num_traits::cast::FromPrimitive;
use std::fmt::Debug;
use std::ops::{Add, Sub, Mul, Div};

pub trait Float: float::Float + FromPrimitive + Debug {}
impl<F> Float for F where F: float::Float + FromPrimitive + Debug {}

pub trait PVOps<F>:		
		Add<Self, Output = Self> +
		Sub<Self, Output = Self> +
		Mul<F, Output = Self> +
		Div<F, Output = Self> +
		Zero

		where Self: Sized,
			  F: Float {}

pub trait Point<F>: 
		Into<<Self as Point<F>>::Vector> +
		Clone +
		Copy + 
		PVOps<F>

		where Self: Sized,
			  F: Float {
	type Vector: Vector<F>;
}

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