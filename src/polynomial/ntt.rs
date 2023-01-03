use crate::{
    array_math::{pointwise_add_inplace, pointwise_sub_inplace},
    coefficient::{self, Coefficient},
    polynomial::{plain::PlainPolynomial, Coefficients, ZETAS},
};
use std::{
    iter::{zip, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
    slice,
};

use super::NB_COEFFICIENTS;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct NTTPolynomial {
    coefficients: Coefficients,
}

impl NTTPolynomial {
    pub fn into_plain(mut self) -> PlainPolynomial {
        let f = 41978;
        let mut zeta_it = ZETAS.iter().rev();

        for chunk_size in (0..8).map(|n| 1 << n) {
            let double_chunks_it = self.coefficients.chunks_mut(2 * chunk_size);
            for double_chunks in double_chunks_it {
                let &zeta = zeta_it.next().unwrap();
                let (lchunk, rchunk) = double_chunks.split_at_mut(chunk_size);
                for (lx, rx) in zip(lchunk, rchunk) {
                    let tmp = *lx;
                    *lx = tmp + *rx;
                    *rx = tmp - *rx;
                    *rx = coefficient::reduce_montgomery(*rx as i64 * -zeta as i64);
                }
            }
        }

        for coeff in self.coefficients.iter_mut() {
            *coeff = coefficient::reduce_montgomery(*coeff as i64 * f);
        }

        PlainPolynomial::from(self.coefficients)
    }

    pub fn map_inplace(&mut self, f: &mut impl FnMut(Coefficient) -> Coefficient) {
        for coeff in self.coefficients.iter_mut() {
            *coeff = f(*coeff);
        }
    }
}

impl From<Coefficients> for NTTPolynomial {
    fn from(coefficients: Coefficients) -> Self {
        Self { coefficients }
    }
}

impl Add for NTTPolynomial {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl AddAssign for NTTPolynomial {
    fn add_assign(&mut self, rhs: Self) {
        pointwise_add_inplace(&mut self.coefficients, rhs.coefficients)
    }
}

impl Sum for NTTPolynomial {
    fn sum<It: Iterator<Item = Self>>(it: It) -> Self {
        let init = Self::from([0; NB_COEFFICIENTS]);
        it.fold(init, |acc, poly| acc + poly)
    }
}

impl Sub for NTTPolynomial {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self {
        self -= rhs;
        self
    }
}

impl SubAssign for NTTPolynomial {
    fn sub_assign(&mut self, rhs: Self) {
        pointwise_sub_inplace(&mut self.coefficients, rhs.coefficients);
    }
}

impl Mul for NTTPolynomial {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self {
        self *= rhs;
        self
    }
}

impl Mul for &NTTPolynomial {
    type Output = NTTPolynomial;

    fn mul(self, rhs: Self) -> NTTPolynomial {
        let mut retval = NTTPolynomial::from(self.coefficients);
        retval *= rhs;
        retval
    }
}

impl MulAssign for NTTPolynomial {
    fn mul_assign(&mut self, rhs: Self) {
        *self *= &rhs;
    }
}

impl MulAssign<&Self> for NTTPolynomial {
    fn mul_assign(&mut self, rhs: &Self) {
        for (self_coeff, &rhs_coeff) in zip(self.coefficients.iter_mut(), rhs.coefficients.iter()) {
            *self_coeff = coefficient::reduce_montgomery(*self_coeff as i64 * rhs_coeff as i64);
        }
    }
}

impl Default for NTTPolynomial {
    fn default() -> Self {
        Self::from([0; NB_COEFFICIENTS])
    }
}

impl<'a> IntoIterator for &'a NTTPolynomial {
    type Item = &'a Coefficient;
    type IntoIter = slice::Iter<'a, Coefficient>;

    fn into_iter(self) -> Self::IntoIter {
        self.coefficients.iter()
    }
}
