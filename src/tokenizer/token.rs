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
