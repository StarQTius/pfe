use crypto::{digest::Digest, sha3};
use itertools::{iproduct, Itertools};
use std::mem::size_of;

#[cfg(test)]
mod tests;

pub type PolynomialCoeff = u32;
pub type Polynomial = [PolynomialCoeff; POLYNOMIAL_DEGREE];

pub const K: u16 = 8;
pub const L: u16 = 7;
pub const SEED_SIZE: usize = 32;

const Q: PolynomialCoeff = 8380417;
const POLYNOMIAL_DEGREE: usize = 256;
const SAMPLE_INTEGER_SIZE: usize = 3;

pub fn expand_a(seed: &[u8; SEED_SIZE]) -> [[Polynomial; L as usize]; K as usize] {
    iproduct!(0..K, 0..L)
        .map(|(i, j)| (i, expand_vec(seed, (i << 8) + j)))
        .group_by(|(i, _)| *i)
        .into_iter()
        .map(|(_, it)| {
            it.map(|(_, poly)| poly)
                .collect::<Vec<Polynomial>>()
                // `group_by()` should have resulted in an iterator of iterators each of count `L`
                .try_into()
                .unwrap()
        })
        .collect::<Vec<[Polynomial; L as usize]>>()
        // `group_by()` should have resulted in an iterator of count `K`
        .try_into()
        .unwrap()
}

fn expand_vec(seed: &[u8; SEED_SIZE], nonce: u16) -> Polynomial {
    let mut hasher = sha3::Sha3::shake_128();
    let mut block_buf = [0; size_of::<PolynomialCoeff>()];

    hasher.input(&seed[..SEED_SIZE]);
    hasher.input(&nonce.to_le_bytes());

    (0..)
        .map(|_| {
            hasher.result(&mut block_buf[..SAMPLE_INTEGER_SIZE]);
            block_buf[SAMPLE_INTEGER_SIZE - 1] &= 0x7f;

            PolynomialCoeff::from_le_bytes(block_buf)
        })
        .filter(|coeff| coeff < &Q)
        .take(POLYNOMIAL_DEGREE)
        .collect::<Vec<PolynomialCoeff>>()
        // Should not fail since we took exactly `POLYNOMIAL_DEGREE` elements
        .try_into()
        .unwrap()
}
