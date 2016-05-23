// Vector/Point macros
#[macro_export]
macro_rules! n_pointvector {
    (ops $lhs:ident; $rhs:ident {$($field:ident),*}) => {
        impl<F: Float> ::std::ops::Add<$rhs<F>> for $lhs<F> {
            type Output = $lhs<F>;

            fn add(self, rhs: $rhs<F>) -> $lhs<F> {
                $lhs {
                    $($field: self.$field + rhs.$field),*
                }
            }
        }

        impl<F: Float> ::std::ops::Sub<$rhs<F>> for $lhs<F> {
            type Output = $lhs<F>;

            fn sub(self, rhs: $rhs<F>) -> $lhs<F> {
                $lhs {
                    $($field: self.$field - rhs.$field),*
                }
            }
        }
    };

    (float ops $name:ident {$($field:ident),+}) => {
        impl<F: Float> Mul<F> for $name<F> {
            type Output = $name<F>;

            fn mul(self, rhs: F) -> $name<F> {
                $name {
                    $($field: self.$field * rhs),+
                }
            }
        }

        impl<F: Float> Div<F> for $name<F> {
            type Output = $name<F>;

            fn div(self, rhs: F) -> $name<F> {
                $name {
                    $($field: self.$field / rhs),+
                }
            }
        }

        impl<F: Float> Neg for $name<F> {
            type Output = $name<F>;

            fn neg(self) -> $name<F> {
                $name {
                    $($field: -self.$field),+
                }
            }
        }
    };

    (struct $dims:expr; $name:ident {$($field:ident: $f_ty:ident),+} $sibling:ident) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name<F: Float> {
            $(pub $field: F),+
        }

        impl<F: Float> ::std::convert::Into<[F; $dims]> for $name<F> {
            fn into(self) -> [F; $dims] {
                [$(self.$field),*]
            }
        }

        impl<F: Float> ::std::convert::Into<($($f_ty),*)> for $name<F> {
            fn into(self) -> ($($f_ty),*) {
                ($(self.$field),*)
            }
        }

        impl<F: Float> ::std::convert::From<$sibling<F>> for $name<F> {
            fn from(sib: $sibling<F>) -> $name<F> {
                $name {
                    $($field: sib.$field),*
                }
            }
        }

        n_pointvector!(ops $name; $sibling {$($field),+});
        n_pointvector!(ops $name; $name {$($field),+});
        n_pointvector!(float ops $name {$($field),+});
    };

    ($dims:expr; $p_name:ident, $v_name:ident {$($field:ident),+}) => {
        n_pointvector!(struct $dims; $p_name {$($field: F),+} $v_name);
        n_pointvector!(struct $dims; $v_name {$($field: F),+} $p_name);

        impl<F: Float + FromPrimitive> $v_name<F> {
            pub fn len(self) -> F {
                ($(self.$field.powi(2) +)+ F::from_f32(0.0).unwrap()).sqrt()
            }

            pub fn normalize(self) -> $v_name<F> {
                self / self.len()
            }
        }
    }
}



// Polynomial Macros
macro_rules! count {
    ($idc:tt) => (1);
    ($($element:tt),*) => {{$(count!($element) +)* 0}};
}

#[macro_export]
macro_rules! n_bezier {
    ($name:ident {
        $($field:ident: $weight:expr),+
    } derived {
        $($dleft:ident - $dright:ident: $dweight:expr),+
    }) => {
        #[derive(Debug, Clone)]
        pub struct $name<F> where F: ::num::Float + ::num::FromPrimitive {
            $(pub $field: F),+
        }

        impl<F> $name<F> where F: ::num::Float + ::num::FromPrimitive {
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
                const COUNT: i32 = count!($($dleft),+) - 1;
                let mut factor = COUNT + 1;

                $(
                    factor -= 1;
                    let $dleft =
                        t1.powi(factor) *
                        t.powi(COUNT-factor) *
                        (self.$dleft - self.$dright) *
                        F::from_i32($dweight * (COUNT + 1)).unwrap();
                )+
                $($dleft +)+ F::from_f32(0.0).unwrap()
            }
        }
    }
}