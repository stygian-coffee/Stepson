use std::collections::HashMap;
use std::convert::TryInto;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::repl::{FromRepl, ReplCompletion};
use crate::serializable::{DeserializeError, Serializable};

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.param.AsmId
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq, FromRepl)]
#[repr(u8)]
pub enum AsmId {
    Normal = 0,
    Voice = 1,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.param.AsmOnOffValue
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq, FromRepl)]
#[repr(u8)]
pub enum AsmOnOffValue {
    Off = 0,
    On = 1,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.param.AsmSettingType
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq, FromRepl)]
#[repr(u8)]
pub enum AsmSettingType {
    OnOff = 0,
    LevelAdjustment = 1,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.param.NcAsmEffect
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq, FromRepl)]
#[repr(u8)]
pub enum NcAsmEffect {
    Off = 0,
    On = 1,
    AdjustmentInProgress = 16,
    AdjustmentCompletion = 17,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.param.NcAsmInquiredType
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq, FromRepl)]
#[repr(u8)]
pub enum NcAsmInquiredType {
    NoUse = 0,
    NoiseCancelling = 1,
    NoiseCancellingAndAmbientSoundMode = 2,
    AmbientSoundMode = 3,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.NcAsmSettingType
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq, FromRepl)]
#[repr(u8)]
pub enum NcAsmSettingType {
    OnOff = 0,
    LevelAdjustment = 1,
    DualSingleOff = 2,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.NcDualSingleValue
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq, FromRepl)]
#[repr(u8)]
pub enum NcDualSingleValue {
    Off = 0,
    Single = 1,
    Dual = 2,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.NcOnOffValue
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq, FromRepl)]
#[repr(u8)]
pub enum NcOnOffValue {
    Off = 0,
    On = 1,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.param.NcSettingType
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq, FromRepl)]
#[repr(u8)]
pub enum NcSettingType {
    OnOff = 0,
    LevelAdjustment = 1,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.param.NcSettingValue
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq, FromRepl)]
#[repr(u8)]
pub enum NcSettingValue {
    Off = 0,
    On = 1,
}

#[derive(Debug, FromRepl)]
//TODO understand asm level (i.e. the u8)
pub struct NcAsmSetParam(
    NcAsmInquiredType,
    NcAsmEffect,
    NcAsmSettingType,
    NcDualSingleValue,
    AsmSettingType,
    AsmId,
    u8,
);

impl Serializable for NcAsmSetParam {
    fn serialize(&self) -> Vec<u8> {
        vec![
            self.0.into(),
            self.1.into(),
            self.2.into(),
            self.3.into(),
            self.4.into(),
            self.5.into(),
            self.6.into(),
        ]
    }

    fn deserialize(_bytes: &[u8]) -> Result<Self, DeserializeError> {
        unimplemented!()
    }
}

#[derive(Debug, FromRepl)]
//TODO understand asm level (i.e. the u8)
pub struct NcAsmNtfyParam(
    NcAsmInquiredType,
    NcAsmEffect,
    NcAsmSettingType,
    NcDualSingleValue,
    AsmSettingType,
    AsmId,
    u8,
);

impl Serializable for NcAsmNtfyParam {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DeserializeError> {
        Ok(Self(
            bytes[0].try_into()?,
            bytes[1].try_into()?,
            bytes[2].try_into()?,
            bytes[3].try_into()?,
            bytes[4].try_into()?,
            bytes[5].try_into()?,
            bytes[6],
        ))
    }
}
