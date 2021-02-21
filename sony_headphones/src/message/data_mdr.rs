pub mod nc_asm;

use num_enum::{IntoPrimitive, FromPrimitive};

use crate::repl::{FromRepl, ReplCompletion, ParseError};
use crate::serializable::{DeserializeError, Serializable};

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.Command
#[derive(Clone, Copy, Debug, IntoPrimitive, FromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandType {
    //NcAsmGetParam = 102, // Noise Cancelling and/or Ambient Sound Mode
    NcAsmSetParam = 104,
    NcAsmNtfyParam = 105,
    #[num_enum(default)]
    Unknown,
}

#[derive(Debug, FromRepl)]
pub enum Command {
    //NcAsmGetParam(nc_asm::NcAsmGetParam),
    NcAsmSetParam(nc_asm::NcAsmSetParam),
    NcAsmNtfyParam(nc_asm::NcAsmNtfyParam),
    Unknown(Vec<u8>),
}

#[derive(Debug)]
pub struct DataMdr {
    pub command: Command
}

impl FromRepl for DataMdr {
    fn from_repl<'a, T>(words: &mut T) -> Result<Self, ParseError> where
        T: Iterator<Item=&'a str> {
        Ok(Self { command: Command::from_repl(words)? })
    }
}

impl ReplCompletion for DataMdr {
    fn complete<'a, T>(words: T, pos: usize) -> (usize, Vec<String>) where
        T: Iterator<Item=&'a str> {
        Command::complete(words, pos)
    }
}

impl Serializable for DataMdr {
    fn serialize(&self) -> Vec<u8> {
        //TODO clean this up a bit to not repeat the match
        let command_type = match self.command {
            Command::NcAsmSetParam(_) => CommandType::NcAsmSetParam,
            Command::NcAsmNtfyParam(_) => CommandType::NcAsmNtfyParam,
            Command::Unknown(_) => CommandType::Unknown,
        };
        let mut bytes = match &self.command {
            Command::NcAsmSetParam(x) => x.serialize(),
            Command::NcAsmNtfyParam(x) => x.serialize(),
            Command::Unknown(x) => x.clone(),
        };

        let mut ret = vec![command_type.into()];
        ret.append(&mut bytes);
        ret
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DeserializeError> {
        let command_type = bytes[0].into();
        let command = match command_type {
            CommandType::NcAsmSetParam => Command::NcAsmSetParam(nc_asm::NcAsmSetParam::deserialize(&bytes[1..])?),
            CommandType::NcAsmNtfyParam => Command::NcAsmNtfyParam(nc_asm::NcAsmNtfyParam::deserialize(&bytes[1..])?),
            CommandType::Unknown => Command::Unknown(bytes[1..].to_vec()),
        };
        Ok(Self { command })
    }
}
