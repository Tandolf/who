use std::fmt::{Display, Formatter};

use nom::{bits, combinator::map, complete::take, error::Error, sequence::tuple, Finish, IResult};

use super::{Buffer, DeSerialize, Serialize};

//  The header contains the following fields:
//
//                                  1  1  1  1  1  1
//    0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
//  +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//  |                      ID                       |
//  +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//  |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
//  +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//  |                    QDCOUNT                    |
//  +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//  |                    ANCOUNT                    |
//  +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//  |                    NSCOUNT                    |
//  +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//  |                    ARCOUNT                    |
//  +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    // A 16 bit identifier assigned by the program that generates any kind of query
    pub id: u16,

    // A one bit field that specifies whether this message is a query (0), or a response (1).
    pub qr: bool,

    // A four bit field that specifies kind of query in this message.
    pub opcode: Opcode,

    // Authoritative Answer - this bit is valid in responses, and specifies that the responding
    // name server is an authority for the domain name in question section.
    pub aa: bool,

    // TrunCation - specifies that this message was truncated due to length greater than that
    // permitted on the transmission channel.
    pub tc: bool,

    // Recursion Desired - If RD is set, it directs the name server to pursue the query recursively.
    pub rd: bool,

    // Recursion Available - this be is set or cleared in a response, and denotes whether recursive query support is available in the name server.
    pub ra: bool,

    // Z Reserved for future use.  Must be zero in all queries and responses.
    pub z: u8,

    // Response code - this 4 bit field is set as part of responses.
    pub r_code: ResponseCode,

    // QDCOUNT an unsigned 16 bit integer specifying the number of entries in the question section.
    pub qd_count: u16,

    // ANCOUNT an unsigned 16 bit integer specifying the number of resource records in the answer section.
    pub an_count: u16,

    // NSCOUNT an unsigned 16 bit integer specifying the number of name server resource records in the authority records section.
    pub ns_count: u16,

    // ARCOUNT an unsigned 16 bit integer specifying the number of resource records in the additional records section.
    pub ar_count: u16,
}

impl Header {
    pub fn new(
        id: u16,
        qr: bool,
        opcode: Opcode,
        aa: bool,
        tc: bool,
        rd: bool,
        ra: bool,
        r_code: ResponseCode,
        qd_count: u16,
        an_count: u16,
        ns_count: u16,
        ar_count: u16,
    ) -> Header {
        Self {
            id,
            qr,
            opcode,
            aa,
            tc,
            rd,
            ra,
            z: 0x00,
            r_code,
            qd_count,
            an_count,
            ns_count,
            ar_count,
        }
    }

    // Default header when making a plain request
    pub(crate) fn request() -> Header {
        Header::new(
            0x0002,
            false,
            Opcode::Query,
            false,
            false,
            true,
            false,
            ResponseCode::NoError,
            1,
            0,
            0,
            1,
        )
    }
}

impl Serialize for Header {
    fn serialize(&self) -> Result<Vec<u8>, anyhow::Error> {
        let flags_upper: u8 = (self.qr as u8) << 7;
        let flags_upper = match self.opcode {
            Opcode::Query => flags_upper | (Opcode::Query as u8) << 3,
            Opcode::IQuery => flags_upper | ((Opcode::IQuery as u8) << 3),
            Opcode::Status => flags_upper | ((Opcode::Status as u8) << 3),
            _ => flags_upper,
        };

        let flags_upper = flags_upper | (self.aa as u8) << 2 | (self.tc as u8) << 1 | self.rd as u8;
        let flags_lower = (self.ra as u8) << 7 | self.z << 4;

        let flags_lower = match self.r_code {
            ResponseCode::NoError => flags_lower,
            ResponseCode::FormatError => flags_lower | ResponseCode::FormatError as u8,
            ResponseCode::ServerFailure => flags_lower | ResponseCode::ServerFailure as u8,
            ResponseCode::NameError => flags_lower | ResponseCode::NameError as u8,
            ResponseCode::NotImplemented => flags_lower | ResponseCode::NotImplemented as u8,
            ResponseCode::Refused => flags_lower | ResponseCode::Refused as u8,
        };

        Ok(vec![
            (self.id >> 8) as u8,
            self.id as u8,
            flags_upper,
            flags_lower,
            (self.qd_count >> 8) as u8,
            self.qd_count as u8,
            (self.an_count >> 8) as u8,
            self.an_count as u8,
            (self.ns_count >> 8) as u8,
            self.ns_count as u8,
            (self.ar_count >> 8) as u8,
            self.ar_count as u8,
        ])
    }
}

