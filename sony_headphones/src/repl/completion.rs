use std::collections::HashMap;

use rustyline::{Context, Helper, Result};
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

pub struct CompletionMap<'a>(HashMap<&'a str, Option<&'a Self>>);

impl<'a> From<HashMap<&'a str, Option<&'a Self>>> for CompletionMap<'a> {
    fn from(map: HashMap<&'a str, Option<&'a Self>>) -> Self {
        Self(map)
    }
}

impl<'a> From<CompletionMap<'a>> for HashMap<&'a str, Option<&'a CompletionMap<'a>>> {
    fn from(map: CompletionMap<'a>) -> Self {
        map.0
    }
}

pub trait ReplCompletion {
    fn possible_completions<'a>() -> CompletionMap<'a>;
}

pub(super) struct ReplHelper<'a> {
    completions: CompletionMap<'a>,
}

impl<'a> ReplHelper<'a> {
    pub fn new(completions: CompletionMap<'a>) -> Self {
        Self { completions }
    }

    fn complete_from<'b, T>(mut words: T, pos: usize, completions: Option<&CompletionMap<'_>>)
        -> Result<(usize, Vec<<Self as Completer>::Candidate>)> where
        T: Iterator<Item=&'b str> {
        let completions = match completions {
            Some(c) => c,
            None => return Ok((pos, Vec::new())),
        };

        let first_word = match words.next() {
            Some(w) => w,
            None => return Ok((pos, completions.0.keys().map(|s| s.to_string()).collect())),
        };

        if completions.0.contains_key(first_word) {
            Self::complete_from(words, pos, *completions.0.get(first_word).unwrap())
        } else {
            Ok((pos - first_word.len(), completions.0.keys().map(|s| s.to_string())
                .filter(|s| s.starts_with(first_word)).collect()))
        }
    }
}

impl<'a> Completer for ReplHelper<'a> {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>)
        -> Result<(usize, Vec<Self::Candidate>)> {
        let words = line.split_whitespace();
        Self::complete_from(words, pos, Some(&self.completions))
    }
}

impl<'a> Highlighter for ReplHelper<'a> {}

impl<'a> Hinter for ReplHelper<'a> {
    type Hint = String;
}

impl<'a> Validator for ReplHelper<'a> {}

impl<'a> Helper for ReplHelper<'a> {}
