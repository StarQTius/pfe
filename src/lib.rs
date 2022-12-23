use crypto::{digest::Digest, sha3};
use itertools::{iproduct, Itertools};
use std::{
    array::from_fn,
    iter::{once, repeat, Iterator},
    mem::{size_of, MaybeUninit},
    num::Wrapping,
};

#[cfg(test)]
mod tests;

pub type PolynomialCoeff = i64;
pub type Polynomial = [PolynomialCoeff; POLYNOMIAL_DEGREE];
pub type PublicKey = [u8; PUBLIC_KEY_SIZE];
pub type SecretKey = [u8; SECRET_KEY_SIZE];
pub type Signature = [u8; SIGNATURE_SIZE];

pub const K: usize = 8;
pub const L: usize = 7;
pub const SEED_SIZE: usize = 64;

const PUBLIC_KEY_SIZE: usize = SEED_SIZE / 2 + K as usize * T1_PACKED_SIZE;
const SECRET_KEY_SIZE: usize = 3 * SEED_SIZE / 2
    + L as usize * ETA_PACKED_SIZE
    + K as usize * ETA_PACKED_SIZE
    + K * T0_PACKED_SIZE;
const OMEGA: usize = 75;
const BETA: usize = 120;
const ETA_PACKED_SIZE: usize = 96;
const T0_PACKED_SIZE: usize = 416;
const T1_PACKED_SIZE: usize = 320;
const POLYZ_PACKED_SIZE: usize = 640;
const POLYVECH_PACKED_SIZE: usize = OMEGA + K;
const Q: PolynomialCoeff = 8380417;
const Q_MOD_2POW32_INVERSE: i32 = 58728449;
const ETA: PolynomialCoeff = 2;
const POLYNOMIAL_DEGREE: usize = 256;
const SAMPLE_INTEGER_SIZE: usize = 3;
const GAMMA1: PolynomialCoeff = 1 << 19;
const GAMMA2: PolynomialCoeff = (Q - 1) / 32;
const D: PolynomialCoeff = 13;
const TAU: usize = 60;
const SIGNATURE_SIZE: usize = SEED_SIZE / 2 + L * POLYZ_PACKED_SIZE + POLYVECH_PACKED_SIZE;

const ZETAS: [i32; POLYNOMIAL_DEGREE] = [
    0, 25847, -2608894, -518909, 237124, -777960, -876248, 466468, 1826347, 2353451, -359251,
    -2091905, 3119733, -2884855, 3111497, 2680103, 2725464, 1024112, -1079900, 3585928, -549488,
    -1119584, 2619752, -2108549, -2118186, -3859737, -1399561, -3277672, 1757237, -19422, 4010497,
    280005, 2706023, 95776, 3077325, 3530437, -1661693, -3592148, -2537516, 3915439, -3861115,
    -3043716, 3574422, -2867647, 3539968, -300467, 2348700, -539299, -1699267, -1643818, 3505694,
    -3821735, 3507263, -2140649, -1600420, 3699596, 811944, 531354, 954230, 3881043, 3900724,
    -2556880, 2071892, -2797779, -3930395, -1528703, -3677745, -3041255, -1452451, 3475950,
    2176455, -1585221, -1257611, 1939314, -4083598, -1000202, -3190144, -3157330, -3632928, 126922,
    3412210, -983419, 2147896, 2715295, -2967645, -3693493, -411027, -2477047, -671102, -1228525,
    -22981, -1308169, -381987, 1349076, 1852771, -1430430, -3343383, 264944, 508951, 3097992,
    44288, -1100098, 904516, 3958618, -3724342, -8578, 1653064, -3249728, 2389356, -210977, 759969,
    -1316856, 189548, -3553272, 3159746, -1851402, -2409325, -177440, 1315589, 1341330, 1285669,
    -1584928, -812732, -1439742, -3019102, -3881060, -3628969, 3839961, 2091667, 3407706, 2316500,
    3817976, -3342478, 2244091, -2446433, -3562462, 266997, 2434439, -1235728, 3513181, -3520352,
    -3759364, -1197226, -3193378, 900702, 1859098, 909542, 819034, 495491, -1613174, -43260,
    -522500, -655327, -3122442, 2031748, 3207046, -3556995, -525098, -768622, -3595838, 342297,
    286988, -2437823, 4108315, 3437287, -3342277, 1735879, 203044, 2842341, 2691481, -2590150,
    1265009, 4055324, 1247620, 2486353, 1595974, -3767016, 1250494, 2635921, -3548272, -2994039,
    1869119, 1903435, -1050970, -1333058, 1237275, -3318210, -1430225, -451100, 1312455, 3306115,
    -1962642, -1279661, 1917081, -2546312, -1374803, 1500165, 777191, 2235880, 3406031, -542412,
    -2831860, -1671176, -1846953, -2584293, -3724270, 594136, -3776993, -2013608, 2432395, 2454455,
    -164721, 1957272, 3369112, 185531, -1207385, -3183426, 162844, 1616392, 3014001, 810149,
    1652634, -3694233, -1799107, -3038916, 3523897, 3866901, 269760, 2213111, -975884, 1717735,
    472078, -426683, 1723600, -1803090, 1910376, -1667432, -1104333, -260646, -3833893, -2939036,
    -2235985, -420899, -2286327, 183443, -976891, 1612842, -3545687, -554416, 3919660, -48306,
    -1362209, 3937738, 1400424, -846154, 1976782,
];

