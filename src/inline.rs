use crate::input::*;
use crate::lexer::*;
use crate::raw::*;
use crate::token::*;
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
  special_bytes: [bool; 256],
  maybe_tokens: VecDeque<Token<MaybeInlineToken>>,
  tokens: Vec<Token<InlineToken>>,
  // token index, repeat
  pub close_code_deque: VecDeque<(usize, usize)>,
  pub close_link_deque: VecDeque<usize>,
}

impl<'a> InlineParser<'a> {
  pub fn new(bytes: &'a [u8]) -> Self {
    let mut special_bytes = [false; 256];
    let specials = [b'*', b'_', b'~', b'[', b']', b'`', b'<'];
    for &byte in &specials {
      special_bytes[byte as usize] = true;
    }
    Self {
      bytes,
      special_bytes,
      close_code_deque: VecDeque::new(),
      close_link_deque: VecDeque::new(),
      maybe_tokens: VecDeque::new(),
      tokens: vec![],
    }
  }

  pub fn parse(&mut self, raws: &Vec<Span>) -> Vec<Token<InlineToken>> {
    self.parse_raws(raws);
    self.process_tokens();
    mem::replace(&mut self.tokens, vec![])
  }

  pub fn parse_raws(&mut self, raws: &Vec<Span>) {
    let mut maybe_tokens: VecDeque<Token<MaybeInlineToken>> = VecDeque::new();
    let mut close_code_deque = VecDeque::new();
    let mut close_link_deque = VecDeque::new();
    iterate_raws(
      raws,
      self.bytes,
      &self.special_bytes,
      |callback_type| match callback_type {
        CallbackType::SpecialByte(raw_bytes, raw_start, start) => {
          let byte = raw_bytes[start];
          match byte {
            b'`' => {
              let (_, repeat) = ch_repeat(&raw_bytes[start..], byte);
              close_code_deque.push_back((maybe_tokens.len(), repeat));
              maybe_tokens.push_back(Token {
                value: MaybeInlineToken::InlineCode {
                  repeat,
                  can_open: true,
                },
                span: Span {
                  start: raw_start + start,
                  end: raw_start + start + repeat,
                },
              });
              return CallbackReturn::Move(repeat);
            }
            b'*' | b'_' | b'~' => {
              let (_, repeat) = ch_repeat(&raw_bytes[start..], byte);
              let can_open = is_left_flanking_delimiter(raw_bytes, start, start + repeat);
              let can_close = is_right_flanking_delimiter(raw_bytes, start, start + repeat);
              if !can_open && !can_close {
                return CallbackReturn::Text(repeat);
              }
              if byte == b'~' && repeat == 2 {
                maybe_tokens.push_back(Token {
                  value: MaybeInlineToken::Emphasis {
                    ch: byte,
                    repeat,
                    can_open,
                    can_close,
                  },
                  span: Span {
                    start: raw_start + start,
                    end: raw_start + start + repeat,
                  },
                });
                return CallbackReturn::Move(repeat);
              } else if byte != b'~' {
                maybe_tokens.push_back(Token {
                  value: MaybeInlineToken::Emphasis {
                    ch: byte,
                    repeat,
                    can_open,
                    can_close,
                  },
                  span: Span {
                    start: raw_start + start,
                    end: raw_start + start + repeat,
                  },
                });
                return CallbackReturn::Move(repeat);
              }
              return CallbackReturn::Text(repeat);
            }
            b'[' => maybe_tokens.push_back(Token {
              value: MaybeInlineToken::LinkStart,
              span: Span {
                start: raw_start + start,
                end: raw_start + start + 1,
              },
            }),
            b']' => {
              close_link_deque.push_back(maybe_tokens.len());
              maybe_tokens.push_back(Token {
                value: MaybeInlineToken::LinkEnd,
                span: Span {
                  start: raw_start + start,
                  end: raw_start + start + 1,
                },
              });
            }
            b'<' => {
              if let Some(size) = uri(&raw_bytes[start..]) {
                maybe_tokens.push_back(Token {
                  value: MaybeInlineToken::AutoLink(false),
                  span: Span {
                    start: raw_start + start,
                    end: raw_start + start + size,
                  },
                });
                return CallbackReturn::Move(size);
              } else {
                return CallbackReturn::Text(1);
              }
            }
            _ => {}
          }
          CallbackReturn::None
        }
        CallbackType::EscapedByte(raw_bytes, raw_start, start) => {
          let byte = raw_bytes[start + 1];
          if byte == b'`' {
            let (_, repeat) = ch_repeat(&raw_bytes[start + 1..], byte);
            close_code_deque.push_back((maybe_tokens.len(), repeat));
            maybe_tokens.push_back(Token {
              value: MaybeInlineToken::InlineCode {
                repeat,
                can_open: false,
              },
              span: Span {
                start: raw_start + start + 1,
                end: raw_start + start + 1 + repeat,
              },
            });
            return CallbackReturn::Move(repeat + 1);
          } else {
            maybe_tokens.push_back(Token {
              value: MaybeInlineToken::EscapedText,
              span: Span {
                start: raw_start + start,
                end: raw_start + start + 2,
              },
            });
          }
          CallbackReturn::None
        }
        CallbackType::Text(raw_bytes, raw_start, start, end) => {
          maybe_tokens.push_back(Token {
            value: MaybeInlineToken::Text,
            span: Span {
              start: raw_start + start,
              end: raw_start + end,
            },
          });
          CallbackReturn::None
        }
        CallbackType::HardBreak(raw_start, start, end) => {
          maybe_tokens.push_back(Token {
            value: MaybeInlineToken::HardBreak,
            span: Span {
              start: raw_start + start,
              end: raw_start + end,
            },
          });
          CallbackReturn::None
        }
        CallbackType::SoftBreak(raw_start, start, end) => {
          maybe_tokens.push_back(Token {
            value: MaybeInlineToken::SoftBreak,
            span: Span {
              start: raw_start + start,
              end: raw_start + end,
            },
          });
          CallbackReturn::None
        }
        _ => CallbackReturn::None,
      },
    );
    self.maybe_tokens = maybe_tokens;
    self.close_code_deque = close_code_deque;
    self.close_link_deque = close_link_deque;
  }

  fn process_tokens(&mut self) {
    let len = self.maybe_tokens.len();
    while !self.maybe_tokens.is_empty() {
      let index = len - self.maybe_tokens.len();
      let maybe_token = self.maybe_tokens.pop_front().unwrap();
      match maybe_token {
        Token {
          value: MaybeInlineToken::InlineCode { repeat, can_open },
          span,
        } => {}
        Token {
          value: MaybeInlineToken::Text,
          span,
        } => {
          self.push_text(span);
        }
        _ => {}
      }
    }
  }

  fn push_text(&mut self, span: Span) {
    if let Some(Token {
      value: InlineToken::Text,
      span: text_span,
    }) = self.tokens.last_mut()
    {
      if text_span.end == span.start {
        text_span.end = span.end;
        return;
      }
    }
    self.tokens.push(Token {
      value: InlineToken::Text,
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
  let mut inlineParser = InlineParser::new(bytes);
  let tokens = inlineParser.parse(&raws);
  println!("{:?}", tokens);
}
