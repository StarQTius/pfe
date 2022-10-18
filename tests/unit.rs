extern crate nom;

#[cfg(test)]

mod tests {
    use nom::{
        sequence::{pair, delimited},
        character::complete::char,
        bytes::complete::{tag, take_until, take_while, take},
        multi::{many0, separated_list0},
        IResult};

    #[derive(PartialEq, Debug)]
    struct Fixture {
        m: Vec<u8>,
        pk: Vec<u8>,
        sk: Vec<u8>,
        sig: Vec<u8>,
        seed: Vec<u8>,
        a: Vec<Vec<Vec<u32>>>,
        s: Vec<Vec<i32>>,
        y: Vec<Vec<i32>>,
        w1: Vec<Vec<i32>>,
        w0: Vec<Vec<i32>>,
        t1: Vec<Vec<i32>>,
        t0: Vec<Vec<i32>>,
        c: Vec<i8>
    }

    fn parse_fixture(s: &str) -> IResult<&str, Fixture> {
        let (s, _) = take_until("\n")(s)?;
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

        let m = parse_byte_vector(m)?.1;
        let pk = parse_byte_vector(pk)?.1;
        let sk = parse_byte_vector(sk)?.1;
        let sig = parse_byte_vector(sig)?.1;
        let seed = parse_byte_vector(seed)?.1;
        let a = parse_matrix(a)?.1;
        let s_ = parse_poly_list(s_)?.1;
        let y = parse_poly_list(y)?.1;
        let w1 = parse_poly_list(w1)?.1;
        let w0 = parse_poly_list(w0)?.1;
        let t1 = parse_poly_list(t1)?.1;
        let t0 = parse_poly_list(t0)?.1;
        let c = parse_ones_vector(c)?.1;

        Ok((s, Fixture{m, pk, sk, sig, seed, a, s : s_, y, w1, w0, t1, t0, c}))
    }

    fn parse_byte_vector(s: &str) -> IResult<&str, Vec<u8>> {
        let (s, char_vec) = many0(take(2u8))(s)?;
        let byte_vec = char_vec.iter()
                 .map(|s| u8::from_str_radix(s, 16).unwrap())
                 .collect();

        Ok((s, byte_vec))
    }
    
    fn parse_ones_vector(s: &str) -> IResult<&str, Vec<i8>> {
        let (s, char_vec) = parse_bracket_list(s)?;
        let byte_vec = char_vec.iter()
                 .map(|s| i8::from_str_radix(s, 10).unwrap())
                 .collect();

        Ok((s, byte_vec))
    }

    fn parse_matrix(s: &str) -> IResult<&str, Vec<Vec<Vec<u32>>>> {
        let (s, char_vec) = delimited(
            char('('),
            separated_list0(tag(";\n     "), parse_bracket_lists),
            char(')')
        )(s)?;

        let mat = char_vec.iter().map(|v| -> Vec<Vec<u32>> {
            v.iter().map(|v| -> Vec<u32> {
                v.iter().map(|s| u32::from_str_radix(s, 10).unwrap()).collect()
            }).collect()
        }).collect();

        Ok((s, mat))
    }

    fn parse_bracket_lists(s: &str) -> IResult<&str, Vec<Vec<&str>>> {
        separated_list0(tag(", "), parse_bracket_list)(s)
    }

    fn parse_poly_list(s: &str) -> IResult<&str, Vec<Vec<i32>>> {
        let (s, char_vec) = delimited(
            char('('),
            separated_list0(pair(tag(",\n"), take_while(is_space)), parse_bracket_list),
            char(')')
        )(s)?;

        let mat = char_vec.iter().map(|v| -> Vec<i32> {
            v.iter().map(|s| { i32::from_str_radix(s, 10).unwrap() }).collect()
        }).collect();

        Ok((s, mat))
    }

    fn parse_bracket_list(s: &str) -> IResult<&str, Vec<&str>> {
        delimited(char('['), separated_list0(char(','), take_trimmed_integer), char(']'))(s)
    }

    fn take_trimmed_integer(s: &str) -> IResult<&str, &str> {
        delimited(take_while(is_space), take_while(is_minus_or_digit), take_while(is_space))(s)
    }

    fn is_space(c: char) -> bool {
        c == ' '
    }

    fn is_minus_or_digit(c: char) -> bool {
        c.is_digit(10) || c == '-'
    }

    #[test]
    fn expand() {
        assert_eq!(parse_bracket_list("[123, -132, 0, 0]").unwrap().1, vec!["123", "-132", "0", "0"]);

        let fixture_str = std::fs::read_to_string("tests/fixtures.txt").unwrap();
        assert_eq!(parse_fixture(fixture_str.as_str()).unwrap().1, Fixture{
            m: Vec::new(),
            pk: Vec::new(),
            sk: Vec::new(),
            sig: Vec::new(),
            seed: Vec::new(),
            a: Vec::new(),
            s: Vec::new(),
            y: Vec::new(),
            w1: Vec::new(),
            w0: Vec::new(),
            t1: Vec::new(),
            t0: Vec::new(),
            c: Vec::new()
        });
    }
}
