#![allow(dead_code)]
use anyhow::{Context, Ok};

use super::{header::DNSHeader, DeSerialize, Serialize};

#[derive(Debug, Clone)]
pub struct DNSPackage {
    header: DNSHeader,
    body: DNSBody,
}

impl Serialize for DNSPackage {
    fn serialize(&self) -> Result<Vec<u8>, anyhow::Error> {
        let mut h = self.header.serialize().context("serializing header")?;
        let mut b = self.body.serialize().context("serializing body")?;

        h.append(&mut b);
        Ok(h)
    }
}

impl DeSerialize for DNSPackage {
    type Item = DNSPackage;

    fn deserialize(raw: &[u8]) -> Result<Self::Item, anyhow::Error> {
        Ok(Self {
            header: DNSHeader::deserialize(&raw[..12])?,
            body: DNSBody::deserialize(&raw[12..]),
        })
    }
}

impl DNSPackage {
    pub fn single(name: impl Into<String>) -> DNSPackage {
        Self {
            header: DNSHeader::request(),
            body: DNSBody::new(name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DNSBody {
    qname: String,
    qtype: u16,
    qclass: u16,
}

impl DNSBody {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            qname: name.into(),
            qtype: 0x0001,
            qclass: 0x0001,
        }
    }

    fn deserialize(raw: &[u8]) -> DNSBody {
        DNSBody {
            qname: "".to_owned(),
            qtype: 1,
            qclass: 1,
        }
    }
}

impl DeSerialize for DNSBody {
    type Item = DNSBody;

    fn deserialize(raw: &[u8]) -> Result<Self::Item, anyhow::Error> {
        todo!()
    }
}

impl Serialize for DNSBody {
    fn serialize(&self) -> Result<Vec<u8>, anyhow::Error> {
        let labels = self.qname.split('.');

        let mut body = Vec::new();
        for label in labels {
            let length: u8 = label.chars().count().try_into().unwrap();
            let label = label.as_bytes();
            body.push(length);
            body.extend(label);
        }
        body.push(0);
        body.push((self.qtype >> 8) as u8);
        body.push(self.qtype as u8);
        body.push((self.qclass >> 8) as u8);
        body.push(self.qclass as u8);
        Ok(body)
    }
}

// OPCODE
//
// A four bit field that specifies kind of query in this message. This value is set by the
// originator of a query and copied into the response.  The values are:
#[derive(Debug, Clone)]
pub enum Opcode {
    // a standard query (QUERY)
    Query = 0,
    // an inverse query (IQUERY)
    IQuery = 1,
    // a server status request (STATUS)
    Status = 2,
    // reserved for future use (value 3-15)
    Reserved,
}

// RCODE Response code - this 4 bit field is set as part of responses.  The values have the following interpretation:
#[derive(Debug, Clone)]
pub enum ResponseCode {
    // No error condition
    NoError = 0,

    // Format error: The name server was unable to interpret the query.
    FormatError = 1,

    // Server failure: The name server was unable to process this query due to a problem with the name server.
    ServerFailure = 2,

    // Name Error: This code signifies that the domain name referenced in the query does not exist.
    NameError = 3,

    // Not Implemented: The name server does not support the requested kind of query.
    NotImplemented = 4,

    // Refused: The name server refuses to perform the specified operation for policy reasons.
    Refused = 5,
}

#[cfg(test)]
mod test {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn serilize_header() {
        let query: &[u8] = &[
            0x00, 0x02, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];

        let q = DNSPackage::single("foobar");
        let q = q.header;
        let bytes = q.serialize().unwrap();

        assert_eq!(&query, &bytes);
    }

    #[test]
    fn serilize_body() {
        let query: &[u8] = &[
            0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
        ];

        let q = DNSBody::new("google.com");
        let bytes = q.serialize().unwrap();

        assert_eq!(&query, &bytes);
    }

    #[test]
    fn serilize_query() {
        let query: &[u8] = &[
            0x00, 0x02, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x06, 0x67,
            0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
        ];

        let q = DNSPackage::single("google.com");
        let bytes = q.serialize().unwrap();

        assert_eq!(&query, &bytes);
    }
}
