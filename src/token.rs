#[cfg(test)]
use serde::Serialize;
#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub struct Token<'source> {
  pub start: usize,
  pub end: usize,
  pub body: TokenBody<'source>,
}

impl<'source> Default for Token<'source> {
  fn default() -> Self {
    Token {
      start: 0,
      end: 0,
      body: TokenBody::Root,
    }
  }
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub enum HeadingLevel {
  H1,
  H2,
  H3,
  H4,
  H5,
  H6,
}

impl HeadingLevel {
  pub fn new(level: usize) -> Option<HeadingLevel> {
    match level {
      1 => Some(HeadingLevel::H1),
      2 => Some(HeadingLevel::H2),
      3 => Some(HeadingLevel::H3),
      4 => Some(HeadingLevel::H4),
      5 => Some(HeadingLevel::H5),
      6 => Some(HeadingLevel::H6),
      _ => None,
    }
  }
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub enum TokenBody<'source> {
  // transition token body
  Raw(&'source str),
  // final token body
  Root,
  Paragraph,
  Text(&'source str),
  ATXHeading(HeadingLevel),
  SetextHeading(HeadingLevel),
  IndentedCode,
  ThematicBreak,
  Code(&'source str),
  BlankLine,
  BlockQuote(usize),
}
