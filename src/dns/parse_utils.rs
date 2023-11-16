use nom::branch::alt;
use nom::bytes::complete::take;
use nom::combinator::map;
use nom::error::{Error, ErrorKind, ParseError};
use nom::number::complete::{be_u128, be_u16, be_u32, u8};
use nom::IResult;
use nom::{bits, Err};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str;
use std::time::Duration;

use super::bit_parsers::parse_ptr;
use super::{QClass, QType};

pub type VResult<I, O> = IResult<I, O, Error<I>>;

pub enum CtrlByte {
    Length(u8),
    Ptr(u16),
    Null,
}

fn parse_nullbyte(buffer: &[u8]) -> IResult<&[u8], CtrlByte> {
    if buffer[0] == 0x00 {
        map(take(1usize), |_: &[u8]| CtrlByte::Null)(buffer)
    } else {
        Err(Err::Error(Error::from_error_kind(buffer, ErrorKind::Eof)))
    }
}

fn parse_length_byte(buffer: &[u8]) -> IResult<&[u8], CtrlByte> {
    map(take(1usize), |value: &[u8]| CtrlByte::Length(value[0]))(buffer)
}

fn resolve_next(buffer: &[u8]) -> IResult<&[u8], CtrlByte> {
    alt((bits(parse_ptr), parse_nullbyte, parse_length_byte))(buffer)
}

// pub fn parse_names<'a>(
//     buffer: &'a [u8],
//     source: &'a [u8],
//     tokens: &mut Vec<String>,
// ) -> VResult<&'a [u8], String> {
//     let mut b = buffer;
//     loop {
//         let next = b[0];
//         if is_ptr(next) {
//             let (buf, index) = ptr_value(b)?;
//             let (_, _) = parse_names(&source[index..], source, tokens)?;
//             b = buf;
//             break;
//         } else if next == 0x00 {
//             break;
//         }
//         let (buf, length) = u8::<&[u8], Error<&[u8]>>(b)?;
//         let (buf, token) = take_token(buf, length as usize)?;
//         tokens.push(token.to_owned());
//         b = buf;
//     }
//     Ok((&b, tokens.join(".")))
// }

pub fn parse_names<'a>(
    buffer: &'a [u8],
    source: &'a [u8],
    tokens: &mut Vec<String>,
) -> VResult<&'a [u8], String> {
    let mut b = buffer;
    loop {
        if let Ok((buffer, ctrl_byte)) = resolve_next(b) {
            match ctrl_byte {
                CtrlByte::Length(length) => {
                    let (buffer, token) = take_token(buffer, length as usize)?;
                    tokens.push(token.to_owned());
                    b = buffer;
                }
                CtrlByte::Ptr(index) => {
                    let (_, _) = parse_names(&source[index as usize..], source, tokens)?;
                    b = buffer;
                    break;
                }
                CtrlByte::Null => {
                    b = buffer;
                    break;
                }
            }
        }
    }
    Ok((&b, tokens.join(".")))
}
// deserializes names in DNSRecords, format is ascii chars prefixed by a length, and ending with a
// null termination. Example:
//
// 0x06 0x67 0x6f 0x6f 0x67 0x6c 0x65 0x03 0x63 0x6f 0x6d 0x00
//
// 6google3com -> google.com
//
pub fn parse_name(buffer: &[u8]) -> VResult<&[u8], String> {
    let mut tokens: Vec<String> = Vec::new();

    // Storing a reference outside the loop for mutation
    let mut buffer = buffer;
    // Index used for caching if cache available
    while buffer[0] != 0x00 {
        let (buf, length) = u8::<&[u8], Error<&[u8]>>(buffer)?;
        let (buf, token) = take_token(buf, length as usize)?;
        tokens.push(token.to_owned());
        buffer = buf;
    }
    let token = tokens.join(".");
    Ok((&buffer[1..], token))
}

pub fn take_token(buffer: &[u8], length: usize) -> VResult<&[u8], &str> {
    map(take(length), |v| str::from_utf8(v).unwrap())(buffer)
}

