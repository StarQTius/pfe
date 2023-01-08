use crate::{
    array_math::{pointwise_add_inplace, pointwise_sub_inplace},
    coefficient::{self, Coefficient},
    packing::Pack,
    polynomial::{ntt::NTTPolynomial, Coefficients, ZETAS},
    TryCollectArray,
};
use std::{
    iter::zip,
    ops::{Add, AddAssign, Sub, SubAssign},
    slice,
};

use super::NB_COEFFICIENTS;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PlainPolynomial {
    coefficients: Coefficients,
}

impl PlainPolynomial {
    pub fn into_ntt(mut self) -> NTTPolynomial {
        let mut zeta_it = ZETAS[1..].iter();

        for chunk_size in (0..8).rev().map(|n| 1 << n) {
            let double_chunks_it = self.coefficients.chunks_mut(2 * chunk_size);
            for double_chunks in double_chunks_it {
                let &zeta = zeta_it.next().unwrap();
                let (lchunk, rchunk) = double_chunks.split_at_mut(chunk_size);
                for (lx, rx) in zip(lchunk, rchunk) {
                    let tmp = coefficient::reduce_montgomery(*rx as i64 * zeta as i64);
                    *rx = *lx - tmp;
                    *lx += tmp;
                }
            }
        }

        NTTPolynomial::from(self.coefficients)
    }

    pub fn map_inplace(&mut self, f: &mut impl FnMut(Coefficient) -> Coefficient) {
        for coeff in self.coefficients.iter_mut() {
            *coeff = f(*coeff);
        }
    }

    pub fn map_fork(
        &mut self,
        f: &mut impl FnMut(Coefficient) -> (Coefficient, Coefficient),
    ) -> Self {
        let mut other_coefficients: Coefficients = [0; NB_COEFFICIENTS];
        for (self_coeff, other_coeff) in
            zip(self.coefficients.iter_mut(), other_coefficients.iter_mut())
        {
            (*self_coeff, *other_coeff) = f(*self_coeff);
        }
        Self::from(other_coefficients)
    }

    pub fn max(&self) -> Coefficient {
        self.coefficients
            .iter()
            .copied()
            .map(coefficient::abs)
            .max()
            .unwrap()
    }

    pub fn dump(&self) -> [i64; NB_COEFFICIENTS] {
        self.coefficients.map(|x| x as i64)
    }
}

impl From<Coefficients> for PlainPolynomial {
    fn from(coefficients: Coefficients) -> Self {
        Self { coefficients }
    }
}

impl Add for PlainPolynomial {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl AddAssign for PlainPolynomial {
    fn add_assign(&mut self, rhs: Self) {
        pointwise_add_inplace(&mut self.coefficients, rhs.coefficients)
    }
}

impl Sub for PlainPolynomial {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self {
        self -= rhs;
        self
    }
}

impl SubAssign for PlainPolynomial {
    fn sub_assign(&mut self, rhs: Self) {
        pointwise_sub_inplace(&mut self.coefficients, rhs.coefficients);
    }
}

impl Pack for PlainPolynomial {
    type Input = Coefficient;
    type Output = u8;

    fn pack_inplace<const IN_N: usize, const OUT_N: usize>(
        &self,
        f: &impl Fn(&[Self::Input; IN_N]) -> [Self::Output; OUT_N],
        output: &mut [Self::Output],
    ) {
        assert!(NB_COEFFICIENTS / IN_N == output.len() / OUT_N);
        assert!(NB_COEFFICIENTS % IN_N == 0);
        assert!(output.len() % OUT_N == 0);

        let coefficients_chunk_it = self.coefficients.chunks_exact(IN_N);
        let output_chunk_it = output.chunks_exact_mut(OUT_N);

        assert!(coefficients_chunk_it.remainder().is_empty());

        for (coeff_chunk, output_chunk) in zip(coefficients_chunk_it, output_chunk_it) {
            // Unwrapping is safe here because out chunks are of the right size
            output_chunk.copy_from_slice(&f(coeff_chunk.try_into().unwrap()));
        }
    }

    fn unpack<const IN_N: usize, const OUT_N: usize>(
        packed: &[Self::Output],
        f: &impl Fn(&[Self::Output; OUT_N]) -> [Self::Input; IN_N],
    ) -> Self {
        let it = packed
            .chunks_exact(OUT_N)
            .flat_map(TryFrom::try_from)
            .flat_map(f);

        Self::from(it.try_collect_array().unwrap())
    }
}

impl Default for PlainPolynomial {
    fn default() -> Self {
        Self::from([0; NB_COEFFICIENTS])
    }
}

impl<'a> IntoIterator for &'a PlainPolynomial {
    type Item = &'a Coefficient;
    type IntoIter = slice::Iter<'a, Coefficient>;

    fn into_iter(self) -> Self::IntoIter {
        self.coefficients.iter()
    }
}
