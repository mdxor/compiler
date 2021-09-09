use crate::token;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;
enum State {
  Block,
  Inline,
  CodeBlock,
  JSXBlock,
  JSXInline,
}

struct Lexer<'a> {
  source: &'a str,
  state: State,
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str) -> Self {
    Lexer {
      source,
      state: State::Block,
    }
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
      static ref RE: Regex = Regex::new(r"^ *").unwrap();
    }
    self.skip_by_regex(&*RE);
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

  fn move_by_str(&mut self, str: &'a str) -> &'a str {
    self.move_by(str.len())
  }

  fn next_token(&mut self) -> Option<token::Token<'a>> {
    if self.source.is_empty() {
      None
    } else {
      match self.state {
        State::Block => self.next_block_token(),
        State::Inline => self.next_inline_token(),
        _ => None,
      }
    }
  }

  fn next_block_token(&mut self) -> Option<token::Token<'a>> {
    self.state = State::Inline;
    self.skip_block_start_whitespace();
    lazy_static! {
      static ref RE: Regex = Regex::new(r"^(?:[^ \n])*").unwrap();
    }
    let caps = RE.captures(self.source).unwrap();
    let match_str = caps.get(0).map_or("", |m| m.as_str());
    if let Some(token) = token::match_block_token(match_str) {
      self.move_by_str(match_str);
      // TODO
      self.skip_whitespace();
      Some(token)
    } else {
      self.next_token()
    }
  }

  fn next_inline_token(&mut self) -> Option<token::Token<'a>> {
    let mut chars = self.source.chars();
    let mut token_size: usize = 0;
    let mut escaped = false;
    let mut is_text = false;
    loop {
      if let Some(char) = chars.next() {
        if escaped {
          token_size += char.len_utf8() + 1;
        } else {
          match char {
            '\n' => {
              if token_size == 0 {
                self.move_by(1);
                return Some(token::Token::Newline);
              }
              break;
            }
            '\\' => {
              escaped = true;
              token_size += 1;
            }
            _ => token_size += char.len_utf8(),
          }
        }
      } else {
        break;
      }
    }
    if token_size > 0 {
      Some(token::Token::Text(self.move_by(token_size)))
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn lexer(source: &str) -> Vec<token::Token> {
    let mut _lexer = Lexer::new(source);
    let mut tokens = vec![];
    loop {
      if let Some(token) = _lexer.next_token() {
        tokens.push(token)
      } else {
        break;
      }
    }
    tokens
  }
  #[test]
  fn parse_1() {
    let re = regex::Regex::new(r"abc").unwrap();
    assert_eq!(
      lexer("# 123\n123"),
      vec![
        token::Token::Heading1,
        token::Token::Text("123"),
        token::Token::Newline,
        token::Token::Text("123"),
      ]
    );
  }
}
