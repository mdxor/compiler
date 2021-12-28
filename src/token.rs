#[cfg(test)]
use serde::Serialize;
#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub struct Item<T> {
  pub start: usize,
  pub end: usize,
  pub value: T,
}

impl<T: Default> Default for Item<T> {
  fn default() -> Self {
    Item {
      start: 0,
      end: 0,
      value: <T>::default(),
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

  pub fn to_str(&self) -> &str {
    match self {
      HeadingLevel::H1 => "h1",
      HeadingLevel::H2 => "h2",
      HeadingLevel::H3 => "h3",
      HeadingLevel::H4 => "h4",
      HeadingLevel::H5 => "h5",
      HeadingLevel::H6 => "h6",
    }
  }
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub struct FencedCode<'source> {
  pub language: &'source str,
  pub meta: &'source str,
  // ` or ~
  pub keyword: u8,
  pub keyword_size: usize,
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub struct LinkDefinition<'source> {
  pub label: &'source str,
  pub url: &'source str,
  pub title: String,
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub enum Align {
  Left,
  Center,
  Right,
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub enum Token<'source> {
  Raw(&'source str),
  Root,
  Paragraph,
  ATXHeading(HeadingLevel),
  SetextHeading(HeadingLevel),
  IndentedCode(usize),
  ThematicBreak,
  Code(&'source str),
  BlankLine,
  BlockQuote(usize),
  FencedCode,
  List(u8, bool, &'source str), // list character, is_tight, ordered index
  ListItem(usize),              // indent
  LinkDefinition,
  Table,
  TableHead,
  TableCell(&'source str, bool, bool),
  TableAlignment,
  TableAlign(Align),
  TableRow,
}

impl<'source> Default for Token<'source> {
  fn default() -> Self {
    Token::Root
  }
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub enum InlineToken<'source> {
  Root,
  Text(&'source str),
  MaybeLinkStart,
  // keyword, repeat, can open, can close
  MaybeEmphasis(u8, usize, bool, bool),
  // keyword, repeat
  EmphasisStart(u8, usize),
  EmphasisEnd(u8, usize),
  InlineCodeStart,
  InlineCodeEnd,
  Code,
}

impl<'source> Default for InlineToken<'source> {
  fn default() -> Self {
    InlineToken::Root
  }
}
