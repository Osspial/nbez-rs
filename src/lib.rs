extern crate num;

#[macro_use]
mod macros;
mod nbez;
pub mod traitdefs;
pub mod traits;
pub use nbez::*;
use traitdefs::Float;

#[inline]
fn lerp<F: Float>(a: F, b: F, factor: F) -> F {
    let fact1 = F::from_f32(1.0).unwrap() - factor;
    a * factor + b * fact1
}

// There are macros in place to make it easier to create new bezier structs, as they can be created
// with a very consistent pattern. However, those macros are also written in a very consistent pattern
// which unfortunately is significantly harder, if not impossible, to create with a traditional
// macro. So, the macro invocations are generated with the build script and then inserted here.
include!(concat!(env!("OUT_DIR"), "/macro_invocs.rs"));

impl<F: traitdefs::Float> Vector2d<F> {
    pub fn perp(self) -> Vector2d<F> {
        Vector2d {
            x: -self.y,
            y: self.x
        }
    }
}