//! Various trait aliases to simplify generics in other modules. Every trait in this module is
//! implemented for all possible valid types.
use num_traits;
use std::fmt::Debug;
use std::ops::{Add, Sub, Mul, Div, Neg};

pub trait Float: num_traits::float::Float + num_traits::cast::FromPrimitive + Debug {}
impl<F> Float for F where F: num_traits::float::Float + num_traits::cast::FromPrimitive + Debug {}

pub trait PVOps<F, Other>:
		Add<Other, Output = Self> +
		Sub<Other, Output = Self> +
		
		Add<Self, Output = Self> +
		Sub<Self, Output = Self> +
		Mul<F, Output = Self> +
		Div<F, Output = Self> +
		Neg<Output = Self>

		where Self: Sized,
			  F: Float,
			  Other: PVOps<F, Self> {}

pub trait Point<F, V>: 
		AsRef<[F]> + 
		AsMut<[F]> +
		From<V> +
		Clone + 
		PVOps<F, V>

		where Self: Sized,
			  F: Float,
			  V: Vector<F, Self> {}

impl<F, P, V> Point<F, V> for P 
        where F: Float,
        	  V: Vector<F, P>,
              P: AsRef<[F]> + AsMut<[F]> + From<V> + Clone + PVOps<F, V> {}

pub trait Vector<F: Float, P: Point<F, Self>>: 
		AsRef<[F]> + 
		AsMut<[F]> + 
		From<P> +
		Clone +
		PVOps<F, P> {}

impl<F, P, V> Vector<F, P> for V 
        where F: Float, 
              P: Point<F, V>, 
              V: AsRef<[F]> + AsMut<[F]> + From<P> + Clone + PVOps<F, P> {}