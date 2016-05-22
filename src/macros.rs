// Vector/Point macros
macro_rules! npoint_ops {
    ($lhs:ty; $rhs:ty = $output:ident<$g_name:ident: $g_ty:ident> {$($field:ident),*}) => {
        impl<$g_name: $g_ty> ::std::ops::Add<$rhs> for $lhs {
            type Output = $output<$g_name>;

            fn add(self, rhs: $rhs) -> $output<$g_name> {
                $output {
                    $($field: self.$field + rhs.$field),*
                }
            }
        }
    }
}

#[macro_export]
macro_rules! impl_npoint {
    // auxiliary/entry point workaround made by durka42
    // read the last rule of the macro first
    (@go $dims: expr; $name:ident<$g_name:ident: $g_ty:ident> {$($field:ident: $f_ty:ty),*} $fields:tt $(,$sibling:ty)*) => {
                                                              // ^ match the first duplicate of the fields
                                                                                            // ^ keep the second one as a tt
        #[derive(Debug, Clone, Copy)]
        pub struct $name<$g_name: $g_ty> {
            $(
                pub $field: $f_ty
            ),*
        }

        impl<$g_name: $g_ty> ::std::convert::Into<[$g_name; $dims]> for $name<$g_name> {
            fn into(self) -> [$g_name; $dims] {
                [$(self.$field),*]
            }
        }

        impl<$g_name: $g_ty> ::std::convert::Into<($($f_ty),*)> for $name<$g_name> {
            fn into(self) -> (F, F) {
                ($(self.$field),*)
            }
        }

        npoint_ops!($name<$g_name>; $name<$g_name> = $name<$g_name: $g_ty> {x, y});

        $(
            // pass the tt-wrapped fields to auxiliary macro
            impl_npoint!(@sib $name $g_name $g_ty $sibling $fields);
            npoint_ops!($name<$g_name>; $sibling = $name<$g_name: $g_ty> {x, y});
        )*
    };
    
    // auxiliary rule
    (@sib $name:ident $g_name:ident $g_ty:ident $sibling:ty {$($field:ident: $f_ty:ty),*}) => {
                                                // ^ finally destructure fields out of tt here
        impl<$g_name: $g_ty> ::std::convert::From<$sibling> for $name<$g_name> {
            fn from(sib: $sibling) -> $name<$g_name> {
                $name {
                    $($field: sib.$field),*
                }
            }
        }
    };
    
    // entry point rule
    ($dims: expr; $name:ident<$g_name:ident: $g_ty:ident> $fields:tt $(,$sibling:ty)*) => {
                                                          // ^ match fields all together as a tt
        impl_npoint!(@go
            $dims;
            $name<$g_name: $g_ty>
            $fields // duplicate the fields
            $fields
            $(,$sibling)*
        );
    };
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