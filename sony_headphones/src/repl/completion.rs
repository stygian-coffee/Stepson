use rustyline::{Context, Helper, Result};
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

use super::Repl;

pub trait ReplCompletion {
    fn complete<'a, T>(words: T, pos: usize) -> (usize, Vec<String>) where
        T: Iterator<Item=&'a str>;
}

pub(super) struct ReplHelper;

impl Completer for ReplHelper {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>)
        -> Result<(usize, Vec<Self::Candidate>)> {
        let words = line.split_whitespace();

        let (write_pos, candidates) = Repl::complete(words, pos);
        Ok((write_pos, candidates)) //TODO fails for e.g. s + space + tab
    }
}

impl Highlighter for ReplHelper {}

impl Hinter for ReplHelper {
    type Hint = String;
}

impl Validator for ReplHelper {}

impl Helper for ReplHelper {}
