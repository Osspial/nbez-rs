use num::{Float, FromPrimitive};

macro_rules! count {
    ($idc:tt) => (1);
    ($($element:tt),*) => {{$(count!($element) +)* 0}};
}

macro_rules! n_bezier {
    ($name:ident {
        $($field:ident: $weight:expr),+
    } derived {
        $($dleft:ident - $dright:ident: $dweight:expr),+
    }) => {
        #[derive(Debug, Clone)]
        pub struct $name<F> where F: Float + FromPrimitive {
            $($field: F),+
        }

        impl<F> $name<F> where F: Float + FromPrimitive {
            pub fn new($($field: F),+) -> $name<F> {
                $name {
                    $($field: $field),+
                }
            }

            pub fn interp(&self, t: F) -> F {
                let zero = F::from_f32(0.0).unwrap();
                let one  = F::from_f32(1.0).unwrap();
                assert!(zero <= t && t <= one);
                self.interp_unbounded(t)
            }

            pub fn interp_unbounded(&self, t: F) -> F {
                let t1 = F::from_f32(1.0).unwrap() - t;
                const COUNT: i32 = count!($($field),+) - 1;
                let mut factor = COUNT + 1;

                $(
                    factor -= 1;
                    let $field =
                        t1.powi(factor) * 
                        t.powi(COUNT-factor) * 
                        self.$field * 
                        F::from_i32($weight).unwrap();
                )+
                $($field +)+ F::from_f32(0.0).unwrap()
            }

            pub fn derivative(&self, t: F) -> F {
                let zero = F::from_f32(0.0).unwrap();
                let one  = F::from_f32(1.0).unwrap();
                assert!(zero <= t && t <= one);
                self.derivative_unbounded(t)
            }

            pub fn derivative_unbounded(&self, t: F) -> F {
                let t1 = F::from_f32(1.0).unwrap() - t;
                const COUNT: i32 = count!($($field),+) - 2;
                let mut factor = COUNT + 1;

                $(
                    factor -= 1;
                    let $dleft =
                        t1.powi(factor) *
                        t.powi(COUNT-factor) *
                        (self.$dleft - self.$dright) *
                        F::from_i32($dweight).unwrap();
                )+
                $($dleft +)+ F::from_f32(0.0).unwrap()
            }
        }
    }
}

n_bezier!{BezCubePoly {
    start: 1,
    ctrl0: 3,
    ctrl1: 3,
    end:   1
} derived {
    ctrl0 - start: 1,
    ctrl1 - ctrl0: 2,
    end   - ctrl1: 1
}}