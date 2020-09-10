use std::borrow::Cow;
#[derive(Debug, PartialEq)]
pub enum Error<'a> {
    CardSizeNotRespected(usize),
    MandatoryKeywordMissing(Cow<'a, str>),
    MustBe8BytesLong(&'a [u8]),
    NomError(nom::Err<(&'a [u8], nom::error::ErrorKind)>),
    SimpleKeywordBadValue,
    BitpixBadValue,
    NaxisBadValue,
    NaxisSizeBadValue,
    NaxisSizeNotFound,
    MandatoryValueError(&'a str),
    NegativeOrNullNaxis,
    NegativeOrNullNaxisSize(usize),
}

impl<'a> From<nom::Err<(&'a [u8], nom::error::ErrorKind)>> for Error<'a> {
    fn from(nom_err: nom::Err<(&'a [u8], nom::error::ErrorKind)>) -> Self {
        Error::NomError(nom_err)
    }
}
