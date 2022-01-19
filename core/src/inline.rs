use crate::jsx_parser::*;
use crate::lexer::*;
use crate::md_lexer::*;
use crate::token::*;
use std::collections::VecDeque;
use std::str;

pub struct InlineParser<'a> {
  source: &'a str,
  bytes: &'a [u8],
  raws: &'a Vec<Span>,
  special_bytes: [bool; 256],
  maybe_tokens: VecDeque<Token<InlineToken>>,
  index: usize,
  text_start: usize,
  // token index, ch, repeat
  open_delimiters: Vec<(usize, u8, usize)>,
  link_delimiters: VecDeque<usize>,
  delimiter_bottom: Option<usize>,
  // raw index, token index,
  open_link: Option<(usize, usize)>,
  pos: usize,
}

impl<'a> InlineParser<'a> {
  pub fn new(source: &'a str, bytes: &'a [u8], raws: &'a Vec<Span>) -> Self {
    let mut special_bytes = [false; 256];
    let specials = [b'*', b'_', b'~', b'[', b']', b'`', b'<', b'!', b'\r', b'\n'];
    for &byte in &specials {
      special_bytes[byte as usize] = true;
    }
    let pos = if raws.len() > 0 { raws[0].start } else { 0 };
    Self {
      source,
      bytes,
      raws,
      special_bytes,
      maybe_tokens: VecDeque::new(),
      index: 0,
      pos,
      text_start: pos,
      open_link: None,
      delimiter_bottom: None,
      open_delimiters: vec![],
      link_delimiters: VecDeque::new(),
    }
  }

  pub fn parse(&mut self) -> AST<Token<InlineToken>> {
    self.parse_raws();
    let start = self.raws.first().unwrap().start;
    let (children, end) = self.parse_tokens();
    AST {
      children,
      span: Span { start, end },
    }
  }

  fn parse_raws(&mut self) {
    while self.index < self.raws.len() {
      if self.pos >= self.raws[self.index].end {
        self.index += 1;
        if self.index == self.raws.len() {
          break;
        }
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

  fn scan_link_url_title(&mut self) -> Option<(Span, Vec<Span>)> {
    let mut index = self.index;
    if let Some((pos, url_span, title_spans)) = link_url_title(|| {
      if let Some(span) = self.raws.get(index) {
        let bytes = &self.bytes[span.start..span.end];
        if index == self.index {
          let bytes = &self.bytes[self.pos + 1..span.end];
          return Some((bytes, self.pos + 1));
        } else {
          return Some((bytes, span.start));
        }
      } else {
        return None;
      }
    }) {
      self.maybe_tokens.push_back(Token {
        value: InlineToken::LinkEnd,
        span: Span {
          start: self.pos,
          end: pos + 1,
        },
      });
      self.index = index;
      self.pos = pos + 1;
      self.text_start = self.pos;
      return Some((url_span, title_spans));
    }
    None
  }

  fn scan_inline_code(&mut self, repeat: usize) -> bool {
    let mut index = self.index;
    let mut pos = self.pos + repeat - 1;
    let mut codes: Vec<Span> = vec![];
    let mut raw = &self.raws[index];
    let mut raw_bytes = &self.bytes[raw.start..raw.end];
    let mut code_start = self.pos + repeat;
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
      // TODO
      if raw_bytes[i] == b'`' {
        let (_, end_repeat) = ch_repeat(&raw_bytes[i..], b'`');
        if end_repeat == repeat {
          codes.push(Span {
            start: code_start,
            end: pos,
          });
          pos += repeat;
          self.maybe_tokens.push_back(Token {
            value: InlineToken::Code(codes),
            span: Span {
              start: self.pos,
              end: pos,
            },
          });
          return self.forward_pos(pos - self.pos);
        }
      }
    }
    false
  }

  fn match_inlink_delimiter(&mut self) {
    while !self.link_delimiters.is_empty() {
      let token_index = self.link_delimiters.pop_front().unwrap();
      self.match_delimiter(token_index);
    }
  }

  fn match_delimiter(&mut self, token_index: usize) {
    if let Token {
      value:
        InlineToken::MaybeEmphasis {
          ch,
          repeat,
          can_open,
          can_close,
        },
      ..
    } = self.maybe_tokens[token_index]
    {
      if can_close && !self.open_delimiters.is_empty() {
        let mut delimiter_index = self.open_delimiters.len() - 1;
        loop {
          let (open_token_index, open_ch, open_repeat) = self.open_delimiters[delimiter_index];
          if let Some(bottom) = self.delimiter_bottom {
            if open_token_index < bottom {
              break;
            }
          }
          if ch == open_ch && open_repeat == repeat {
            self.maybe_tokens[token_index].value = InlineToken::EmphasisEnd;
            self.maybe_tokens[open_token_index].value = InlineToken::EmphasisStart;
            self.open_delimiters.truncate(delimiter_index);
          }
          if delimiter_index == 0 {
            break;
          } else {
            delimiter_index -= 1;
          }
        }
      }
      if can_open {
        self.open_delimiters.push((token_index, ch, repeat));
      }
    }
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
        if byte == b'~' && repeat != 2 {
          self.pos += repeat;
          return true;
        }
        let can_open = is_left_flanking_delimiter(raw_bytes, start, start + repeat);
        let can_close = is_right_flanking_delimiter(raw_bytes, start, start + repeat);
        if !can_open && !can_close {
          self.pos += repeat;
          return true;
        }

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
        self.forward_pos(repeat);
        if let Some((raw_index, _)) = self.open_link {
          if raw_index == self.index {
            self.link_delimiters.push_back(self.maybe_tokens.len() - 1);
            return true;
          }
        } else {
          self.match_delimiter(self.maybe_tokens.len() - 1);
        }
        return true;
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
        self.open_link = Some((self.index, self.maybe_tokens.len()));
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
        if let Some((raw_index, token_index)) = self.open_link {
          self.open_link = None;
          if raw_index == self.index {
            if let Some((url, title)) = self.scan_link_url_title() {
              self.delimiter_bottom = Some(token_index);
              self.match_inlink_delimiter();
              self.delimiter_bottom = None;
              self.maybe_tokens[token_index].value = InlineToken::LinkStart { url, title };
              return true;
            }
          }
        }
        return false;
      }
      b'<' => {
        if let Some(size) = uri(bytes) {
          self.maybe_tokens.push_back(Token {
            value: InlineToken::AutoLink(false),
            span: Span {
              start: self.pos,
              end: self.pos + size,
            },
          });
          return self.forward_pos(size);
        } else {
          let spans = VecDeque::from(vec![Span {
            start: self.pos,
            end: raw.end,
          }]);
          let mut parser = JSXParser::new(self.source, self.bytes, &spans);
          if let Some((element, end, _)) = parser.jsx() {
            let span = Span {
              start: self.pos,
              end,
            };
            self.forward_pos(end - self.pos);
            self.maybe_tokens.push_back(Token {
              value: InlineToken::JSX(element),
              span,
            });
            true
          } else {
            false
          }
        }
      }
      b'\r' | b'\n' => {
        let size = if byte == b'\r' { 2 } else { 1 };
        let mut span = Span {
          start: self.pos,
          end: self.pos + size,
        };
        self.forward_pos(size);
        if self.index != self.raws.len() - 1 {
          if let Some(Token {
            value: InlineToken::TextSegment,
            span: text_span,
          }) = self.maybe_tokens.back_mut()
          {
            let bytes = &self.bytes[text_span.start..text_span.end];
            let spaces = rev_spaces0(bytes);
            if spaces >= 2 {
              let end = text_span.end - spaces;
              text_span.end = end;
              span.start = end;
              self.maybe_tokens.push_back(Token {
                value: InlineToken::HardBreak,
                span,
              });
              return true;
            }
          }
          self.maybe_tokens.push_back(Token {
            value: InlineToken::SoftBreak,
            span,
          });
        }
        return true;
      }
      _ => {
        return false;
      }
    }
  }

