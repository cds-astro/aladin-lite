use std::collections::HashSet;
#[derive(Debug, PartialEq)]
pub struct PrimaryHeader<'a> {
    pub keys: HashSet<&'a str>,
    pub cards: Vec<(&'a str, FITSHeaderKeyword<'a>)>,
}

use crate::error::Error;
impl<'a> PrimaryHeader<'a> {
    pub(crate) fn new(mut input: &'a [u8]) -> MyResult<&'a [u8], Self> {
        let mut cards = Vec::new();
        let mut keys = HashSet::new();

        let mut end = false;

        let mut simple = false;
        let mut naxis = false;
        let mut bitpix = false;

        while !end {
            let (input_next, card) = parse_card(input)?;
            input = input_next;
            if card == FITSHeaderKeyword::End {
                // Do not push the END keyword to the cards
                end = true;
            } else {
                let key = match card {
                    FITSHeaderKeyword::Simple => {
                        simple = true;
                        "SIMPLE"
                    }
                    FITSHeaderKeyword::Bitpix(_) => {
                        bitpix = true;
                        "BITPIX"
                    }
                    FITSHeaderKeyword::Naxis(_) => {
                        naxis = true;
                        "NAXIS"
                    },
                    FITSHeaderKeyword::Blank(_) => "BLANK",
                    FITSHeaderKeyword::NaxisSize { name, .. } => name,
                    FITSHeaderKeyword::Comment(_) => "COMMENT",
                    FITSHeaderKeyword::History(_) => "HISTORY",
                    FITSHeaderKeyword::Other { name, .. } => std::str::from_utf8(name).unwrap(),
                    _ => unreachable!(),
                };

                cards.push((key, card));
                keys.insert(key);
            }
        }
        use std::borrow::Cow;
        // Check mandatory keys are present
        if !simple {
            Err(Error::MandatoryKeywordMissing(Cow::Borrowed("SIMPLE")))
        } else if !bitpix {
            Err(Error::MandatoryKeywordMissing(Cow::Borrowed("BITPIX")))
        } else if !naxis {
            Err(Error::MandatoryKeywordMissing(Cow::Borrowed("NAXIS")))
        } else {
            // Check the NAXISM
            let naxis = &cards.iter().find(|(name, _)| name == &"NAXIS").unwrap().1;

            if let FITSHeaderKeyword::Naxis(naxis) = naxis {
                for idx_axis in 0..*naxis {
                    let key = String::from("NAXIS") + &(idx_axis + 1).to_string();
                    if !keys.contains(&key as &str) {
                        return Err(Error::MandatoryKeywordMissing(key.into()));
                    }
                }
            } else {
                unreachable!();
            }

            let header = Self { cards, keys };

            Ok((input, header))
        }
    }

    pub(crate) fn get_naxis(&self) -> usize {
        if let Some(&FITSHeaderKeyword::Naxis(naxis)) = self.get("NAXIS") {
            naxis
        } else {
            unreachable!();
        }
    }

    pub(crate) fn get_blank(&self) -> f64 {
        if let Some(&FITSHeaderKeyword::Blank(blank)) = self.get("BLANK") {
            blank
        } else {
            unreachable!();
        }
    }

    pub(crate) fn get_axis_size(&self, idx: usize) -> Option<usize> {
        // NAXIS indexes begins at 1 instead of 0
        let naxis = String::from("NAXIS") + &(idx + 1).to_string();
        if let Some(FITSHeaderKeyword::NaxisSize { size, .. }) = self.get(&naxis) {
            Some(*size)
        } else {
            None
        }
    }

    pub(crate) fn get_bitpix(&self) -> &BitpixValue {
        if let Some(FITSHeaderKeyword::Bitpix(bitpix)) = self.get("BITPIX") {
            bitpix
        } else {
            unreachable!();
        }
    }

    pub fn get(&self, key: &str) -> Option<&FITSHeaderKeyword> {
        if self.keys.contains(key) {
            let card = &self.cards.iter().find(|card| key == card.0).unwrap().1;

            Some(card)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FITSHeaderKeyword<'a> {
    Simple,
    Bitpix(BitpixValue),
    Naxis(usize),
    NaxisSize {
        name: &'a str,
        // Index of the axis
        idx: usize,
        // Size of the axis
        size: usize,
    },
    Blank(f64),
    // TODO we will probably need a Cow<str> here
    // because we have to delete simple quote doublons
    Comment(&'a str),
    History(&'a str),
    Other {
        name: &'a [u8],
        value: FITSKeywordValue<'a>,
    },
    End,
}

use crate::card_value::FITSKeywordValue;
#[derive(Debug, PartialEq)]
pub enum BitpixValue {
    U8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

type MyResult<'a, I, O> = Result<(I, O), Error<'a>>;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{alphanumeric1, digit1, multispace0, space0, one_of},
    character::is_space,
    combinator::recognize,
    sequence::{pair, preceded},
    IResult,
};

const KEYWORD_BYTES_LENGTH: usize = 8;
pub(self) fn parse_card(header: &[u8]) -> MyResult<&[u8], FITSHeaderKeyword> {
    // First parse the keyword
    let (header, keyword) = preceded(multispace0, parse_card_keyword)(header)?;
    // We stop consuming tokens after the exit
    if keyword == b"END" {
        return Ok((header, FITSHeaderKeyword::End))
    }

    let (header, value) = parse_card_value(header)?;

    match (keyword, value) {
        // SIMPLE = true check
        (b"SIMPLE", value) => match value {
            FITSKeywordValue::Logical(true) => Ok((header, FITSHeaderKeyword::Simple)),
            _ => Err(Error::MandatoryValueError("SIMPLE")),
        },
        // BITPIX in {8, 16, 32, 64, -32, -64} check
        (b"BITPIX", value) => match value {
            FITSKeywordValue::FloatingPoint(bitpix) => {
                let bitpix = bitpix as i32;
                let bitpix = match bitpix {
                    8 => BitpixValue::U8,
                    16 => BitpixValue::I16,
                    32 => BitpixValue::I32,
                    64 => BitpixValue::I64,
                    -32 => BitpixValue::F32,
                    -64 => BitpixValue::F64,
                    _ => return Err(Error::BitpixBadValue),
                };

                Ok((header, FITSHeaderKeyword::Bitpix(bitpix)))
            }
            _ => Err(Error::MandatoryValueError("BITPIX")),
        },
        // NAXIS > 0 integer check
        (b"NAXIS", value) => match value {
            FITSKeywordValue::FloatingPoint(naxis) => {
                if naxis <= 0.0 {
                    Err(Error::NegativeOrNullNaxis)
                } else {
                    Ok((header, FITSHeaderKeyword::Naxis(naxis as usize)))
                }
            }
            _ => Err(Error::MandatoryValueError("NAXIS")),
        },
        // BLANK value
        (b"BLANK", value) => match value {
            FITSKeywordValue::FloatingPoint(blank) => {
                Ok((header, FITSHeaderKeyword::Blank(blank)))
            },
            _ => Err(Error::MandatoryValueError("BLANK")),
        },
        // Comment associated to a string check
        (b"COMMENT", value) => match value {
            FITSKeywordValue::CharacterString(str) => { Ok((header, FITSHeaderKeyword::Comment(str))) },
            _ => Err(Error::MandatoryValueError("COMMENT")),
        },
        // History associated to a string check
        (b"HISTORY", value) => match value {
            FITSKeywordValue::CharacterString(str) => Ok((header, FITSHeaderKeyword::History(str))),
            _ => Err(Error::MandatoryValueError("HISTORY")),
        },
        ([b'N', b'A', b'X', b'I', b'S', ..], value) => {
            let name = std::str::from_utf8(keyword).unwrap();
            let (_, idx_axis) =
                (preceded(tag(b"NAXIS"), digit1)(keyword) as IResult<&[u8], &[u8]>).unwrap();

            let idx_axis = std::str::from_utf8(idx_axis)
                .map(|str| str.parse::<usize>().unwrap())
                .unwrap();
            if let FITSKeywordValue::FloatingPoint(size) = value {
                if size <= 0.0 {
                    Err(Error::NegativeOrNullNaxisSize(idx_axis))
                } else {
                    // Check the value
                    Ok((
                        header,
                        FITSHeaderKeyword::NaxisSize {
                            name,
                            idx: idx_axis,
                            size: size as usize,
                        },
                    ))
                }
            } else {
                Err(Error::MandatoryValueError(name))
            }
        }
        (keyword, value) => {
            Ok((
                header,
                FITSHeaderKeyword::Other {
                    name: keyword,
                    value,
                },
            ))
        }
    }
}

pub(crate) fn parse_card_keyword(buf: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        recognize(pair(tag(b"NAXIS"), digit1)),
        take_till(|c| c == b' ' || c == b'\t' || c == b'=')
    ))(buf)
}

use crate::card_value::*;
pub(crate) fn parse_card_value(buf: &[u8]) -> IResult<&[u8], FITSKeywordValue> {
    preceded(
        white_space0,
        alt((
            preceded(
                tag(b"= "),
                alt((
                    parse_character_string,
                    parse_logical,
                    parse_float,
                )),
            ),
            parse_undefined,
        )),
    )(buf)
}

#[cfg(test)]
mod tests {
    use super::{parse_card, Error, FITSHeaderKeyword, FITSKeywordValue};
    #[test]
    fn test_parse_card() {
        assert_eq!(
            parse_card(
                b"AZSDFGFC=                    T                                                  "
            ),
            Ok((
                b"                                                  " as &[u8],
                FITSHeaderKeyword::Other {
                    name: b"AZSDFGFC",
                    value: FITSKeywordValue::Logical(true)
                }
            ))
        );
        assert_eq!(
            parse_card(
                b"CDS_1=                     T                                                  "
            ),
            Ok((
                b"                                                  " as &[u8],
                FITSHeaderKeyword::Other {
                    name: b"CDS_1",
                    value: FITSKeywordValue::Logical(true)
                }
            ))
        );
    }
}
