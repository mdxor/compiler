use crate::jsx;
#[cfg(test)]
use serde::Serialize;

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct AST<'a> {
  pub blocks: Vec<BlockToken<'a>>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct Heading<'a> {
  pub level: u8,
  pub content: Vec<InlineBlock<'a>>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct MCode<'a> {
  pub code: &'a str,
  pub language: &'a str,
  pub metastring: &'a str,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub enum BlockToken<'a> {
  Heading(Heading<'a>),
  Newline,
  // single line code
  SCode(&'a str),
  // multi line code
  MCode(MCode<'a>),
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
