use crate::{PolynomialCoeff, SEED_SIZE};

use nom::{
    bytes::complete::{tag, take, take_until, take_while},
    character::complete::char,
    multi::{many0, separated_list0},
    sequence::{delimited, pair},
    IResult,
};
use std::{
    fs::File,
    io::{BufReader, Read},
};

const N: u32 = 100;
const FIXTURE_TEXT_SIZE_MAX: u32 = 1000000;
static mut FIXTURES: Vec<Fixture> = Vec::new();

#[derive(PartialEq, Debug, Default)]
pub struct Fixture {
    pub count: u32,
    pub m: Vec<u8>,
    pub pk: Vec<u8>,
    pub sk: Vec<u8>,
    pub sig: Vec<u8>,
    pub seed: [u8; SEED_SIZE],
    pub a: Vec<Vec<Vec<PolynomialCoeff>>>,
    pub s: Vec<Vec<i32>>,
    pub y: Vec<Vec<i32>>,
    pub w1: Vec<Vec<i32>>,
    pub w0: Vec<Vec<i32>>,
    pub t1: Vec<Vec<i32>>,
    pub t0: Vec<Vec<i32>>,
    pub c: Vec<i8>,
}

pub fn fixtures() -> &'static Vec<Fixture> {
    unsafe { &FIXTURES }
}

#[ctor::ctor]
unsafe fn make_fixtures() {
    let mut buf = BufReader::new(File::open("rsrc/fixtures.txt").unwrap())
        .take(u64::from(N * FIXTURE_TEXT_SIZE_MAX));
    let mut s = String::new();

    buf.read_to_string(&mut s).unwrap();

    FIXTURES = (0..N)
        .scan((s.as_str(), Fixture::default()), |(s, _), _| {
            let (remainder, fixture) = parse_fixture(s).ok()?;
            *s = remainder;
            Some(fixture)
        })
        .collect();

    assert!((0..N).eq(FIXTURES.iter().map(|fixture| fixture.count)));
}

fn parse_fixture(s: &str) -> IResult<&str, Fixture> {
    let (s, _) = tag("count = ")(s)?;
    let (s, count) = take_until("\n")(s)?;
    let (s, _) = take(1u8)(s)?;

    let (s, _) = tag("m = ")(s)?;
    let (s, m) = take_until("\n")(s)?;
    let (s, _) = take(1u8)(s)?;

    let (s, _) = tag("pk = ")(s)?;
    let (s, pk) = take_until("\n")(s)?;
    let (s, _) = take(1u8)(s)?;

    let (s, _) = tag("sk = ")(s)?;
    let (s, sk) = take_until("\n")(s)?;
    let (s, _) = take(1u8)(s)?;

    let (s, _) = tag("sig = ")(s)?;
    let (s, sig) = take_until("\n")(s)?;
    let (s, _) = take(1u8)(s)?;

    let (s, _) = tag("seed = ")(s)?;
    let (s, seed) = take_until("\n")(s)?;
    let (s, _) = take(1u8)(s)?;

    let (s, _) = tag("A = ")(s)?;
    let (s, a) = take_until("\ns =")(s)?;
    let (s, _) = take(1u8)(s)?;

    let (s, _) = tag("s = ")(s)?;
    let (s, s_) = take_until("\ny =")(s)?;
    let (s, _) = take(1u8)(s)?;

    let (s, _) = tag("y = ")(s)?;
    let (s, y) = take_until("\nw1 =")(s)?;
    let (s, _) = take(1u8)(s)?;

    let (s, _) = tag("w1 = ")(s)?;
    let (s, w1) = take_until("\nw0 =")(s)?;
    let (s, _) = take(1u8)(s)?;

    let (s, _) = tag("w0 = ")(s)?;
    let (s, w0) = take_until("\nt1 =")(s)?;
    let (s, _) = take(1u8)(s)?;

    let (s, _) = tag("t1 = ")(s)?;
    let (s, t1) = take_until("\nt0 =")(s)?;
    let (s, _) = take(1u8)(s)?;

    let (s, _) = tag("t0 = ")(s)?;
    let (s, t0) = take_until("\nc =")(s)?;
    let (s, _) = take(1u8)(s)?;

    let (s, _) = tag("c = ")(s)?;
    let (s, c) = take_until("\n\n")(s)?;
    let (s, _) = take(2u8)(s)?;

    let count = u32::from_str_radix(count, 10).unwrap();
    let m = parse_byte_vector(m)?.1;
    let pk = parse_byte_vector(pk)?.1;
    let sk = parse_byte_vector(sk)?.1;
    let sig = parse_byte_vector(sig)?.1;
    let seed = parse_byte_vector(seed)?.1[..SEED_SIZE].try_into().unwrap();
    let a = parse_matrix(a)?.1;
    let s_ = parse_poly_list(s_)?.1;
    let y = parse_poly_list(y)?.1;
    let w1 = parse_poly_list(w1)?.1;
    let w0 = parse_poly_list(w0)?.1;
    let t1 = parse_poly_list(t1)?.1;
    let t0 = parse_poly_list(t0)?.1;
    let c = parse_ones_vector(c)?.1;

    Ok((
        &s,
        Fixture {
            count,
            m,
            pk,
            sk,
            sig,
            seed,
            a,
            s: s_,
            y,
            w1,
            w0,
            t1,
            t0,
            c,
        },
    ))
}

