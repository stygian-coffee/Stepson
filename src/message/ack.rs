use crate::serializable::{DeserializeError, Serializable};

#[derive(Debug)]
pub struct Ack {}

impl Serializable for Ack {
    fn serialize(&self) -> Vec<u8> {
        vec![]
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DeserializeError> {
        match bytes.len() {
            0 => Ok(Self {}),
            _ => Err(DeserializeError::InvalidLength(bytes[0])),
        }
    }
}
