use crate::packing::Pack;
use coefficient::Coefficient;
use core::{
    array::from_fn,
    iter::{once, zip, Iterator},
    mem::{size_of, MaybeUninit},
};
use counter::Counter;
use itertools::Itertools;
use polynomial::{ntt::NTTPolynomial, plain::PlainPolynomial, NB_COEFFICIENTS};
use sha3::{
    digest::{ExtendableOutput, ExtendableOutputReset, Update, XofReader},
    Shake256,
};
use subarray::Subarray;
use vector::{Matrix, Vector};

mod array_math;
mod coefficient;
pub mod counter;
mod expand;
mod packing;
mod polynomial;
mod vector;
#[macro_use]
mod subarray;

#[cfg(test)]
mod tests;

pub type Polynomial = [coefficient::Coefficient; POLYNOMIAL_DEGREE];
pub type PublicKey = [u8; PUBLIC_KEY_SIZE];
pub type SecretKey = [u8; SECRET_KEY_SIZE];
pub type Signature = [u8; SIGNATURE_SIZE];

pub const K: usize = 8;
pub const L: usize = 7;
pub const SEED_SIZE: usize = 64;
pub const HALF_SEED_SIZE: usize = 32;
const Q: coefficient::Coefficient = 8380417;
const Q_MOD_2POW32_INVERSE: coefficient::Coefficient = 58728449;

const PUBLIC_KEY_SIZE: usize = SEED_SIZE / 2 + K as usize * T1_PACKED_SIZE;
const SECRET_KEY_SIZE: usize = 3 * SEED_SIZE / 2
    + L as usize * ETA_PACKED_SIZE
    + K as usize * ETA_PACKED_SIZE
    + K * T0_PACKED_SIZE;
const OMEGA: usize = 75;
const BETA: Coefficient = 120;
const ETA_PACKED_SIZE: usize = 96;
const T0_PACKED_SIZE: usize = 416;
const T1_PACKED_SIZE: usize = 320;
const POLYZ_PACKED_SIZE: usize = 640;
const POLYVECH_PACKED_SIZE: usize = OMEGA + K;
const ETA: coefficient::Coefficient = 2;
const POLYNOMIAL_DEGREE: usize = 256;
// const SAMPLE_INTEGER_SIZE: usize = 3;
const GAMMA1: coefficient::Coefficient = 1 << 19;
const GAMMA2: coefficient::Coefficient = (Q - 1) / 32;
const D: coefficient::Coefficient = 13;
const TAU: usize = 60;
const SIGNATURE_SIZE: usize = SEED_SIZE / 2 + L * POLYZ_PACKED_SIZE + POLYVECH_PACKED_SIZE;

pub fn make_keys<Ctr: Counter>(
    byte_stream: impl Iterator<Item = u8>,
) -> Option<(PublicKey, SecretKey)> {
    let mut hasher = Shake256::default();

    let mut rho = [0u8; SEED_SIZE / 2];
    let mut rho_prime = [0u8; SEED_SIZE];
    let mut key = [0u8; SEED_SIZE / 2];

    let seed: [u8; SEED_SIZE / 2] = byte_stream.try_collect_array()?;
    hasher.update(&seed);

    let mut reader = hasher.finalize_xof_reset();
    reader.read(&mut rho);
    reader.read(&mut rho_prime);
    reader.read(&mut key);

    let a = expand::expand_a(Ctr::new(&rho));
    let s1 = expand::expand_s::<L>(Ctr::new(subarr!(rho_prime[..HALF_SEED_SIZE])), 0);
    let s2 = expand::expand_s::<K>(Ctr::new(subarr!(rho_prime[..HALF_SEED_SIZE])), L as u16);
    let (t0, t1) = (make_w(&a, &s1.clone().into_ntt()) + s2.clone()).power2round();

    let pk = pack_public_key(&rho, t1);

    let mut tr = [0; SEED_SIZE / 2];

    hasher.update(&pk);

    let mut reader = hasher.finalize_xof_reset();
    reader.read(&mut tr);

    let sk = make_private_key(&rho, &tr, &key, t0, s1, s2)?;

    Some((pk, sk))
}

