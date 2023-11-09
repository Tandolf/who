use nom::error::{Error, ErrorKind, ParseError};
use nom::Err;
use nom::{bits::complete::take, combinator::map, IResult};

use super::parse_utils::CtrlByte;

type BitInput<'a> = (&'a [u8], usize);

fn is_ptr(input: BitInput) -> IResult<BitInput, bool> {
    map(take(2usize), |bits: u8| bits == 0b0000_0011)(input)
}

pub fn parse_ptr(input: BitInput) -> IResult<BitInput, CtrlByte> {
    let (output, is_ptr) = is_ptr(input)?;
    if is_ptr {
        map(take(14usize), |value: u16| CtrlByte::Ptr(value))(output)
    } else {
        Err(Err::Error(Error::from_error_kind(input, ErrorKind::Eof)))
    }
}
