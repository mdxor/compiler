use crate::jsx;
use crate::rule::Rule;
use crate::token;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
  static ref RULE: Rule = Rule::new();
}

pub struct Lexer<'a> {
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

  pub fn tokenize(&mut self) -> Result<token::AST<'a>, &'a str> {
    let mut blocks = vec![];
    loop {
      if self.source.is_empty() {
        break;
      } else {
        let token = self.scan_block()?;
        blocks.push(token);
      }
    }
    let ast = token::AST { blocks };
    Ok(ast)
  }

  fn scan_block(&mut self) -> Result<token::Block<'a>, &'a str> {
    if RULE.indented_code.is_match(self.source) {
      self.move_by(4);
      return Ok(self.scan_single_line_code());
    } else {
      // <=3 whitespace
      self.skip_whitespace();
      if let Some(caps) = RULE.empty_atx_heading.captures(self.source) {
        let size = caps.get(1).unwrap().as_str().len();
        self.move_by(caps.get(0).unwrap().as_str().len());
        return Ok(token::Block::Leaf(token::LeafBlock::Heading(
          token::Heading {
            level: size as u8,
            inlines: vec![],
          },
        )));
      }
      if let Some(caps) = RULE.atx_heading.captures(self.source) {
        let size = caps.get(1).unwrap().as_str().len();
        return self.scan_heading(size);
      } else if self.source.starts_with("```") {
        return Ok(self.scan_multiple_line_code());
      }
      return self.scan_paragraph();
    }
  }

  fn scan_paragraph(&mut self) -> Result<token::Block<'a>, &'a str> {
    let inlines = self.scan_inlines()?;
    Ok(token::Block::Leaf(token::LeafBlock::Paragraph(
      token::Paragraph { inlines },
    )))
  }

  fn scan_heading(&mut self, level: usize) -> Result<token::Block<'a>, &'a str> {
    self.move_by(level + 1);
    let mut inlines = self.scan_inlines()?;

    // A closing sequence of # characters is optional:
    // # Title ### => <h1>Title<h1/>
    if !inlines.is_empty() {
      let last_inline = inlines.pop().unwrap();
      let mut closing_size = 0;
      if let token::Inline::Text(text) = last_inline {
        if let Some(caps) = RULE.closing_atx_heading.captures(text) {
          closing_size = caps.get(0).unwrap().as_str().len();
          inlines.push(token::Inline::Text(&text[..text.len() - closing_size]));
        }
      }
      if closing_size == 0 {
        inlines.push(last_inline);
      }
    }
    let heading = token::Heading {
      level: level as u8,
      inlines,
    };
    Ok(token::Block::Leaf(token::LeafBlock::Heading(heading)))
  }

  fn scan_inlines(&mut self) -> Result<Vec<token::Inline<'a>>, &'a str> {
    self.skip_whitespace();
    let mut inlines = vec![];
    loop {
      if self.source.is_empty() || self.source.starts_with("\n") {
        if self.source.starts_with("\n") {
          self.move_by(1);
        }
        return Ok(inlines);
      } else if self.source.starts_with("`") {
      } else {
        inlines.append(&mut self.scan_inline_text());
      }
    }
  }

  fn scan_inline_text(&mut self) -> Vec<token::Inline<'a>> {
    let mut chars = self.source.chars();
    let mut text_size = 0;
    let mut texts = vec![];
    let mut escaped = false;
    loop {
      if let Some(char) = chars.next() {
        if !escaped {
          match char {
            '\n' | '`' => {
              texts.push(token::Inline::Text(self.move_by(text_size)));
              break;
            }
            '\\' => {
              escaped = !escaped;
            }
            _ => {
              text_size += char.len_utf8();
            }
          }
        } else {
          match char {
            '\n' => {
              if text_size > 0 {
                texts.push(token::Inline::Text(self.move_by(text_size + 1)));
              }
              break;
            }
            '\\' => {
              if text_size > 0 {
                texts.push(token::Inline::Text(self.move_by(text_size)));
                text_size = 0;
              }
              texts.push(token::Inline::Text(r"\"));
            }
            '`' | '#' => {
              if text_size > 0 {
                texts.push(token::Inline::Text(self.move_by(text_size)));
                text_size = 0;
              }
              self.move_by(1);
              texts.push(token::Inline::Text(self.move_by(1)));
            }
            _ => {
              text_size += char.len_utf8() + 1;
            }
          }
          escaped = false;
        }
      } else {
        if escaped {
          text_size += 1;
        }
        texts.push(token::Inline::Text(self.move_by(text_size)));
        break;
      }
    }
    texts
  }

  fn scan_single_line_by_end_char(&mut self, end_char: char) -> &'a str {
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
        return self.move_by(size);
      }
    }
    self.move_by(size + 1)
  }

  fn scan_single_line_code(&mut self) -> token::Block<'a> {
    let code = self.scan_single_line_by_end_char('\n');
    token::Block::Leaf(token::LeafBlock::IndentedCode(code))
  }

  fn scan_multiple_line_code(&mut self) -> token::Block<'a> {
    lazy_static! {
      static ref CODE_END_REGEX: Regex = Regex::new(r"(^ {0,3}|\n {0,3})``` *\n?").unwrap();
    }
    self.move_by(3);
    let mut language = "";
    let mut metastring = "";
    let mut code = "";

    self.skip_whitespace();
    let mut code_info = self.scan_single_line_by_end_char('\n');
    if code_info.ends_with("\n") {
      code_info = &code_info[..code_info.len() - 1];
    }
    if !code_info.is_empty() {
      if let Some(index) = code_info.find(" ") {
        language = &code_info[0..index].trim();
        metastring = &code_info[index..].trim_start();
      } else {
        language = code_info.trim();
      }
    }

    if let Some(captures) = CODE_END_REGEX.captures(self.source) {
      let end_token = captures.get(0).unwrap().as_str();
      let code_size = self.source.find(end_token).unwrap();
      code = self.move_by(code_size);
      self.move_by(end_token.len());
    } else {
      code = self.move_by(self.source.len());
    }
    token::Block::Leaf(token::LeafBlock::FencedCode(token::FencedCode {
      code,
      language,
      metastring,
    }))
  }

  fn scan_jsx(&mut self, is_inline: bool) -> Result<jsx::JSXNode<'a>, &'a str> {
    let mut jsx_parser = jsx::JSXParser::new(self.source, 0, is_inline);
    let jsx_node = jsx_parser.parse();
    self.move_by(jsx_parser.size);
    jsx_node
  }
}