fn make_private_key(
    rho: &[u8; SEED_SIZE / 2],
    tr: &[u8; SEED_SIZE / 2],
    key: &[u8; SEED_SIZE / 2],
    t0: Vector<PlainPolynomial, K>,
    s1: Vector<PlainPolynomial, L>,
    s2: Vector<PlainPolynomial, K>,
) -> Option<SecretKey> {
    let mut retval = [0; SECRET_KEY_SIZE];

    let retval_slices = retval.partition_mut(&[
        SEED_SIZE / 2,
        SEED_SIZE / 2,
        SEED_SIZE / 2,
        L * ETA_PACKED_SIZE,
        K * ETA_PACKED_SIZE,
        K * T0_PACKED_SIZE,
    ]);

    let packed_t0: [_; K * T0_PACKED_SIZE] = t0.pack(&t0_packer);
    let packed_s1: [_; L * ETA_PACKED_SIZE] = s1.pack(&eta_packer);
    let packed_s2: [_; K * ETA_PACKED_SIZE] = s2.pack(&eta_packer);

    retval_slices[0].copy_from_slice(rho);
    retval_slices[1].copy_from_slice(key);
    retval_slices[2].copy_from_slice(tr);
    retval_slices[3].copy_from_slice(&packed_s1);
    retval_slices[4].copy_from_slice(&packed_s2);
    retval_slices[5].copy_from_slice(&packed_t0);

    Some(retval)
}

fn t0_packer(chunk: &[Coefficient; 8]) -> [u8; 13] {
    let chunk = chunk.map(|coeff| (1 << (D - 1)) - coeff);

    [
        chunk[0],
        (chunk[0] >> 8) | (chunk[1] << 5),
        (chunk[1] >> 3),
        (chunk[1] >> 11) | (chunk[2] << 2),
        (chunk[2] >> 6) | (chunk[3] << 7),
        (chunk[3] >> 1),
        (chunk[3] >> 9) | (chunk[4] << 4),
        (chunk[4] >> 4),
        (chunk[4] >> 12) | (chunk[5] << 1),
        (chunk[5] >> 7) | (chunk[6] << 6),
        (chunk[6] >> 2),
        (chunk[6] >> 10) | (chunk[7] << 3),
        (chunk[7] >> 5),
    ]
    .map(|coeff| coeff as u8)
}

fn eta_packer(chunk: &[Coefficient; 8]) -> [u8; 3] {
    let chunk = chunk.map(|coeff| ETA - coeff);

    [
        chunk[0] | (chunk[1] << 3) | (chunk[2] << 6),
        (chunk[2] >> 2) | (chunk[3] << 1) | (chunk[4] << 4) | (chunk[5] << 7),
        (chunk[5] >> 1) | (chunk[6] << 2) | (chunk[7] << 5),
    ]
    .map(|coeff| coeff as u8)
}

fn pack_public_key(rho: &[u8; SEED_SIZE / 2], t1: Vector<PlainPolynomial, K>) -> PublicKey {
    let packed_t1: [_; K * T1_PACKED_SIZE] = t1.pack(&|chunk: &[Coefficient; 4]| {
        [
            chunk[0],
            chunk[0] >> 8 | chunk[1] << 2,
            chunk[1] >> 6 | chunk[2] << 4,
            chunk[2] >> 4 | chunk[3] << 6,
            chunk[3] >> 2,
        ]
        .map(|n| n as u8)
    });

    let mut retval = [0; PUBLIC_KEY_SIZE];

    retval[..SEED_SIZE / 2].copy_from_slice(rho);
    retval[SEED_SIZE / 2..].copy_from_slice(&packed_t1);

    retval
}

fn make_w(
    a: &Matrix<NTTPolynomial, L, K>,
    y: &Vector<NTTPolynomial, L>,
) -> Vector<PlainPolynomial, K> {
    (a * y).reduce_32().into_plain().caddq()
}

