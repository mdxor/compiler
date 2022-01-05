use crate::input::*;
use crate::lexer::*;
use crate::raw::*;
use crate::token::*;
use std::collections::VecDeque;

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

struct ParserHelper {
  // token index, repeat
  pub close_code_deque: VecDeque<(usize, usize)>,
  pub close_link_deque: VecDeque<usize>,
}
impl ParserHelper {
  pub fn new() -> Self {
    Self {
      close_code_deque: VecDeque::new(),
      close_link_deque: VecDeque::new(),
    }
  }
}

pub struct InlineParser<'a> {
  bytes: &'a [u8],
  special_bytes: [bool; 256],
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
    }
  }

  pub fn parse(&self, raws: &Vec<Span>) -> Vec<Token<InlineToken>> {
    let (mut tokens, mut parser_helper) = self.parse_raws(raws);
    self.process_tokens(&mut tokens);
    tokens
  }

  pub fn parse_raws(&self, raws: &Vec<Span>) -> (Vec<Token<InlineToken>>, ParserHelper) {
    let mut tokens: Vec<Token<InlineToken>> = vec![];
    let mut parser_helper = ParserHelper::new();
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
              parser_helper
                .close_code_deque
                .push_back((tokens.len(), repeat));
              tokens.push(Token {
                value: InlineToken::MaybeInlineCode {
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
                tokens.push(Token {
                  value: InlineToken::MaybeEmphasis {
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
                tokens.push(Token {
                  value: InlineToken::MaybeEmphasis {
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
            b'[' => tokens.push(Token {
              value: InlineToken::MaybeLinkStart,
              span: Span {
                start: raw_start + start,
                end: raw_start + start + 1,
              },
            }),
            b']' => {
              parser_helper.close_link_deque.push_back(tokens.len());
              tokens.push(Token {
                value: InlineToken::MaybeLinkEnd,
                span: Span {
                  start: raw_start + start,
                  end: raw_start + start + 1,
                },
              });
            }
            b'<' => {
              if let Some(size) = uri(&raw_bytes[start..]) {
                tokens.push(Token {
                  value: InlineToken::AutoLink(false),
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
            parser_helper
              .close_code_deque
              .push_back((tokens.len(), repeat));
            tokens.push(Token {
              value: InlineToken::MaybeInlineCode {
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
            tokens.push(Token {
              value: InlineToken::EscapedText,
              span: Span {
                start: raw_start + start,
                end: raw_start + start + 2,
              },
            });
          }
          CallbackReturn::None
        }
        CallbackType::Text(raw_bytes, raw_start, start, end) => {
          tokens.push(Token {
            value: InlineToken::Text,
            span: Span {
              start: raw_start + start,
              end: raw_start + end,
            },
          });
          CallbackReturn::None
        }
        CallbackType::HardBreak(raw_start, start, end) => {
          tokens.push(Token {
            value: InlineToken::HardBreak,
            span: Span {
              start: raw_start + start,
              end: raw_start + end,
            },
          });
          CallbackReturn::None
        }
        CallbackType::SoftBreak(raw_start, start, end) => {
          tokens.push(Token {
            value: InlineToken::SoftBreak,
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

    self.process_tokens(&mut tokens);
    (tokens, parser_helper)
  }

  fn process_tokens(&self, tokens: &mut Vec<Token<InlineToken>>) {
    let mut emphasis_stack: Vec<(u8, usize, usize)> = vec![];
    let mut index = 0;
    let len = tokens.len();
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
  let inlineParser = InlineParser::new(bytes);
  let tokens = inlineParser.parse(&raws);
  println!("{:?}", tokens);
}
