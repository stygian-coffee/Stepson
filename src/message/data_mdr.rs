pub mod nc_asm;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::serializable::{DeserializeError, Serializable};

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.Command
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandType {
    NcAsmGetParam = 102, // Noise Cancelling and/or Ambient Sound Mode
    NcAsmSetParam = 104,
    NcAsmNtfyParam = 105,
}

#[derive(Debug)]
pub enum Command {
    //NcAsmGetParam(nc_asm::NcAsmGetParam),
    NcAsmSetParam(nc_asm::NcAsmSetParam),
    NcAsmNtfyParam(nc_asm::NcAsmNtfyParam),
}

impl Command {
    fn command_type(&self) -> CommandType {
        match self {
            //Command::NcAsmGetParam(_) => CommandType::NcAsmGetParam,
            Command::NcAsmSetParam(_) => CommandType::NcAsmSetParam,
            Command::NcAsmNtfyParam(_) => CommandType::NcAsmNtfyParam,
        }
    }
}

impl Serializable for Command {
    fn serialize(&self) -> Vec<u8> {
        match self {
            //Command::NcAsmGetParam(_) => CommandType::NcAsmGetParam,
            Command::NcAsmSetParam(x) => x.serialize(),
            Command::NcAsmNtfyParam(x) => x.serialize(),
        }
    }

    fn deserialize(_bytes: &[u8]) -> Result<Self, DeserializeError> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct DataMdr {
    pub command: Command
}

impl Serializable for DataMdr {
    fn serialize(&self) -> Vec<u8> {
        let mut ret = vec![self.command.command_type().into()];
        ret.append(&mut self.command.serialize());
        ret
    }

    fn deserialize(_bytes: &[u8]) -> Result<Self, DeserializeError> {
        unimplemented!()
    }
}
