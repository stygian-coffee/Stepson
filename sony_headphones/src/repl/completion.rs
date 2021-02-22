use std::collections::HashMap;

use rustyline::{Context, Helper, Result};
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

use super::Repl;

pub trait ReplCompletion {
    fn completion_map<T>() -> HashMap<String, Option<fn(T, usize) -> (usize, Vec<String>)>>
        where T: Iterator<Item=String>, Self: Sized;

    fn complete<T>(mut words: T, pos: usize) -> (usize, Vec<String>) where
        T: Iterator<Item=String>, Self: Sized {
        let completion_map: HashMap<String, _> = Self::completion_map().into_iter()
            .map(|(k, v)| (k.to_string(), v)).collect();

        let first_word = match words.next() {
            Some(w) => w,
            None => return (0, completion_map.into_iter().map(|(k, _)| k).collect()),
        };

        if completion_map.contains_key(&first_word.to_string()) {
            match completion_map.get(&first_word).unwrap() {
                Some(f) => f(words, pos),
                None => (pos, vec![])
            }
        } else {
            (pos - first_word.len(), completion_map.into_iter()
                .filter_map(|(k, _)| {
                    match k.starts_with(&first_word) {
                        true => Some(k),
                        false => None,
                    }
                }).collect())
        }
    }
}

impl dyn ReplCompletion {
}

pub(super) struct ReplHelper;

impl Completer for ReplHelper {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>)
        -> Result<(usize, Vec<Self::Candidate>)> {
        let words = line.split_whitespace().map(|s| s.to_string());

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
