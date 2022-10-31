use super::*;
use pretty_assertions::assert_eq;

mod fixtures;

#[test]
fn expand() {
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
