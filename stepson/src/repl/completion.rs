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

type CompletionBranch = (String, CompletionTree);

pub struct CompletionTree {
    branches: Box<dyn Iterator<Item = CompletionBranch>>,
}

impl CompletionTree {
    pub fn new(branches: Vec<CompletionBranch>) -> Self {
        Self {
            branches: Box::new(branches.into_iter()),
        }
    }

    pub fn empty() -> Self {
        Self {
            branches: Box::new(vec![].into_iter()),
        }
    }

    fn traverse(&self, words: Vec<String>) -> Vec<String> {
        unimplemented!()
    }
}

impl Iterator for CompletionTree {
    type Item = CompletionBranch;

    fn next(&mut self) -> Option<Self::Item> {
        self.branches.next()
    }
}

pub trait ReplCompletion {
    fn completion_tree(cx: &CompletionContext) -> CompletionTree;
}

pub trait ReplCompletionStateful {
    fn completion_tree(&self, cx: &CompletionContext) -> CompletionTree;
}

impl ReplCompletion for u8 {
    fn completion_tree(_cx: &CompletionContext) -> CompletionTree {
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

        let completion_tree = self.data.borrow().completion_tree(&cx);
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