fn parse_byte_vector(s: &str) -> IResult<&str, Vec<u8>> {
    let (s, char_vec) = many0(take(2u8))(s)?;
    let byte_vec = char_vec
        .iter()
        .map(|s| u8::from_str_radix(s, 16).unwrap())
        .collect();

    Ok((s, byte_vec))
}

fn parse_ones_vector(s: &str) -> IResult<&str, Vec<i8>> {
    let (s, char_vec) = parse_bracket_list(s)?;
    let byte_vec = char_vec
        .iter()
        .map(|s| i8::from_str_radix(s, 10).unwrap())
        .collect();

    Ok((s, byte_vec))
}

fn parse_matrix(s: &str) -> IResult<&str, Vec<Vec<Vec<u32>>>> {
    let (s, char_vec) = delimited(
        char('('),
        separated_list0(tag(";\n     "), parse_bracket_lists),
        char(')'),
    )(s)?;

    let mat = char_vec
        .iter()
        .map(|v| -> Vec<Vec<u32>> {
            v.iter()
                .map(|v| -> Vec<u32> {
                    v.iter()
                        .map(|s| u32::from_str_radix(s, 10).unwrap())
                        .collect()
                })
                .collect()
        })
        .collect();

    Ok((s, mat))
}

fn parse_bracket_lists(s: &str) -> IResult<&str, Vec<Vec<&str>>> {
    separated_list0(tag(", "), parse_bracket_list)(s)
}

fn parse_poly_list(s: &str) -> IResult<&str, Vec<Vec<i32>>> {
    let (s, char_vec) = delimited(
        char('('),
        separated_list0(pair(tag(",\n"), take_while(is_space)), parse_bracket_list),
        char(')'),
    )(s)?;

    let mat = char_vec
        .iter()
        .map(|v| -> Vec<i32> {
            v.iter()
                .map(|s| i32::from_str_radix(s, 10).unwrap())
                .collect()
        })
        .collect();

    Ok((s, mat))
}

fn parse_bracket_list(s: &str) -> IResult<&str, Vec<&str>> {
    delimited(
        char('['),
        separated_list0(char(','), take_trimmed_integer),
        char(']'),
    )(s)
}

fn take_trimmed_integer(s: &str) -> IResult<&str, &str> {
    delimited(
        take_while(is_space),
        take_while(is_minus_or_digit),
        take_while(is_space),
    )(s)
}

fn is_space(c: char) -> bool {
    c == ' '
}

fn is_minus_or_digit(c: char) -> bool {
    c.is_digit(10) || c == '-'
}
