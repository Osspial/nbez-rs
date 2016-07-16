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

        impl<F: $crate::traitdefs::Float> std::ops::Mul<$rhs<F>> for $lhs<F> {
            type Output = $lhs<F>;

            fn mul(self, rhs: $rhs<F>) -> $lhs<F> {
                $lhs {
                    $($field: self.$field * rhs.$field),*
                }
            }
        }

        impl<F: $crate::traitdefs::Float> std::ops::Div<$rhs<F>> for $lhs<F> {
            type Output = $lhs<F>;

            fn div(self, rhs: $rhs<F>) -> $lhs<F> {
                $lhs {
                    $($field: self.$field / rhs.$field),+
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
        #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

        impl<F: $crate::traitdefs::Float> num_traits::identities::Zero for $name<F> {
            fn zero() -> $name<F> {
                $name {
                    $($field: F::zero()),+
                }
            }

            fn is_zero(&self) -> bool {
                *self == $name::<F>::zero()
            }
        }

        impl<F: $crate::traitdefs::Float> num_traits::identities::One for $name<F> {
            fn one() -> $name<F> {
                $name {
                    $($field: F::one()),+
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


/// Create an n-order bezier polynomial.
///
/// `$doc`: the documentation for the struct
///
/// `$order`: the order of the curve
///
/// `$sum`: the sum of all numbers between 1 and $order
///
/// `$name`: the name of the struct
///
/// `$field`: the name of the field for the various points on the polynomial
///
/// `$weight`: the weight of each of the fields
///
/// `$start`: the first field
///
/// `$left`, `$right`: a pair of adjacent fields. Used for slope and curve elevation
///
/// `$end`: the last field
macro_rules! n_bezier {
    ($doc:expr, $order:expr, $sum:expr; $name:ident {
        $($field:ident: $weight:expr),+
    } {
        $start:ident;
        $($left:ident, $right:ident: $dweight:expr),+;
        $end:ident;
    } elevated $elevated:ident<$($est:ty),+>) => {
        #[derive(Debug, Clone, Copy)]
        #[doc=$doc]
        pub struct $name<F = f32, P = $crate::Point2d<F>, V = $crate::Vector2d<F>>
                where F: $crate::traitdefs::Float,
                      P: $crate::traitdefs::Point<F, V>,
                      V: $crate::traitdefs::Vector<F, P> {
            $(pub $field: P),+,
            __marker: std::marker::PhantomData<(V, F)>
        }

        impl<F, P, V> $name<F, P, V>
                where F: $crate::traitdefs::Float,
                      P: $crate::traitdefs::Point<F, V>,
                      V: $crate::traitdefs::Vector<F, P> {
            pub fn new($($field: P),+) -> $name<F, P, V> {
                $name {
                    $($field: $field),+,
                    __marker: std::marker::PhantomData
                }
            }
        }

        impl<F, P, V> $crate::BezCurve<F> for $name<F, P, V>
                where P: $crate::traitdefs::Point<F, V>,
                      V: $crate::traitdefs::Vector<F, P>,
                      F: $crate::traitdefs::Float {
            type Point = P;
            type Vector = V;
            type Elevated = $elevated<$($est),+>;

            fn from_slice(slice: &[P]) -> Option<$name<F, P, V>> {
                use $crate::OrderStatic;
                if slice.len() - 1 != $name::<F, P, V>::order_static() {
                    None
                } else {
                    let mut index = -1;
                    Some($name {$($field: {
                        index += 1;
                        slice[index as usize]
                    }),+,
                    __marker: std::marker::PhantomData})
                }
            }

            fn interp_unbounded(&self, t: F) -> P {
                let t1 = F::from_f32(1.0).unwrap() - t;
                const COUNT: i32 = $order;
                let mut factor = COUNT + 1;

                $(
                    factor -= 1;
                    let $field =
                        self.$field *
                        t1.powi(factor) * 
                        t.powi(COUNT-factor) *  
                        F::from_i32($weight).unwrap();
                )+
                $($field +)+ P::zero()
            }

            fn slope_unbounded(&self, t: F) -> V {
                let t1 = F::from_f32(1.0).unwrap() - t;
                const COUNT: i32 = $order - 1;
                let mut factor = COUNT + 1;

                $(
                    factor -= 1;
                    let $right =
                        (self.$right - self.$left) *
                        t1.powi(factor) *
                        t.powi(COUNT-factor) *
                        F::from_i32($dweight * (COUNT + 1)).unwrap();
                )+
                V::from($($right +)+ P::zero())
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

            fn split(&self, t: F) -> Option<($name<F, P, V>, $name<F, P, V>)> {
                use $crate::lerp;


                if $order == 1 {
                    let interp = lerp(self.$start, self.$end, t);
                    Some((
                        $name::from_slice([self.$start, interp].as_ref()).unwrap(),
                        $name::from_slice([interp, self.$end].as_ref()).unwrap()
                    ))
                } else {
                    // `self`'s points as a slice
                    let pslice = self.as_ref();

                    const LERP_LEN: usize = $sum;
                    let mut lerps = [P::zero(); LERP_LEN];

                    // Populate `lerps` with linear interpolations of points and other elements of lerps.
                    //
                    // What that means is that the first few indices will be filled with direct interpolations of
                    // points in the curve, and subsequent indices will interpolate between those points, and so on.
                    // For example, with the fourth-order polynomial containing these points:
                    //
                    // [0.0, 1.0, 0.5, 1.0, 0.0]
                    // 
                    // The first four indicies of `lerps` will contain interpolations between those points, like so
                    // (if splitting the curve at t = 0.5):
                    //
                    // 0.0 & 1.0    1.0 & 0.5    0.5 & 1.0    1.0 & 0.0
                    // [  0.5,         0.75,        0.75,        0.5    ...
                    //
                    // The next three indicies will be interpolations between *those* points, like so:
                    //
                    //     0.5 & 0.75    0.75 & 0.75    0.75 & 0.5
                    // ...    0.625,         0.75,          0.625  ...
                    // 
                    // The next two interpolate between the above three points, and the last point interpolates between
                    // those two.
                    let mut offset = $order;
                    let mut offset_threshold = $order * 2 - 1;
                    for i in 0..LERP_LEN {
                        if i < $order {
                            lerps[i] = lerp(pslice[i], pslice[i+1], t);
                        } else {
                            if i >= offset_threshold {
                                offset -= 1;
                                offset_threshold += offset - 1;
                            }
                            lerps[i] = lerp(lerps[i - offset], lerps[i - offset + 1], t);
                        }
                    }

                    // The length of the `points` array
                    const POINTS_LEN: usize = $order * 2 + 1;

                    // An array containing the control points that compose the split curves, with one point of overlap
                    // where the curves meet.
                    let mut points = [P::zero(); POINTS_LEN];
                    points[0] = self.$start;
                    points[$order] = lerps[LERP_LEN - 1];
                    points[POINTS_LEN - 1] = self.$end;


                    // The index in `lerps` that we're going to be accessing for `points`
                    let mut lerp_index = 0;

                    // Move the lerped points from `lerp` to `points`
                    for i in 1..$order {
                        points[i] = lerps[lerp_index];
                        points[POINTS_LEN - 1 - i] = lerps[lerp_index + $order - i];
                        
                        lerp_index += $order - i + 1;
                    }


                    Some((
                        $name::from_slice(&points[..$order + 1]).expect("left poly fail"),
                        $name::from_slice(&points[$order..]).expect("right poly fail")
                    ))
                }
            }

            fn order(&self) -> usize {
                use $crate::OrderStatic;
                $name::<F, P, V>::order_static()
            }
        }

        impl<F, P, V> $crate::OrderStatic for $name<F, P, V> 
                where F: $crate::traitdefs::Float,
                      P: $crate::traitdefs::Point<F, V>,
                      V: $crate::traitdefs::Vector<F, P> {
            #[inline]
            fn order_static() -> usize {
                $order
            }
        }

        impl<F, P, V> ::std::convert::From<[P; $order + 1]> for $name<F, P, V> 
                where F: $crate::traitdefs::Float,
                      P: $crate::traitdefs::Point<F, V>,
                      V: $crate::traitdefs::Vector<F, P> {
            fn from(array: [P; $order + 1]) -> $name<F, P, V> {
                use $crate::BezCurve;
                $name::from_slice(&array[..]).unwrap()
            }
        }

        impl<F, P, V> ::std::convert::AsRef<[P]> for $name<F, P, V> 
                where F: $crate::traitdefs::Float,
                      P: $crate::traitdefs::Point<F, V>,
                      V: $crate::traitdefs::Vector<F, P> {
            fn as_ref(&self) -> &[P] {
                use std::slice;
                unsafe {
                    slice::from_raw_parts(self as *const $name<F, P, V> as *const P, $order + 1)
                }
            }
        }

        impl<F, P, V> ::std::convert::AsMut<[P]> for $name<F, P, V> 
                where F: $crate::traitdefs::Float,
                      P: $crate::traitdefs::Point<F, V>,
                      V: $crate::traitdefs::Vector<F, P> {
            fn as_mut(&mut self) -> &mut [P] {
                use std::slice;
                unsafe {
                    slice::from_raw_parts_mut(self as *mut $name<F, P, V> as *mut P, $order + 1)
                }
            }
        }
    }
}
