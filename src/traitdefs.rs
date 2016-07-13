//! Various trait aliases to simplify generics in other modules. Every trait in this module is
//! implemented for all possible valid types.
use num_traits;
use std::fmt::Debug;

pub trait Float: num_traits::float::Float + num_traits::cast::FromPrimitive + Debug {}
impl<F> Float for F where F: num_traits::float::Float + num_traits::cast::FromPrimitive + Debug {}

pub trait Point<F: Float>: AsRef<[F]> + AsMut<[F]> + Clone {}
impl<F, P> Point<F> for P 
        where F: Float,
              P: AsRef<[F]> + AsMut<[F]> + Clone {}

pub trait Vector<F: Float, P: Point<F>>: AsRef<[F]> + AsMut<[F]> + From<P> {}
impl<F, P, V> Vector<F, P> for V 
        where F: Float, 
              P: Point<F>, 
              V: AsRef<[F]> + AsMut<[F]> + From<P> {}