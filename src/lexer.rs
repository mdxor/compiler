use crate::jsx;
use crate::scanner::Scanner;
use crate::token;
const MAX: usize = usize::max_value();
pub struct Lexer<'source> {
  _source: &'source str,
  _bytes: &'source [u8],
  offset: usize,
}

impl<'source> Scanner<'source> for Lexer<'source> {
  fn source(&mut self) -> &'source str {
    &self._source[self.offset..]
  }

  fn bytes(&mut self) -> &'source [u8] {
    &self._bytes[self.offset..]
  }

  fn cur(&mut self) -> u8 {
    self._bytes[self.offset]
  }

  fn forward_slice(&mut self, size: usize) -> &'source str {
    let result = &self.source()[0..size];
    self.offset += size;
    result
  }

  fn forward(&mut self, size: usize) {
    self.offset += size;
  }
}

impl<'source> Lexer<'source> {
  pub fn new(source: &'source str) -> Self {
    Lexer {
      _source: source,
      _bytes: source.as_bytes(),
      offset: 0,
    }
  }

  fn scan_atx_heading(&mut self) -> Option<token::LeafBlock> {
    if let Some(size) = self.scan_block_starting_token(b'#', 1, 6) {
      Some(token::LeafBlock::ATXHeading(size as u8))
    } else {
      None
    }
  }

  fn scan_setext_heading(&mut self) -> Option<token::LeafBlock> {
    if self.match_keyword_cur_line(b'=', false) {
      Some(token::LeafBlock::SetextHeading(1))
    } else if self.match_keyword_cur_line(b'-', false) {
      Some(token::LeafBlock::SetextHeading(2))
    } else {
      None
    }
  }

  fn scan_thematic_break(&mut self) -> Option<token::LeafBlock> {
    if self.match_keywords_cur_line(vec![b'*', b'-', b'_'], true) {
      Some(token::LeafBlock::ThematicBreak)
    } else {
      None
    }
  }

  fn scan_fenced_code(&mut self) -> Option<token::LeafBlock> {
    if let Some(size) = self.scan_block_starting_token(b'`', 3, MAX) {
      Some(token::LeafBlock::FencedCode(size))
    } else {
      None
    }
  }

  fn scan_block(&mut self) -> Option<token::Block> {
    if let Some(heading) = self.scan_atx_heading() {
      Some(token::Block::Leaf(heading))
    } else if let Some(heading) = self.scan_setext_heading() {
      Some(token::Block::Leaf(heading))
    } else if let Some(thematic_break) = self.scan_thematic_break() {
      Some(token::Block::Leaf(thematic_break))
    } else if let Some(fenced_code) = self.scan_fenced_code() {
      Some(token::Block::Leaf(fenced_code))
    } else {
      None
    }
  }

  fn read_block(&mut self) -> Option<token::Block> {
    if self.source().is_empty() {
      return None;
    }
    let whitespace_size = self.count_starting_whitespace();
    if whitespace_size >= 4 {
      Some(token::Block::Leaf(token::LeafBlock::IndentedCode))
    } else {
      self.forward(whitespace_size);
      if let Some(block) = self.scan_block() {
        Some(block)
      } else {
        Some(token::Block::Leaf(token::LeafBlock::Paragraph))
      }
    }
  }

  fn read_inline(&mut self, multiple_lines: bool) {}
}
