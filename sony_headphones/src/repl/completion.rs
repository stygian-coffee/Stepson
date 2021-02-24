use std::collections::HashMap;

use rustyline::{Context, Helper, Result};
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

use super::Repl;

pub trait ReplCompletion {
    fn completion_map(words: &Vec<String>)
        -> HashMap<String, Option<fn(Vec<String>, usize) -> (usize, Vec<String>)>>
        where Self: Sized;

    fn complete(mut words: Vec<String>, pos: usize) -> (usize, Vec<String>) where
        Self: Sized {
        let completion_map: HashMap<String, _> = Self::completion_map(&words).into_iter()
            .map(|(k, v)| (k.to_string(), v)).collect();

        let first_word = match words.pop() {
            Some(w) => w,
            None => return (0, completion_map.into_iter().map(|(k, _)| k).collect()),
        };

        if !words.is_empty() {
            // if we are here, then we aren't at the last word in the line
            match completion_map.get(&first_word).map(|opt| opt.as_ref()).flatten() {
                Some(f) => f(words, pos),
                None => (pos, vec![]),
            }
        } else {
            // if we are here, then words consists only of first_word
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

impl ReplCompletion for u8 {
    fn completion_map(_words: &Vec<String>)
        -> HashMap<String, Option<fn(Vec<String>, usize) -> (usize, Vec<String>)>> {
        HashMap::new()
    }
}

pub(super) struct ReplHelper;

impl Completer for ReplHelper {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>)
        -> Result<(usize, Vec<Self::Candidate>)> {
        let mut words = if line.ends_with(|c: char| c.is_whitespace()) {
            vec![String::new()]
        } else {
            vec![]
        };
        words.append(&mut line.split_whitespace().rev().map(|s| s.to_string()).collect());

        let (write_pos, mut candidates) = Repl::complete(words, pos);
        for s in &mut candidates {
            s.push(' ');
        }
        Ok((write_pos, candidates))
    }
}

impl Highlighter for ReplHelper {}

impl Hinter for ReplHelper {
    type Hint = String;
}

impl Validator for ReplHelper {}

impl Helper for ReplHelper {}
