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

lazy_static! {
  static ref KEYCHARSET: HashSet<&'static char> = {
    let keyCharSet: HashSet<&'static char> = ['#', '\\', '`'].into_iter().collect();
    keyCharSet
  };
}

struct Lexer<'a> {
  source: &'a str,
  chars: std::str::Chars<'a>,
  state: State,
  token: String,
  escaped: bool,
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str) -> Self {
    Lexer {
      source,
      chars: source.chars(),
      state: State::Block,
      token: String::new(),
      escaped: false,
    }
  }

  fn skip_by_captures(&mut self, captures: regex::Captures) {
    let skip_str = captures.get(0).map_or("", |m| m.as_str());
    self.move_by(skip_str.len());
  }

  fn skip_whitespace(&mut self) {
    lazy_static! {
      static ref RE: Regex = Regex::new(r"^ *").unwrap();
    }
    self.skip_by_captures(RE.captures(self.source).unwrap());
  }

  fn skip_block_start_whitespace(&mut self) {
    lazy_static! {
      static ref RE: Regex = Regex::new(r"^ {0,3}").unwrap();
    }
    self.skip_by_captures(RE.captures(self.source).unwrap());
  }

  fn move_by(&mut self, size: usize) -> &'a str {
    let result = &self.source[..size + 1];
    self.source = &self.source[size + 1..];
    result
  }

  fn move_by_str(&mut self, str: &'a str) -> &'a str {
    self.move_by(str.len())
  }

  fn next_token(&mut self) -> Option<token::Token> {
    if self.source.is_empty() {
      None
    } else {
      match self.state {
        State::Block => self.next_block_token(),
        _ => None,
      }
    }
  }

  fn next_block_token(&mut self) -> Option<token::Token> {
    self.state = State::Inline;
    self.skip_block_start_whitespace();
    lazy_static! {
      static ref RE: Regex = Regex::new(r"^(?:[^ \n])*").unwrap();
    }
    let caps = RE.captures(self.source).unwrap();
    let match_str = caps.get(0).map_or("", |m| m.as_str());
    if let Some(token) = token::match_block_token(match_str) {
      self.move_by_str(match_str);
      Some(token)
    } else {
      self.next_token()
    }
  }
}

// #[cfg(test)]
// mod tests {
//   use super::*;

//   fn lexer(code: &str) -> Vec<token::Token> {
//     let mut lexer = Lexer::new(code);
//     let mut tokens = vec![];
//     loop {
//       if let Some(token) = lexer.next_token() {
//         tokens.push(token)
//       } else {
//         break;
//       }
//     }
//     tokens
//   }
//   #[test]
//   fn parse_1() {
//     println!("{}", "早n".find("n").unwrap());
//     println!("{}", '早'.len_utf8());
//     assert_eq!(
//       lexer("# 123 #\n123"),
//       vec![
//         token::Token::Heading1,
//         token::Token::Text("123"),
//         token::Token::Heading1,
//         token::Token::Text("123"),
//       ]
//     );
//   }
// }
