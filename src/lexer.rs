use crate::jsx;
use crate::lexer_base::LexerBase;
use crate::token;

pub struct Lexer<'source> {
  _source: &'source str,
  _bytes: &'source [u8],
  offset: usize,
}

impl<'source> LexerBase<'source> for Lexer<'source> {
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
    if let Some(size) = self.scan_block_starting_token(b'#', 6) {
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

  fn scan_block(&mut self) -> Option<token::Block> {
    if let Some(heading) = self.scan_atx_heading() {
      Some(token::Block::Leaf(heading))
    } else if let Some(heading) = self.scan_setext_heading() {
      Some(token::Block::Leaf(heading))
    } else {
      None
    }
  }

  fn read_block(&mut self) -> Option<token::Block> {
    if self.source().is_empty() {
      return None;
    }
    // TODO: handle Indented Code, List
    let whitespace_size = self.count_starting_whitespace();
    if whitespace_size < 4 {
      self.forward(whitespace_size);
      None
    } else {
      if let Some(block) = self.scan_block() {
        Some(block)
      } else {
        Some(token::Block::Leaf(token::LeafBlock::Paragraph))
      }
    }
  }

  fn read_inline(&mut self, multiple_lines: bool) {}
}
