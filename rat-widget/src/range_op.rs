//!
//! Bounded numeric operations.
//!

/// Bounded numeric add and subtract.
pub trait RangeOp
where
    Self: Sized,
{
    type Step;

    /// Addition. Bounded to min/max.
    fn add_clamp(self, delta: Self::Step, bounds: (Self, Self)) -> Self;

    /// Subtraction. Bounded to min/max.
    fn sub_clamp(self, delta: Self::Step, bounds: (Self, Self)) -> Self;
}

macro_rules! u_range_op {
    ($value_ty:ty, $step_ty:ty) => {
        impl RangeOp for $value_ty {
            type Step = $step_ty;

            #[inline(always)]
            fn add_clamp(self, delta: Self::Step, bounds: (Self, Self)) -> Self {
                self.saturating_add(delta).clamp(bounds.0, bounds.1)
            }

            #[inline(always)]
            fn sub_clamp(self, delta: Self::Step, bounds: (Self, Self)) -> Self {
                self.saturating_sub(delta).clamp(bounds.0, bounds.1)
            }
        }
    };
}

macro_rules! i_range_op {
    ($value_ty:ty, $step_ty:ty) => {
        impl RangeOp for $value_ty {
            type Step = $step_ty;

            #[inline(always)]
            fn add_clamp(self, delta: Self::Step, bounds: (Self, Self)) -> Self {
                self.saturating_add_unsigned(delta)
                    .clamp(bounds.0, bounds.1)
            }

            #[inline(always)]
            fn sub_clamp(self, delta: Self::Step, bounds: (Self, Self)) -> Self {
                self.saturating_sub_unsigned(delta)
                    .clamp(bounds.0, bounds.1)
            }
        }
    };
}

u_range_op!(u8, u8);
u_range_op!(u16, u16);
u_range_op!(u32, u32);
u_range_op!(u64, u64);
u_range_op!(u128, u128);
u_range_op!(usize, usize);
i_range_op!(i8, u8);
i_range_op!(i16, u16);
i_range_op!(i32, u32);
i_range_op!(i64, u64);
i_range_op!(i128, u128);
i_range_op!(isize, usize);

impl RangeOp for f32 {
    type Step = f32;

    #[inline(always)]
    fn add_clamp(self, delta: Self::Step, bounds: (Self, Self)) -> Self {
        (self + delta).clamp(bounds.0, bounds.1)
    }

    #[inline(always)]
    fn sub_clamp(self, delta: Self::Step, bounds: (Self, Self)) -> Self {
        (self - delta).clamp(bounds.0, bounds.1)
    }
}

impl RangeOp for f64 {
    type Step = f64;

    #[inline(always)]
    fn add_clamp(self, delta: Self::Step, bounds: (Self, Self)) -> Self {
        (self + delta).clamp(bounds.0, bounds.1)
    }

    #[inline(always)]
    fn sub_clamp(self, delta: Self::Step, bounds: (Self, Self)) -> Self {
        (self - delta).clamp(bounds.0, bounds.1)
    }
}
