use crate::document::*;
use crate::input::*;
use crate::lexer::*;
use crate::token::*;

pub struct BlockParser<'source> {
  source: &'source str,
  document: Document<'source>,
  spine: Vec<ContainerBlock>,
  last_leaf_end: usize,
}
impl<'source> BlockParser<'source> {
  pub fn new(source: &'source str) -> Self {
    BlockParser {
      source,
      spine: Vec::new(),
      document: Document::new(source),
      last_leaf_end: 0,
    }
  }

  pub fn parse(&mut self) -> AST {
    let mut blocks = self.scan_blocks();
    AST {
      blocks,
      span: Span {
        start: 0,
        end: self.source.len(),
      },
    }
  }

  fn scan_blocks(&mut self) -> Vec<Token<BlockToken>> {
    let mut blocks = vec![];
    let level = self.spine.len();
    while self.document.start() < self.source.len() {
      if level != self.spine.len() {
        break;
      }
      let block = self.scan_block();
      match block {
        Token {
          value: BlockToken::Paragraph {
            raws: mut next_raws,
          },
          span,
        } => {
          let end = span.end;
          if let Some(Token {
            value: BlockToken::Paragraph { raws },
            span,
          }) = blocks.last_mut()
          {
            span.end = end;
            raws.push(next_raws.pop().unwrap());
          } else {
            blocks.push(Token {
              value: BlockToken::Paragraph { raws: next_raws },
              span,
            });
          }
          continue;
        }
        Token {
          value: BlockToken::SetextHeading { level, raws },
          span,
        } => {
          let Span { start, end } = span;
          if let Some(Token {
            value: BlockToken::Paragraph { .. },
            ..
          }) = blocks.last()
          {
            if let Some(Token {
              value: BlockToken::Paragraph { raws },
              span: p_span,
            }) = blocks.pop()
            {
              blocks.push(Token {
                value: BlockToken::SetextHeading { level: level, raws },
                span: Span {
                  start: p_span.start,
                  end: end,
                },
              });
            }
          } else {
            blocks.push(Token {
              value: BlockToken::Paragraph {
                raws: vec![Span { start, end }],
              },
              span,
            });
          }
        }
        _ => {
          blocks.push(block);
        }
      }
    }
    blocks
  }

  // TODO: indent
  fn scan_list_items(&mut self, ch: u8) -> Vec<Token<BlockToken>> {
    let mut blocks = vec![];
    let level = self.spine.len();
    while self.document.start() < self.source.len() {
      if level != self.spine.len() {
        break;
      }
      let spaces = self.document.spaces0();
      let bytes = self.document.bytes();
      if let Some((size, marker_size, end_indent)) = list_item_start(bytes) {
        if bytes[marker_size - 1] != ch {
          break;
        }
        let start_indent = self.document.spaces();
        let start = self.document.start();
        self.document.forward(size);
        let indent = start_indent + marker_size + end_indent;
        self.spine.push(ContainerBlock::ListItem(indent));

        let item_blocks = self.scan_blocks();
        blocks.push(Token {
          value: BlockToken::ListItem {
            blocks: item_blocks,
            indent: start_indent + marker_size + end_indent,
          },
          span: Span {
            start,
            end: self.last_leaf_end,
          },
        });
      }
      break;
    }
    blocks
  }

  fn scan_block(&mut self) -> Token<BlockToken> {
    self.document.spaces0();
    if let Some(block) = self.scan_blank_line() {
      self.finish_leaf_block();
      return block;
    }
    if let Some(block) = self.scan_container_block() {
      return block;
    }
    let block = self.scan_leaf_block();
    self.finish_leaf_block();
    block
  }

  fn scan_blank_line(&mut self) -> Option<Token<BlockToken>> {
    let (_, eol_size) = eol(self.document.bytes())?;
    Some(Token {
      value: BlockToken::BlankLine,
      span: Span {
        start: self.document.start(),
        end: self.document.forward(eol_size),
      },
    })
  }

  fn scan_fenced_code(&mut self) -> Token<BlockToken> {
    Token {
      value: BlockToken::Paragraph { raws: vec![] },
      span: Span { start: 0, end: 0 },
    }
  }

  fn scan_indented_code(&mut self) -> Token<BlockToken> {
    Token {
      value: BlockToken::Paragraph { raws: vec![] },
      span: Span { start: 0, end: 0 },
    }
  }

