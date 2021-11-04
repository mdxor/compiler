use crate::jsx;
use crate::token;

pub struct Lexer<'source> {
  _source: &'source str,
  _bytes: &'source [u8],
  offset: usize,
}
const BLOCK_TOKEN_MAX_SIZE: usize = 6;
impl<'source> Lexer<'source> {
  pub fn new(source: &'source str) -> Self {
    Lexer {
      _source: source,
      _bytes: source.as_bytes(),
      offset: 0,
    }
  }

  fn source(&mut self) -> &'source str {
    &self._source[self.offset..]
  }

  fn bytes(&mut self) -> &'source [u8] {
    &self._bytes[self.offset..]
  }

  fn cur(&mut self) -> u8 {
    self._bytes[self.offset]
  }

  fn move_by(&mut self, size: usize) -> &'source str {
    let result = &self.source()[0..size];
    self.offset += size;
    result
  }

  fn scan_blank_line(&mut self) -> Option<usize> {
    let mut size = 0;
    for &b in self.bytes() {
      if b == b' ' {
        size += 1;
      } else if b == b'\n' {
        return Some(size);
      } else {
        return None;
      }
    }
    Some(size)
  }

  fn count_starts_whitespace(&mut self) -> usize {
    let mut size = 0;
    for &b in self.bytes() {
      if b == b' ' {
        size += 1;
      } else {
        break;
      }
    }
    size
  }

  fn scan_token(&mut self) -> Option<&'source str> {
    let mut size = 0;
    for &b in self.bytes() {
      if b == b' ' || b == b'\n' {
        break;
      } else {
        size += 1;
        if size > BLOCK_TOKEN_MAX_SIZE {
          return None;
        }
      }
    }
    Some(&self.source()[size..])
  }

  fn skip_whitespace(&mut self) {
    let size = self.count_starts_whitespace();
    self.move_by(size);
  }

  fn match_block_start_token(&mut self, t: &'source str) -> Option<token::Block> {
    match t {
      "#" => Some(token::Block::Leaf(token::LeafBlock::Heading1)),
      "##" => Some(token::Block::Leaf(token::LeafBlock::Heading2)),
      "###" => Some(token::Block::Leaf(token::LeafBlock::Heading3)),
      "####" => Some(token::Block::Leaf(token::LeafBlock::Heading4)),
      "#####" => Some(token::Block::Leaf(token::LeafBlock::Heading5)),
      "######" => Some(token::Block::Leaf(token::LeafBlock::Heading6)),
      _ => None,
    }
  }

  fn read_block(&mut self) -> Option<token::Block> {
    if self.source().is_empty() {
      return None;
    }
    // TODO: handle Indented Code, List
    let whitespace_size = self.count_starts_whitespace();
    if whitespace_size < 4 {
      self.move_by(whitespace_size);
      None
    } else {
      let maybe_token = self.scan_token();
      let mut block: Option<token::Block> = None;
      if let Some(t) = maybe_token {
        block = self.match_block_start_token(t);
        if let Some(_) = block {
          self.move_by(t.len());
        }
      }
      if let None = block {
        block = Some(token::Block::Leaf(token::LeafBlock::Paragraph));
      }
      block
    }
  }

  fn read_inline(&mut self, multiple_lines: bool) {}
}
