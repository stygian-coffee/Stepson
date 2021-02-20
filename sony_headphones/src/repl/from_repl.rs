pub use derive::FromRepl;

use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("expected argument")]
    ExpectedArgument,
    #[error("unexpected argument")]
    UnexpectedArgument,
    #[error("unknown argument: {0}")]
    UnknownArgument(String),
}

pub trait FromRepl {
    fn from_repl<'a, T>(words: &mut T) -> Result<Self, ParseError> where
        Self: Sized,
        T: Iterator<Item=&'a str>;
}

impl FromRepl for u8 {
    fn from_repl<'a, T>(words: &mut T) -> Result<Self, ParseError> where
        T: Iterator<Item=&'a str> {
        let word = match words.next() {
            Some(w) => w,
            None => return Err(ParseError::ExpectedArgument),
        };
        Ok(u8::from_str(word)?)
    }
}

impl FromRepl for Vec<u8> {
    fn from_repl<'a, T>(words: &mut T) -> Result<Self, ParseError> where
        T: Iterator<Item=&'a str> {
        Ok(words.map(|w| u8::from_str(w)).collect::<Result<Vec<u8>, _>>()?)
    }
}