  // list and block quote
  fn scan_container_block(&mut self) -> Option<Token<BlockToken>> {
    let bytes = self.document.bytes();
    let start = self.document.start();
    if let Some((size, level)) = block_quote(bytes) {
      self.document.forward(size);
      self.spine.push(ContainerBlock::BlockQuote(level));
      let blocks = self.scan_blocks();
      return Some(Token {
        value: BlockToken::BlockQuote { blocks, level },
        span: Span {
          start,
          end: self.last_leaf_end,
        },
      });
    } else if let Some((size, marker_size, end_indent)) = list_item_start(bytes) {
      let ch = bytes[marker_size - 1];
      let order_span = Span {
        start,
        end: start + marker_size - 1,
      };
      self.spine.push(ContainerBlock::List(ch));
      let mut blocks = self.scan_list_items(ch);
      return Some(Token {
        value: BlockToken::List {
          blocks,
          ch,
          order_span,
          is_tight: false,
        },
        span: Span {
          start,
          end: self.last_leaf_end,
        },
      });
    }
    None
  }

  fn scan_leaf_block(&mut self) -> Token<BlockToken> {
    let bytes = self.document.bytes();
    let start = self.document.start();
    if let Some(size) = thematic_break(bytes) {
      return Token {
        value: BlockToken::ThematicBreak,
        span: Span {
          start,
          end: self.document.forward(size),
        },
      };
    }
    if let Some((start_size, level)) = atx_heading_start(bytes) {
      let (line_size, _) = one_line(bytes);
      let end = self.document.forward(start_size + line_size);
      return Token {
        value: BlockToken::ATXHeading {
          level: HeadingLevel::new(level).unwrap(),
          raws: vec![Span {
            start: start + line_size,
            end,
          }],
        },
        span: Span { start, end },
      };
    }
    if let Some((size)) = setext_heading(bytes) {
      let level = if bytes[0] == b'=' { 1 } else { 2 };
      let end = self.document.forward(size);
      return Token {
        value: BlockToken::SetextHeading {
          level: HeadingLevel::new(level).unwrap(),
          raws: vec![],
        },
        span: Span { start, end },
      };
    }
    // TODO: lind definition, table, jsx, fenced code, indented code
    let (line_size, _) = one_line(bytes);
    let end = self.document.forward(line_size);
    return Token {
      value: BlockToken::Paragraph {
        raws: vec![Span { start, end }],
      },
      span: Span { start, end },
    };
  }

  fn finish_leaf_block(&mut self) {
    self.last_leaf_end = self.document.start();
    let (size, level) = self.continue_container();
    self.document.forward(size);
    while level < self.spine.len() {
      self.spine.pop();
    }
  }

  fn continue_container(&mut self) -> (usize, usize) {
    let mut spine_level = 0;
    let mut size = 0;
    let mut offset = 0;
    let (bytes, mut spaces) = spaces0(self.document.bytes());
    offset = spaces;
    while spine_level < self.spine.len() {
      let container_block = &self.spine[spine_level];
      if let ContainerBlock::BlockQuote(level) = container_block {
        if spaces < 4 {
          if let Some((quote_size, quote_level)) = block_quote(&bytes[offset..]) {
            if *level == quote_level {
              offset += quote_size;
              size = offset;
              spaces = spaces0(&bytes[size..]).1;
              offset += spaces;
              spine_level += 1;
              continue;
            }
          }
        }
      } else if let ContainerBlock::List(ch) = container_block {
        if let Some(ContainerBlock::ListItem(indent)) = self.spine.get(spine_level + 1) {
          if spaces >= *indent {
            size = offset - (size - *indent);
            spine_level += 2;
            continue;
          }
        }
        if let Some((_, marker_size, end_indent)) = list_item_start(&bytes[offset..]) {
          if *ch == bytes[size + marker_size - 1] {
            spine_level += 1;
          }
        }
      }
      break;
    }
    (size, spine_level)
  }

  // table, link def
  fn interrupt_paragraph_like(&mut self) -> Option<usize> {
    let (size, level) = self.continue_container();
    if level == self.spine.len() {
      self.document.reset_spaces();
      if !self.interrupt_paragraph(size) {
        return Some(size);
      }
    }
    None
  }

  fn interrupt_paragraph(&mut self, offset: usize) -> bool {
    let bytes = &self.document.bytes()[offset..];
    let (bytes, spaces) = spaces0(bytes);
    if spaces >= 4 {
      return true;
    }
    if atx_heading_start(bytes).is_none()
      && eol(bytes).is_none()
      && bytes[0] != b'>'
      && thematic_break(bytes).is_none()
      && setext_heading(bytes).is_none()
      && open_fenced_code(bytes).is_none()
    {
      return false;
    }
    true
  }
}
