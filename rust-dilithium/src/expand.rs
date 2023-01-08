use crate::{
    coefficient,
    counter::Counter,
    polynomial::{ntt::NTTPolynomial, plain::PlainPolynomial, NB_COEFFICIENTS},
    subarr_mut,
    subarray::Subarray,
    vector::{Matrix, Vector},
    TryCollectArray, ETA, GAMMA1, K, L, Q,
};
use itertools::{iproduct, Itertools};
use std::mem::size_of;

pub fn expand_a(mut ctr: impl Counter) -> Matrix<NTTPolynomial, L, K> {
    const _23BITS_MASK: coefficient::Coefficient = (1 << 23) - 1;
    const _23BITS_MASK_SIZE: usize = 3;

    let mut block_buf = [0; size_of::<coefficient::Coefficient>()];

    let retval_chunks = iproduct!(0..K as u16, 0..L as u16)
        // For each of the `K * L` coefficients of our return value, we generate a polynomial with
        // rejection sampling
        .map(|(i, j)| {
            ctr.reset(256 * i + j);

            let polynomial_it = sample_polynomial(0, Q - 1, |_| {
                *subarr_mut!(block_buf[.._23BITS_MASK_SIZE]) = ctr.squeeze();
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

pub fn expand_s<const N: usize>(mut ctr: impl Counter, nonce: u16) -> Vector<PlainPolynomial, N> {
    let mut block_buf_opt: Option<[u8; 1]> = None;

    // For each of the `N` coefficients of our return value, we generate a polynomial with
    // rejection sampling
    let retval_it = (nonce..nonce + N as u16).map(|nonce| {
        ctr.reset(nonce);
        block_buf_opt = None;

        let polynomial_it = sample_polynomial(0, 14, |_| match block_buf_opt {
            None => {
                let block_buf: [u8; 1] = ctr.squeeze();
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

pub fn expand_y(mut ctr: impl Counter, nonce: u16) -> Vector<PlainPolynomial, L> {
    let mut block_buf = [0; 3];

    // For each of the `L` coefficients of our return value, we generate a polynomial with
    // rejection sampling
    let retval_it = (L as u16 * nonce..L as u16 * (nonce + 1)).map(|nonce| {
        ctr.reset(nonce);

        let polynomial_it = sample_polynomial(1 - GAMMA1, GAMMA1, |i| {
            let k = 4 * (i % 2);

            if k == 0 {
                block_buf = ctr.squeeze();
            } else {
                block_buf[0] = block_buf[2];
                *subarr_mut!(block_buf[1..3]) = ctr.squeeze();
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
