// Vector/Point macros
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
                $crate::check_t_bounds(t);
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

            pub fn slope(&self, t: F) -> F {
                $crate::check_t_bounds(t);
                self.slope_unbounded(t)
            }

            pub fn slope_unbounded(&self, t: F) -> F {
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

        impl<F> ::std::ops::Deref for $name<F> where F: ::num::Float + ::num::FromPrimitive {
            type Target = [F];
            fn deref(&self) -> &[F] {
                use std::slice;
                unsafe {
                    slice::from_raw_parts(self as *const $name<F> as *const F, count!($($field),+))
                }
            }
        }

        impl<F> ::std::ops::DerefMut for $name<F> where F: ::num::Float + ::num::FromPrimitive {
            fn deref_mut(&mut self) -> &mut [F] {
                use std::slice;
                unsafe {
                    slice::from_raw_parts_mut(self as *mut $name<F> as *mut F, count!($($field),+))
                }
            }
        }
    }
}


macro_rules! bez_composite {
    ($name:ident<$poly:ident> {
        $($field:ident: $($n_field:ident),+;)+
    } -> <$point:ident; $vector:ident>;
        $($p_field:ident, $set_field:ident: $($s_field:ident),+;)+) => 
    {
        #[derive(Debug, Clone)]
        pub struct $name<F: ::num::Float + ::num::FromPrimitive> {
            $(pub $field: $poly<F>),+
        }

        impl<F: ::num::Float + ::num::FromPrimitive> $name<F> {
            pub fn new($($($n_field: F),+),+) -> $name<F> {
                $name {
                    $($field: $poly::new($($n_field),+)),+
                }
            }

            $(
                pub fn $p_field(&self) -> $point<F> {
                    $point {
                        $($s_field: self.$s_field.$p_field),+
                    }
                }

                pub fn $set_field(&mut self, $($s_field: F),+) {
                    $(self.$s_field.$p_field = $s_field;)+
                }
            )+

            pub fn interp(&self, t: F) -> $point<F> {
                $crate::check_t_bounds(t);
                self.interp_unbounded(t)
            }

            pub fn interp_unbounded(&self, t: F) -> $point<F> {                
                $point {
                    $($field: self.$field.interp_unbounded(t)),+
                }
            }

            pub fn slope(&self, t: F) -> $vector<F> {
                $crate::check_t_bounds(t);
                self.slope_unbounded(t)
            }

            pub fn slope_unbounded(&self, t: F) -> $vector<F> {                
                $vector {
                    $($field: self.$field.slope_unbounded(t)),+
                }
            }
        }
    }
}