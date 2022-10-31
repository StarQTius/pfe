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
        assert_eq!(fixture.s, expand_s(&fixture.seed), "{}", i);
    }
}

#[test]
fn test_expand_y() {
    let fixtures = fixtures::fixtures();

    for (i, fixture) in fixtures.iter().enumerate() {
        assert_eq!(fixture.y, expand_y(&fixture.seed), "{}", i);
    }
}
