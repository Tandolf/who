use nom::{bits, combinator::map, complete::take, error::Error, Finish, IResult};

use super::{DeSerialize, Serialize};

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
#[derive(Debug, Clone)]
pub struct DNSHeader {
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

impl DNSHeader {
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
    ) -> DNSHeader {
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
    pub(crate) fn request() -> DNSHeader {
        DNSHeader::new(
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

impl Serialize for DNSHeader {
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

fn parse_flags(input: BitInput) -> IResult<BitInput, DNSHeader> {
    let (input, id) = parse_u16(input)?;
    dbg!(&id);
    let (input, qr) = parse_bool(input)?;
    dbg!(&qr);
    let (input, opcode) = parse_opcode(input)?;
    dbg!(&opcode);
    let (input, aa) = parse_bool(input)?;
    dbg!(&aa);
    let (input, rc) = parse_bool(input)?;
    dbg!(&rc);
    let (input, rd) = parse_bool(input)?;
    dbg!(&rd);
    let (input, ra) = parse_bool(input)?;
    dbg!(&ra);
    let (input, _) = skip(input, 3)?;
    let (input, r_code) = parse_rcode(input)?;
    dbg!(&r_code);
    let (input, qd_count) = parse_u16(input)?;
    dbg!(&qd_count);
    let (input, an_count) = parse_u16(input)?;
    dbg!(&an_count);
    let (input, ns_count) = parse_u16(input)?;
    dbg!(&ns_count);
    let (input, ar_count) = parse_u16(input)?;
    dbg!(&ar_count);

    Ok((
        input,
        DNSHeader::new(
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

fn skip(i: BitInput, value: usize) -> IResult<BitInput, ()> {
    map(take(value), |_bits: u8| ())(i)
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

impl DeSerialize for DNSHeader {
    type Item = DNSHeader;
    fn deserialize(raw: &[u8]) -> Result<Self::Item, anyhow::Error> {
        let header =
            bits::<&[u8], DNSHeader, Error<(&[u8], usize)>, Error<&[u8]>, _>(parse_flags)(raw)
                .finish()
                .unwrap() // Handle error better so that anyhow works
                .1;

        Ok(header)
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
mod tests {
    use pretty_assertions::assert_eq;

    use crate::dns::DeSerialize;

    use super::DNSHeader;

    #[test]
    fn parse_qr() {
        let bytes = vec![0x00, 0x02, 0x80];

        let msg = DNSHeader::deserialize(&bytes).unwrap();

        assert_eq!(msg.qr, true);
    }
}
