use std::cell::RefCell;
use std::rc::Rc;

use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};

use super::ReplData;

#[derive(Debug)]
pub struct CompletionContext {
    pub all_words: Vec<String>,
    pub current_pos: usize,
}

type CompletionBranch = (String, Box<dyn FnOnce() -> CompletionTree>);

pub struct CompletionTree {
    branches: Vec<CompletionBranch>,
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
    fn completion_tree(cx: std::rc::Rc<CompletionContext>) -> CompletionTree;

    fn lazy_completion_tree(cx: Rc<CompletionContext>) -> Box<dyn FnOnce() -> CompletionTree> {
        Box::new(|| Self::completion_tree(cx))
    }
}

pub trait ReplCompletionStateful {
    fn completion_tree(&self, cx: std::rc::Rc<CompletionContext>) -> CompletionTree;
}

impl ReplCompletion for u8 {
    fn completion_tree(_cx: std::rc::Rc<CompletionContext>) -> CompletionTree {
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

        let cx = CompletionContext {
            all_words: words.clone(),
            current_pos: 0,
        };

        let completion_tree = self.data.borrow().completion_tree(Rc::new(cx));
        let mut candidates = completion_tree.traverse(words);
        for s in &mut candidates {
            s.push(' ');
        }
        Ok((pos, candidates))
    }
}

impl Highlighter for ReplHelper {}

impl Hinter for ReplHelper {
    type Hint = String;
}

impl Validator for ReplHelper {}

impl Helper for ReplHelper {}
