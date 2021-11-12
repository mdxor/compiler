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
  SetextHeading(SetextHeading<'source>),
  IndentedCode(IndentedCode<'source>),
  FencedCode(FencedCode<'source>),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct ATXHeading<'source> {
  pub level: usize,
  pub raw_inlines: &'source str,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct SetextHeading<'source> {
  pub level: usize,
  pub raw: &'source str,
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
