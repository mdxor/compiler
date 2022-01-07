use crate::input::*;
use crate::lexer::*;
use crate::token::*;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::mem;

fn is_left_flanking_delimiter(bytes: &[u8], start: usize, end: usize) -> bool {
  let len = bytes.len();
  if end >= len || bytes[end].is_ascii_whitespace() {
    return false;
  }
  let next_byte = if let Some(c) = bytes.get(end + 1) {
    c
  } else {
    return true;
  };
  // TODO
  if !next_byte.is_ascii_punctuation()
    || (start == 0
      || bytes[start - 1].is_ascii_whitespace()
      || bytes[start - 1].is_ascii_punctuation())
  {
    return true;
  }
  return false;
}

fn is_right_flanking_delimiter(bytes: &[u8], start: usize, end: usize) -> bool {
  let len = bytes.len();
  if start == 0 || bytes[start - 1].is_ascii_whitespace() {
    return false;
  }
  let prev_byte = if let Some(c) = bytes.get(start - 1) {
    c
  } else {
    return true;
  };
  // TODO
  if !prev_byte.is_ascii_punctuation()
    || (end >= len || bytes[end].is_ascii_whitespace() || bytes[end].is_ascii_punctuation())
  {
    return true;
  }
  return false;
}

pub struct InlineParser<'a> {
  bytes: &'a [u8],
  raws: &'a Vec<Span>,
  special_bytes: [bool; 256],
  maybe_tokens: VecDeque<Token<InlineToken>>,
  tokens: Vec<Token<InlineToken>>,
  code_map: HashMap<usize, usize>,
  close_link_deque: VecDeque<usize>,
  index: usize,
  text_start: usize,
  // raw index, token index,
  open_link: Option<(usize, usize)>,
  pos: usize,
}

impl<'a> InlineParser<'a> {
  pub fn new(bytes: &'a [u8], raws: &'a Vec<Span>) -> Self {
    let mut special_bytes = [false; 256];
    let specials = [b'*', b'_', b'~', b'[', b']', b'`', b'<', b'!', b'\r', b'\n'];
    for &byte in &specials {
      special_bytes[byte as usize] = true;
    }
    Self {
      bytes,
      raws,
      special_bytes,
      code_map: HashMap::new(),
      close_link_deque: VecDeque::new(),
      maybe_tokens: VecDeque::new(),
      tokens: vec![],
      index: 0,
      text_start: 0,
      open_link: None,
      pos: 0,
    }
  }

  pub fn parse(&mut self) -> Vec<Token<InlineToken>> {
    self.parse_raws();
    self.process_tokens();
    mem::replace(&mut self.tokens, vec![])
  }

  fn parse_raws(&mut self) {
    while self.pos < self.raws[self.index].end && self.index < self.raws.len() {
      if self.pos >= self.raws[self.index].end {
        self.index += 1;
        self.text_start = if self.pos > self.raws[self.index].start {
          self.pos
        } else {
          self.raws[self.index].start
        };
        continue;
      }
      let raw = &self.raws[self.index];
      let byte = self.bytes[self.pos];
      if self.special_bytes[byte as usize] {
        self.handle_text();
        if self.handle_special_byte(&raw) {
          continue;
        }
      } else if byte == b'\\' {
        if self.pos + 1 < raw.end {
          if self.bytes[self.pos + 1].is_ascii_punctuation() {
            self.handle_text();
            self.maybe_tokens.push_back(Token {
              value: InlineToken::TextSegment,
              span: Span {
                start: self.pos + 1,
                end: self.pos + 2,
              },
            });
            self.pos += 1;
            self.text_start = self.pos;
          }
        }
      }
      self.pos += 1;
      if self.pos == self.raws[self.index].end {
        self.handle_text();
      }
    }
  }

  fn handle_text(&mut self) {
    if self.text_start < self.pos {
      self.maybe_tokens.push_back(Token {
        value: InlineToken::TextSegment,
        span: Span {
          start: self.text_start,
          end: self.pos,
        },
      });
      self.text_start = self.pos;
    }
  }

  fn forward_pos(&mut self, n: usize) -> bool {
    self.pos += n;
    self.text_start = self.pos;
    true
  }

  fn scan_link_url_title(&mut self) -> bool {
    let mut index = self.index;
    if let Some((pos, url_span, title_spans)) = link_url_title(|| {
      if let Some(span) = self.raws.get(index) {
        let bytes = &self.bytes[span.start..span.end];
        if index == self.index {
          let bytes = &self.bytes[self.pos..span.end];
          return Some((bytes, self.pos));
        } else {
          return Some((bytes, span.start));
        }
      } else {
        return None;
      }
    }) {
      self.maybe_tokens.push_back(Token {
        value: InlineToken::LinkUrlTitle {
          url: url_span,
          title: title_spans,
          start_index: self.index,
        },
        span: Span {
          start: self.pos,
          end: pos + 1,
        },
      });
      self.index = index;
      self.pos = pos + 1;
      return true;
    }
    false
  }

