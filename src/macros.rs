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
        impl<F: Float> ::std::ops::Mul<F> for $name<F> {
            type Output = $name<F>;

            fn mul(self, rhs: F) -> $name<F> {
                $name {
                    $($field: self.$field * rhs),+
                }
            }
        }

        impl<F: Float> ::std::ops::Div<F> for $name<F> {
            type Output = $name<F>;

            fn div(self, rhs: F) -> $name<F> {
                $name {
                    $($field: self.$field / rhs),+
                }
            }
        }

        impl<F: Float> ::std::ops::Neg for $name<F> {
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

        impl<F: Float> $name<F> {
            pub fn new($($field: F),+) -> $name<F> {
                $name {
                    $($field: $field),+
                }
            }
        }

        impl<'a, F: Float> ::std::convert::From<&'a [F]> for $name<F> {
            fn from(slice: &'a [F]) -> $name<F> {
                assert_eq!(slice.len(), count!($($field),+));
                let mut index = -1;
                $name{$($field: {
                    index += 1;
                    slice[index as usize]
                }),+}
            }
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

        impl<F: Float> ::std::convert::AsRef<[F]> for $name<F> {
            fn as_ref(&self) -> &[F] {
                use std::slice;
                unsafe {
                    slice::from_raw_parts(self as *const $name<F> as *const F, $dims)
                }
            }
        }

        impl<F: Float> ::std::convert::AsMut<[F]> for $name<F> {
            fn as_mut(&mut self) -> &mut [F] {
                use std::slice;
                unsafe {
                    slice::from_raw_parts_mut(self as *mut $name<F> as *mut F, $dims)
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


macro_rules! check_t_bounds {
    ($t: expr) => {{
        let zero = F::from_f32(0.0).unwrap();
        let one  = F::from_f32(1.0).unwrap();
        if !(zero <= $t && $t <= one) {
            return None;
        }
    }}
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
        #[derive(Debug, Clone, Copy)]
        pub struct $name<F> where F: ::num::Float + ::num::FromPrimitive {
            $(pub $field: F),+
        }

        impl<F> $name<F> where F: ::num::Float + ::num::FromPrimitive {
            pub fn new($($field: F),+) -> $name<F> {
                $name {
                    $($field: $field),+
                }
            }
        }

        impl<F> $crate::traits::BezCurve<F> for $name<F>
                where F: ::num::Float + ::num::FromPrimitive {
            type Point = F;
            type Vector = F;

            fn from_slice(slice: &[F]) -> Option<$name<F>> {
                if slice.len() - 1 != $name::<F>::order_static().unwrap() {
                    None
                } else {
                    let mut index = -1;
                    Some($name {$($field: {
                        index += 1;
                        slice[index as usize]
                    }),+})
                }
            }

            fn interp_unbounded(&self, t: F) -> F {
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

            fn slope_unbounded(&self, t: F) -> F {
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

            #[inline]
            fn order_static() -> Option<usize> {
                Some(count!($($field),+)-1)            
            }
        }

        impl<F> ::std::convert::AsRef<[F]> for $name<F> where F: ::num::Float + ::num::FromPrimitive {
            fn as_ref(&self) -> &[F] {
                use std::slice;
                unsafe {
                    slice::from_raw_parts(self as *const $name<F> as *const F, count!($($field),+))
                }
            }
        }

        impl<F> ::std::convert::AsMut<[F]> for $name<F> where F: ::num::Float + ::num::FromPrimitive {
            fn as_mut(&mut self) -> &mut [F] {
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
        $($dim:ident = $($dfield:ident),+;)+) => 
    {
        #[derive(Debug, Clone, Copy)]
        pub struct $name<F: ::num::Float + ::num::FromPrimitive> {
            $(pub $field: $point<F>),+
        }

        impl<F: ::num::Float + ::num::FromPrimitive> $name<F> {
            pub fn new($($($n_field: F),+),+) -> $name<F> {
                $name {
                    $($field: $point::new($($n_field),+)),+
                }
            }
        }

        impl<F> $crate::traits::BezCurve<F> for $name<F>
                where F: ::num::Float + ::num::FromPrimitive {
            type Point = $point<F>;
            type Vector = $vector<F>;

            fn from_slice(slice: &[$point<F>]) -> Option<$name<F>> {
                if slice.len() - 1 == $name::<F>::order_static().unwrap() {
                    None
                } else {
                    let mut index = -1;
                    Some( $name {$($field: {
                        index += 1;
                        slice[index as usize]
                    }),+})  
                }
            }

            fn interp_unbounded(&self, t: F) -> $point<F> {
                use $crate::traits::BezCurve;

                $(let $dim = $poly::new($(self.$dfield.$dim),+);)+
                $point::new($($dim.interp_unbounded(t)),+)
            }

            fn slope_unbounded(&self, t: F) -> $vector<F> {
                use $crate::traits::BezCurve;

                $(let $dim = $poly::new($(self.$dfield.$dim),+);)+
                $vector::new($($dim.slope_unbounded(t)),+)
            }

            #[inline]
            fn order_static() -> Option<usize> {
                $poly::<F>::order_static()
            }
        }

        impl<F> ::std::convert::AsRef<[$point<F>]> for $name<F> where F: ::num::Float + ::num::FromPrimitive {
            fn as_ref(&self) -> &[$point<F>] {
                use std::slice;
                unsafe {
                    slice::from_raw_parts(self as *const $name<F> as *const $point<F>, count!($($field),+))
                }
            }
        }

        impl<F> ::std::convert::AsMut<[$point<F>]> for $name<F> where F: ::num::Float + ::num::FromPrimitive {
            fn as_mut(&mut self) -> &mut [$point<F>] {
                use std::slice;
                unsafe {
                    slice::from_raw_parts_mut(self as *mut $name<F> as *mut $point<F>, count!($($field),+))
                }
            }
        }
    };
}