pub fn make_challenge(seed: &[u8; SEED_SIZE / 2]) -> PlainPolynomial {
    const FIRST_TAU_BITS_MASK: u64 = (1 << TAU) - 1;

    let mut hasher = Shake256::default();
    let mut sign_bits_buf = [0u8; size_of::<u64>()];
    let mut retval = [0; POLYNOMIAL_DEGREE];

    hasher.update(seed);

    let mut reader = hasher.finalize_xof();
    reader.read(&mut sign_bits_buf);

    let mut sign_bits = u64::from_le_bytes(sign_bits_buf) & FIRST_TAU_BITS_MASK;

    for last_bit_index in POLYNOMIAL_DEGREE - TAU..POLYNOMIAL_DEGREE {
        let chosen_bit_index = (0..)
            .map(|_| {
                let mut buf = [0u8; 1];
                reader.read(&mut buf);
                buf[0] as usize
            })
            .find(|&bit_index| bit_index <= last_bit_index)
            // We can unwrap safely since the iterator never ends
            .unwrap();
        retval[last_bit_index] = retval[chosen_bit_index];
        retval[chosen_bit_index] = 1 - 2 * (sign_bits & 1) as coefficient::Coefficient;
        sign_bits >>= 1;
    }

    PlainPolynomial::from(retval)
}

pub fn sign<Ctr: Counter>(msg: &[u8], sk: &SecretKey) -> Signature {
    let [rho, key, tr, packed_s1, packed_s2, packed_t0] = sk.partition(&[
        SEED_SIZE / 2,
        SEED_SIZE / 2,
        SEED_SIZE / 2,
        L * ETA_PACKED_SIZE,
        K * ETA_PACKED_SIZE,
        K * T0_PACKED_SIZE,
    ]);

    let s1: Vector<PlainPolynomial, L> = Pack::unpack(packed_s1, &eta_unpacker);
    let s2: Vector<PlainPolynomial, K> = Pack::unpack(packed_s2, &eta_unpacker);
    let t0: Vector<PlainPolynomial, K> = Pack::unpack(packed_t0, &t0_unpacker);

    let s1 = s1.into_ntt();
    let s2 = s2.into_ntt();
    let t0 = t0.into_ntt();

    let mut hasher = Shake256::default();
    hasher.update(tr);
    hasher.update(msg);

    let mut mu = [0u8; SEED_SIZE];
    let mut reader = hasher.finalize_xof_reset();
    reader.read(&mut mu);

    let mut rho_prime = [0u8; SEED_SIZE];

    hasher.update(key);
    hasher.update(&mu);

    let mut reader = hasher.finalize_xof_reset();
    reader.read(&mut rho_prime);

    // Unwrapping is safe here because the slice is of the right size
    let a = expand::expand_a(Ctr::new(rho.as_ref().try_into().unwrap()));

    let sample_signature = |nonce| {
        let y = expand::expand_y(Ctr::new(subarr!(rho_prime[..HALF_SEED_SIZE])), nonce);

        let z = y.clone().into_ntt();
        let w = make_w(&a, &z);

        let (w0, w1) = w.decompose();
        let mut challenge_seed = [0u8; SEED_SIZE / 2];

        let packed_w1: [_; K * NB_COEFFICIENTS / 2] =
            w1.pack(&|chunk: &[Coefficient; 2]| [(chunk[0] | (chunk[1] << 4)) as u8]);

        hasher.update(&mu);
        hasher.update(&packed_w1);

        let mut reader = hasher.finalize_xof_reset();
        reader.read(&mut challenge_seed);

        let challenge = make_challenge(&challenge_seed).into_ntt();
        let z = ((s1.clone() * &challenge).into_plain() + y).reduce_32();
        if z.max() >= GAMMA1 - BETA {
            return None;
        }

        let h = (s2.clone() * &challenge).into_plain();
        let w0 = (w0 - h).reduce_32();
        if w0.max() >= GAMMA2 - BETA {
            return None;
        }

        let h = (t0.clone() * &challenge).into_plain().reduce_32();
        if h.max() >= GAMMA2 - BETA {
            return None;
        }

        let h = (t0.clone() * &challenge).into_plain().reduce_32();
        if h.max() >= GAMMA2 {
            return None;
        }

        let (hint, hint_bits_count) = make_hint(&(w0 + h), &w1);
        if hint_bits_count > OMEGA {
            return None;
        }

        Some(make_signature(&challenge_seed, &z, &hint))
    };

    // Will not panic since input iterator is infinite
    (0..).find_map(sample_signature).unwrap()
}

