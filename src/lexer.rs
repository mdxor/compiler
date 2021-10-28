use crate::jsx;
use crate::rule::Rule;
use crate::token;
use lazy_static::lazy_static;
use std::mem;

lazy_static! {
  static ref RULE: Rule = Rule::new();
}

pub struct Lexer<'a> {
  source: &'a str,
  offset: usize,
  inlines_source: &'a str,
  pub blocks: Vec<token::Block<'a>>,
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str) -> Self {
    Lexer {
      source,
      offset: 0,
      inlines_source: "",
      blocks: vec![],
    }
  }

  fn cur(&mut self) -> &'a str {
    &self.source[self.offset..]
  }

  fn skip_whitespace(&mut self) -> usize {
    if let Some(caps) = RULE.whitespace.captures(self.cur()) {
      let size = caps.get(0).unwrap().as_str().len();
      self.move_by(size);
      size
    } else {
      0
    }
  }

  fn move_by(&mut self, size: usize) -> &'a str {
    let result = &self.source[self.offset..self.offset + size];
    self.offset += size;
    result
  }

  fn move_inlines_by(&mut self, size: usize) -> &'a str {
    let result = &self.inlines_source[..size];
    self.inlines_source = &self.inlines_source[size..];
    result
  }

  pub fn tokenize(&mut self) -> Result<token::AST<'a>, &'a str> {
    loop {
      if self.cur().is_empty() {
        if !self.inlines_source.is_empty() {
          let block = self.scan_paragraph()?;
          self.blocks.push(block);
        }
        break;
      } else {
        self.scan_block()?;
      }
    }
    let blocks = mem::take(&mut self.blocks);
    Ok(token::AST { blocks })
  }

  fn scan_block(&mut self) -> Result<(), &'a str> {
    let mut block_option: Option<token::Block<'a>> = None;
    let _inlines_source = self.inlines_source;
    if let Some(block) = self.scan_blank_line() {
      block_option = Some(block);
    } else if let Some(block) = self.scan_single_line_code() {
      block_option = Some(block);
    } else {
      // <=3 whitespace
      self.skip_whitespace();
      if let Some(block) = self.scan_empty_atx_heading() {
        block_option = Some(block);
      } else if let Some(block) = self.scan_heading()? {
        block_option = Some(block);
      } else if let Some(block) = self.scan_fenced_code() {
        block_option = Some(block);
      } else if let Some(block) = self.scan_thematic_break() {
        block_option = Some(block);
      }
    }

    if let Some(block) = block_option {
      if !_inlines_source.is_empty() {
        self.inlines_source = _inlines_source;
        let p = self.scan_paragraph()?;
        self.blocks.push(p);
      }
      self.blocks.push(block);
    } else {
      if let Some(caps) = RULE.line.captures(self.cur()) {
        let size = caps.get(0).unwrap().as_str().len();
        self.move_by(size);
        self.inlines_source =
          &self.source[self.offset - size - self.inlines_source.len()..self.offset];
      }
    }
    Ok(())
  }

  fn scan_blank_line(&mut self) -> Option<token::Block<'a>> {
    if let Some(caps) = RULE.blank_line.captures(self.cur()) {
      let size = caps.get(0).unwrap().as_str().len();
      self.move_by(size);
      Some(token::Block::Leaf(token::LeafBlock::BlankLine))
    } else {
      None
    }
  }

  fn scan_thematic_break(&mut self) -> Option<token::Block<'a>> {
    if let Some(caps) = RULE.thematic_break.captures(self.cur()) {
      let size = caps.get(0).unwrap().as_str().len();
      self.move_by(size);
      Some(token::Block::Leaf(token::LeafBlock::ThematicBreak))
    } else {
      None
    }
  }

  fn scan_paragraph(&mut self) -> Result<token::Block<'a>, &'a str> {
    if self.inlines_source.ends_with("\n") {
      self.inlines_source = &self.inlines_source[..self.inlines_source.len() - 1];
    }
    let inlines = self.scan_inlines()?;

    Ok(token::Block::Leaf(token::LeafBlock::Paragraph(
      token::Paragraph { inlines },
    )))
  }

  fn scan_empty_atx_heading(&mut self) -> Option<token::Block<'a>> {
    if let Some(caps) = RULE.empty_atx_heading.captures(self.cur()) {
      let size = caps.get(1).unwrap().as_str().len();
      self.move_by(caps.get(0).unwrap().as_str().len());
      Some(token::Block::Leaf(token::LeafBlock::Heading(
        token::Heading {
          level: size as u8,
          inlines: vec![],
        },
      )))
    } else {
      None
    }
  }

  fn scan_heading(&mut self) -> Result<Option<token::Block<'a>>, &'a str> {
    if let Some(caps) = RULE.atx_heading.captures(self.cur()) {
      let size = caps.get(1).unwrap().as_str().len();
      let level = size as u8;
      self.move_by(size + 1);
      self.skip_whitespace();

      if let Some(caps) = RULE.line.captures(self.cur()) {
        let size = caps.get(0).unwrap().as_str().len();
        let line_size = caps.get(1).map_or("", |v| v.as_str()).len();
        self.inlines_source = &self.source[self.offset..self.offset + line_size];
        if let Some(caps) = RULE.closing_atx_heading.captures(self.inlines_source) {
          self.inlines_source =
            &self.inlines_source[..line_size - caps.get(0).unwrap().as_str().len()];
        }

        self.move_by(size);
        let inlines = self.scan_inlines()?;
        let heading = token::Heading { level, inlines };
        return Ok(Some(token::Block::Leaf(token::LeafBlock::Heading(heading))));
      }
      return Ok(Some(token::Block::Leaf(token::LeafBlock::Heading(
        token::Heading {
          level: size as u8,
          inlines: vec![],
        },
      ))));
    }
    Ok(None)
  }

  fn scan_inlines(&mut self) -> Result<Vec<token::Inline<'a>>, &'a str> {
    let mut inlines = vec![];
    loop {
      if self.inlines_source.is_empty() {
        return Ok(inlines);
      } else if self.inlines_source.starts_with("`") {
      } else {
        inlines.append(&mut self.scan_inline_text());
      }
    }
  }

  fn scan_inline_text(&mut self) -> Vec<token::Inline<'a>> {
    let mut chars = self.inlines_source.chars();
    let mut text_size = 0;
    let mut texts = vec![];
    let mut escaped = false;
    loop {
      if let Some(char) = chars.next() {
        if !escaped {
          match char {
            '`' => {
              texts.push(token::Inline::Text(self.move_inlines_by(text_size)));
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
            '\\' => {
              if text_size > 0 {
                texts.push(token::Inline::Text(self.move_inlines_by(text_size)));
                text_size = 0;
              }
              texts.push(token::Inline::Text(r"\"));
            }
            '`' | '#' => {
              if text_size > 0 {
                texts.push(token::Inline::Text(self.move_inlines_by(text_size)));
                text_size = 0;
              }
              self.move_inlines_by(1);
              texts.push(token::Inline::Text(self.move_inlines_by(1)));
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
        texts.push(token::Inline::Text(self.move_inlines_by(text_size)));
        break;
      }
    }
    texts
  }

  fn scan_single_line_by_end_char(&mut self, end_char: char) -> &'a str {
    let mut size = 0;
    let mut chars = self.cur().chars();
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

  fn scan_single_line_code(&mut self) -> Option<token::Block<'a>> {
    if RULE.indented_code.is_match(self.cur()) {
      self.move_by(4);
      let code = self.scan_single_line_by_end_char('\n');
      Some(token::Block::Leaf(token::LeafBlock::IndentedCode(code)))
    } else {
      None
    }
  }

  fn scan_fenced_code(&mut self) -> Option<token::Block<'a>> {
    if RULE.fenced_code.is_match(self.cur()) {
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
      if let Some(captures) = RULE.fenced_code_end.captures(self.cur()) {
        let end_token = captures.get(0).unwrap().as_str();
        let code_size = self.cur().find(end_token).unwrap();
        code = self.move_by(code_size);
        self.move_by(end_token.len());
      } else {
        code = self.move_by(self.source.len() - self.offset);
      }
      Some(token::Block::Leaf(token::LeafBlock::FencedCode(
        token::FencedCode {
          code,
          language,
          metastring,
        },
      )))
    } else {
      None
    }
  }

  fn scan_jsx(&mut self, is_inline: bool) -> Result<jsx::JSXNode<'a>, &'a str> {
    let mut jsx_parser = jsx::JSXParser::new(self.cur(), 0, is_inline);
    let jsx_node = jsx_parser.parse();
    self.move_by(jsx_parser.size);
    jsx_node
  }
}
