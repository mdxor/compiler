#[cfg(test)]
use serde::Serialize;

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub struct ATXHeading<'source> {
  pub level: usize,
  pub raw_inlines: &'source str,
}
