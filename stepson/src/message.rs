pub mod ack;
pub mod data_mdr;

use std::convert::TryInto;

use num_enum::{FromPrimitive, IntoPrimitive};

use crate::repl::{CompletionTree, FromRepl, ParseError, ReplCompletion};
use crate::serializable::{DeserializeError, Serializable};

/// com.sony.songpal.tandemfamily.DataType
#[derive(Clone, Copy, Debug, IntoPrimitive, FromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum DataType {
    Ack = 1,
    DataMdr = 12,
    #[num_enum(default)]
    Unknown,
}

#[derive(Debug, FromRepl)]
pub enum Data {
    Ack(ack::Ack),
    DataMdr(data_mdr::DataMdr),
    Unknown(Vec<u8>),
}

impl Data {
    pub fn data_type(&self) -> DataType {
        match self {
            Data::Ack(_) => DataType::Ack,
            Data::DataMdr(_) => DataType::DataMdr,
            Data::Unknown(_) => DataType::Unknown,
        }
    }
}

// Message format:
// MESSAGE_START
// escape_specials(
//     u8: data_type
//     u8: sequence_number
//     u32: data length
//     [u8; _]: data
//     u8: checksum mod 0xff (excluding MESSAGE_START)
// )
// MESSAGE_END
//
// escape / unescape specials is in com.sony.songpal.tandemfamily.message.a.b

pub const MESSAGE_START: u8 = 62;
pub const MESSAGE_END: u8 = 60;
pub const ESCAPE_CHAR: u8 = 61;

/// com.sony.songpal.tandemfamily.message.b
#[derive(Debug)]
pub struct Message {
    pub sequence_number: u8,
    pub data: Data,
}

impl Message {
    pub fn requires_ack(&self) -> bool {
        match self.data.data_type() {
            DataType::DataMdr => true,
            _ => false,
        }
    }
}

impl FromRepl for Message {
    fn from_repl<'a, T>(words: &mut T) -> Result<Self, ParseError>
    where
        T: Iterator<Item = &'a str>,
    {
        Ok(Self {
            sequence_number: 0,
            data: Data::from_repl(words)?,
        })
    }
}

impl ReplCompletion for Message {
    fn completion_tree() -> CompletionTree {
        Data::completion_tree()
    }
}

impl Serializable for Message {
    fn serialize(&self) -> Vec<u8> {
        let mut data = escape_specials(&match &self.data {
            Data::Ack(x) => x.serialize(),
            Data::DataMdr(x) => x.serialize(),
            Data::Unknown(x) => x.clone(),
        });

        // message start, data type, sequence number
        let mut ret = vec![
            MESSAGE_START,
            self.data.data_type().into(),
            self.sequence_number.into(),
        ];

        // data length
        ret.extend_from_slice(&(data.len() as u32).to_be_bytes());

        // data
        ret.append(&mut data);

        // checksum
        ret.push(checksum(&ret[1..]));

        // message end
        ret.push(MESSAGE_END);

        ret
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DeserializeError> {
        let bytes = unescape_specials(bytes)?;

        if bytes[0] != MESSAGE_START {
            return Err(DeserializeError::InvalidStartOfMessage(bytes[0]));
        }

        let data_type = DataType::from(bytes[1]);
        let sequence_number = bytes[2];
        let data_len = u32::from_be_bytes(bytes[3..7].try_into().unwrap()); //TODO
        let data = match data_type {
            DataType::Ack => Data::Ack(ack::Ack::deserialize(&bytes[7..(7 + data_len as usize)])?),
            DataType::DataMdr => Data::DataMdr(data_mdr::DataMdr::deserialize(
                &bytes[7..(7 + data_len as usize)],
            )?),
            DataType::Unknown => Data::Unknown(bytes[7..(7 + data_len as usize)].to_vec()),
        };
        let chksum = bytes[(7 + data_len as usize)];

        if chksum != checksum(&bytes[1..(7 + data_len as usize)]) {
            return Err(DeserializeError::InvalidChecksum(chksum));
        }

        if bytes[(7 + data_len as usize) + 1] != MESSAGE_END {
            return Err(DeserializeError::InvalidEndOfMessage(
                bytes[(7 + data_len as usize) + 1],
            ));
        }

        Ok(Self {
            sequence_number,
            data,
        })
    }
}

fn checksum(s: &[u8]) -> u8 {
    s.iter().sum()
}

fn escape_specials(s: &[u8]) -> Vec<u8> {
    let mut new = Vec::new();
    for b in s {
        match b {
            &MESSAGE_START | &MESSAGE_END | &ESCAPE_CHAR => {
                new.push(ESCAPE_CHAR);
            }
            _ => {}
        }
        new.push(*b);
    }
    new
}

fn unescape_specials(s: &[u8]) -> Result<Vec<u8>, DeserializeError> {
    let mut new = Vec::new();
    let mut iter = s.iter();
    loop {
        let b = match iter.next() {
            Some(b) => b,
            None => break,
        };
        match b {
            //&MESSAGE_START | &MESSAGE_END => return Err(DeserializeError::ExpectedEscape(*b)),
            &ESCAPE_CHAR => {
                let escaped = match iter.next() {
                    Some(b) => b,
                    None => return Err(DeserializeError::EscapeEof),
                };
                match escaped {
                    &MESSAGE_START | &MESSAGE_END => {
                        new.push(*escaped);
                    }
                    _ => return Err(DeserializeError::InvalidEscape(*escaped)),
                }
            }
            _ => new.push(*b),
        }
    }
    Ok(new)
}
