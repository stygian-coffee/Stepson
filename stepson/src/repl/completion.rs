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

pub struct CompletionTree { }

impl CompletionTree {
    fn traverse(&self, words: Vec<String>) -> Vec<String> {
        unimplemented!()
    }
}

pub trait ReplCompletion {
    fn completion_tree(cx: &CompletionContext) -> CompletionTree;
}

pub trait ReplCompletionStateful {
    fn completion_tree(&self, cx: &CompletionContext) -> CompletionTree;
}

// impl ReplCompletion for u8 {
//     fn completion_map(_cx: &CompletionContext) -> Vec<(String, Option<CompleteMethod>)> {
//         vec![]
//     }
// }

pub(super) struct ReplHelper {
    data: Rc<RefCell<ReplData>>,
}

impl Completer for ReplHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
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

        let mut completion_tree = self.data.borrow().completion_tree(&cx);
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
