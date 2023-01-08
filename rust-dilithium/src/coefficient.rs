use crate::{D, GAMMA2, Q, Q_MOD_2POW32_INVERSE};
use std::num::Wrapping;

pub type Coefficient = i32;

pub fn reduce_montgomery(coeff: i64) -> Coefficient {
    const Q_64: i64 = Q as i64;
    const Q_MOD_2POW32_INVERSE_64: i64 = Q_MOD_2POW32_INVERSE as i64;

    let tmp = (Wrapping(coeff) * Wrapping(Q_MOD_2POW32_INVERSE_64)).0 as Coefficient;
    ((coeff - tmp as i64 * Q_64) >> 32) as Coefficient
}

pub fn reduce_32(n: Coefficient) -> Coefficient {
    n - ((n + (1 << 22)) >> 23) * Q
}

pub fn caddq(n: Coefficient) -> Coefficient {
    if n >= 0 {
        n
    } else {
        n + Q
    }
}

pub fn decompose(n: Coefficient) -> (Coefficient, Coefficient) {
    let n1 = (n + 127) >> 7;
    let mut n1 = (n1 * 1025 + (1 << 21)) >> 22;
    n1 &= 15;

    let mut n0 = n - n1 * 2 * GAMMA2;
    n0 -= (((Q - 1) / 2 - n0) >> 31) & Q;

    (n0, n1)
}

pub fn power2round(coeff: Coefficient) -> (Coefficient, Coefficient) {
    let a1 = (coeff + (1 << (D - 1)) - 1) >> D;
    (coeff - (a1 << D), a1)
}

pub fn abs(n: Coefficient) -> Coefficient {
    n - ((n >> 31) & (2 * n))
}
