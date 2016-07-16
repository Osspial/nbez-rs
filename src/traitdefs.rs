//! Various trait aliases to simplify generics in other modules. Every trait in this module is
//! implemented for all possible valid types.
use num_traits;
use std::fmt::Debug;
use std::ops::{Add, Sub, Mul, Div, Neg};

pub trait Float: num_traits::float::Float + num_traits::cast::FromPrimitive + Debug {}
impl<F> Float for F where F: num_traits::float::Float + num_traits::cast::FromPrimitive + Debug {}

pub trait PVOps<F>:		
		Add<Self, Output = Self> +
		Sub<Self, Output = Self> +
		Mul<F, Output = Self> +
		Div<F, Output = Self> +
		Neg<Output = Self>

		where Self: Sized,
			  F: Float {}

impl<F: Float, PV> PVOps<F> for PV
		where PV:			
			Add<PV, Output = PV> +
			Sub<PV, Output = PV> +
			Mul<F, Output = PV> +
			Div<F, Output = PV> +
			Neg<Output = PV> {}

pub trait Point<F, V>: 
		From<V> +
		Clone + 
		PVOps<F>

		where Self: Sized,
			  F: Float,
			  V: Vector<F, Self> {}

impl<F, P, V> Point<F, V> for P 
        where F: Float,
        	  V: Vector<F, P>,
              P: From<V> + Clone + PVOps<F> {}

pub trait Vector<F: Float, P: Point<F, Self>>: 
		From<P> +
		Clone +
		PVOps<F> {}

impl<F, P, V> Vector<F, P> for V 
        where F: Float, 
              P: Point<F, V>, 
              V: From<P> + Clone + PVOps<F> {}