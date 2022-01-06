use crate::input::*;
use crate::lexer::*;
use crate::raw::*;
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
  special_bytes: [bool; 256],
  maybe_tokens: VecDeque<Token<MaybeInlineToken>>,
  tokens: Vec<Token<InlineToken>>,
  pub close_link_deque: VecDeque<usize>,
}

impl<'a> InlineParser<'a> {
  pub fn new(bytes: &'a [u8]) -> Self {
    let mut special_bytes = [false; 256];
    let specials = [b'*', b'_', b'~', b'[', b']', b'`', b'<', b'!', b'\r', b'\n'];
    for &byte in &specials {
      special_bytes[byte as usize] = true;
    }
    Self {
      bytes,
      special_bytes,
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

    // repeat, token index
    let mut code_map: HashMap<usize, usize> = HashMap::new();
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
              if let Some(index) = code_map.get(&repeat) {
                maybe_tokens[*index].value = MaybeInlineToken::InlineCodeStart;
                let mut index = *index + 1;
                while index < maybe_tokens.len() {
                  maybe_tokens[index].value = MaybeInlineToken::Code;
                  index += 1;
                }
                maybe_tokens.push_back(Token {
                  value: MaybeInlineToken::InlineCodeEnd,
                  span: Span {
                    start: raw_start + start,
                    end: raw_start + start + repeat,
                  },
                });
                code_map.remove(&repeat);
              } else {
                code_map.insert(repeat, maybe_tokens.len());
                maybe_tokens.push_back(Token {
                  value: MaybeInlineToken::InlineCode,
                  span: Span {
                    start: raw_start + start,
                    end: raw_start + start + repeat,
                  },
                });
              }
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
            b'!' => {
              if let Some(next_byte) = raw_bytes.get(start + 1) {
                if *next_byte == b'[' {
                  maybe_tokens.push_back(Token {
                    value: MaybeInlineToken::LinkStart,
                    span: Span {
                      start: raw_start + start,
                      end: raw_start + start + 2,
                    },
                  });
                  return CallbackReturn::Move(2);
                }
              }
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
            b'\r' | b'\n' => {
              let size = if byte == b'\r' { 2 } else { 1 };
              maybe_tokens.push_back(Token {
                value: MaybeInlineToken::LineBreak,
                span: Span {
                  start: raw_start + start,
                  end: raw_start + start + size,
                },
              });
              return CallbackReturn::Move(size);
            }
            _ => {}
          }
          CallbackReturn::None
        }
        CallbackType::EscapedByte(raw_bytes, raw_start, start) => {
          let byte = raw_bytes[start + 1];
          if byte == b'`' {
            let (_, repeat) = ch_repeat(&raw_bytes[start + 1..], byte);
            if let Some(index) = code_map.get(&repeat) {
              maybe_tokens[*index].value = MaybeInlineToken::InlineCode;
              let mut index = *index + 1;
              while index < maybe_tokens.len() {
                maybe_tokens[index].value = MaybeInlineToken::Code;
                index += 1;
              }
              code_map.remove(&repeat);
              maybe_tokens.push_back(Token {
                value: MaybeInlineToken::Text,
                span: Span {
                  start: raw_start + start,
                  end: raw_start + start + 1,
                },
              });
              maybe_tokens.push_back(Token {
                value: MaybeInlineToken::InlineCodeEnd,
                span: Span {
                  start: raw_start + start + 1,
                  end: raw_start + start + repeat,
                },
              });
              return CallbackReturn::Move(repeat + 1);
            } else {
              return CallbackReturn::Text(repeat + 1);
            }
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
        _ => CallbackReturn::None,
      },
    );
    self.maybe_tokens = maybe_tokens;
    self.close_link_deque = close_link_deque;
  }

  fn process_inline_code(&mut self, span: Span) {
    let mut depth = 1;
    let start = span.end;
    let mut end = span.end;
    self.tokens.push(Token {
      value: InlineToken::InlineCodeStart,
      span,
    });
    let mut codes = vec![];
    while depth > 0 {
      let maybe_token = self.maybe_tokens.pop_front().unwrap();
      let span = maybe_token.span;
      let value = maybe_token.value;
      match value {
        MaybeInlineToken::InlineCodeStart => depth += 1,
        MaybeInlineToken::InlineCodeEnd => depth -= 1,
        _ => {}
      }
      if depth != 0 {
        end = span.end;
        codes.push(span);
      } else {
        self.tokens.push(Token {
          value: InlineToken::Code(codes),
          span: Span { start, end },
        });
        self.tokens.push(Token {
          value: InlineToken::InlineCodeEnd,
          span,
        });
        return;
      }
    }
  }

  fn process_link(&mut self) {}

  fn process_tokens(&mut self) {
    let len = self.maybe_tokens.len();
    while !self.maybe_tokens.is_empty() {
      let index = len - self.maybe_tokens.len();
      let maybe_token = self.maybe_tokens.pop_front().unwrap();
      match maybe_token {
        Token {
          value: MaybeInlineToken::InlineCodeStart,
          span,
        } => {
          self.process_inline_code(span);
        }
        Token {
          value: MaybeInlineToken::LinkStart,
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
  let mut inline_parser = InlineParser::new(bytes);
  let tokens = inline_parser.parse(&raws);
  println!("{:?}", tokens);
}
