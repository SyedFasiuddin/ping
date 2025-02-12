use crate::blob::Blob;
use crate::parse;
use cookie_factory as cf;
use custom_debug_derive::Debug;
use std::io;

use nom::{
    combinator::map,
    number::complete::{be_u16, be_u32, be_u8},
    sequence::tuple,
    Parser,
};

#[derive(Debug, Clone)]
pub enum Type {
    EchoReply,
    DestinationUnreachable(DestinationUnreachable),
    EchoRequest,
    TimeExceeded(TimeExceeded),
    // type, code
    Other(u8, u8),
}

#[derive(Debug, Clone)]
pub enum DestinationUnreachable {
    HostUnreachable,
    Other(u8),
}

#[derive(Debug, Clone)]
pub enum TimeExceeded {
    TTLExpired,
    Other(u8),
}

impl From<(u8, u8)> for Type {
    fn from(x: (u8, u8)) -> Self {
        let (typ, code) = x;

        match typ {
            0 => Self::EchoReply,
            3 => Self::DestinationUnreachable(code.into()),
            8 => Self::EchoRequest,
            11 => Self::TimeExceeded(code.into()),
            _ => Self::Other(typ, code),
        }
    }
}

impl From<u8> for DestinationUnreachable {
    fn from(x: u8) -> Self {
        match x {
            1 => Self::HostUnreachable,
            x => Self::Other(x),
        }
    }
}

impl From<u8> for TimeExceeded {
    fn from(x: u8) -> Self {
        match x {
            0 => Self::TTLExpired,
            x => Self::Other(x),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub typ: Type,

    #[debug(skip)]
    pub checksum: u16,

    #[debug(format = "{:?}")]
    pub header: Header,
    pub payload: Blob,
}

impl Packet {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        let original_i = i;

        let (i, typ) = {
            let (i, (typ, code)) = tuple((be_u8, be_u8))(i)?;
            (i, Type::from((typ, code)))
        };
        let (i, checksum) = be_u16(i)?;
        let (i, header) = match typ {
            Type::EchoRequest => map(Echo::parse, Header::EchoRequest)(i)?,
            Type::EchoReply => map(Echo::parse, Header::EchoReply)(i)?,
            _ => map(be_u32, Header::Other)(i)?,
        };
        let payload = Blob::new(i);

        let res = Self {
            typ,
            checksum,
            header,
            payload,
        };

        let serialized = cf::gen_simple(res.serialize(), Vec::new()).unwrap();
        assert_eq!(original_i, &serialized[..]);

        Ok((i, res))
    }

    pub fn serialize_no_checksum<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::{bytes::be_u16, sequence::tuple};

        tuple((
            self.header.serialize_type_and_code(),
            be_u16(0),
            self.header.serialize(),
            self.payload.serialize(),
        ))
    }

    pub fn serialize<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::{bytes::le_u16, combinator::slice};

        move |out| {
            let mut buf = cf::gen_simple(self.serialize_no_checksum(), Vec::new())?;
            let checksum = crate::ipv4::checksum(&buf);
            cf::gen_simple(le_u16(checksum), &mut buf[2..])?;
            slice(buf)(out)
        }
    }

    pub fn as_ipv4_payload(self) -> crate::ipv4::Payload {
        crate::ipv4::Payload::ICMP(self)
    }
}

#[derive(Debug, Clone)]
pub struct Echo {
    #[debug(format = "{:04x}")]
    pub identifier: u16,
    #[debug(format = "{:04x}")]
    pub sequence_number: u16,
}

impl Echo {
    fn parse(i: parse::Input) -> parse::Result<Self> {
        map(tuple((be_u16, be_u16)), |(identifier, sequence_number)| {
            Echo {
                identifier,
                sequence_number,
            }
        })
        .parse(i)
    }

    pub fn serialize<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::{bytes::be_u16, sequence::tuple};
        tuple((be_u16(self.identifier), be_u16(self.sequence_number)))
    }

    pub fn as_echo_request<P: AsRef<[u8]>>(self, payload: P) -> Packet {
        Packet {
            typ: Type::EchoRequest,
            checksum: 0,
            header: Header::EchoRequest(self),
            payload: Blob::new(payload.as_ref()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Header {
    EchoRequest(Echo),
    EchoReply(Echo),
    Other(u32),
}

impl Header {
    fn serialize<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::bytes::be_u32;

        move |out| match self {
            Self::EchoRequest(echo) | Self::EchoReply(echo) => echo.serialize()(out),
            Self::Other(x) => be_u32(*x)(out),
        }
    }

    fn serialize_type_and_code<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, sequence::tuple};

        move |out| match self {
            Self::EchoRequest(_) => tuple((be_u8(8), be_u8(0)))(out),
            Self::EchoReply(_) => tuple((be_u8(0), be_u8(0)))(out),
            _ => unimplemented!(),
        }
    }
}
