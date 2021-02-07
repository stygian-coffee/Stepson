use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::serializable::{DeserializeError, Serializable};

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.param.AsmId
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum AsmId {
    Normal = 0,
    Voice = 1,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.param.AsmOnOffValue
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum AsmOnOffValue {
    Off = 0,
    On = 1,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.param.AsmSettingType
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum AsmSettingType {
    OnOff = 0,
    LevelAdjustment = 1,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.param.NcAsmEffect
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum NcAsmEffect {
    Off = 0,
    On = 1,
    AdjustmentInProgress = 16,
    AdjustmentCompletion = 17,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.param.NcAsmInquiredType
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum NcAsmInquiredType {
    NoUse = 0,
    NoiseCancelling = 1,
    NoiseCancellingAndAmbientSoundMode = 2,
    AmbientSoundMode = 3,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.NcAsmSettingType
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum NcAsmSettingType {
    OnOff = 0,
    LevelAdjustment = 1,
    DualSingleOff = 2,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.NcDualSingleValue
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum NcDualSingleValue {
    Off = 0,
    Single = 1,
    Dual = 2,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.NcOnOffValue
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum NcOnOffValue {
    Off = 0,
    On = 1,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.param.NcSettingType
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum NcSettingType {
    OnOff = 0,
    LevelAdjustment = 1,
}

/// com.sony.songpal.tandemfamily.message.mdr.v1.table1.param.NcSettingValue
#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum NcSettingValue {
    Off = 0,
    On = 1,
}

#[derive(Debug)]
pub struct NcAsmSetParam {
    pub nc_asm_inquired_type: NcAsmInquiredType,
    pub nc_asm_effect: NcAsmEffect,
    pub nc_asm_setting_type: NcAsmSettingType,
    pub nc_dual_single_value: NcDualSingleValue,
    pub asm_setting_type: AsmSettingType,
    pub asm_id: AsmId,
    pub asm_level: u8, //TODO understand level
}

impl Serializable for NcAsmSetParam {
    fn serialize(&self) -> Vec<u8> {
        vec![
            self.nc_asm_inquired_type.into(),
            self.nc_asm_effect.into(),
            self.nc_asm_setting_type.into(),
            self.nc_dual_single_value.into(),
            self.asm_setting_type.into(),
            self.asm_id.into(),
            self.asm_level.into(),
        ]
    }

    fn deserialize(_bytes: &[u8]) -> Result<Self, DeserializeError> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct NcAsmNtfyParam {
    pub nc_asm_inquired_type: NcAsmInquiredType,
    pub nc_asm_effect: NcAsmEffect,
    pub nc_asm_setting_type: NcAsmSettingType,
    pub nc_dual_single_value: NcDualSingleValue,
    pub asm_setting_type: AsmSettingType,
    pub asm_id: AsmId,
    pub asm_level: u8, //TODO understand level
}

impl Serializable for NcAsmNtfyParam {
    fn serialize(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn deserialize(_bytes: &[u8]) -> Result<Self, DeserializeError> {
        unimplemented!()
    }
}
