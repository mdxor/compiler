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

  fn skip_by_regex(&mut self, regex: &regex::Regex) {
    if let Some(caps) = regex.captures(self.source) {
      let skip_str = caps.get(0).map_or("", |m| m.as_str());
      if !skip_str.is_empty() {
        self.move_by(skip_str.len());
      }
    }
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

  fn skip_block_start_whitespace(&mut self) {
    lazy_static! {
      static ref RE: Regex = Regex::new(r"^ {0,3}").unwrap();
    }
    self.skip_by_regex(&*RE);
  }

  fn move_by(&mut self, size: usize) -> &'a str {
    let result = &self.source[..size];
    self.source = &self.source[size..];
    result
  }

  fn tokenize(&mut self) -> Result<Vec<token::Token>, &'a str> {
    Err("")
  }

  fn scan_jsx(&mut self, is_inline: bool) {}
}
