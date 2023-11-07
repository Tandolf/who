use std::fmt::Display;

use nom::Finish;

use crate::Buffer;

use super::{
    parse_utils::{parse_name, parse_qclass, parse_qtype},
    DeSerialize, QClass, QType, Serialize,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    qname: String,
    qtype: QType,
    qclass: QClass,
}

impl Question {
    pub fn new(name: impl Into<String>, qtype: QType, qclass: QClass) -> Self {
        Self {
            qname: name.into(),
            qtype,
            qclass,
        }
    }
}

impl Serialize for Question {
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
        let qtype = match self.qtype {
            QType::A => QType::A as u8,
            QType::NS => todo!(),
            QType::MD => todo!(),
            QType::MF => todo!(),
            QType::CNAME => todo!(),
            QType::SOA => todo!(),
            QType::MB => todo!(),
            QType::MG => todo!(),
            QType::MR => todo!(),
            QType::NULL => todo!(),
            QType::WKS => todo!(),
            QType::PTR => todo!(),
            QType::HINFO => todo!(),
            QType::MINFO => todo!(),
            QType::MX => todo!(),
            QType::TXT => QType::TXT as u8,
            QType::AXFR => todo!(),
            QType::MAILB => todo!(),
            QType::MAILA => todo!(),
            QType::STAR => todo!(),
        };
        body.push(0);
        body.push(qtype);

        let qclass = match self.qclass {
            QClass::IN => QClass::IN as u8,
            QClass::CS => QClass::CS as u8,
            QClass::CH => QClass::CH as u8,
            QClass::HS => QClass::HS as u8,
            QClass::STAR => QClass::STAR as u8,
        };
        body.push(0);
        body.push(qclass);
        Ok(body)
    }
}

impl<'a> DeSerialize<'a> for Question {
    type Item = (&'a mut Buffer<'a>, Question);

    fn deserialize(buffer: &'a mut Buffer<'a>) -> Result<Self::Item, anyhow::Error> {
        let (buf, name) = parse_name(buffer.current).finish().unwrap();
        let (buf, qtype) = parse_qtype(buf).finish().unwrap();
        let (buf, qclass) = parse_qclass(buf).finish().unwrap();

        buffer.current = buf;

        Ok((buffer, Question::new(name, qtype, qclass)))
    }
}

impl Display for Question {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}\t\t\t{}\t{}", self.qname, self.qclass, self.qtype)
    }
}

#[cfg(test)]
mod tests {
    use super::Question;
    use crate::dns::{Buffer, DeSerialize, QClass, QType};

    // ========== Question Section ===========
    // 0462 6c6f 67                                     -> length 4 + ASCII "blog"
    // 0c 746f 6572 6b74 756d 6c61 7265                 -> length 12 + text "toerktumlare"
    // 0363 6f6d                                        -> length 3 + text "com"
    // 00                                               -> null termination
    // 0001                                             -> QType
    // 0001                                             -> QClass
    //
    #[test]
    fn deserialize_question() {
        let raw = vec![
            0x04, 0x62, 0x6c, 0x6f, 0x67, 0x0c, 0x74, 0x6f, 0x65, 0x72, 0x6b, 0x74, 0x75, 0x6d,
            0x6c, 0x61, 0x72, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01, 0x00, 0x01,
        ];

        let mut buffer = Buffer {
            current: &raw,
            source: &raw,
        };
        let (_, actual) = Question::deserialize(&mut buffer).unwrap();

        let expected = Question::new("blog.toerktumlare.com", QType::A, QClass::IN);
        assert_eq!(expected, actual)
    }
}
