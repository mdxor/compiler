use crate::jsx;
use crate::rule::Rule;
use crate::token;
use lazy_static::lazy_static;

lazy_static! {
  static ref RULE: Rule = Rule::new();
}

pub struct Lexer<'a> {
  _source: &'a str,
  _bytes: &'a [u8],
  offset: usize,
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str) -> Self {
    Lexer {
      _source: source,
      _bytes: source.as_bytes(),
      offset: 0,
    }
  }

  fn source(&mut self) -> &'a str {
    &self._source[self.offset..]
  }

  fn cur(&mut self) -> u8 {
    self._bytes[self.offset]
  }

  fn move_by(&mut self, size: usize) -> &'a str {
    let result = &self.source()[0..size];
    self.offset += size;
    result
  }

  fn read_block(&mut self) {}

  fn read_inline(&mut self, multiple_lines: bool) {}
}