pub fn parse_qclass(buffer: &[u8]) -> VResult<&[u8], QClass> {
    map(be_u16, |value: u16| match value {
        1 => QClass::IN,
        2 => QClass::CS,
        3 => QClass::CH,
        4 => QClass::HS,
        5 => QClass::STAR,
        _ => panic!("Unknown QClass returned: {}", value),
    })(buffer)
}

pub fn parse_qtype(buffer: &[u8]) -> VResult<&[u8], QType> {
    map(be_u16, |value: u16| match value {
        1 => QType::A,
        5 => QType::CNAME,
        16 => QType::TXT,
        28 => QType::AAAA,
        _ => panic!("Unknown QType returned: {}", value),
    })(buffer)
}

pub fn parse_rdlength(buffer: &[u8]) -> VResult<&[u8], u16> {
    be_u16(buffer)
}

pub fn parse_ipv4(buffer: &[u8]) -> VResult<&[u8], Ipv4Addr> {
    map(be_u32, |value: u32| Ipv4Addr::from(value))(buffer)
}

pub fn parse_ipv6(buffer: &[u8]) -> VResult<&[u8], Ipv6Addr> {
    map(be_u128, |value: u128| Ipv6Addr::from(value))(buffer)
}

pub fn parse_ttl(buffer: &[u8]) -> VResult<&[u8], Duration> {
    map(be_u32, |value| Duration::new(value.try_into().unwrap(), 0))(buffer)
}

pub fn is_ptr(byte: u8) -> bool {
    byte >> 6 == 3
}

pub fn ptr_value(buffer: &[u8]) -> VResult<&[u8], usize> {
    map(take(2usize), |v: &[u8]| {
        let x = ((v[0] & 0b0011_1111) as usize) << 8;
        let y = v[1] as usize;
        x + y
    })(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_parsing() {
        let buffer = vec![
            0x07, 0x74, 0x77, 0x69, 0x74, 0x74, 0x65, 0x72, 0x03, 0x63, 0x6f, 0x6d, 0x00,
        ];

        let (_, name) = parse_name(&buffer).unwrap();

        assert_eq!("twitter.com", &name)
    }

    #[test]
    fn parse_name_multiple_tokens() {
        let buffer = vec![
            0x04, 0x62, 0x6c, 0x6f, 0x67, 0x0c, 0x74, 0x6f, 0x65, 0x72, 0x6b, 0x74, 0x75, 0x6d,
            0x6c, 0x61, 0x72, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
        ];

        let (buffer, name) = parse_name(&buffer).unwrap();

        assert_eq!("blog.toerktumlare.com", &name);
        assert_eq!(0, buffer.len());
    }

    #[test]
    fn is_pointer() {
        let v = 0xC0;
        assert!(is_ptr(v));
    }

    #[test]
    fn is_not_pointer() {
        let v = 0x3F;
        assert!(!is_ptr(v));
    }

    #[test]
    fn get_ptr_value() {
        let buffer = vec![0xC1, 0x01];
        let (_, actual) = ptr_value(&buffer).unwrap();
        assert_eq!(257usize, actual)
    }

    #[test]
    fn parse_string_with_pointers() {
        let source = vec![
            0x00, 0x00, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
            0x00, 0x00, 0x03, 0x6e, 0x73, 0x31, 0xc0, 0x02,
        ];
        let buffer = vec![0x03, 0x6e, 0x73, 0x31, 0xc0, 0x02];

        let mut v = Vec::new();
        let (_, actual) = parse_names(&buffer, &source, &mut v).unwrap();
        assert_eq!("ns1.google.com", actual)
    }

    #[test]
    fn parse_string_with_two_pointers() {
        let source = vec![
            0x00, 0x00, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
            0x00, 0x00, 0x03, 0x6e, 0x73, 0x31, 0xc0, 0x02,
        ];
        let buffer = vec![0x03, 0x6e, 0x73, 0x31, 0xc0, 0x02];

        let mut v = Vec::new();
        let (_, actual) = parse_names(&buffer, &source, &mut v).unwrap();
        assert_eq!("ns1.google.com", actual)
    }
}
