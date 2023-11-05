use nom::bytes::complete::take;
use nom::combinator::map;
use nom::number::complete::{be_u16, be_u32, u8};
use nom::{error::VerboseError, IResult};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::str;
use std::time::Duration;

use super::{QClass, QType};

type VResult<I, O> = IResult<I, O, VerboseError<I>>;

// deserializes names in DNSRecords, format is ascii chars prefixed by a length, and ending with a
// null termination. Example:
//
// 0x06 0x67 0x6f 0x6f 0x67 0x6c 0x65 0x03 0x63 0x6f 0x6d 0x00
//
// 6google3com -> google.com
//
pub fn parse_name(buffer: &[u8]) -> VResult<&[u8], String> {
    parse_name_cached(buffer, None, buffer)
}
pub fn parse_name_cached<'a>(
    buffer: &'a [u8],
    cache: Option<&mut HashMap<u32, String>>,
    source: &'a [u8],
) -> VResult<&'a [u8], String> {
    let mut tokens: Vec<String> = Vec::new();

    if buffer[0] != 0x00 {
        // Storing a reference outside the loop for mutation
        let mut buffer = buffer;
        dbg!(source.len());
        dbg!(buffer[1..].len());

        // Index used for caching if cache available
        let index = source.len() - buffer.len();
        loop {
            let (buf, length) = u8::<&[u8], VerboseError<&[u8]>>(buffer).unwrap();
            let (buf, token) = take_token(buf, length as usize).unwrap();
            tokens.push(token.to_owned());
            buffer = buf;
            if buf[0] == 0x00 {
                break;
            };
        }
        let token = tokens.join(".");
        if let Some(cache) = cache {
            dbg!(&cache);
            dbg!(&index);
            cache.insert(index as u32, token.clone());
        }
        return Ok((&buffer[1..], token.clone()));
    }
    panic!("No string") // fix error handling
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

    #[test]
    fn parse_name_with_cache() {
        let original = vec![
            0x00, 0x00, 0x00, 0x04, 0x62, 0x6c, 0x6f, 0x67, 0x0c, 0x74, 0x6f, 0x65, 0x72, 0x6b,
            0x74, 0x75, 0x6d, 0x6c, 0x61, 0x72, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
        ];

        let buffer = vec![
            0x04, 0x62, 0x6c, 0x6f, 0x67, 0x0c, 0x74, 0x6f, 0x65, 0x72, 0x6b, 0x74, 0x75, 0x6d,
            0x6c, 0x61, 0x72, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
        ];

        let mut cache = HashMap::new();
        let (buffer, name) = parse_name_cached(&buffer, Some(&mut cache), &original).unwrap();

        assert_eq!("blog.toerktumlare.com", &name);
        assert_eq!(&name, &cache[&4]);
        assert_eq!(0, buffer.len());
    }
}
