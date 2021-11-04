use crate::jsx;
#[cfg(test)]
use serde::Serialize;

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
  JSX,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub enum Block {
  Container(ContainerBlock),
  Leaf(LeafBlock),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub enum ContainerBlock {
  BlockQuote,
  ListItem,
  List,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub enum LeafBlock {
  ThematicBreak,
  Heading1,
  Heading2,
  Heading3,
  Heading4,
  Heading5,
  Heading6,
  IndentedCode,
  FencedCode,
  JSX,
  Paragraph,
  BlankLine,
  Table,
}