type BitInput<'a> = (&'a [u8], usize);

fn parse_header(input: BitInput) -> IResult<BitInput, Header> {
    let (
        input,
        (id, qr, opcode, aa, rc, rd, ra, _, r_code, qd_count, an_count, ns_count, ar_count),
    ) = tuple((
        parse_u16,
        parse_bool,
        parse_opcode,
        parse_bool,
        parse_bool,
        parse_bool,
        parse_bool,
        skip3,
        parse_rcode,
        parse_u16,
        parse_u16,
        parse_u16,
        parse_u16,
    ))(input)?;

    Ok((
        input,
        Header::new(
            id, qr, opcode, aa, rc, rd, ra, r_code, qd_count, an_count, ns_count, ar_count,
        ),
    ))
}

fn parse_u16(i: BitInput) -> IResult<BitInput, u16> {
    take(16usize)(i)
}

fn parse_bool(i: BitInput) -> IResult<BitInput, bool> {
    map(take(1usize), |bits: u8| bits > 0)(i)
}

fn parse_opcode(i: BitInput) -> IResult<BitInput, Opcode> {
    map(take(4usize), |bit: u8| match bit {
        0 => Opcode::Query,
        1 => Opcode::IQuery,
        2 => Opcode::Status,
        3 => Opcode::Reserved,
        _ => panic!("Illegal OpCode value: {:#02x}", bit),
    })(i)
}

fn skip3(i: BitInput) -> IResult<BitInput, ()> {
    map(take(3usize), |_bits: u8| ())(i)
}

fn parse_rcode(i: BitInput) -> IResult<BitInput, ResponseCode> {
    map(take(4usize), |bit: u8| match bit {
        0 => ResponseCode::NoError,
        1 => ResponseCode::FormatError,
        2 => ResponseCode::ServerFailure,
        3 => ResponseCode::NameError,
        4 => ResponseCode::NotImplemented,
        5 => ResponseCode::Refused,
        _ => panic!("Illegal ResponseCode value: {:#02x}", bit),
    })(i)
}

impl<'a> DeSerialize<'a> for Header {
    type Item = (&'a mut Buffer<'a>, Header);

    fn deserialize(buffer: &'a mut Buffer<'a>) -> Result<Self::Item, anyhow::Error> {
        let (buf, header) =
            bits::<_, _, Error<(&[u8], usize)>, Error<&[u8]>, _>(parse_header)(buffer.current)
                .finish()
                .map_err(|e| {
                    anyhow::Error::msg(format!("Error at: {:?}, with code: {:?}", e.input, e.code))
                })?;

        buffer.current = buf;
        Ok((buffer, header))
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            ";; OPCODE: {}, STATUS: {} id: {}\n;; {}, {}, {},\n;; QUERY: {}, ANSWERS: {}, AUTHORITY: {}, ADDITIONAL: {}\n",
            self.opcode, self.r_code, self.id, if self.qr { "qr"} else {""}, if self.rd {"rd"} else {""}, if self.ra {"ra"} else {""}, self.qd_count, self.an_count, self.ns_count, self.ar_count
        )
    }
}

// OPCODE
//
// A four bit field that specifies kind of query in this message. This value is set by the
// originator of a query and copied into the response.  The values are:
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Display for Opcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// RCODE Response code - this 4 bit field is set as part of responses.  The values have the following interpretation:
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Display for ResponseCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::{Header, Opcode, ResponseCode};
    use crate::dns::{Buffer, DeSerialize};
    use pretty_assertions::assert_eq;

    #[test]
    fn header_deserialize() {
        let raw = vec![
            0x00, 0x02, 0x81, 0x80, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];

        let expected = Header::new(
            2,
            true,
            Opcode::Query,
            false,
            false,
            true,
            true,
            ResponseCode::NoError,
            1,
            0,
            0,
            1,
        );

        let mut buffer = Buffer {
            current: &raw,
            source: &raw,
        };
        let (_, actual) = Header::deserialize(&mut buffer).unwrap();

        assert_eq!(expected, actual);
    }
}
