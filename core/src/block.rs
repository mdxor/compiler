use crate::document::*;
use crate::input::*;
use crate::jsx_parser::*;
use crate::lexer::*;
use crate::token::*;
use std::collections::VecDeque;
use std::mem::replace;
use std::str;

pub struct BlockParser<'source> {
  source: &'source str,
  document: Document<'source>,
  spine: Vec<ContainerBlock>,
  last_leaf_end: usize,
  tmp_tokens: VecDeque<Token<BlockToken>>,
}
impl<'source> BlockParser<'source> {
  pub fn new(source: &'source str) -> Self {
    BlockParser {
      source,
      spine: Vec::new(),
      document: Document::new(source),
      last_leaf_end: 0,
      tmp_tokens: VecDeque::new(),
    }
  }

  pub fn parse(&mut self) -> AST<Token<BlockToken>> {
    let spans = VecDeque::from(vec![Span {
      start: 0,
      end: self.source.len(),
    }]);
    let mut parser = JSXParser::new(self.source, self.document.bytes, &spans);
    let import_export_size = parser.js_import_export();
    self.document.forward_to(import_export_size);
    let mut blocks = self.scan_blocks();
    AST {
      children: blocks,
      span: Span {
        start: import_export_size,
        end: self.source.len(),
      },
    }
  }

  fn scan_blocks(&mut self) -> Vec<Token<BlockToken>> {
    let mut blocks = vec![];
    let level = self.spine.len();
    let mut is_prev_paragraph = false;
    while self.document.start() < self.source.len() {
      if level != self.spine.len() {
        break;
      }
      let block = self.scan_block(is_prev_paragraph);
      is_prev_paragraph = self.process_block(&mut blocks, block);
      while !self.tmp_tokens.is_empty() {
        let block = self.tmp_tokens.pop_front().unwrap();
        self.process_block(&mut blocks, block);
      }
    }
    blocks
  }

