use core::{
    array::from_fn,
    iter::zip,
    ops::{Add, AddAssign, Mul, Sub, SubAssign},
    slice,
};

use crate::{
    array_math::{pointwise_add_inplace, pointwise_sub_inplace, scalar_mul},
    coefficient::{self, Coefficient},
    packing::Pack,
    polynomial::{ntt::NTTPolynomial, plain::PlainPolynomial, NB_COEFFICIENTS},
    TryCollectArray, D,
};

pub type Matrix<Scalar, const N: usize, const M: usize> = Vector<Vector<Scalar, N>, M>;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Vector<Scalar, const N: usize> {
    coefficients: [Scalar; N],
}

impl<const N: usize> Vector<NTTPolynomial, N> {
    pub fn into_plain(self) -> Vector<PlainPolynomial, N> {
        Vector::from(
            self.coefficients
                .into_iter()
                .map(|poly| poly.into_plain())
                .try_collect_array()
                .unwrap(),
        )
    }

    pub fn reduce_32(mut self) -> Self {
        self.map_inplace(&mut coefficient::reduce_32);
        self
    }

    fn map_inplace(&mut self, f: &mut impl FnMut(Coefficient) -> Coefficient) {
        for coeff in self.coefficients.iter_mut() {
            coeff.map_inplace(f);
        }
    }
}

impl<const N: usize> Vector<PlainPolynomial, N> {
    pub fn into_ntt(self) -> Vector<NTTPolynomial, N> {
        Vector::from(
            self.coefficients
                .into_iter()
                .map(|poly| poly.into_ntt())
                .try_collect_array()
                .unwrap(),
        )
    }

    pub fn reduce_32(mut self) -> Self {
        self.map_inplace(&mut coefficient::reduce_32);
        self
    }

    pub fn caddq(mut self) -> Self {
        self.map_inplace(&mut coefficient::caddq);
        self
    }

    pub fn decompose(mut self) -> (Self, Self) {
        let other = self.map_fork(&mut coefficient::decompose);
        (self, other)
    }

    pub fn power2round(mut self) -> (Self, Self) {
        let other = self.map_fork(&mut coefficient::power2round);
        (self, other)
    }

    pub fn max(&self) -> Coefficient {
        self.coefficients
            .iter()
            .map(|coeff| coeff.max())
            .max()
            .unwrap()
    }

    pub fn use_hint(mut self, hint: &[[bool; NB_COEFFICIENTS]; N]) -> Self {
        let mut hint_it = hint.iter().flatten();
        self.map_inplace(&mut |coeff| {
            let &hint = hint_it.next().unwrap();
            let (a0, a1) = coefficient::decompose(coeff);

            if hint {
                (a1 + (if a0 > 0 { 1 } else { -1 })) & 15
            } else {
                a1
            }
        });

        self
    }

    pub fn shift_d(mut self) -> Self {
        self.map_inplace(&mut |coeff| coeff << D);
        self
    }

    fn map_inplace(&mut self, f: &mut impl FnMut(Coefficient) -> Coefficient) {
        for coeff in self.coefficients.iter_mut() {
            coeff.map_inplace(f);
        }
    }

    fn map_fork(&mut self, f: &mut impl FnMut(Coefficient) -> (Coefficient, Coefficient)) -> Self {
        let retval_it = self.coefficients.iter_mut().map(|coeff| coeff.map_fork(f));
        Self::from(retval_it.try_collect_array().unwrap())
    }

    pub fn dump(&self) -> [[i64; 256]; N] {
        let mut it = self.coefficients.iter().map(|coeff| coeff.dump());
        from_fn(|_| it.next().unwrap())
    }
}

impl<Scalar: AddAssign, const N: usize> Add for Vector<Scalar, N> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        pointwise_add_inplace(&mut self.coefficients, rhs.coefficients);
        self
    }
}

impl<Scalar: SubAssign, const N: usize> Sub for Vector<Scalar, N> {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self {
        pointwise_sub_inplace(&mut self.coefficients, rhs.coefficients);
        self
    }
}

impl<const N: usize> Mul for &Vector<NTTPolynomial, N> {
    type Output = NTTPolynomial;

    fn mul(self, rhs: Self) -> NTTPolynomial {
        scalar_mul(&self.coefficients, &rhs.coefficients)
    }
}

impl<const N: usize> Mul<&NTTPolynomial> for Vector<NTTPolynomial, N> {
    type Output = Vector<NTTPolynomial, N>;

    fn mul(mut self, rhs: &NTTPolynomial) -> Self::Output {
        for coeff in self.coefficients.iter_mut() {
            *coeff *= rhs;
        }

        self
    }
}

impl<const N: usize, const M: usize> Mul<&Vector<NTTPolynomial, M>>
    for &Matrix<NTTPolynomial, M, N>
{
    type Output = Vector<NTTPolynomial, N>;

    fn mul(self, rhs: &Vector<NTTPolynomial, M>) -> Self::Output {
        let retval_it = self.coefficients.iter().map(|coeff| coeff * rhs);
        From::from(retval_it.try_collect_array().unwrap())
    }
}

impl<T, const N: usize> From<[T; N]> for Vector<T, N> {
    fn from(coefficients: [T; N]) -> Self {
        Self { coefficients }
    }
}

impl<T: Pack, const N: usize> Pack for Vector<T, N> {
    type Input = <T as Pack>::Input;
    type Output = <T as Pack>::Output;

    fn pack_inplace<const IN_N: usize, const OUT_N: usize>(
        &self,
        f: &impl Fn(&[Self::Input; IN_N]) -> [Self::Output; OUT_N],
        output: &mut [Self::Output],
    ) {
        let coeff_it = self.coefficients.iter();
        let output_chunk_it = output.chunks_exact_mut(output.len() / self.coefficients.len());

        for (coeff, output_chunk) in zip(coeff_it, output_chunk_it) {
            coeff.pack_inplace(f, output_chunk);
        }
    }

    fn unpack<const IN_N: usize, const OUT_N: usize>(
        packed: &[Self::Output],
        f: &impl Fn(&[Self::Output; OUT_N]) -> [Self::Input; IN_N],
    ) -> Self {
        let it = packed
            .chunks_exact(packed.len() / N)
            .map(|chunk| T::unpack(chunk, f));

        Self::from(it.try_collect_array().unwrap())
    }
}

impl<T, const N: usize> Default for Vector<T, N>
where
    T: Default,
{
    fn default() -> Self {
        Self::from(from_fn(|_| T::default()))
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a Vector<T, N> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.coefficients.iter()
    }
}
