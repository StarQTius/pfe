macro_rules! subarray_mut {
    ($array:ident[$start:tt..$end:tt]) => {
        $array.subarray_mut::<{ $end - $start }, $start, $end>()
    };
    ($array:ident[..$end:tt]) => {
        $array.subarray_mut::<$end, 0, $end>()
    };
    ($array:ident[$start:tt..]) => {
        $array.subarray_mut::<{ $array.len() - $start }, $start, $array.len()>()
    };
}

pub(crate) use subarray_mut;

macro_rules! const_assert_lesser {
    ($lhs:expr, $rhs:expr) => {
        let _ = AssertLesser::<$lhs, $rhs>::GUARD;
    };
}

macro_rules! const_assert_lesser_or_equal {
    ($lhs:expr, $rhs:expr) => {
        let _ = AssertLesserOrEqual::<$lhs, $rhs>::GUARD;
    };
}

pub trait Subarray {
    type Item;

    fn subarray<const LENGTH: usize, const START: usize, const END: usize>(
        &self,
    ) -> &[Self::Item; LENGTH];
    fn subarray_mut<const LENGTH: usize, const START: usize, const END: usize>(
        &mut self,
    ) -> &mut [Self::Item; LENGTH];
}

impl<T, const N: usize> Subarray for [T; N] {
    type Item = T;

    fn subarray<const LENGTH: usize, const START: usize, const END: usize>(
        &self,
    ) -> &[Self::Item; LENGTH] {
        const_assert_lesser!(LENGTH, N);
        const_assert_lesser_or_equal!(END, N);

        self[START..END].try_into().unwrap()
    }

    fn subarray_mut<const LENGTH: usize, const START: usize, const END: usize>(
        &mut self,
    ) -> &mut [Self::Item; LENGTH] {
        const_assert_lesser!(LENGTH, N);
        const_assert_lesser_or_equal!(END, N);

        (&mut self[START..END]).try_into().unwrap()
    }
}

trait Assert {
    const GUARD: ();
}

struct AssertLesser<const LHS: usize, const RHS: usize>;

impl<const LHS: usize, const RHS: usize> Assert for AssertLesser<LHS, RHS> {
    const GUARD: () = assert!(LHS < RHS);
}

struct AssertLesserOrEqual<const LHS: usize, const RHS: usize>;

impl<const LHS: usize, const RHS: usize> Assert for AssertLesserOrEqual<LHS, RHS> {
    const GUARD: () = assert!(LHS <= RHS);
}
