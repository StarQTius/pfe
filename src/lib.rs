use crypto::{digest::Digest, sha3};
use itertools::{iproduct, Itertools};
use std::mem::size_of;

#[cfg(test)]
mod tests;

pub type PolynomialCoeff = i64;
pub type Polynomial = [PolynomialCoeff; POLYNOMIAL_DEGREE];

pub const K: u16 = 8;
pub const L: u16 = 7;
pub const SEED_SIZE: usize = 64;

const Q: PolynomialCoeff = 8380417;
const ETA: PolynomialCoeff = 2;
const POLYNOMIAL_DEGREE: usize = 256;
const SAMPLE_INTEGER_SIZE: usize = 3;
const HALF_SEED_SIZE: usize = SEED_SIZE / 2;
const GAMMA: PolynomialCoeff = 1 << 19;

pub fn expand_a(seed: &[u8; SEED_SIZE]) -> [[Polynomial; L as usize]; K as usize] {
    let mut hasher = sha3::Sha3::shake_128();
    let mut block_buf = [0; size_of::<PolynomialCoeff>()];

    iproduct!(0..K, 0..L)
        .map(|(i, j)| {
            hasher.reset();
            hasher.input(&seed[..HALF_SEED_SIZE]);
            hasher.input(&((i << 8) + j).to_le_bytes());

            (
                i,
                expand_polynomial(0, Q - 1, |_| {
                    hasher.result(&mut block_buf[..SAMPLE_INTEGER_SIZE]);
                    block_buf[SAMPLE_INTEGER_SIZE - 1] &= 0x7f;

                    PolynomialCoeff::from_le_bytes(block_buf)
                }),
            )
        })
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

pub fn expand_s(seed: &[u8; SEED_SIZE]) -> [Polynomial; L as usize] {
    let mut hasher = sha3::Sha3::shake_256();
    let mut block_buf_opt: Option<[u8; 1]> = None;

    (0..L)
        .map(|nonce| {
            hasher.reset();
            hasher.input(&seed[..SEED_SIZE]);
            hasher.input(&nonce.to_le_bytes());
            block_buf_opt = None;

            expand_polynomial(0, 14, |_| match block_buf_opt {
                None => {
                    let mut block_buf = [0; 1];
                    hasher.result(&mut block_buf);
                    block_buf_opt = Some(block_buf);
                    (block_buf[0] & 0xf) as PolynomialCoeff
                }
                Some(block_buf) => {
                    block_buf_opt = None;
                    (block_buf[0] >> 4) as PolynomialCoeff
                }
            })
            .map(|coeff| ETA - (coeff % (2 * ETA + 1)))
        })
        .collect::<Vec<Polynomial>>()
        // Should not fail since we are iterating over `L` elements
        .try_into()
        .unwrap()
}

pub fn expand_y(seed: &[u8; SEED_SIZE]) -> [Polynomial; L as usize] {
    let mut hasher = sha3::Sha3::shake_256();
    let mut block_buf = [0; 3];

    (0..L)
        .map(|nonce| {
            hasher.reset();
            hasher.input(&seed[..SEED_SIZE]);
            hasher.input(&nonce.to_le_bytes());

            expand_polynomial(1 - GAMMA, GAMMA, |i| {
                let k = 4 * (i % 2);

                if k == 0 {
                    hasher.result(&mut block_buf);
                } else {
                    block_buf[0] = block_buf[2];
                    hasher.result(&mut block_buf[1..]);
                }

                let mut coeff = (block_buf[0] as PolynomialCoeff) >> k;
                coeff |= (block_buf[1] as PolynomialCoeff) << (8 - k);
                coeff |= (block_buf[2] as PolynomialCoeff) << (16 - k);
                coeff &= 0xfffff;
                GAMMA - coeff
            })
        })
        .collect::<Vec<Polynomial>>()
        // Should not fail since we are iterating over `L` elements
        .try_into()
        .unwrap()
}

fn expand_polynomial(
    inf: PolynomialCoeff,
    sup: PolynomialCoeff,
    generator: impl FnMut(i32) -> PolynomialCoeff,
) -> Polynomial {
    (0..)
        .map(generator)
        .filter(|coeff| &inf <= coeff && coeff <= &sup)
        .take(POLYNOMIAL_DEGREE)
        .collect::<Vec<PolynomialCoeff>>()
        // Should not fail since we took exactly `POLYNOMIAL_DEGREE` elements
        .try_into()
        .unwrap()
}
