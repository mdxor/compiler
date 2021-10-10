use crate::jsx;
#[cfg(test)]
use serde::Serialize;

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct AST<'a> {
  pub blocks: Vec<Block<'a>>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct Heading<'a> {
  pub level: u8,
  pub inlines: Vec<Inline<'a>>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct Paragraph<'a> {
  pub inlines: Vec<Inline<'a>>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub enum Inline<'a> {
  BackslashEscape,
  CodeSpan,
  StrikeThrough,
  Emphasis,
  Strong,
  Link,
  AutoLink,
  HardLineBreak,
  SoftLineBreak,
  Text(&'a str),
  JSX(jsx::JSXNode<'a>),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct FencedCode<'a> {
  pub code: &'a str,
  pub language: &'a str,
  pub metastring: &'a str,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct BlockQuote<'a> {
  blocks: Vec<Block<'a>>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub enum Block<'a> {
  Container(ContainerBlock<'a>),
  Leaf(LeafBlock<'a>),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub enum ContainerBlock<'a> {
  BlockQuote(BlockQuote<'a>),
  ListItem,
  List,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub enum LeafBlock<'a> {
  ThematicBreak,
  Heading(Heading<'a>),
  IndentedCode(&'a str),
  FencedCode(FencedCode<'a>),
  JSX(jsx::JSXNode<'a>),
  Paragraph(Paragraph<'a>),
  Table,
}
