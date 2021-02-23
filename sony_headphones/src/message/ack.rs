use std::collections::HashMap;

use crate::serializable::{DeserializeError, Serializable};
use crate::repl::{FromRepl, ReplCompletion, ParseError};

#[derive(Debug)]
pub struct Ack {}

impl FromRepl for Ack {
    fn from_repl<'a, T>(_words: &mut T) -> Result<Self, ParseError> where
        T: Iterator<Item=&'a str> {
        Ok(Self {})
    }
}

impl ReplCompletion for Ack {
    fn completion_map(_words: &Vec<String>)
        -> HashMap<String, Option<fn(Vec<String>, usize) -> (usize, Vec<String>)>> {
        HashMap::new()
    }
}

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
