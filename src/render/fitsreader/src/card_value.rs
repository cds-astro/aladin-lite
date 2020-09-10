use nom::{
    branch::alt,
    bytes::complete::{tag, escaped, take_till, take_while},
    character::complete::{alphanumeric0, char, digit1, space0, alpha0, one_of},
    combinator::{map, value, opt},
    sequence::{delimited, preceded, tuple, pair},
    number::complete::float,
    combinator::recognize,
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum FITSKeywordValue<'a> {
    IntegerNumber(i64),
    Logical(bool),
    CharacterString(&'a str),
    FloatingPoint(f64),
    Undefined,
}

pub(crate) fn white_space0(s: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while(|s| s == b' ')(s)
}

pub(crate) fn parse_undefined(buf: &[u8]) -> IResult<&[u8], FITSKeywordValue> {
    value(FITSKeywordValue::Undefined, white_space0)(buf)
}

pub(crate) fn parse_character_string(buf: &[u8]) -> IResult<&[u8], FITSKeywordValue> {
    map(
        preceded(
            space0,
            delimited(
                char('\''),
                take_till(|c| c == b'\''),
                char('\'')
            )
        ),
        |str: &[u8]| {
            let str = std::str::from_utf8(str).unwrap();
            FITSKeywordValue::CharacterString(str)
        },
    )(buf)
}

pub(crate) fn parse_logical(buf: &[u8]) -> IResult<&[u8], FITSKeywordValue> {
    preceded(
        space0,
        alt((
            value(FITSKeywordValue::Logical(true), char('T')),
            value(FITSKeywordValue::Logical(false), char('F')),
        )),
    )(buf)
}

pub(crate) fn parse_integer(buf: &[u8]) -> IResult<&[u8], FITSKeywordValue> {
    preceded(
        space0,
        map(
        recognize(
                pair(
                    opt(
                        alt((char('+'), char('-')))
                    ),
                    digit1
                )
            ),
            |bytes: &[u8]| {
                let string = std::str::from_utf8(bytes).unwrap();
                let value = string.parse::<i64>().unwrap();
                FITSKeywordValue::IntegerNumber(value)
            }
        )
    )(buf)
}

pub(crate) fn parse_float(buf: &[u8]) -> IResult<&[u8], FITSKeywordValue> {
    preceded(
        space0,
        map(
            float,
            |val| {
                FITSKeywordValue::FloatingPoint(val as f64)
            }
        )
    )(buf)
}

#[cfg(test)]
mod tests {
    use super::{parse_integer, parse_character_string, parse_float, FITSKeywordValue};

    #[test]
    fn test_integer() {
        assert_eq!(
            parse_integer(b"      -4545424"),
            Ok((b"" as &[u8], FITSKeywordValue::IntegerNumber(-4545424)))
        );
        assert_eq!(
            parse_integer(b"      5506"),
            Ok((b"" as &[u8], FITSKeywordValue::IntegerNumber(5506)))
        );
    }

    #[test]
    fn test_float() {
        assert_eq!(
            parse_float(b"      -32768.0"),
            Ok((b"" as &[u8], FITSKeywordValue::FloatingPoint(-32768.0)))
        );
        assert_eq!(
            parse_float(b"      -32767"),
            Ok((b"" as &[u8], FITSKeywordValue::FloatingPoint(-32767.0)))
        );
    }
    #[test]
    fn test_string() {
        assert_eq!(
            parse_character_string(b"      'sdfs Zdfs MLKKLSFD sdf '"),
            Ok((b"" as &[u8], FITSKeywordValue::CharacterString("sdfs Zdfs MLKKLSFD sdf ")))
        );
    }
}