fn make_signature(
    challenge_seed: &[u8; SEED_SIZE / 2],
    z: &Vector<PlainPolynomial, L>,
    hint: &[[bool; POLYNOMIAL_DEGREE]; K],
) -> Signature {
    fn pack_hint_polynomial(
        poly: &[bool; POLYNOMIAL_DEGREE],
    ) -> (impl Iterator<Item = u8> + '_, u8) {
        let (it1, it2) = poly.iter().tee();
        (
            it1.enumerate().filter(|(_, &b)| b).map(|(i, _)| i as u8),
            it2.filter(|&&b| b).count() as u8,
        )
    }

    let mut retval = [0; SIGNATURE_SIZE];

    let (hint_positions, hint_numbers_by_polynomial): ([_; K], _) =
        unzip(hint.iter().map(pack_hint_polynomial)).unwrap();

    let [retval_challenge_seed, retval_packed_z, retval_packed_hint] =
        retval.partition_mut(&[SEED_SIZE / 2, L * POLYZ_PACKED_SIZE, POLYVECH_PACKED_SIZE]);

    retval_challenge_seed.copy_from_slice(challenge_seed);

    let packed_z: [_; L * POLYZ_PACKED_SIZE] = z.pack(&z_packer);
    retval_packed_z.copy_from_slice(&packed_z);

    for (lhs, rhs) in zip(
        retval_packed_hint.iter_mut(),
        hint_positions.into_iter().flatten(),
    ) {
        *lhs = rhs;
    }

    let accumulated_hint_numbers: [_; K] = hint_numbers_by_polynomial
        .into_iter()
        .scan(0, |acc, n| {
            *acc += n;
            Some(*acc)
        })
        .try_collect_array()
        .unwrap();

    for (lhs, rhs) in zip(
        retval_packed_hint.iter_mut().rev(),
        accumulated_hint_numbers.into_iter().rev(),
    ) {
        *lhs = rhs;
    }

    retval
}

fn unzip<T1, T2, const N: usize>(
    mut it: impl Iterator<Item = (T1, T2)>,
) -> Option<([T1; N], [T2; N])> {
    let mut retval1: [_; N] = from_fn(|_| MaybeUninit::uninit());
    let mut retval2: [_; N] = from_fn(|_| MaybeUninit::uninit());

    for (lhs1, lhs2) in zip(retval1.iter_mut(), retval2.iter_mut()) {
        let (rhs1, rhs2) = it.next()?;
        lhs1.write(rhs1);
        lhs2.write(rhs2);
    }

    unsafe {
        Some((
            retval1.as_ptr().cast::<[T1; N]>().read(),
            retval2.as_ptr().cast::<[T2; N]>().read(),
        ))
    }
}

fn z_packer(chunk: &[Coefficient; 2]) -> [u8; 5] {
    let chunk = chunk.map(|coeff| GAMMA1 - coeff);
    [
        chunk[0],
        chunk[0] >> 8,
        chunk[0] >> 16 | chunk[1] << 4,
        chunk[1] >> 4,
        chunk[1] >> 12,
    ]
    .map(|coeff| coeff as u8)
}

fn make_hint(
    w0: &Vector<PlainPolynomial, K>,
    w1: &Vector<PlainPolynomial, K>,
) -> ([[bool; POLYNOMIAL_DEGREE]; K], usize) {
    let w0_it = w0.into_iter().flatten();
    let w1_it = w1.into_iter().flatten();

    let (hint_it, count_it) = zip(w0_it, w1_it)
        .map(|(&coeff0, &coeff1)| {
            !(-GAMMA2..=GAMMA2).contains(&coeff0) || coeff0 == -GAMMA2 && coeff1 != 0
        })
        .tee();

    let hint = hint_it
        .chunks(NB_COEFFICIENTS)
        .into_iter()
        .filter_map(|it| it.try_collect_array())
        .try_collect_array()
        .unwrap();

    (hint, count_it.filter(|&b| b).count())
}