  fn scan_inline_code(&mut self, repeat: usize) -> bool {
    let mut index = self.index;
    let mut pos = self.pos + repeat - 1;
    let mut codes: Vec<Span> = vec![];
    let mut raw = &self.raws[index];
    let mut raw_bytes = &self.bytes[raw.start..raw.end];
    let mut code_start = self.pos + repeat - 1;
    loop {
      if pos == raw.end {
        index += 1;
        if index == self.raws.len() {
          return false;
        }
        codes.push(Span {
          start: code_start,
          end: raw.end,
        });
        raw = &self.raws[index];
        raw_bytes = &self.bytes[raw.start..raw.end];
        pos = raw.start;
        code_start = raw.start;
      } else {
        pos += 1;
      }
      let i = pos - raw.start;
      if raw_bytes[i] == b'`' {
        let (_, end_repeat) = ch_repeat(&raw_bytes[i..], b'`');
        if end_repeat == repeat {
          codes.push(Span {
            start: code_start,
            end: i,
          });
          pos += repeat;
          self.maybe_tokens.push_back(Token {
            value: InlineToken::Code(codes),
            span: Span {
              start: self.pos,
              end: pos,
            },
          });
          self.pos = pos;
          return true;
        }
      }
    }
    false
  }

  fn handle_special_byte(&mut self, raw: &Span) -> bool {
    let raw_bytes = &self.bytes[raw.start..raw.end];
    let bytes = &self.bytes[self.pos..raw.end];
    let byte = self.bytes[self.pos];
    let start = self.pos - raw.start;
    match byte {
      b'`' => {
        let (_, repeat) = ch_repeat(bytes, byte);
        if !self.scan_inline_code(repeat) {
          self.pos += repeat;
        }
        return true;
      }
      b'*' | b'_' | b'~' => {
        let (_, repeat) = ch_repeat(bytes, byte);
        let can_open = is_left_flanking_delimiter(raw_bytes, start, start + repeat);
        let can_close = is_right_flanking_delimiter(raw_bytes, start, start + repeat);
        if !can_open && !can_close {
          return false;
        }
        if byte == b'~' && repeat == 2 {
          self.maybe_tokens.push_back(Token {
            value: InlineToken::MaybeEmphasis {
              ch: byte,
              repeat,
              can_open,
              can_close,
            },
            span: Span {
              start: self.pos,
              end: self.pos + repeat,
            },
          });
          return self.forward_pos(repeat);
        } else if byte != b'~' {
          self.maybe_tokens.push_back(Token {
            value: InlineToken::MaybeEmphasis {
              ch: byte,
              repeat,
              can_open,
              can_close,
            },
            span: Span {
              start: self.pos,
              end: self.pos + repeat,
            },
          });
          return self.forward_pos(repeat);
        }
        return false;
      }
      b'!' => {
        if let Some(next_byte) = raw_bytes.get(start + 1) {
          if *next_byte == b'[' {
            self.maybe_tokens.push_back(Token {
              value: InlineToken::MaybeLinkStart,
              span: Span {
                start: self.pos,
                end: self.pos + 2,
              },
            });
            return self.forward_pos(2);
          }
        }
        return false;
      }
      b'[' => {
        self.maybe_tokens.push_back(Token {
          value: InlineToken::MaybeLinkStart,
          span: Span {
            start: self.pos,
            end: self.pos + 1,
          },
        });
        return self.forward_pos(1);
      }
      b']' => {
        self.close_link_deque.push_back(self.maybe_tokens.len());
        self.maybe_tokens.push_back(Token {
          value: InlineToken::MaybeLinkEnd,
          span: Span {
            start: self.pos,
            end: self.pos + 1,
          },
        });
        self.forward_pos(1);
        self.scan_link_url_title();
        return true;
      }
      b'<' => {
        if let Some(size) = uri(&raw_bytes[start..]) {
          self.maybe_tokens.push_back(Token {
            value: InlineToken::AutoLink(false),
            span: Span {
              start: self.pos,
              end: self.pos + size,
            },
          });
          return self.forward_pos(size);
        } else {
          return false;
        }
      }
      b'\r' | b'\n' => {
        let size = if byte == b'\r' { 2 } else { 1 };
        self.maybe_tokens.push_back(Token {
          value: InlineToken::LineBreak,
          span: Span {
            start: self.pos,
            end: self.pos + size,
          },
        });
        return self.forward_pos(size);
      }
      _ => {
        return false;
      }
    }
  }

  fn process_tokens(&mut self) {
    let len = self.maybe_tokens.len();
    while !self.maybe_tokens.is_empty() {
      let index = len - self.maybe_tokens.len();
      let maybe_token = self.maybe_tokens.pop_front().unwrap();
      match maybe_token {
        Token {
          value: InlineToken::MaybeLinkStart,
          span,
        } => {}
        Token {
          value: InlineToken::TextSegment,
          span,
        } => {
          self.push_text(span);
        }
        _ => {
          self.tokens.push(maybe_token);
        }
      }
    }
  }

  fn push_text(&mut self, span: Span) {
    if let Some(Token {
      value: InlineToken::Text(texts),
      span: text_span,
    }) = self.tokens.last_mut()
    {
      text_span.end = span.end;
      texts.push(span);
      return;
    }
    self.tokens.push(Token {
      value: InlineToken::Text(vec![span.clone()]),
      span,
    });
  }
}

#[test]
fn test_parse_raws() {
  let bytes = b"`123`2*code*  \n1<https://mdxor.com>";
  let raws = vec![
    Span { start: 0, end: 15 },
    Span {
      start: 15,
      end: bytes.len(),
    },
  ];
  let mut inline_parser = InlineParser::new(bytes, &raws);
  let tokens = inline_parser.parse();
  println!("{:?}", tokens);
}
