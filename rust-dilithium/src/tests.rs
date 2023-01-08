use crate::*;
use sha3::{
    digest::{ExtendableOutputReset, Update},
    Shake128, Shake256,
};

mod fixtures;

#[test]
fn test_expand_a() {
    let fixtures = fixtures::fixtures();

    for fixture in fixtures {
        let result = expand::expand_a(fixture.half_seed());
        assert!(result == fixture.a);
    }
}

#[test]
fn test_expand_s() {
    let fixtures = fixtures::fixtures();
    for fixture in fixtures {
        let result = expand::expand_s(&fixture.seed, 0);
        assert!(result == fixture.s);
    }
}

#[test]
fn test_expand_y() {
    let fixtures = fixtures::fixtures();
    for fixture in fixtures {
        let result = expand::expand_y(&fixture.seed, 0);
        assert!(result == fixture.y);
    }
}

#[test]
fn test_make_w_and_t_vecs() {
    let fixtures = fixtures::fixtures();

    for fixture in fixtures {
        let w = make_w(&fixture.a, &fixture.y.clone().into_ntt());
        let (w0, w1) = w.clone().decompose();
        let (t0, t1) = w.power2round();

        assert!(w0 == fixture.w0);
        assert!(w1 == fixture.w1);
        assert!(t0 == fixture.t0);
        assert!(t1 == fixture.t1);
    }
}

#[test]
fn test_make_challenge() {
    let fixtures = fixtures::fixtures();

    for fixture in fixtures {
        let challenge = make_challenge(fixture.half_seed());
        assert!(challenge == fixture.c);
    }
}

#[test]
fn test_make_keys() {
    let fixtures = fixtures::fixtures();
    let mut hasher_128 = Shake128::default();
    let mut hasher_256 = Shake256::default();

    for (i, fixture) in fixtures.iter().enumerate() {
        let mut byte_buf = [0; SEED_SIZE];

        hasher_128.update(&((i * 3 + 1) as u64).to_le_bytes());

        let mut reader_128 = hasher_128.finalize_xof_reset();
        reader_128.read(&mut byte_buf[..SEED_SIZE / 2]).unwrap();

        let (pk, sk) = make_keys(byte_buf.clone().into_iter()).unwrap();
        let mut pk_hash = [0; 32];
        let mut sk_hash = [0; 32];

        hasher_256.update(&pk);

        let mut reader_256 = hasher_256.finalize_xof_reset();
        reader_256.read(&mut pk_hash).unwrap();

        assert!(fixture.pk == pk_hash);

        hasher_256.update(&sk);

        let mut reader_256 = hasher_256.finalize_xof_reset();
        reader_256.read(&mut sk_hash).unwrap();

        assert!(fixture.sk == sk_hash);

        // Not tested separately because fixtures only provide hashes of public and secret keys, so
        // we reuse the keys we generated above

        let signature = sign(&fixture.m, &sk);

        let mut signature_hash = [0; 32];

        hasher_256.update(&signature);

        let mut reader_256 = hasher_256.finalize_xof_reset();
        reader_256.read(&mut signature_hash).unwrap();

        assert!(signature_hash == fixture.sig);
        assert!(verify(&fixture.m, &signature, &pk));
    }
}
