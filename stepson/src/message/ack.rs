use crate::repl::{CompletionContext, CompletionTree, FromRepl, ParseError, ReplCompletion};
use crate::serializable::{DeserializeError, Serializable};

#[derive(Debug)]
pub struct Ack {}

impl FromRepl for Ack {
    fn from_repl<'a, T>(_words: &mut T) -> Result<Self, ParseError>
    where
        T: Iterator<Item = &'a str>,
    {
        Ok(Self {})
    }
}

impl ReplCompletion for Ack {
    fn completion_tree(_cx: &CompletionContext) -> CompletionTree {
        CompletionTree::empty()
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
