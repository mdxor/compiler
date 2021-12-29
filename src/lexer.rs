extern crate nom;
// extern crate pest;
use crate::scan::*;
use nom::{
  branch::alt,
  bytes::complete::{tag, take_while, take_while_m_n},
  character::complete::{alpha1, char, line_ending, not_line_ending, space0, space1},
  character::is_digit,
  combinator::eof,
  combinator::map_res,
  multi::many1,
  sequence::{terminated, tuple},
  IResult,
};
// use pest::Parser;

// #[derive(Parser)]
// #[grammar = "mdx.pest"]
// struct Lexer;

// fn search(rule: Rule, str: &str) -> Option<usize> {
//   if let Ok(pairs) = Lexer::parse(rule, str) {
//     Some(pairs.last().unwrap().as_span().end())
//   } else {
//     None
//   }
// }
fn atx_heading_start(input: &str) -> IResult<&str, (&str, &str)> {
  tuple((
    take_while_m_n(1, 6, |c| c == '#'),
    alt((line_ending, tag(" "), eof)),
  ))(input)
}
// size, level
pub(crate) fn scan_atx_heading_start(input: &str) -> Option<(usize, usize)> {
  if let Ok((_, (opening, follow))) = atx_heading_start(input) {
    return Some((opening.len() + follow.len(), opening.len()));
  }
  None
}

fn open_fenced_code(input: &str) -> IResult<&str, ((&str, &str, &str), &str)> {
  tuple((
    alt((
      tuple((
        tag("```"),
        take_while(|c| c == '`'),
        take_while(|c| c != '`' && c != '\r' && c != '\n'),
      )),
      tuple((
        tag("~~~"),
        take_while(|c| c == '~'),
        take_while(|c| c != '\r' && c != '\n'),
      )),
    )),
    alt((line_ending, space0, eof)),
  ))(input)
}
// size, repeat size, meta
pub(crate) fn scan_open_fenced_code(input: &str) -> Option<(usize, usize, &str)> {
  if let Ok((next_input, ((fenced, fenced_rest, meta), _))) = open_fenced_code(input) {
    Some((
      input.len() - next_input.len(),
      fenced.len() + fenced_rest.len(),
      meta.trim(),
    ))
  } else {
    None
  }
}

fn close_fenced_code(input: &str) -> IResult<&str, (&str, &str)> {
  tuple((
    alt((take_while(|c| c == '`'), take_while(|c| c == '~'))),
    alt((line_ending, space0, eof)),
  ))(input)
}
// size, repeat size
pub(crate) fn scan_close_fenced_code(input: &str) -> Option<(usize, usize)> {
  if let Ok((next_input, (fenced, _))) = close_fenced_code(input) {
    return Some((input.len() - next_input.len(), fenced.len()));
  } else {
    None
  }
}

fn block_quote(input: &str) -> IResult<&str, Vec<&str>> {
  many1(terminated(tag(">"), take_while_m_n(0, 3, |c| c != ' ')))(input)
}
// size, level
pub(crate) fn scan_block_quote(input: &str) -> Option<(usize, usize)> {
  if let Ok((next_input, matches)) = block_quote(input) {
    let last = matches.last().unwrap();
    let spaces = last.len() - last.trim_end().len();
    let size = input.len() - next_input.len() - if spaces > 1 { spaces - 1 } else { 0 };
    Some((size, matches.len()))
  } else {
    None
  }
}

fn list_item_start(input: &str) -> IResult<&str, ((&str, &str), &str)> {
  tuple((
    alt((
      tuple((tag("-"), tag(""))),
      tuple((tag("*"), tag(""))),
      tuple((tag("+"), tag(""))),
      tuple((
        take_while(|v| v > '0' && v < '9'),
        alt((tag("."), tag(")"))),
      )),
    )),
    alt((line_ending, eof, tag(" "))),
  ))(input)
}
// size, marker size
pub(crate) fn scan_list_item_start(input: &str) -> Option<(usize, usize)> {
  if let Ok((next_input, (_, close))) = list_item_start(input) {
    let size = input.len() - next_input.len();
    Some((size, size - close.len()))
  } else {
    None
  }
}

fn setext_heading(input: &str) -> IResult<&str, (&str, &str, &str)> {
  tuple((
    alt((take_while(|c| c == '='), take_while(|c| c == '-'))),
    space0,
    alt((line_ending, eof)),
  ))(input)
}
pub(crate) fn scan_setext_heading(input: &str) -> Option<usize> {
  if let Ok((next_input, _)) = setext_heading(input) {
    Some(input.len() - next_input.len())
  } else {
    None
  }
}

fn thematic_break(input: &str) -> IResult<&str, (Vec<(char, &str)>, &str)> {
  tuple((
    alt((
      many1(tuple((char('*'), space0))),
      many1(tuple((char('-'), space0))),
      many1(tuple((char('_'), space0))),
    )),
    alt((line_ending, eof)),
  ))(input)
}
pub(crate) fn scan_thematic_break(input: &str) -> Option<usize> {
  if let Ok((next_input, _)) = thematic_break(input) {
    Some(input.len() - next_input.len())
  } else {
    None
  }
}

fn blank_line(input: &str) -> IResult<&str, (&str, &str)> {
  tuple((space0, alt((line_ending, eof))))(input)
}
pub(crate) fn scan_blank_line(input: &str) -> Option<usize> {
  if let Ok((next_input, _)) = blank_line(input) {
    Some(input.len() - next_input.len())
  } else {
    None
  }
}
#[test]
fn test_scan_atx_heading_start() {
  // println!("{:?}", scan_atx_heading_start("# 123"));
  println!("{:?}", scan_open_fenced_code("123```\n"));
  // println!("{:?}", scan_list_item_start("- 123"));
  // println!("{:?}", scan_close_fenced_code("```   "));
  // println!("{:?}", scan_blank_line(""));
  // println!("{:?}", scan_setext_heading("===== "));
}
