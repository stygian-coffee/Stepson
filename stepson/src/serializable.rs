use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeserializeError {
    #[error("escape byte plated at end of data")]
    EscapeEof,
    // #[error("expected escape character before {0}")]
    // ExpectedEscape(u8),
    #[error("invalid checksum: {0}")]
    InvalidChecksum(u8),
    #[error("invalid length: {0}")]
    InvalidLength(u8),
    #[error("invalid escape: {0}")]
    InvalidEscape(u8),
    #[error("invalid start of message: {0}")]
    InvalidStartOfMessage(u8),
    #[error("invalid end of message: {0}")]
    InvalidEndOfMessage(u8),
    #[error("unrecognized value: {0}")]
    TryFromPrimitive(u8),
}

impl<T: TryFromPrimitive<Primitive = u8>> From<TryFromPrimitiveError<T>> for DeserializeError {
    fn from(err: TryFromPrimitiveError<T>) -> Self {
        DeserializeError::TryFromPrimitive(err.number)
    }
}

//TODO derive macro for simple structs
pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(bytes: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized;
}
