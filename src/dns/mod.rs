#![allow(dead_code)]

use std::fmt::{self, Display, Formatter};

pub mod bit_parsers;
pub mod header;
pub mod message;
pub mod parse_utils;
pub mod question;
pub mod record;

#[derive(Debug)]
pub struct Buffer<'a> {
    pub current: &'a [u8],
    pub source: &'a [u8],
}

pub trait Serialize {
    fn serialize(&self) -> Result<Vec<u8>, anyhow::Error>;
}

pub trait DeSerialize<'a> {
    type Item;
    fn deserialize(buffer: &'a mut Buffer<'a>) -> Result<Self::Item, anyhow::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QType {
    A = 1,       // 1 a host address
    NS = 2,      // 2 an authoritative name server
    MD = 3,      // 3 a mail destination (Obsolete - use MX)
    MF = 4,      // 4 a mail forwarder (Obsolete - use MX)
    CNAME = 5,   // 5 the canonical name for an alias
    SOA = 6,     // 6 marks the start of a zone of authority
    MB = 7,      // 7 a mailbox domain name (EXPERIMENTAL)
    MG = 8,      // 8 a mail group member (EXPERIMENTAL)
    MR = 9,      // 9 a mail rename domain name (EXPERIMENTAL)
    NULL = 10,   // 10 a null RR (EXPERIMENTAL)
    WKS = 11,    // 11 a well known service description
    PTR = 12,    // 12 a domain name pointer
    HINFO = 13,  // 13 host information
    MINFO = 14,  // 14 mailbox or mail list information
    MX = 15,     // 15 mail exchange
    TXT = 16,    // 16 text strings
    AAAA = 28,   // 28 ipv6 address
    AXFR = 252,  // 252 A request for a transfer of an entire zone
    MAILB = 253, // 253 A request for mailbox-related records (MB, MG or MR)
    MAILA = 254, // 254 A request for mail agent RRs (Obsolete - see MX)
    STAR = 255,  // 255 A request for all records
}

impl Display for QType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// CLASS fields appear in resource records.  The following CLASS mnemonics
// and values are defined:
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QClass {
    IN = 1,     // 1 the Internet
    CS = 2,     // 2 the CSNET class (obsolete)
    CH = 3,     // 3 the CHAOS class
    HS = 4,     // 4 Hesiod [Dyer 87]
    STAR = 255, // 255 any class
}

impl Display for QClass {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
