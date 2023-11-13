#![allow(dead_code)]

use anyhow::{Context, Ok};
use rand::random;

use super::{
    header::Header, question::Question, record::Record, Buffer, DeSerialize, QClass, QType,
    Serialize,
};

#[derive(Debug, Clone)]
pub struct Message {
    pub header: Header,
    pub question: Question,
    pub records: Vec<Record>,
}

impl Serialize for Message {
    fn serialize(&self) -> Result<Vec<u8>, anyhow::Error> {
        let mut h = self.header.serialize().context("serializing header")?;
        let mut b = self.question.serialize().context("serializing body")?;

        h.append(&mut b);
        Ok(h)
    }
}

impl<'a> DeSerialize<'a> for Message {
    type Item = (&'a mut Buffer<'a>, Message);

    fn deserialize(buffer: &'a mut Buffer<'a>) -> Result<Self::Item, anyhow::Error> {
        let (buffer, header) = Header::deserialize(buffer)?;
        let (buffer, question) = Question::deserialize(buffer)?;

        let mut records = Vec::with_capacity(header.an_count as usize);
        let mut buf = buffer;
        for _ in 0..header.an_count {
            let (buffer, record) = Record::deserialize(buf)?;
            records.push(record);
            buf = buffer;
        }

        Ok((
            buf,
            Message {
                header,
                question,
                records,
            },
        ))
    }
}

impl Message {
    pub fn single(name: impl Into<String>) -> Message {
        let id = random::<u16>();
        Self {
            header: Header::request(id),
            question: Question::new(name, QType::A, QClass::IN),
            records: Vec::with_capacity(0),
        }
    }

    pub fn txt(name: impl Into<String>) -> Message {
        let id = random::<u16>();
        Self {
            header: Header::request(id),
            question: Question::new(name, QType::TXT, QClass::IN),
            records: Vec::with_capacity(0),
        }
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

        let q = Message::single("foobar");
        let q = q.header;
        let bytes = q.serialize().unwrap();

        assert_eq!(&query, &bytes);
    }

    #[test]
    fn serilize_query() {
        let query: &[u8] = &[
            0x00, 0x02, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x06, 0x67,
            0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01, 0x00, 0x01,
        ];

        let q = Message::single("google.com");
        let bytes = q.serialize().unwrap();

        assert_eq!(&query, &bytes);
    }

    // 0a00 020f 0035 8d63 008e 0ea1 a4c9 8180  .....5.c........
    // 0001 0005 0000 0000 0462 6c6f 670c 746f  .........blog.to
    // 0462 6c6f 670c 746f
    // 6572 6b74 756d 6c61 7265 0363 6f6d 0000  erktumlare.com..
    // 0100 01c0 0c00 0500 0100 000e 1000 1307  ................
    // 7461 6e64 6f6c 6606 6769 7468 7562 0269  tandolf.github.i
    // 6f00 c033 0001 0001 0000 0e10 0004 b9c7  o..3............
    // 6c99 c033 0001 0001 0000 0e10 0004 b9c7  l..3............
    // 6d99 c033 0001 0001 0000 0e10 0004 b9c7  m..3............
    // 6e99 c033 0001 0001 0000 0e10 0004 b9c7  n..3............
    // 6f99

    // ========== Header section ===========
    // a4c9                                             -> Id
    // 8180                                             -> Flags
    // 0001                                             -> QDCount
    // 0005                                             -> ANCount
    // 0000                                             -> NSCount
    // 0000                                             -> ARCount
    //
    // ========== Question Section ===========
    // 0462 6c6f 67                                     -> length 4 + ASCII "blog"
    // 0c 746f 6572 6b74 756d 6c61 7265                 -> length 12 + text "toerktumlare"
    // 0363 6f6d                                        -> length 3 + text "com"
    // 00                                               -> null termination
    // 0001                                             -> QClass
    // 0001                                             -> QType
    //
    // ======== Resource record CNAME ========
    // c00c                                             -> pointer to byte 12 from the ID-byte (compression) name
    // 0005                                             -> Type (CNAME)
    // 0001                                             -> Class (IN, internet)
    // 0000 0e10                                        -> TTL (time to live)
    // 0013                                             -> Rd length: 19 octets (number of octets in the next field)
    // 0774 616e 646f 6c66 0667 6974 6875 6202 696f     -> cname data (tandolf.github.com)
    // 00                                               -> null termination
    //
    // ========== Resource records A =========
    // c033                                             -> Pointer from ID to name 51 bytes
    // 0001                                             -> Type (A)
    // 0001                                             -> Class (IN, internet)
    // 0000 0e10                                        -> TTL (time to live) 3600 seconds
    // 0004                                             -> number of bytes in the data field (4)
    // b9c7 6c99                                        -> 4 bytes IP-address
    //
    //
    // c033
    // 0001 0001 0000 0e10 0004 b9c7 6d99
    // c033
    // 0001 0001 0000 0e10 0004 b9c7 6e99
    // c033
    // 0001 0001 0000 0e10 0004 b9c7 6f99
    //
    //
}