  fn parse_tokens(&mut self) -> (Vec<Token<InlineToken>>, usize) {
    let mut children = vec![];
    while !self.maybe_tokens.is_empty() {
      let maybe_token = self.maybe_tokens.pop_front().unwrap();
      let mut span = maybe_token.span;
      let token_value = maybe_token.value;
      match token_value {
        InlineToken::MaybeLinkStart
        | InlineToken::TextSegment
        | InlineToken::MaybeLinkStart { .. } => {
          if let Some(Token {
            value: InlineToken::Text(text_spans),
            span: text_span,
          }) = children.last_mut()
          {
            text_span.end = span.end;
            text_spans.push(span);
          } else {
            children.push(Token {
              value: InlineToken::Text(vec![span.clone()]),
              span,
            });
          }
        }
        InlineToken::LinkStart { url, title } => {
          let start = span.start;
          let (text_children, end) = self.parse_tokens();
          children.push(Token {
            value: InlineToken::Link {
              url,
              title,
              text_children,
            },
            span: Span { start, end },
          });
        }
        InlineToken::EmphasisStart => {
          let start = span.start;
          let (em_children, end) = self.parse_tokens();
          children.push(Token {
            value: InlineToken::Emphasis(em_children),
            span: Span { start, end },
          });
        }
        InlineToken::LinkEnd | InlineToken::EmphasisEnd => {
          return (children, span.end);
        }
        _ => children.push(Token {
          value: token_value,
          span,
        }),
      }
    }
    if children.len() == 0 {
      return (children, 0);
    }
    let end = children.last().unwrap().span.end;
    (children, end)
  }
}

// #[test]
// fn test_parse_raws() {
//   let bytes = b"`123`2*code*  \n1<https://mdxor.com>";
//   let raws = vec![
//     Span { start: 0, end: 15 },
//     Span {
//       start: 15,
//       end: bytes.len(),
//     },
//   ];
//   let mut inline_parser = InlineParser::new(bytes, &raws);
//   let tokens = inline_parser.parse();
//   println!("{:?}", tokens);
// }
