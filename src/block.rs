extern crate pest;

use crate::scan::*;
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
  let mut pairs = Lexer::parse(Rule::atx_heading_start, str).ok()?;
  let pair = pairs.next().unwrap();
  let mut size = pair.as_span().end();
  if let Some(line_end) = pair.into_inner().next() {
    let end_span = line_end.as_span();
    let end_size = end_span.end() - end_span.start();
    size -= end_size;
  }
  Some((size))
}

// size, repeat size, meta
pub(crate) fn scan_open_fenced_code(str: &str) -> Option<(usize, usize, &str)> {
  let mut pairs = Lexer::parse(Rule::open_fenced_code, str).ok()?;
  let pair = pairs.next().unwrap();
  let size = pair.as_span().end();
  let mut inner = pair.into_inner();
  let repeat = inner.next().unwrap().as_span().end();
  let meta_span = inner.next().unwrap().as_span();
  let meta = &str[meta_span.start()..meta_span.end()];
  Some((size, repeat, meta))
}

// size, repeat size
pub(crate) fn scan_close_fenced_code(str: &str) -> Option<(usize, usize)> {
  let mut pairs = Lexer::parse(Rule::close_fenced_code, str).ok()?;
  let pair = pairs.next().unwrap();
  let size = pair.as_span().end();
  let repeat = pair.into_inner().next().unwrap().as_span().end();
  Some((size, repeat))
}

// size, level
pub(crate) fn scan_block_quote(bytes: &[u8]) -> Option<(usize, usize)> {
  if bytes.len() > 0 && bytes[0] == b'>' {
    let mut spaces = 0;
    let mut level = 1;
    let size = scan_while(&bytes[1..], |x| match x {
      b'>' => {
        level += 1;
        spaces = 0;
        true
      }
      b' ' => {
        spaces += 1;
        if spaces > 3 {
          false
        } else {
          true
        }
      }
      _ => false,
    }) + 1;
    Some((size, level))
  } else {
    None
  }
}

// size, marker size
pub(crate) fn scan_list_item_start(str: &str) -> Option<(usize, usize)> {
  let mut pairs = Lexer::parse(Rule::list_item_start, str).ok()?;
  let pair = pairs.next().unwrap();
  let mut size = pair.as_span().end();
  let mut inner = pair.into_inner();
  let marker_size = inner.next().unwrap().as_span().end();
  if let Some(line_end) = inner.next() {
    let end_span = line_end.as_span();
    let end_size = end_span.end() - end_span.start();
    size -= end_size;
  }
  Some((size, marker_size))
}

pub(crate) fn scan_setext_heading(str: &str) -> Option<usize> {
  search(Rule::setext_heading, str)
}

pub(crate) fn scan_thematic_break(str: &str) -> Option<usize> {
  search(Rule::thematic_break, str)
}

#[test]
fn test_scan_atx_heading_start() {
  println!("{:?}", scan_atx_heading_start("#\n"));
  println!("{:?}", scan_atx_heading_start("# \n"));
  println!("{:?}", scan_atx_heading_start("# "));
  println!("{:?}", scan_open_fenced_code("```"));
  println!("{:?}", scan_close_fenced_code("````   \n"));
  println!("{:?}", scan_close_fenced_code("````   "));
  println!("{:?}", scan_close_fenced_code("``   "));
}
