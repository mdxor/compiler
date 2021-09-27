use crate::jsx;
#[cfg(test)]
use serde::Serialize;

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct Heading<'a> {
  pub level: u8,
  pub content: Vec<InlineBlock<'a>>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub enum BlockToken<'a> {
  Heading(Heading<'a>),
  Newline,
  // single line code
  SCode(&'a str),
  // multi line code
  MCode(&'a str),
  // TODO: list & table
  BulletList,
  OrderedList,
  TaskList,
  Hr,
  Table,
  JSX(jsx::JSXNode<'a>),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub enum ListLegalBlockToken<'a> {
  Heading(Heading<'a>),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct Img<'a> {
  alt: &'a str,
  url: &'a str,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct Link<'a> {
  label: &'a str,
  url: &'a str,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub enum InlineBlock<'a> {
  Img(Img<'a>),
  Link(Link<'a>),
  Text(&'a str),
  Code(&'a str),
  JSX(jsx::JSXNode<'a>),
}