pub fn make_keys(mut byte_stream: impl Iterator<Item = u8>) -> Option<(PublicKey, SecretKey)> {
    let mut hasher = sha3::Sha3::shake_256();
    let mut rho = [0u8; SEED_SIZE / 2];
    let mut rho_prime = [0u8; SEED_SIZE];
    let mut key = [0u8; SEED_SIZE / 2];

    let seed: [u8; SEED_SIZE / 2] = try_from_fn(|_| byte_stream.next())?;
    hasher.input(&seed);
    hasher.result(&mut rho);
    hasher.result(&mut rho_prime);
    hasher.result(&mut key);

    let a = expand_a(&rho);
    let s1 = expand_s::<L>(&rho_prime, 0);
    let s2 = expand_s::<K>(&rho_prime, L as u16);
    let mut s1_hat = s1;

    s1_hat.iter_mut().for_each(to_ntt);

    let mut t1 = ntt_matrix_product(&a, &s1_hat);

    for (t1_poly, s2_poly) in t1.iter_mut().zip(s2.iter()) {
        for coeff in t1_poly.iter_mut() {
            *coeff = reduce_32(*coeff);
        }

        from_ntt(t1_poly);

        for (t1_coeff, s2_coeff) in t1_poly.iter_mut().zip(s2_poly.iter()) {
            *t1_coeff += s2_coeff;
        }

        caddq(t1_poly);
    }

    let (t0, t1): (Vec<_>, Vec<_>) = t1.iter().map(power2round_poly).unzip();
    let (t0, t1): ([Polynomial; K], [Polynomial; K]) =
        (t0.try_into().unwrap(), t1.try_into().unwrap());

    let pk = make_public_key(&rho, &t1);

    let mut tr = [0; SEED_SIZE / 2];
    hasher.reset();
    hasher.input(&pk);
    hasher.result(&mut tr);

    let sk = make_private_key(&rho, &tr, &key, &t0, &s1, &s2)?;

    Some((pk, sk))
}

fn try_from_fn<T, const N: usize>(mut f: impl FnMut(usize) -> Option<T>) -> Option<[T; N]>
where
    T: Copy + Sized,
{
    let mut retval = [MaybeUninit::uninit(); N];

    for (i, val) in retval.iter_mut().enumerate() {
        val.write(f(i)?);
    }

    unsafe { Some(from_fn(|i| retval[i].assume_init())) }
}

