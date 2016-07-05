// Vector/Point macros
macro_rules! n_pointvector {
    (ops $lhs:ident; $rhs:ident {$($field:ident),*}) => {
        impl<F: $crate::traitdefs::Float> ::std::ops::Add<$rhs<F>> for $lhs<F> {
            type Output = $lhs<F>;

            fn add(self, rhs: $rhs<F>) -> $lhs<F> {
                $lhs {
                    $($field: self.$field + rhs.$field),*
                }
            }
        }

        impl<F: $crate::traitdefs::Float> ::std::ops::Sub<$rhs<F>> for $lhs<F> {
            type Output = $lhs<F>;

            fn sub(self, rhs: $rhs<F>) -> $lhs<F> {
                $lhs {
                    $($field: self.$field - rhs.$field),*
                }
            }
        }
    };

    (float ops $name:ident {$($field:ident),+}) => {
        impl<F: $crate::traitdefs::Float> ::std::ops::Mul<F> for $name<F> {
            type Output = $name<F>;

            fn mul(self, rhs: F) -> $name<F> {
                $name {
                    $($field: self.$field * rhs),+
                }
            }
        }

        impl<F: $crate::traitdefs::Float> ::std::ops::Div<F> for $name<F> {
            type Output = $name<F>;

            fn div(self, rhs: F) -> $name<F> {
                $name {
                    $($field: self.$field / rhs),+
                }
            }
        }

        impl<F: $crate::traitdefs::Float> ::std::ops::Neg for $name<F> {
            type Output = $name<F>;

            fn neg(self) -> $name<F> {
                $name {
                    $($field: -self.$field),+
                }
            }
        }
    };

    (struct $doc:expr, $dims:expr; $name:ident {$($field:ident: $f_ty:ident),+} $sibling:ident) => {
        #[derive(Debug, Clone, Copy)]
        #[doc=$doc]
        pub struct $name<F: $crate::traitdefs::Float> {
            $(pub $field: F),+
        }

        impl<F: $crate::traitdefs::Float> $name<F> {
            pub fn new($($field: F),+) -> $name<F> {
                $name {
                    $($field: $field),+
                }
            }
        }

        impl<F: $crate::traitdefs::Float> ::std::convert::From<[F; $dims]> for $name<F> {
            fn from(array: [F; $dims]) -> $name<F> {
                let mut index = -1;
                $name{$($field: {
                    index += 1;
                    array[index as usize]
                }),+}
            }
        }

        impl<F: $crate::traitdefs::Float> ::std::convert::Into<[F; $dims]> for $name<F> {
            fn into(self) -> [F; $dims] {
                [$(self.$field),*]
            }
        }

        impl<F: $crate::traitdefs::Float> ::std::convert::Into<($($f_ty),*)> for $name<F> {
            fn into(self) -> ($($f_ty),*) {
                ($(self.$field),*)
            }
        }

        impl<F: $crate::traitdefs::Float> ::std::convert::From<$sibling<F>> for $name<F> {
            fn from(sib: $sibling<F>) -> $name<F> {
                $name {
                    $($field: sib.$field),*
                }
            }
        }

        impl<F: $crate::traitdefs::Float> ::std::convert::AsRef<[F]> for $name<F> {
            fn as_ref(&self) -> &[F] {
                use std::slice;
                unsafe {
                    slice::from_raw_parts(self as *const $name<F> as *const F, $dims)
                }
            }
        }

        impl<F: $crate::traitdefs::Float> ::std::convert::AsMut<[F]> for $name<F> {
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

    ($p_doc:expr, $v_doc:expr, $dims:expr; $p_name:ident, $v_name:ident {$($field:ident),+}) => {
        n_pointvector!(struct $p_doc, $dims; $p_name {$($field: F),+} $v_name);
        n_pointvector!(struct $v_doc, $dims; $v_name {$($field: F),+} $p_name);

        impl<F: $crate::traitdefs::Float> $v_name<F> {
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
macro_rules! n_bezier {
    ($doc:expr, $order:expr; $name:ident {
        $($field:ident: $weight:expr),+
    } {
        $start:ident, $next:ident;
        $($left:ident, $right:ident: $dweight:expr),+;
        $penu:ident, $end:ident;
    } elevated $elevated:ident<$($est:ty),+>) => {
        #[derive(Debug, Clone, Copy)]
        #[doc=$doc]
        pub struct $name<F> where F: $crate::traitdefs::Float {
            $(pub $field: F),+
        }

        impl<F> $name<F> where F: $crate::traitdefs::Float {
            pub fn new($($field: F),+) -> $name<F> {
                $name {
                    $($field: $field),+
                }
            }
        }

        impl<F> $crate::BezCurve<F> for $name<F>
                where F: $crate::traitdefs::Float {
            type Point = F;
            type Vector = F;
            type Elevated = $elevated<$($est),+>;

            fn from_slice(slice: &[F]) -> Option<$name<F>> {
                use $crate::OrderStatic;
                if slice.len() - 1 != $name::<F>::order_static() {
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
                const COUNT: i32 = $order;
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
                const COUNT: i32 = $order - 1;
                let mut factor = COUNT + 1;

                $(
                    factor -= 1;
                    let $right =
                        t1.powi(factor) *
                        t.powi(COUNT-factor) *
                        (self.$right - self.$left) *
                        F::from_i32($dweight * (COUNT + 1)).unwrap();
                )+
                $($right +)+ F::from_f32(0.0).unwrap()
            }

            fn elevate(&self) -> $elevated<$($est),+> {
                let mut factor = 0.0;
                let order = F::from_usize(self.order() + 1).unwrap();
                $elevated::from([self.$start, 
                    $({
                        factor += 1.0;
                        let factor = F::from_f32(factor).unwrap();
                        (self.$left * factor +
                        self.$right * (order - factor)) / order
                    },)+
                    self.$end])
            }

            fn order(&self) -> usize {
                use $crate::OrderStatic;
                $name::<F>::order_static()
            }
        }

        impl<F> $crate::OrderStatic for $name<F> 
                where F: $crate::traitdefs::Float {
            #[inline]
            fn order_static() -> usize {
                $order
            }
        }

        impl<F> ::std::convert::From<[F; $order + 1]> for $name<F> 
                where F: $crate::traitdefs::Float {
            fn from(array: [F; $order + 1]) -> $name<F> {
                use $crate::BezCurve;
                $name::from_slice(&array[..]).unwrap()
            }
        }

        impl<F> ::std::convert::AsRef<[F]> for $name<F> 
                where F: $crate::traitdefs::Float {
            fn as_ref(&self) -> &[F] {
                use std::slice;
                unsafe {
                    slice::from_raw_parts(self as *const $name<F> as *const F, $order + 1)
                }
            }
        }

        impl<F> ::std::convert::AsMut<[F]> for $name<F> 
                where F: $crate::traitdefs::Float {
            fn as_mut(&mut self) -> &mut [F] {
                use std::slice;
                unsafe {
                    slice::from_raw_parts_mut(self as *mut $name<F> as *mut F, $order + 1)
                }
            }
        }
    }
}


macro_rules! bez_composite {
    ($doc:expr, $order:expr; $name:ident<$poly:ident> {
        $($field:ident: $($n_field:ident),+;)+
    } -> <$point:ident; $vector:ident> {
        $($dim:ident = $($dfield:ident),+;)+
    } elevated $elevated:ident<$($est:ty),+> {
        $($eindex:expr => $($edim:ident),+;)+
    } chained $chain:ident) => 
    {
        #[derive(Debug, Clone, Copy)]
        #[doc=$doc]
        pub struct $name<F: $crate::traitdefs::Float> {
            $(pub $field: $point<F>),+
        }

        impl<F: $crate::traitdefs::Float> $name<F> {
            pub fn new($($($n_field: F),+),+) -> $name<F> {
                $name {
                    $($field: $point::new($($n_field),+)),+
                }
            }

            $(
                pub fn $dim(&self) -> $poly<F> {
                    $poly::new($(self.$dfield.$dim),+)
                }
            )+
        }

        impl<F> $crate::BezCurve<F> for $name<F>
                where F: $crate::traitdefs::Float {
            type Point = $point<F>;
            type Vector = $vector<F>;
            type Elevated = $elevated<$($est),+>;

            fn from_slice(slice: &[$point<F>]) -> Option<$name<F>> {
                use $crate::OrderStatic;
                if slice.len() - 1 != $name::<F>::order_static() {
                    None
                } else {
                    let mut index = -1;
                    Some($name {$($field: {
                        index += 1;
                        slice[index as usize]
                    }),+})  
                }
            }

            fn interp_unbounded(&self, t: F) -> $point<F> {
                use $crate::BezCurve;

                $point::new($(self.$dim().interp_unbounded(t)),+)
            }

            fn slope_unbounded(&self, t: F) -> $vector<F> {
                use $crate::BezCurve;

                $vector::new($(self.$dim().slope_unbounded(t)),+)
            }

            fn elevate(&self) -> $elevated<$($est),+> {
                use $crate::BezCurve;
                
                $(let $dim = self.$dim().elevate();)+
                $elevated::from([$($point::new($($edim.as_ref()[$eindex]),+)),+])
            }

            fn order(&self) -> usize {
                use $crate::OrderStatic;
                $poly::<F>::order_static()
            }
        }

        impl<F> $crate::OrderStatic for $name<F>
                where F: $crate::traitdefs::Float {
            #[inline]
            fn order_static() -> usize {
                use $crate::OrderStatic;
                $poly::<F>::order_static()
            }
        }

        impl<F> ::std::convert::From<[$point<F>; $order + 1]> for $name<F>
                where F: $crate::traitdefs::Float {
            fn from(array: [$point<F>; $order + 1]) -> $name<F> {
                use $crate::BezCurve;
                $name::from_slice(&array[..]).unwrap()
            }
        }

        impl<F> ::std::convert::AsRef<[$point<F>]> for $name<F>
                where F: $crate::traitdefs::Float {
            fn as_ref(&self) -> &[$point<F>] {
                use std::slice;
                unsafe {
                    slice::from_raw_parts(self as *const $name<F> as *const $point<F>, $order + 1)
                }
            }
        }

        impl<F> ::std::convert::AsMut<[$point<F>]> for $name<F>
                where F: $crate::traitdefs::Float {
            fn as_mut(&mut self) -> &mut [$point<F>] {
                use std::slice;
                unsafe {
                    slice::from_raw_parts_mut(self as *mut $name<F> as *mut $point<F>, $order + 1)
                }
            }
        }
    };
}