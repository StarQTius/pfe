use crate::{
    coefficient,
    polynomial::{ntt::NTTPolynomial, plain::PlainPolynomial, NB_COEFFICIENTS},
    vector::{Matrix, Vector},
    TryCollectArray, ETA, GAMMA1, K, L, Q, SEED_SIZE,
};
use crypto::{digest::Digest, sha3};
use itertools::{iproduct, Itertools};
use std::mem::size_of;

pub fn expand_a(seed: &[u8; SEED_SIZE / 2]) -> Matrix<NTTPolynomial, L, K> {
    const _23BITS_MASK: coefficient::Coefficient = (1 << 23) - 1;
    const _23BITS_MASK_SIZE: usize = 3;

    let mut hasher = sha3::Sha3::shake_128();
    let mut block_buf = [0; size_of::<coefficient::Coefficient>()];

    let retval_chunks = iproduct!(0..K as u16, 0..L as u16)
        // For each of the `K * L` coefficients of our return value, we generate a polynomial with
        // rejection sampling
        .map(|(i, j)| {
            hasher.reset();
            hasher.input(seed);
            hasher.input(&((i << 8) + j).to_le_bytes());

            let polynomial_it = sample_polynomial(0, Q - 1, |_| {
                hasher.result(&mut block_buf[.._23BITS_MASK_SIZE]);

                coefficient::Coefficient::from_le_bytes(block_buf) & _23BITS_MASK
            });

            NTTPolynomial::from(polynomial_it.try_collect_array().unwrap())
        })
        // We divide the result into `K` chunks of size `L` and each chunk is used to create a
        // `Vector` of size L
        .chunks(L);

    let retval_it = retval_chunks
        .into_iter()
        .filter_map(|it| it.try_collect_array())
        .map(Vector::from);

    // Therefore, unwrapping here should be safe
    Matrix::from(retval_it.try_collect_array().unwrap())
}

pub fn expand_s<const N: usize>(seed: &[u8; SEED_SIZE], nonce: u16) -> Vector<PlainPolynomial, N> {
    let mut hasher = sha3::Sha3::shake_256();
    let mut block_buf_opt: Option<[u8; 1]> = None;

    // For each of the `N` coefficients of our return value, we generate a polynomial with
    // rejection sampling
    let retval_it = (nonce..nonce + N as u16).map(|nonce| {
        hasher.reset();
        hasher.input(&seed[..SEED_SIZE]);
        hasher.input(&nonce.to_le_bytes());
        block_buf_opt = None;

        let polynomial_it = sample_polynomial(0, 14, |_| match block_buf_opt {
            None => {
                let mut block_buf = [0; 1];
                hasher.result(&mut block_buf);
                block_buf_opt = Some(block_buf);
                (block_buf[0] & 0xf) as coefficient::Coefficient
            }
            Some(block_buf) => {
                block_buf_opt = None;
                (block_buf[0] >> 4) as coefficient::Coefficient
            }
        })
        .map(|coeff| ETA - (coeff % (2 * ETA + 1)));

        PlainPolynomial::from(polynomial_it.try_collect_array().unwrap())
    });

    // Therefore, unwrapping should be safe here
    Vector::from(retval_it.try_collect_array().unwrap())
}

pub fn expand_y(seed: &[u8; SEED_SIZE], nonce: u16) -> Vector<PlainPolynomial, L> {
    let mut hasher = sha3::Sha3::shake_256();
    let mut block_buf = [0; 3];

    // For each of the `L` coefficients of our return value, we generate a polynomial with
    // rejection sampling
    let retval_it = (L as u16 * nonce..L as u16 * (nonce + 1)).map(|nonce| {
        hasher.reset();
        hasher.input(&seed[..SEED_SIZE]);
        hasher.input(&nonce.to_le_bytes());

        let polynomial_it = sample_polynomial(1 - GAMMA1, GAMMA1, |i| {
            let k = 4 * (i % 2);

            if k == 0 {
                hasher.result(&mut block_buf);
            } else {
                block_buf[0] = block_buf[2];
                hasher.result(&mut block_buf[1..]);
            }

            let mut coeff = (block_buf[0] as coefficient::Coefficient) >> k;
            coeff |= (block_buf[1] as coefficient::Coefficient) << (8 - k);
            coeff |= (block_buf[2] as coefficient::Coefficient) << (16 - k);
            coeff &= 0xfffff;
            GAMMA1 - coeff
        });

        PlainPolynomial::from(polynomial_it.try_collect_array().unwrap())
    });

    // Therefore, unwrapping should be safe here
    Vector::from(retval_it.try_collect_array().unwrap())
}

fn sample_polynomial(
    inf: coefficient::Coefficient,
    sup: coefficient::Coefficient,
    generator: impl FnMut(usize) -> coefficient::Coefficient,
) -> impl Iterator<Item = coefficient::Coefficient> {
    (0usize..)
        .map(generator)
        .filter(move |coeff| (inf..=sup).contains(coeff))
        .take(NB_COEFFICIENTS)
}
