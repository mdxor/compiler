use crate::jsx;
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

  fn skip_whitespace(&mut self) -> usize {
    lazy_static! {
      static ref WHITESPACE_REGEX: Regex = Regex::new("^ +").unwrap();
    }
    if let Some(caps) = WHITESPACE_REGEX.captures(self.source) {
      let size = caps.get(0).unwrap().as_str().len();
      self.move_by(size);
      size
    } else {
      0
    }
  }

  fn move_by(&mut self, size: usize) -> &'a str {
    let result = &self.source[..size];
    self.source = &self.source[size..];
    result
  }

  fn tokenize(&mut self) -> Result<Vec<token::BlockToken>, &'a str> {
    let mut tokens = vec![];
    loop {
      if self.source.is_empty() {
        break;
      } else {
        let token = self.scan_block_token()?;
        tokens.push(token);
      }
    }
    Ok(tokens)
  }

  fn scan_block_token(&mut self) -> Result<token::BlockToken<'a>, &'a str> {
    lazy_static! {
      static ref HEADING_START_REGEX: Regex = Regex::new("^(#{1,6}) ").unwrap();
    }
    if self.source.starts_with("    ") {
      self.move_by(4);
      return Ok(self.scan_single_line_code());
    } else {
      self.skip_whitespace();
      if let Some(caps) = HEADING_START_REGEX.captures(self.source) {
        let size = caps.get(1).unwrap().as_str().len();
        self.move_by(size);
        return self.scan_heading(size as u8);
      }
    }
    Err("")
  }

  fn scan_heading(&mut self, level: u8) -> Result<token::BlockToken<'a>, &'a str> {
    let content = self.scan_inline_blocks()?;
    let heading = token::Heading { level, content };
    Ok(token::BlockToken::Heading(heading))
  }

  fn scan_inline_blocks(&mut self) -> Result<Vec<token::InlineBlock<'a>>, &'a str> {
    Ok(vec![])
  }

  fn scan_by_end_char(&mut self, end_char: char) -> &'a str {
    let mut size = 0;
    let mut chars = self.source.chars();
    loop {
      if let Some(char) = chars.next() {
        if char == end_char || char == '\n' {
          break;
        } else {
          size += char.len_utf8();
        }
      } else {
        break;
      }
    }
    self.move_by(size)
  }

  fn scan_single_line_code(&mut self) -> token::BlockToken<'a> {
    let code = self.scan_by_end_char('\n');
    token::BlockToken::SCode(code)
  }

  fn scan_jsx(&mut self, is_inline: bool) -> Result<jsx::JSXNode<'a>, &'a str> {
    let mut jsx_parser = jsx::JSXParser::new(self.source, 0, is_inline);
    let jsx_node = jsx_parser.parse();
    self.move_by(jsx_parser.size);
    jsx_node
  }
}
