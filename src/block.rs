extern crate pest;

use pest::Parser;

#[derive(Parser)]
#[grammar = "mdx.pest"]
struct Lexer;

fn search(rule: Rule, str: &str) -> Option<usize> {
  if let Ok(pairs) = Lexer::parse(rule, str) {
    Some(pairs.last().unwrap().as_span().end())
  } else {
    None
  }
}

pub(crate) fn scan_atx_heading_start(str: &str) -> Option<usize> {
  search(Rule::atx_heading_start, str)
}

pub(crate) fn scan_open_fenced_code(str: &str) -> Option<usize> {
  search(Rule::open_code_fence, str)
}
