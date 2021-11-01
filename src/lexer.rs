use crate::jsx;
use crate::rule::Rule;
use crate::token;
use lazy_static::lazy_static;

lazy_static! {
  static ref RULE: Rule = Rule::new();
}

pub struct Lexer<'a> {
  _source: &'a str,
  offset: usize,
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str) -> Self {
    Lexer {
      _source: source,
      offset: 0,
    }
  }

  fn source(&mut self) -> &'a str {
    &self._source[self.offset..]
  }

  fn read_block(&mut self) {}
}
