use core::{
    iter::{zip, Sum},
    ops::{AddAssign, Mul, SubAssign},
};

pub fn pointwise_add_inplace<T: AddAssign, const N: usize>(lhs: &mut [T; N], rhs: [T; N]) {
    for (lhs_coeff, rhs_coeff) in zip(lhs, rhs) {
        *lhs_coeff += rhs_coeff;
    }
}

pub fn pointwise_sub_inplace<T: SubAssign, const N: usize>(lhs: &mut [T; N], rhs: [T; N]) {
    for (lhs_coeff, rhs_coeff) in zip(lhs, rhs) {
        *lhs_coeff -= rhs_coeff;
    }
}

pub fn scalar_mul<'a, T, const N: usize>(lhs: &'a [T; N], rhs: &'a [T; N]) -> T
where
    T: Sum,
    &'a T: Mul<Output = T> + 'a,
{
    zip(lhs, rhs)
        .map(|(lhs_coeff, rhs_coeff)| lhs_coeff * rhs_coeff)
        .sum()
}
