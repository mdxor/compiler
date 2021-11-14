#[cfg(test)]
use serde::Serialize;

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct TokenResult<'source> {
  pub size: usize,
  pub token: Token<'source>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub enum Token<'source> {
  ATXHeading(ATXHeading<'source>),
  ThematicBreak,
  SetextHeading(SetextHeading),
  IndentedCode(IndentedCode<'source>),
  FencedCode(FencedCode<'source>),
  LinkDefinition(LinkDefinition<'source>),
  BlockQuote,
  BulletListItem,
  OrderedListItem(OrderedListItem),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct ATXHeading<'source> {
  pub level: usize,
  pub raw_inlines: &'source str,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct SetextHeading {
  pub level: usize,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct IndentedCode<'source> {
  pub codes: Vec<&'source str>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct FencedCode<'source> {
  pub code: &'source str,
  pub meta_string: &'source str,
  pub language: &'source str,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct LinkDefinition<'source> {
  pub href: &'source str,
  pub title: &'source str,
  pub label: &'source str,
}
#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct OrderedListItem {
  pub order: u32,
}
