use std::cell::RefCell;
use std::rc::Rc;

use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};

use super::ReplData;

type CompletionBranch = (String, Box<dyn FnOnce() -> CompletionTree>);

pub struct CompletionTree {
    pub branches: Vec<CompletionBranch>,
}

impl CompletionTree {
    pub fn new(branches: Vec<CompletionBranch>) -> Self {
        Self { branches }
    }

    pub fn empty() -> Self {
        Self { branches: vec![] }
    }

    pub fn lazy_empty() -> Box<fn() -> Self> {
        Box::new(|| Self::empty())
    }

    fn traverse(self, mut words: Vec<String>) -> Vec<String> {
        let first_word = match words.pop() {
            Some(w) => w,
            None => return self.branches.into_iter().map(|(s, _)| s.clone()).collect(),
        };

        if words.is_empty() {
            // if that was the last word
            self.branches
                .into_iter()
                .filter_map(|(s, _)| {
                    if s.starts_with(&first_word) {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            match self.branches.into_iter().find(|(s, _)| s == &first_word) {
                Some((_, f)) => f().traverse(words),
                None => vec![],
            }
        }
    }
}

pub trait ReplCompletion {
    fn completion_tree() -> CompletionTree;

    fn lazy_completion_tree() -> Box<dyn FnOnce() -> CompletionTree> {
        Box::new(|| Self::completion_tree())
    }
}

pub trait ReplCompletionStateful {
    fn lazy_completion_tree(&self) -> Box<dyn FnOnce() -> CompletionTree>;
}

impl ReplCompletion for u8 {
    fn completion_tree() -> CompletionTree {
        CompletionTree::empty()
    }
}

pub(super) struct ReplHelper {
    pub data: Rc<RefCell<ReplData>>,
}

impl Completer for ReplHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
        // The words in the line in reverse order, starting with an empty "word" if the line
        // ends with whitespace.
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

        let last_word = match words.get(0) {
            Some(w) => w.clone(),
            None => String::new(),
        };

        let completion_tree = self.data.borrow().lazy_completion_tree()();
        let mut candidates = completion_tree.traverse(words);

        for s in &mut candidates {
            s.push(' ');
        }
        Ok((pos - last_word.len(), candidates))
    }
}

impl Highlighter for ReplHelper {}

impl Hinter for ReplHelper {
    type Hint = String;
}

impl Validator for ReplHelper {}

impl Helper for ReplHelper {}
