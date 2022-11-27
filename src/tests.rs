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
            .zip(expand_a(&fixture.seed).iter().flatten())
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
            .zip(expand_s(&fixture.seed).iter())
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
