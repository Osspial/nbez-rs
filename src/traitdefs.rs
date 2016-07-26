//! Various trait aliases to simplify generics in other modules. Every trait in this module is
//! implemented for all possible valid types.
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

impl<F: Float, PV> PVOps<F> for PV
		where PV:			
			Add<PV, Output = PV> +
			Sub<PV, Output = PV> +
			Mul<F, Output = PV> +
			Div<F, Output = PV> +
			Zero {}

pub trait Point<F, V>: 
		Into<V> +
		Clone +
		Copy + 
		PVOps<F>

		where Self: Sized,
			  F: Float,
			  V: Vector<F> {}

impl<F, P, V> Point<F, V> for P 
        where F: Float,
        	  V: Vector<F>,
              P: Into<V> + Clone + Copy + PVOps<F> {}

pub trait Vector<F: Float>: 
		Clone +
		Copy +
		PVOps<F> {}

impl<F, V> Vector<F> for V 
        where F: Float, 
              V: Clone + Copy + PVOps<F> {}