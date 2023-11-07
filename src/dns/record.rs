use nom::Finish;
use std::fmt::Display;
use std::net::Ipv4Addr;
use std::time::Duration;

use super::parse_utils::parse_ipv4;
use super::parse_utils::parse_name;
use super::parse_utils::parse_qclass;
use super::parse_utils::parse_qtype;
use super::parse_utils::parse_rdlength;
use super::parse_utils::parse_string;
use super::parse_utils::parse_ttl;
use super::parse_utils::VResult;
use super::Buffer;
use super::{DeSerialize, QClass, QType};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RData {
    A(Ipv4Addr),
    CNAME(String),
    TXT(String),
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

fn parse_record<'a>(buffer: &'a [u8], source: &'a [u8]) -> VResult<&'a [u8], Record> {
    // If a pointer, then get the value from the cache
    let (buffer, name) = match buffer[0] {
        0xC0 => {
            let index = buffer[1] as usize;
            let (_, name) = parse_name(&source[index..])?;
            (&buffer[2..], name)
        }
        _ => parse_name(buffer)?,
    };

    let (buffer, qtype) = parse_qtype(buffer)?;
    let (buffer, qclass) = parse_qclass(buffer)?;
    let (buffer, ttl) = parse_ttl(buffer)?;
    let (buffer, rd_length) = parse_rdlength(buffer)?;

    let (buffer, rdata) = match qtype {
        QType::A => {
            let (buffer, address) = parse_ipv4(buffer)?;
            (buffer, RData::A(address))
        }
        QType::CNAME => {
            let (buffer, name) = parse_name(buffer)?;
            (buffer, RData::CNAME(name))
        }
        QType::TXT => {
            let (buffer, txt) = parse_string(buffer, rd_length.into())?;
            (buffer, RData::TXT(txt.to_owned()))
        }
        _ => unimplemented!(),
    };

    Ok((
        buffer,
        Record::new(name.clone(), qtype, qclass, ttl, rd_length, rdata),
    ))
}

impl<'a> DeSerialize<'a> for Record {
    type Item = (&'a mut Buffer<'a>, Record);

    fn deserialize(buffer: &'a mut Buffer<'a>) -> Result<Self::Item, anyhow::Error> {
        let (_, record) = parse_record(buffer.current, buffer.source)
            .finish()
            .map_err(|e| {
                anyhow::Error::msg(format!("Error at: {:?}, with code: {:?}", e.input, e.code))
            })?;
        Ok((buffer, record))
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ";{}\t\t\t{}\t{}", self.name, self.qclass, self.qtype)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_record() {
        let raw = vec![
            0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01,
            0x00, 0x01, 0x00, 0x00, 0x0e, 0x10, 0x00, 0x04, 0x01, 0x02, 0x03, 0x04,
        ];

        let mut buffer = Buffer {
            current: &raw,
            source: &raw,
        };
        let (_, actual) = Record::deserialize(&mut buffer).unwrap();

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