trait Partition {
    type Slice;

    fn partition<const N: usize>(self, sizes: &[usize; N]) -> [Self::Slice; N];
}

trait PartitionMut {
    type SliceMut;

    fn partition_mut<const N: usize>(self, sizes: &[usize; N]) -> [Self::SliceMut; N];
}

impl<'a> Partition for &'a [u8] {
    type Slice = &'a [u8];

    fn partition<const N: usize>(self, sizes: &[usize; N]) -> [Self::Slice; N] {
        assert!(sizes.iter().all(|&size| size > 0));
        assert!(sizes.iter().sum::<usize>() == self.len());

        let mut it = sizes.iter().chain(once(&0));
        let mut counter = *it.next().unwrap();
        let mut retval_it = self.split_inclusive(|_| {
            counter -= 1;
            if counter == 0 {
                counter = *it.next().unwrap();
                true
            } else {
                false
            }
        });

        // Should never panic since `sizes` is as large as the return value
        from_fn(|_| retval_it.next().unwrap())
    }
}

impl<'a> Partition for &'a mut [u8] {
    type Slice = &'a [u8];

    fn partition<const N: usize>(self, sizes: &[usize; N]) -> [Self::Slice; N] {
        self.partition_mut(sizes).map(|slice| &*slice)
    }
}

impl<'a> PartitionMut for &'a mut [u8] {
    type SliceMut = &'a mut [u8];

    fn partition_mut<const N: usize>(self, sizes: &[usize; N]) -> [Self::SliceMut; N] {
        assert!(sizes.iter().all(|&size| size > 0));
        assert!(sizes.iter().sum::<usize>() == self.len());

        let mut it = sizes.iter().chain(once(&0));
        let mut counter = *it.next().unwrap();
        let mut retval_it = self.split_inclusive_mut(|_| {
            counter -= 1;
            if counter == 0 {
                counter = *it.next().unwrap();
                true
            } else {
                false
            }
        });

        // Should never panic since `sizes` is as large as the return value
        from_fn(|_| retval_it.next().unwrap())
    }
}

fn eta_unpacker(chunk: &[u8; 3]) -> [Coefficient; 8] {
    const _3BITS_MASK: coefficient::Coefficient = 7;

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
    .map(|n| n as Coefficient)
    .map(|n| ETA - (n & _3BITS_MASK))
}

fn t0_unpacker(chunk: &[u8; 13]) -> [Coefficient; 8] {
    const DBITS_MASK: coefficient::Coefficient = 0x1fff;

    let chunk = chunk.map(|byte| byte as Coefficient);

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
    .map(|n| n as Coefficient)
    .map(|n| (DBITS_MASK / 2 + 1) - (n & DBITS_MASK))
}

pub fn verify<Ctr: Counter>(msg: &[u8], signature: &Signature, pk: &PublicKey) -> bool {
    let mut hasher = Shake256::default();

    let [rho, packed_t1] = pk.partition(&[SEED_SIZE / 2, K * T1_PACKED_SIZE]);
    let t1: Vector<PlainPolynomial, K> = Pack::unpack(packed_t1, &t1_unpacker);

    let [expected_challenge_seed, packed_z, packed_hint] =
        signature.partition(&[SEED_SIZE / 2, L * POLYZ_PACKED_SIZE, POLYVECH_PACKED_SIZE]);
    let z: Vector<PlainPolynomial, L> = Pack::unpack(packed_z, &z_unpacker);
    if z.max() >= GAMMA1 - BETA {
        return false;
    }

    let hint = if let Some(hint) = unpack_hint(packed_hint.try_into().unwrap()) {
        hint
    } else {
        return false;
    };

    let mut mu = [0u8; SEED_SIZE / 2];
    hasher.update(pk);

    let mut reader = hasher.finalize_xof_reset();
    reader.read(&mut mu);

    hasher.update(&mu);
    hasher.update(msg);

    let mut mu = [0u8; SEED_SIZE];
    let mut reader = hasher.finalize_xof_reset();
    reader.read(&mut mu);

    let challenge = make_challenge(expected_challenge_seed.try_into().unwrap()).into_ntt();
    let a = expand::expand_a(Ctr::new(rho.try_into().unwrap()));
    let w1 = &a * &z.into_ntt();
    let t1 = t1.shift_d().into_ntt() * &challenge;
    let w1 = (w1 - t1).reduce_32().into_plain().caddq().use_hint(&hint);

    let packed_w1: [_; POLYNOMIAL_DEGREE * K / 2] =
        w1.pack(&|slice: &[_; 2]| [(slice[0] | slice[1] << 4) as u8]);

    let mut challenge_seed = [0u8; SEED_SIZE / 2];

    hasher.update(&mu);
    hasher.update(&packed_w1);

    let mut reader = hasher.finalize_xof_reset();
    reader.read(&mut challenge_seed);

    challenge_seed == expected_challenge_seed
}