fn make_private_key(
    &rho: &[u8; SEED_SIZE / 2],
    &tr: &[u8; SEED_SIZE / 2],
    &key: &[u8; SEED_SIZE / 2],
    t0: &[Polynomial; K],
    s1: &[Polynomial; L],
    s2: &[Polynomial; K],
) -> Option<SecretKey> {
    let mut it = rho
        .into_iter()
        .chain(key)
        .chain(tr)
        .chain(s1.iter().flat_map(pack_eta_polynomial))
        .chain(s2.iter().flat_map(pack_eta_polynomial))
        .chain(t0.iter().flat_map(pack_t0_polynomial));

    try_from_fn(|_| it.next())
}

fn pack_t0_polynomial(poly: &Polynomial) -> impl Iterator<Item = u8> + '_ {
    poly.chunks(8).flat_map(|chunk| {
        let tmp: [_; 8] = std::array::from_fn(|i| ((1 << (D - 1)) - chunk[i]) as u32);
        [
            tmp[0] as u8,
            (tmp[0] >> 8) as u8 | (tmp[1] << 5) as u8,
            (tmp[1] >> 3) as u8,
            (tmp[1] >> 11) as u8 | (tmp[2] << 2) as u8,
            (tmp[2] >> 6) as u8 | (tmp[3] << 7) as u8,
            (tmp[3] >> 1) as u8,
            (tmp[3] >> 9) as u8 | (tmp[4] << 4) as u8,
            (tmp[4] >> 4) as u8,
            (tmp[4] >> 12) as u8 | (tmp[5] << 1) as u8,
            (tmp[5] >> 7) as u8 | (tmp[6] << 6) as u8,
            (tmp[6] >> 2) as u8,
            (tmp[6] >> 10) as u8 | (tmp[7] << 3) as u8,
            (tmp[7] >> 5) as u8,
        ]
    })
}

fn pack_eta_polynomial(poly: &Polynomial) -> impl Iterator<Item = u8> + '_ {
    poly.chunks(8).flat_map(|chunk| {
        let tmp: [_; 8] = std::array::from_fn(|i| (ETA - chunk[i]) as u32);
        [
            tmp[0] as u8 | (tmp[1] << 3) as u8 | (tmp[2] << 6) as u8,
            (tmp[2] >> 2) as u8 | (tmp[3] << 1) as u8 | (tmp[4] << 4) as u8 | (tmp[5] << 7) as u8,
            (tmp[5] >> 1) as u8 | (tmp[6] << 2) as u8 | (tmp[7] << 5) as u8,
        ]
    })
}

fn make_public_key(rho: &[u8; SEED_SIZE / 2], t1: &[Polynomial; K]) -> [u8; PUBLIC_KEY_SIZE] {
    let mut retval = [0; PUBLIC_KEY_SIZE];

    retval[..SEED_SIZE / 2].copy_from_slice(rho);

    let t1_it = t1.iter().flat_map(|poly| poly.chunks(4));
    let retval_it = retval[SEED_SIZE / 2..].chunks_mut(5);

    for (t1_chunk, retval_chunk) in t1_it.zip(retval_it) {
        retval_chunk[0] = t1_chunk[0] as u8;
        retval_chunk[1] = ((t1_chunk[0] >> 8) | (t1_chunk[1] << 2)) as u8;
        retval_chunk[2] = ((t1_chunk[1] >> 6) | (t1_chunk[2] << 4)) as u8;
        retval_chunk[3] = ((t1_chunk[2] >> 4) | (t1_chunk[3] << 6)) as u8;
        retval_chunk[4] = (t1_chunk[3] >> 2) as u8;
    }

    retval
}

