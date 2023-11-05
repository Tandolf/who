use nom::error::VerboseError;
use nom::Finish;
use nom::IResult;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::time::Duration;

use super::parse_utils::parse_ipv4;
use super::parse_utils::parse_name;
use super::parse_utils::parse_qclass;
use super::parse_utils::parse_qtype;
use super::parse_utils::parse_rdlength;
use super::parse_utils::parse_ttl;
use super::Global;
use super::{DeSerialize, QClass, QType};

type VResult<I, O> = IResult<I, O, VerboseError<I>>;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RData {
    A(Ipv4Addr),
    CNAME(String),
}

// Resource record format
//
// The answer, authority, and additional sections all share the same
// format: a variable number of resource records, where the number of
// records is specified in the corresponding count field in the header.
// Each resource record has the following format:
//                                     1  1  1  1  1  1
//       0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
//     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//     |                                               |
//     /                                               /
//     /                      NAME                     /
//     |                                               |
//     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//     |                      TYPE                     |
//     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//     |                     CLASS                     |
//     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//     |                      TTL                      |
//     |                                               |
//     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//     |                   RDLENGTH                    |
//     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--|
//     /                     RDATA                     /
//     /                                               /
//     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    // a domain name to which this resource record pertains.
    name: String,

    // two octets containing one of the RR type codes.
    // This field specifies the meaning of the data in the RDATA field.
    qtype: QType,

    // two octets which specify the class of the data in the
    // RDATA field.
    qclass: QClass,

    // a 32 bit unsigned integer that specifies the time
    // interval (in seconds) that the resource record may be
    // cached before it should be discarded.  Zero values are
    // interpreted to mean that the RR can only be used for the
    // transaction in progress, and should not be cached.
    ttl: Duration,

    // an unsigned 16 bit integer that specifies the length in
    // octets of the RDATA field.
    rd_length: u16,

    // a variable length string of octets that describes the
    // resource.  The format of this information varies
    // according to the TYPE and CLASS of the resource record.
    // For example, the if the TYPE is A and the CLASS is IN,
    // the RDATA field is a 4 octet ARPA Internet address.
    rdata: RData,
}

impl Record {
    pub fn new(
        name: String,
        qtype: QType,
        qclass: QClass,
        ttl: Duration,
        rd_length: u16,
        rdata: RData,
    ) -> Self {
        Self {
            name,
            qtype,
            qclass,
            ttl,
            rd_length,
            rdata,
        }
    }
}

fn get_cached<'a>(
    buffer: &'a [u8],
    cache: &HashMap<u32, String>,
) -> Result<(&'a [u8], String), anyhow::Error> {
    let index = buffer[1];
    let name = cache.get(&(index as u32)).unwrap();
    Ok((&buffer[2..], name.clone()))
}

fn parse_record<'a>(
    buf: &'a [u8],
    global: &Global<'a>,
) -> Result<(&'a [u8], Record), anyhow::Error> {
    // If a pointer, then get the value from the cache
    let (buffer, name) = match buf[0] {
        0xC0 => get_cached(buf, &global.cache).unwrap(),
        _ => parse_name(buf).finish().unwrap(),
    };

    let (buffer, qtype) = parse_qtype(buffer).finish().unwrap();
    let (buffer, qclass) = parse_qclass(buffer).finish().unwrap();
    let (buffer, ttl) = parse_ttl(buffer).finish().unwrap();
    let (buffer, rd_length) = parse_rdlength(buffer).finish().unwrap();

    let (buffer, rdata) = match qtype {
        QType::A => {
            let (buffer, address) = parse_ipv4(buffer).finish().unwrap();
            (buffer, RData::A(address))
        }
        _ => unimplemented!(),
    };

    Ok((
        buffer,
        Record::new(name.clone(), qtype, qclass, ttl, rd_length, rdata),
    ))
}

impl<'a> DeSerialize<'a> for Record {
    type Item = (&'a [u8], Record);

    fn deserialize(buffer: &'a [u8], global: &mut Global<'a>) -> Result<Self::Item, anyhow::Error> {
        let (buffer, record) = parse_record(buffer, global)?;
        Ok((buffer, record))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_record() {
        let buffer = vec![
            0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01,
            0x00, 0x01, 0x00, 0x00, 0x0e, 0x10, 0x00, 0x04, 0x01, 0x02, 0x03, 0x04,
        ];

        let mut global = Global {
            cache: HashMap::new(),
            source: &buffer,
        };
        let (_, actual) = Record::deserialize(&buffer, &mut global).unwrap();

        let expected = Record::new(
            "google.com".to_owned(),
            QType::A,
            QClass::IN,
            Duration::new(3600, 0),
            4,
            RData::A(Ipv4Addr::new(1, 2, 3, 4)),
        );

        assert_eq!(expected, actual);
    }
}
