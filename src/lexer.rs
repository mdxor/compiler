use crate::jsx;
use crate::token;

pub struct Lexer<'source> {
  _source: &'source str,
  _bytes: &'source [u8],
  offset: usize,
}
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

  fn skip_whitespace(&mut self) {
    let size = self.count_starts_whitespace();
    self.move_by(size);
  }

  fn scan_block_start_token(&mut self, keyword: u8, max_size: usize) -> Option<usize> {
    let mut size = 0;
    for &b in self.bytes() {
      if b == b' ' || b == b'\n' {
        break;
      } else if b == keyword {
        size += 1;
        if size > max_size {
          return None;
        }
      } else {
        return None;
      }
    }
    if size == 0 {
      None
    } else {
      Some(size)
    }
  }

  fn scan_single_keyword_cur_line(
    &mut self,
    keyword: u8,
    allow_internal_spaces: bool,
  ) -> Option<usize> {
    let mut size = 0;
    let mut starting_spaces = true;
    let mut ending_spaces = false;
    for &b in self.bytes() {
      if b == b'\n' {
        if size > 0 {
          size += 1;
        }
        break;
      }
      if starting_spaces {
        if b == keyword {
          starting_spaces = false;
          size += 1;
        } else if b == b' ' {
          size += 1
        } else {
          return None;
        }
      } else if ending_spaces {
        if b == b' ' {
          size += 1
        } else {
          return None;
        }
      } else {
        if b == keyword {
          size += 1
        } else if b == b' ' {
          if !allow_internal_spaces {
            ending_spaces = true;
          }
          size += 1;
        } else {
          return None;
        }
      }
    }
    if size == 0 {
      None
    } else {
      Some(size)
    }
  }

  fn scan_atx_heading(&mut self) -> Option<token::LeafBlock> {
    if let Some(size) = self.scan_block_start_token(b'#', 6) {
      match size {
        1 => Some(token::LeafBlock::Heading1),
        2 => Some(token::LeafBlock::Heading2),
        3 => Some(token::LeafBlock::Heading3),
        4 => Some(token::LeafBlock::Heading4),
        5 => Some(token::LeafBlock::Heading5),
        6 => Some(token::LeafBlock::Heading6),
        _ => None,
      }
    } else {
      None
    }
  }

  fn scan_block(&mut self) -> Option<token::Block> {
    if let Some(heading) = self.scan_atx_heading() {
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
    let whitespace_size = self.count_starts_whitespace();
    if whitespace_size < 4 {
      self.move_by(whitespace_size);
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
