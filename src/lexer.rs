use crate::token;
use lazy_static::lazy_static;
use regex::Regex;

struct Lexer<'a> {
  source: &'a str,
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str) -> Self {
    Lexer { source }
  }

  fn skip_whitespace(&mut self) {
    lazy_static! {
      static ref WHITESPACE_REGEX: Regex = Regex::new("^ *").unwrap();
    }
    if let Some(caps) = WHITESPACE_REGEX.captures(self.source) {
      let size = caps.get(0).unwrap().as_str().len();
      self.move_by(size);
    }
  }

  fn move_by(&mut self, size: usize) -> &'a str {
    let result = &self.source[..size];
    self.source = &self.source[size..];
    result
  }

  fn tokenize(&mut self) -> Result<Vec<token::BlockToken>, &'a str> {
    let mut tokens = vec![];
    Ok(tokens)
  }

  fn scan_jsx(&mut self, is_inline: bool) {}
}
