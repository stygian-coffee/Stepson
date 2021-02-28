use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};

use super::Repl;

#[derive(Debug)]
pub struct CompletionContext {
    pub all_words: Vec<String>,
    pub current_pos: usize,
}

pub type CompleteMethod = fn(Vec<String>, usize, CompletionContext) -> (usize, Vec<String>);
pub trait ReplCompletion {
    fn completion_map(cx: &CompletionContext) -> Vec<(String, Option<CompleteMethod>)>
    where
        Self: Sized;

    fn complete(
        mut words: Vec<String>,
        pos: usize,
        mut cx: CompletionContext,
    ) -> (usize, Vec<String>)
    where
        Self: Sized,
    {
        let completion_map = Self::completion_map(&cx);

        let first_word = match words.pop() {
            Some(w) => w,
            None => return (0, completion_map.into_iter().map(|(k, _)| k).collect()),
        };

        if !words.is_empty() {
            // if we are here, then we aren't at the last word in the line
            match completion_map
                .into_iter()
                .find(|(k, _)| k == &first_word)
                .map(|(_, v)| v)
                .flatten()
            {
                Some(f) => {
                    cx.current_pos += 1;
                    f(words, pos, cx)
                }
                None => (pos, vec![]),
            }
        } else {
            // if we are here, then words consists only of first_word
            (
                pos - first_word.len(),
                completion_map
                    .into_iter()
                    .filter_map(|(k, _)| match k.starts_with(&first_word) {
                        true => Some(k),
                        false => None,
                    })
                    .collect(),
            )
        }
    }
}

impl ReplCompletion for u8 {
    fn completion_map(_cx: &CompletionContext) -> Vec<(String, Option<CompleteMethod>)> {
        vec![]
    }
}

pub(super) struct ReplHelper;

impl Completer for ReplHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
        let mut words = if line.ends_with(|c: char| c.is_whitespace()) {
            vec![String::new()]
        } else {
            vec![]
        };
        words.append(
            &mut line
                .split_whitespace()
                .rev()
                .map(|s| s.to_string())
                .collect(),
        );

        let cx = CompletionContext {
            all_words: words.clone(),
            current_pos: 0,
        };

        let (write_pos, mut candidates) = Repl::complete(words, pos, cx);
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