fn ntt_matrix_product(a: &[[Polynomial; L]; K], u: &[Polynomial; L]) -> [Polynomial; K] {
    a.iter()
        .map(|line| ntt_dot_product(line, u))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn ntt_dot_product(lvec: &[Polynomial; L], rvec: &[Polynomial; L]) -> Polynomial {
    lvec.iter()
        .zip(rvec.iter())
        .map(|(lpoly, rpoly)| ntt_product(lpoly, rpoly))
        .fold([0; POLYNOMIAL_DEGREE], ntt_sum)
}

pub fn expand_a(seed: &[u8]) -> [[Polynomial; L]; K] {
    let mut hasher = sha3::Sha3::shake_128();
    let mut block_buf = [0; size_of::<PolynomialCoeff>()];

    iproduct!(0..K as u16, 0..L as u16)
        .map(|(i, j)| {
            hasher.reset();
            hasher.input(seed);
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
        .collect::<Vec<[Polynomial; L]>>()
        // `group_by()` should have resulted in an iterator of count `K`
        .try_into()
        .unwrap()
}

pub fn expand_s<const N: usize>(seed: &[u8; SEED_SIZE], nonce: u16) -> [Polynomial; N] {
    let mut hasher = sha3::Sha3::shake_256();
    let mut block_buf_opt: Option<[u8; 1]> = None;

    (nonce..nonce + N as u16)
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

pub fn expand_y(seed: &[u8; SEED_SIZE], nonce: u16) -> [Polynomial; L] {
    let mut hasher = sha3::Sha3::shake_256();
    let mut block_buf = [0; 3];

    (L as u16 * nonce..L as u16 * (nonce + 1))
        .map(|nonce| {
            hasher.reset();
            hasher.input(&seed[..SEED_SIZE]);
            hasher.input(&nonce.to_le_bytes());

            expand_polynomial(1 - GAMMA1, GAMMA1, |i| {
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
                GAMMA1 - coeff
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

pub fn to_ntt(poly: &mut Polynomial) {
    let poly_len = poly.len();
    let mut zeta_it = ZETAS[1..].iter();

    for chunk_size in (0..8).map(|n| poly_len >> (n + 1)) {
        let double_chunks_it = poly.chunks_mut(2 * chunk_size);
        for double_chunks in double_chunks_it {
            let zeta = *zeta_it.next().unwrap() as i64;
            let (lchunk, rchunk) = double_chunks.split_at_mut(chunk_size);
            for (lx, rx) in lchunk.iter_mut().zip(rchunk) {
                let tmp = reduce_montgomery(*rx * zeta);
                *rx = *lx - tmp;
                *lx += tmp;
            }
        }
    }
}

pub fn from_ntt(ntt_poly: &mut Polynomial) {
    let f = 41978;
    let mut zeta_it = ZETAS.iter().rev();

    for chunk_size in (0..8).map(|n| 1 << n) {
        let double_chunks_it = ntt_poly.chunks_mut(2 * chunk_size);
        for double_chunks in double_chunks_it {
            let zeta = *zeta_it.next().unwrap() as i64;
            let (lchunk, rchunk) = double_chunks.split_at_mut(chunk_size);
            for (lx, rx) in lchunk.iter_mut().zip(rchunk) {
                let tmp = *lx;
                *lx = tmp + *rx;
                *rx = tmp - *rx;
                *rx = reduce_montgomery(*rx * -zeta);
            }
        }
    }

    for coeff in ntt_poly {
        *coeff = reduce_montgomery(*coeff * f);
    }
}

pub fn caddq(poly: &mut Polynomial) {
    for neg_val in poly.iter_mut().filter(|&&mut val| val < 0) {
        *neg_val += Q;
    }
}

pub fn ntt_product(lpoly: &Polynomial, rpoly: &Polynomial) -> Polynomial {
    lpoly
        .iter()
        .zip(rpoly)
        .map(|(lhs, rhs)| reduce_montgomery(lhs * rhs))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

pub fn ntt_sum(lpoly: Polynomial, rpoly: Polynomial) -> Polynomial {
    lpoly
        .iter()
        .zip(rpoly)
        .map(|(lhs, rhs)| lhs + rhs)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

pub fn make_w_and_t_vecs(
    a: &[[Polynomial; L]; K],
    mut y: [Polynomial; L],
) -> (
    [Polynomial; K],
    [Polynomial; K],
    [Polynomial; K],
    [Polynomial; K],
) {
    y.iter_mut().for_each(to_ntt);

    let mut w: Vec<_> = a
        .iter()
        .map(|line_vec| {
            line_vec
                .iter()
                .zip(y)
                .map(|(ntt_lpoly, ntt_rpoly)| ntt_product(ntt_lpoly, &ntt_rpoly))
                .fold([0 as PolynomialCoeff; 256], ntt_sum)
                .map(reduce_32)
        })
        .collect();

    w.iter_mut().for_each(from_ntt);
    w.iter_mut().for_each(caddq);

    let (w0, w1): (Vec<_>, Vec<_>) = w.iter().map(decompose_poly).unzip();
    let (t0, t1): (Vec<_>, Vec<_>) = w.iter().map(power2round_poly).unzip();

    (
        w0.try_into().unwrap(),
        w1.try_into().unwrap(),
        t0.try_into().unwrap(),
        t1.try_into().unwrap(),
    )
}

fn reduce_montgomery(n: PolynomialCoeff) -> PolynomialCoeff {
    let wrapped_n = Wrapping(n as i32);
    let tmp = (wrapped_n * Wrapping(Q_MOD_2POW32_INVERSE)).0 as i64;
    (n - tmp * Q) >> 32
}

fn reduce_32(n: PolynomialCoeff) -> PolynomialCoeff {
    n - ((n + (1 << 22)) >> 23) * Q
}

fn decompose_poly(poly: &Polynomial) -> (Polynomial, Polynomial) {
    let (vec1, vec2): (Vec<_>, Vec<_>) = poly.iter().map(decompose_coeff).unzip();
    (vec1.try_into().unwrap(), vec2.try_into().unwrap())
}

fn decompose_coeff(a: &PolynomialCoeff) -> (PolynomialCoeff, PolynomialCoeff) {
    let a1 = (a + 127) >> 7;
    let mut a1 = (a1 * 1025 + (1 << 21)) >> 22;
    a1 &= 15;

    let mut a0 = a - a1 * 2 * GAMMA2;
    a0 -= (((Q - 1) / 2 - a0) >> 31) & Q;

    (a0, a1)
}

fn power2round_poly(poly: &Polynomial) -> (Polynomial, Polynomial) {
    let (poly0, poly1): (Vec<_>, Vec<_>) = poly.iter().map(power2round).unzip();
    (poly0.try_into().unwrap(), poly1.try_into().unwrap())
}

fn power2round(coeff: &PolynomialCoeff) -> (PolynomialCoeff, PolynomialCoeff) {
    let a1 = (coeff + (1 << (D - 1)) - 1) >> D;
    (coeff - (a1 << D), a1)
}

pub fn make_challenge(seed: &[u8]) -> Polynomial {
    const FIRST_TAU_BITS_MASK: u64 = (1 << TAU) - 1;

    let mut hasher = sha3::Sha3::shake_256();
    let mut sign_bits_buf = [0u8; size_of::<u64>()];
    let mut retval: Polynomial = [0; POLYNOMIAL_DEGREE];

    hasher.input(seed);
    hasher.result(&mut sign_bits_buf);

    let mut sign_bits = u64::from_le_bytes(sign_bits_buf) & FIRST_TAU_BITS_MASK;

    for last_bit_index in POLYNOMIAL_DEGREE - TAU..POLYNOMIAL_DEGREE {
        let chosen_bit_index = (0..)
            .map(|_| {
                let mut buf = [0u8; 1];
                hasher.result(&mut buf);
                buf[0] as usize
            })
            .find(|&bit_index| bit_index <= last_bit_index)
            // We can unwrap safely since the iterator never ends
            .unwrap();
        retval[last_bit_index] = retval[chosen_bit_index];
        retval[chosen_bit_index] = 1 - 2 * (sign_bits & 1) as PolynomialCoeff;
        sign_bits >>= 1;
    }

    retval
}

pub fn sign(msg: &[u8], sk: &SecretKey) -> Signature {
    let [rho, key, tr, s1, s2, t0] = split_by_sizes(
        sk,
        &[
            SEED_SIZE / 2,
            SEED_SIZE / 2,
            SEED_SIZE / 2,
            L * ETA_PACKED_SIZE,
            K * ETA_PACKED_SIZE,
            K * T0_PACKED_SIZE,
        ],
    );

    // Should not panic as long as slices are of correct sizes
    let mut s1: [_; L] = extract_vector(s1, unpack_eta_polynomial).unwrap();
    let mut s2: [_; K] = extract_vector(s2, unpack_eta_polynomial).unwrap();
    let mut t0: [_; K] = extract_vector(t0, unpack_t0_polynomial).unwrap();

    let mut hasher = sha3::Sha3::shake_256();
    hasher.input(tr);
    hasher.input(msg);

    let mut mu = [0u8; SEED_SIZE];
    hasher.result(&mut mu);

    let mut rho_prime = [0u8; SEED_SIZE];
    hasher.reset();
    hasher.input(key);
    hasher.input(&mu);
    hasher.result(&mut rho_prime);

    let a = expand_a(rho);
    s1.iter_mut().for_each(to_ntt);
    s2.iter_mut().for_each(to_ntt);
    t0.iter_mut().for_each(to_ntt);

    let sample_signature = |nonce| {
        let y = expand_y(&rho_prime, nonce);

        let mut z = y;
        z.iter_mut().for_each(to_ntt);

        let mut w = ntt_matrix_product(&a, &z);
        w.iter_mut().for_each(|poly| {
            for coeff in poly.iter_mut() {
                *coeff = reduce_32(*coeff);
            }
            from_ntt(poly);
            caddq(poly);
        });

        let (w0, w1): (Vec<_>, Vec<_>) = w.iter().map(decompose_poly).unzip();
        let mut w0: [Polynomial; K] = w0.try_into().unwrap();
        let w1: [Polynomial; K] = w1.try_into().unwrap();
        let mut challenge_seed: [u8; SEED_SIZE / 2] = from_fn(|_| 0);

        // Should not panic since `decompose_poly` should have returned a sufficiently long `Vec`
        let packed_w1: [u8; POLYNOMIAL_DEGREE * K / 2] = try_from_it(
            w1.iter()
                .flat_map(|poly| poly.chunks(2).map(|slice| (slice[0] | slice[1] << 4) as u8)),
        )
        .unwrap();

        hasher.reset();
        hasher.input(&mu);
        hasher.input(&packed_w1);
        hasher.result(&mut challenge_seed);

        let mut challenge = make_challenge(&challenge_seed);
        to_ntt(&mut challenge);

        let mut z: [_; L] =
            try_from_it(s1.iter().map(|poly| ntt_product(&challenge, poly))).unwrap();
        z.iter_mut().for_each(from_ntt);
        z.iter_mut()
            .zip(y.iter())
            .for_each(|(lpoly, &rpoly)| *lpoly = ntt_sum(*lpoly, rpoly).map(reduce_32));
        if !z
            .iter()
            .flat_map(|poly| poly.iter())
            .all(|&coeff| abs(coeff) < GAMMA1 - BETA as PolynomialCoeff)
        {
            None?
        }

        let mut h: [Polynomial; K] =
            try_from_it(s2.iter().map(|poly| ntt_product(&challenge, poly))).unwrap();
        h.iter_mut().for_each(from_ntt);
        w0.iter_mut()
            .zip(h.iter())
            .for_each(|(lpoly, &rpoly)| *lpoly = ntt_difference(*lpoly, rpoly).map(reduce_32));
        if !w0
            .iter()
            .flat_map(|poly| poly.iter())
            .all(|&coeff| abs(coeff) < GAMMA2 - BETA as PolynomialCoeff)
        {
            None?
        }

        let mut h: [Polynomial; K] =
            try_from_it(t0.iter().map(|poly| ntt_product(&challenge, poly))).unwrap();
        h.iter_mut().for_each(from_ntt);
        h.iter_mut()
            .for_each(|poly| poly.iter_mut().for_each(|coeff| *coeff = reduce_32(*coeff)));
        if !h
            .iter()
            .flat_map(|poly| poly.iter())
            .all(|&coeff| abs(coeff) < GAMMA2)
        {
            None?
        }

        w0.iter_mut()
            .zip(h.iter())
            .for_each(|(lpoly, &rpoly)| *lpoly = ntt_sum(*lpoly, rpoly));
        let (h, bits_count) = make_hint(&w0, &w1);
        if bits_count > OMEGA {
            None?
        }

        Some(make_signature(challenge_seed.into_iter(), &z, &h))
    };

    // Will not panic since input iterator is infinite
    (0..).find_map(sample_signature).unwrap()
}

fn make_signature(
    challenge_seed_it: impl Iterator<Item = u8>,
    z: &[Polynomial; L],
    hint: &[[bool; POLYNOMIAL_DEGREE]; K],
) -> Signature {
    let (hint_positions, hint_numbers_by_polynomial): (Vec<_>, Vec<_>) =
        hint.iter().map(pack_hint_polynomial).unzip();
    let signature_it = challenge_seed_it
        .chain(z.iter().flat_map(pack_z_polynomial))
        .chain(hint_positions.into_iter().flatten())
        .chain(repeat(0).take(OMEGA - hint_numbers_by_polynomial.iter().sum::<u8>() as usize))
        .chain(hint_numbers_by_polynomial.iter().scan(0, |acc, &n| {
            *acc += n;
            Some(*acc)
        }));

    // Should not panic ?
    try_from_it(signature_it).unwrap()
}

fn pack_hint_polynomial(poly: &[bool; POLYNOMIAL_DEGREE]) -> (impl Iterator<Item = u8> + '_, u8) {
    let (it1, it2) = poly.iter().tee();
    (
        it1.enumerate().filter(|(_, &b)| b).map(|(i, _)| i as u8),
        it2.filter(|&&b| b).count() as u8,
    )
}

fn pack_z_polynomial(poly: &Polynomial) -> impl Iterator<Item = u8> + '_ {
    poly.chunks(2).flat_map(|chunk| {
        let tmp: [_; 2] = from_fn(|i| GAMMA1 - chunk[i]);
        [
            tmp[0],
            tmp[0] >> 8,
            tmp[0] >> 16 | tmp[1] << 4,
            tmp[1] >> 4,
            tmp[1] >> 12,
        ]
        .map(|coeff| coeff as u8)
    })
}

fn make_hint(
    w0: &[Polynomial; K],
    w1: &[Polynomial; K],
) -> ([[bool; POLYNOMIAL_DEGREE]; K], usize) {
    let w0_flat_it = w0.iter().flat_map(|poly| poly.iter());
    let w1_flat_it = w1.iter().flat_map(|poly| poly.iter());
    let (hint_it, count_it) = w0_flat_it
        .zip(w1_flat_it)
        .map(|(&coeff0, &coeff1)| {
            !(-GAMMA2..=GAMMA2).contains(&coeff0) || coeff0 == -GAMMA2 && coeff1 != 0
        })
        .tee();

    (
        try_from_it(
            hint_it
                .chunks(POLYNOMIAL_DEGREE)
                .into_iter()
                .filter_map(try_from_it),
        )
        .unwrap(),
        count_it.filter(|&b| b).count(),
    )
}

pub fn ntt_difference(lpoly: Polynomial, rpoly: Polynomial) -> Polynomial {
    lpoly
        .iter()
        .zip(rpoly)
        .map(|(lhs, rhs)| lhs - rhs)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn abs(n: PolynomialCoeff) -> PolynomialCoeff {
    n - ((n >> 31) & (2 * n))
}

fn split_by_sizes<'a, const N: usize>(slice: &'a [u8], sizes: &[usize; N]) -> [&'a [u8]; N] {
    let (it1, it2) = sizes
        .iter()
        .scan(0, |sum, n| {
            *sum += n;
            Some(*sum)
        })
        .tee();

    let subslices_it = once(0)
        .chain(it1)
        .zip(it2)
        .map(|(start, end)| &slice[start..end]);

    // Should never panic since `sizes` is as large as the return value
    try_from_it(subslices_it).unwrap()
}

fn extract_vector<const N: usize>(
    sk_slice: &[u8],
    extract_poly: impl Fn(&[u8]) -> Option<Polynomial>,
) -> Option<[Polynomial; N]> {
    try_from_it(
        sk_slice
            .chunks_exact(sk_slice.len() / N)
            .filter_map(extract_poly),
    )
}

fn unpack_eta_polynomial(sk_slice: &[u8]) -> Option<Polynomial> {
    const _3BITS_MASK: PolynomialCoeff = 7;
    const SK_CHUNK_LENGTH: usize = 3;

    let it = sk_slice
        .chunks_exact(SK_CHUNK_LENGTH)
        .filter_map(|chunk| {
            let chunk: [_; SK_CHUNK_LENGTH] =
                try_from_it(chunk.iter().map(|&x| x as PolynomialCoeff))?;
            Some(
                [
                    chunk[0],
                    chunk[0] >> 3,
                    (chunk[0] >> 6) | (chunk[1] << 2),
                    chunk[1] >> 1,
                    chunk[1] >> 4,
                    (chunk[1] >> 7) | (chunk[2] << 1),
                    chunk[2] >> 2,
                    chunk[2] >> 5,
                ]
                .into_iter()
                .map(|n| ETA - (n & _3BITS_MASK)),
            )
        })
        .flatten();

    try_from_it(it)
}

fn unpack_t0_polynomial(sk_slice: &[u8]) -> Option<Polynomial> {
    const DBITS_MASK: PolynomialCoeff = 0x1fff;
    const SK_CHUNK_LENGTH: usize = 13;

    let it = sk_slice
        .chunks_exact(SK_CHUNK_LENGTH)
        .filter_map(|chunk| {
            let chunk: [_; SK_CHUNK_LENGTH] =
                try_from_it(chunk.iter().map(|&x| x as PolynomialCoeff))?;
            Some(
                [
                    chunk[0] | (chunk[1] << 8),
                    (chunk[1] >> 5) | (chunk[2] << 3) | (chunk[3] << 11),
                    (chunk[3] >> 2) | (chunk[4] << 6),
                    (chunk[4] >> 7) | (chunk[5] << 1) | (chunk[6] << 9),
                    (chunk[6] >> 4) | (chunk[7] << 4) | (chunk[8] << 12),
                    (chunk[8] >> 1) | (chunk[9] << 7),
                    (chunk[9] >> 6) | (chunk[10] << 2) | (chunk[11] << 10),
                    (chunk[11] >> 3) | (chunk[12] << 5),
                ]
                .into_iter()
                .map(|n| (DBITS_MASK / 2 + 1) - (n & DBITS_MASK)),
            )
        })
        .flatten();

    try_from_it(it)
}

fn try_from_it<T, const N: usize>(mut it: impl Iterator<Item = T>) -> Option<[T; N]>
where
    T: Copy,
{
    try_from_fn(|_| it.next())
}
