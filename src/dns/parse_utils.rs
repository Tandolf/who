use nom::bytes::complete::take;
use nom::combinator::map;
use nom::error::Error;
use nom::number::complete::{be_u16, be_u32, u8};
use nom::{error::VerboseError, IResult};
use std::net::Ipv4Addr;
use std::str;
use std::time::Duration;

use super::{QClass, QType};

pub type VResult<I, O> = IResult<I, O, Error<I>>;

pub fn parse_string(buffer: &[u8], length: usize) -> VResult<&[u8], &str> {
    take_token(buffer, length)
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
        _ => panic!("Unknown QType returned: {}", value),
    })(buffer)
}

pub fn parse_rdlength(buffer: &[u8]) -> VResult<&[u8], u16> {
    be_u16(buffer)
}

pub fn parse_ipv4(buffer: &[u8]) -> VResult<&[u8], Ipv4Addr> {
    map(take(4usize), |value: &[u8]| {
        Ipv4Addr::new(value[0], value[1], value[2], value[3])
    })(buffer)
}

pub fn parse_ttl(buffer: &[u8]) -> VResult<&[u8], Duration> {
    map(be_u32, |value| Duration::new(value.try_into().unwrap(), 0))(buffer)
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
}
