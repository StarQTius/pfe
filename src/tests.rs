use crate::*;
use pretty_assertions::assert_eq;

mod fixtures;

#[test]
fn test_expand_a() {
    let fixtures = fixtures::fixtures();

    for fixture in fixtures {
        for (i, (expected, result)) in fixture
            .a
            .iter()
            .flatten()
            .zip(
                expand_a(fixture.seed[..SEED_SIZE / 2].try_into().unwrap())
                    .iter()
                    .flatten(),
            )
            .enumerate()
        {
            assert_eq!(expected, result, "{}", i);
        }
    }
}

#[test]
fn test_expand_s() {
    let fixtures = fixtures::fixtures();

    for (i, fixture) in fixtures.iter().enumerate() {
        for (j, (lpoly, rpoly)) in fixture
            .s
            .iter()
            .zip(expand_s::<L>(&fixture.seed, 0).iter())
            .enumerate()
        {
            assert_eq!(lpoly, rpoly, "{} -- {}", i, j);
        }
    }
}

#[test]
fn test_expand_y() {
    let fixtures = fixtures::fixtures();

    for (i, fixture) in fixtures.iter().enumerate() {
        for (j, (lpoly, rpoly)) in fixture
            .y
            .iter()
            .zip(expand_y(&fixture.seed).iter())
            .enumerate()
        {
            assert_eq!(lpoly, rpoly, "{} -- {}", i, j);
        }
    }
}

#[test]
fn test_make_w_and_t_vecs() {
    let fixtures = fixtures::fixtures();

    for (i, fixture) in fixtures.iter().enumerate() {
        let (w0, w1, t0, t1) = make_w_and_t_vecs(&fixture.a, fixture.y);
        for (j, (lpoly, rpoly)) in fixture.w0.iter().zip(w0.iter()).enumerate() {
            assert_eq!(lpoly, rpoly, "w0 {} -- {}", i, j);
        }
        for (j, (lpoly, rpoly)) in fixture.w1.iter().zip(w1.iter()).enumerate() {
            assert_eq!(lpoly, rpoly, "w1 {} -- {}", i, j);
        }
        for (j, (lpoly, rpoly)) in fixture.t0.iter().zip(t0.iter()).enumerate() {
            assert_eq!(lpoly, rpoly, "t0 {} -- {}", i, j);
        }
        for (j, (lpoly, rpoly)) in fixture.t1.iter().zip(t1.iter()).enumerate() {
            assert_eq!(lpoly, rpoly, "t1 {} -- {}", i, j);
        }
    }
}

#[test]
fn test_make_challenge() {
    let fixtures = fixtures::fixtures();

    for (i, fixture) in fixtures.iter().enumerate() {
        assert_eq!(fixture.c, make_challenge(&fixture.seed), "{}", i);
    }
}

#[test]
fn test_make_keys() {
    let fixtures = fixtures::fixtures();
    let mut hasher_128 = sha3::Sha3::shake_128();
    let mut hasher_256 = sha3::Sha3::shake_256();

    for (i, fixture) in fixtures.iter().enumerate() {
        let mut byte_buf = [0; SEED_SIZE];

        hasher_128.reset();
        hasher_128.input(&((i * 3 + 1) as u64).to_le_bytes());
        hasher_128.result(&mut byte_buf[..SEED_SIZE / 2]);

        let (pk, sk) = make_keys(byte_buf.into_iter()).unwrap();
        let mut pk_hash = [0; 32];
        let mut sk_hash = [0; 32];

        hasher_256.reset();
        hasher_256.input(&pk);
        hasher_256.result(&mut pk_hash);

        assert_eq!(fixture.pk, pk_hash);

        hasher_256.reset();
        hasher_256.input(&sk);
        hasher_256.result(&mut sk_hash);

        assert_eq!(fixture.sk, sk_hash);
    }
}