  fn process_block(
    &mut self,
    blocks: &mut Vec<Token<BlockToken>>,
    block: Token<BlockToken>,
  ) -> bool {
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
          raws.append(&mut next_raws);
        } else {
          blocks.push(Token {
            value: BlockToken::Paragraph { raws: next_raws },
            span,
          });
        }
        return true;
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
    false
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
    }
    blocks
  }

  fn scan_block(&mut self, is_prev_paragraph: bool) -> Token<BlockToken> {
    self.document.spaces0();
    if let Some(block) = self.scan_blank_line() {
      self.finish_leaf_block();
      return block;
    }
    if !is_prev_paragraph && self.document.spaces() >= 4 {
      let block = self.scan_indented_code();
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

  fn scan_fenced_code(&mut self) -> Option<Token<BlockToken>> {
    let bytes = self.document.bytes();
    let start = self.document.start();
    let (size, repeat, meta_size) = open_fenced_code(bytes)?;
    let ch = bytes[0];
    let meta_span = Span {
      start: start + repeat,
      end: start + repeat + meta_size,
    };
    let mut code_spans = vec![];
    self.document.forward(size);
    loop {
      if let Some(size) = self.continue_container() {
        self.document.forward(size);
        if let Some(size) = close_fenced_code(self.document.bytes(), ch, repeat) {
          self.document.forward(size);
          break;
        }
        let (size, _) = one_line(self.document.bytes());
        code_spans.push(Span {
          start: self.document.start(),
          end: self.document.forward(size),
        });
      } else {
        break;
      }
    }
    Some(Token {
      value: BlockToken::FencedCode {
        meta_span,
        code_spans,
      },
      span: Span {
        start,
        end: self.document.start(),
      },
    })
  }

  fn scan_indented_code(&mut self) -> Token<BlockToken> {
    let start = self.document.start();
    let mut spans = vec![];
    let indent = self.document.spaces();
    let (size, _) = one_line(self.document.bytes());
    spans.push(Span {
      start: start + indent,
      end: self.document.forward(size),
    });
    loop {
      if let Some(container_size) = self.continue_container() {
        let bytes = self.document.bytes();
        let (bytes, spaces) = spaces0(&bytes[container_size..]);
        if spaces < 4 || eol(bytes).is_some() {
          break;
        }
        let consumed_spaces = if spaces <= indent { spaces } else { indent };
        let (line_size, _) = one_line(bytes);
        let start = self.document.start() + consumed_spaces;
        let end = self.document.start() + spaces + line_size;
        spans.push(Span { start, end });
        self.document.forward_to(end);
      } else {
        break;
      }
    }
    Token {
      value: BlockToken::IndentedCode(spans),
      span: Span {
        start,
        end: self.document.start(),
      },
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
      let end = self.document.forward(line_size);
      return Token {
        value: BlockToken::ATXHeading {
          level: HeadingLevel::new(level).unwrap(),
          raws: vec![Span {
            start: start + start_size,
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
    if let Some(block) = self.scan_fenced_code() {
      return block;
    }
    // TODO: lind definition, table
    self.scan_paragraph_like()
  }

  fn scan_paragraph_like(&mut self) -> Token<BlockToken> {
    let bytes = self.document.bytes();
    let start = self.document.start();
    let (line_size, _) = one_line(bytes);
    let end = self.document.forward(line_size);
    let span = Span { start, end };
    let mut tokens: VecDeque<Token<BlockToken>> = VecDeque::new();
    let mut raws_deque = VecDeque::new();
    raws_deque.push_back(span);
    if let Some(_) = single_char(bytes, b'<') {
      loop {
        if let Some(size) = self.continue_paragraph_like() {
          self.document.forward(size);
          let start = self.document.start();
          let (line_size, _) = one_line(self.document.bytes());
          let end = self.document.forward(line_size);
          let span = Span { start, end };
          raws_deque.push_back(span);
        } else {
          break;
        }
      }
      let mut raws: Vec<Span> = vec![];
      while !raws_deque.is_empty() {
        let mut parser = JSXParser::new(self.source, self.document.bytes, &raws_deque);
        if let Some((element, end, index)) = parser.jsx() {
          if !raws.is_empty() {
            let span = Span {
              start: raws.first().unwrap().start,
              end: raws.last().unwrap().end,
            };
            tokens.push_back(Token {
              value: BlockToken::Paragraph {
                raws: replace(&mut raws, vec![]),
              },
              span,
            })
          }
          let start = raws_deque.front().unwrap().start;
          tokens.push_back(Token {
            value: BlockToken::JSX(element),
            span: Span { start, end },
          });
          raws_deque.drain(..index);
        } else {
          raws.push(raws_deque.pop_front().unwrap());
        }
      }
      if !raws.is_empty() {
        let span = Span {
          start: raws.first().unwrap().start,
          end: raws.last().unwrap().end,
        };
        tokens.push_back(Token {
          value: BlockToken::Paragraph {
            raws: replace(&mut raws, vec![]),
          },
          span,
        })
      }
      let token = tokens.pop_front().unwrap();
      self.tmp_tokens = tokens;
      return token;
    }
    return Token {
      value: BlockToken::Paragraph {
        raws: Vec::from(raws_deque),
      },
      span: Span { start, end },
    };
  }

  fn finish_leaf_block(&mut self) {
    self.last_leaf_end = self.document.start();
    let (size, level) = self.resume_container();
    self.document.forward(size);
    while level < self.spine.len() {
      self.spine.pop();
    }
  }

  fn resume_container(&mut self) -> (usize, usize) {
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

  fn continue_container(&mut self) -> Option<usize> {
    if self.source.len() <= self.document.start() {
      return None;
    }
    let (size, level) = self.resume_container();
    if level == self.spine.len() {
      self.document.reset_spaces();
      Some(size)
    } else {
      None
    }
  }

  // table, link def
  fn continue_paragraph_like(&mut self) -> Option<usize> {
    let size = self.continue_container()?;
    if !self.interrupt_paragraph(size) {
      Some(size)
    } else {
      None
    }
  }

  fn interrupt_paragraph(&mut self, offset: usize) -> bool {
    let bytes = &self.document.bytes()[offset..];
    let (bytes, _) = spaces0(bytes);
    if atx_heading_start(bytes).is_none()
      && eol(bytes).is_none()
      && bytes[0] != b'>'
      && list_item_start(bytes).is_none()
      && thematic_break(bytes).is_none()
      && setext_heading(bytes).is_none()
      && open_fenced_code(bytes).is_none()
    {
      return false;
    }
    true
  }
}