fn unpack_hint(packed_hint: &[u8; POLYVECH_PACKED_SIZE]) -> Option<[[bool; POLYNOMIAL_DEGREE]; K]> {
    let (one_indices, polynomial_indices) = packed_hint.split_at(OMEGA);
    if !(polynomial_indices.len() == K
        && polynomial_indices
            .iter()
            .all(|i| (0..OMEGA as u8).contains(i)))
    {
        return None;
    }

    let (polynomial_indices_it1, polynomial_indices_it2) =
        polynomial_indices.iter().map(|&i| i as usize).tee();
    let one_indices_in_polynomial_it = once(0)
        .chain(polynomial_indices_it1)
        .zip(polynomial_indices_it2)
        .map(|(start, end)| &one_indices[start..end]);

    let retval_it = one_indices_in_polynomial_it.filter_map(|slice| {
        if is_strictly_sorted(slice.iter()) {
            let mut poly = [false; POLYNOMIAL_DEGREE];
            slice.iter().for_each(|&i| poly[i as usize] = true);
            Some(poly)
        } else {
            None
        }
    });

    retval_it.try_collect_array()
}

fn is_strictly_sorted<'a>(mut it: impl Iterator<Item = &'a u8>) -> bool {
    let first = if let Some(&first) = it.next() {
        first
    } else {
        return true;
    };
    it.try_fold(
        first,
        |prev, &curr| if prev < curr { Some(curr) } else { None },
    )
    .is_some()
}

fn z_unpacker(chunk: &[u8; 5]) -> [Coefficient; 2] {
    const _20BITS_MASK: Coefficient = (1 << 20) - 1;

    let chunk = chunk.map(|x| x as Coefficient);
    [
        chunk[0] | (chunk[1] << 8) | (chunk[2] << 16),
        (chunk[2] >> 4) | (chunk[3] << 4) | (chunk[4] << 12),
    ]
    .map(|n| GAMMA1 - (n & _20BITS_MASK))
}

fn t1_unpacker(chunk: &[u8; 5]) -> [Coefficient; 4] {
    const _10BITS_MASK: coefficient::Coefficient = (1 << 10) - 1;

    let chunk = chunk.map(|x| x as Coefficient);
    [
        chunk[0] | (chunk[1] << 8),
        (chunk[1] >> 2) | (chunk[2] << 6),
        (chunk[2] >> 4) | (chunk[3] << 4),
        (chunk[3] >> 6) | (chunk[4] << 2),
    ]
    .map(|n| n & _10BITS_MASK)
}

fn try_from_fn<T, const N: usize>(mut f: impl FnMut(usize) -> Option<T>) -> Option<[T; N]> {
    let mut retval: [_; N] = from_fn(|_| MaybeUninit::uninit());

    for (i, val) in retval.iter_mut().enumerate() {
        val.write(f(i)?);
    }

    unsafe { Some(retval.as_ptr().cast::<[T; N]>().read()) }
}

trait TryCollectArray<T, const N: usize> {
    fn try_collect_array(self) -> Option<[T; N]>;
}

impl<T, Iter, const N: usize> TryCollectArray<T, N> for Iter
where
    Iter: Iterator<Item = T>,
{
    fn try_collect_array(mut self) -> Option<[T; N]> {
        try_from_fn(|_| self.next())
    }
}
