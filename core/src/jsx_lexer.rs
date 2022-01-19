use crate::lexer::*;
use crate::token::*;
use std::collections::VecDeque;

pub struct JSXLexer<'a> {
  pub bytes: &'a [u8],
  pub cur_bytes: &'a [u8],
  pos: usize,
  index: usize,
  spans: &'a VecDeque<Span>,
}

impl<'a> JSXLexer<'a> {
  pub fn new(bytes: &'a [u8], spans: &'a VecDeque<Span>) -> Self {
    let pos = if spans.len() > 0 { spans[0].start } else { 0 };
    let cur_bytes = if spans.len() > 0 {
      let Span { start, end } = spans[0];
      &bytes[start..end]
    } else {
      b""
    };
    Self {
      bytes,
      cur_bytes,
      index: 0,
      pos,
      spans,
    }
  }

  fn bytes(&mut self) -> Option<&'a [u8]> {
    if !self.cur_bytes.is_empty() {
      return Some(self.cur_bytes);
    }
    if self.index == self.spans.len() - 1 {
      return None;
    }
    self.index += 1;
    let Span { start, end } = self.spans[self.index];
    self.cur_bytes = &self.bytes[start..end];
    self.pos = start;
    None
  }

  pub fn pos(&self) -> usize {
    self.pos
  }

  fn forward(&mut self, size: usize) -> usize {
    self.pos += size;
    self.cur_bytes = &self.cur_bytes[size..];
    self.pos
  }

  fn skip_spaces_newlines(&mut self) -> Option<&'a [u8]> {
    loop {
      let bytes = self.bytes()?;
      let (bytes, size) = spaces_newlines_0(bytes);
      self.forward(size);
      if !bytes.is_empty() {
        return Some(bytes);
      }
    }
    None
  }

  pub fn read_target_punctuator(&mut self, target: &[u8]) -> Option<Span> {
    let bytes = self.skip_spaces_newlines()?;
    let len = target.len();
    if bytes.len() >= len {
      if target == &bytes[..len] {
        return Some(Span {
          start: self.pos,
          end: self.forward(len),
        });
      }
    }
    None
  }

  pub fn read_token(&mut self) -> Option<JSToken> {
    let bytes = self.skip_spaces_newlines()?;
    let variable_keywords: Vec<&[u8]> = vec![b"null", b"undefined", b"true", b"false"];
    if let Some((_, size)) = identifier(bytes) {
      let word = &bytes[..size];
      let span = Span {
        start: self.pos,
        end: self.forward(size),
      };
      if is_reserved_word(word) {
        if variable_keywords.contains(&word) {
          return Some(JSToken::Keyword(span));
        }
        return None;
      }
      return Some(JSToken::Identifier(span));
    } else if let Some(size) = number(bytes) {
      return Some(JSToken::Number(Span {
        start: self.pos,
        end: self.forward(size),
      }));
    } else if let Some(size) = string_literal(bytes) {
      return Some(JSToken::String(Span {
        start: self.pos,
        end: self.forward(size),
      }));
    }
    let span = self.read_head_punctuator()?;
    Some(JSToken::Punctuator(span))
  }

  pub fn read_identifier(&mut self) -> Option<Span> {
    let bytes = self.skip_spaces_newlines()?;
    let (_, size) = identifier(bytes)?;
    let word = &bytes[..size];
    if !is_reserved_word(word) {
      return Some(Span {
        start: self.pos,
        end: self.forward(size),
      });
    }
    return None;
  }

  pub fn read_keyword(&mut self) -> Option<Span> {
    let bytes = self.skip_spaces_newlines()?;
    let (_, size) = identifier(bytes)?;
    let word = &bytes[..size];
    if is_reserved_word(word) {
      return Some(Span {
        start: self.pos,
        end: self.forward(size),
      });
    }
    return None;
  }

  pub fn read_jsx_text(&mut self) -> Option<Span> {
    let bytes = self.bytes()?;
    let size = take_while(bytes, |ch| ch != b'{' && ch != b'<').1;
    if size > 1 {
      Some(Span {
        start: self.pos,
        end: self.forward(size),
      })
    } else {
      None
    }
  }

  pub fn read_string_literal(&mut self) -> Option<Span> {
    let bytes = self.skip_spaces_newlines()?;
    let size = string_literal(bytes)?;
    return Some(Span {
      start: self.pos,
      end: self.forward(size),
    });
  }

  pub fn read_head_punctuator(&mut self) -> Option<Span> {
    let bytes = self.skip_spaces_newlines()?;
    if let Some(byte) = bytes.get(0) {
      if !byte.is_ascii_punctuation() {
        return None;
      }
      match byte {
        b'{' | b'(' | b'[' | b'<' | b'!' => {}
        _ => {
          return None;
        }
      }
      return Some(Span {
        start: self.pos,
        end: self.forward(1),
      });
    }
    None
  }

  pub fn read_mid_punctuator(&mut self) -> Option<Span> {
    let bytes = self.skip_spaces_newlines()?;
    if let Some(byte) = bytes.get(0) {
      if !byte.is_ascii_punctuation() {
        return None;
      }
      let mut size = 1;
      match byte {
        b'=' | b'|' | b'&' => {
          if let Some(second_byte) = bytes.get(1) {
            if second_byte == byte {
              size += 1;
            } else {
              return None;
            }
          } else {
            return None;
          }
        }
        b'?' | b'.' | b'(' | b'[' => {}
        b'+' | b'-' | b'*' | b'/' => {
          if let Some(b'=') = bytes.get(1) {
            size += 1;
          }
        }
        _ => {
          return None;
        }
      }
      return Some(Span {
        start: self.pos,
        end: self.forward(size),
      });
    }
    None
  }

  pub fn read_template(&mut self) -> Option<JSToken> {
    let bytes = self.bytes()?;
    let len = bytes.len();
    if let Some(b'`') = bytes.get(0) {
      return Some(JSToken::Punctuator(Span {
        start: self.pos,
        end: self.forward(1),
      }));
    }
    if let (Some(b'$'), Some(b'{')) = (bytes.get(0), bytes.get(1)) {
      return Some(JSToken::Punctuator(Span {
        start: self.pos,
        end: self.forward(2),
      }));
    }
    let mut escaped = false;
    for i in 0..len {
      if escaped {
        escaped = false;
        continue;
      }
      if let Some(b'\\') = bytes.get(0) {
        escaped = true;
        continue;
      }
      if let (Some(b'$'), Some(b'{')) = (bytes.get(i), bytes.get(i + 1)) {
        return Some(JSToken::Template(Span {
          start: self.pos,
          end: self.forward(i),
        }));
      }
    }
    return Some(JSToken::Template(Span {
      start: self.pos,
      end: self.forward(len),
    }));
  }

  pub fn finish_jsx(&mut self) -> Option<(usize, usize)> {
    let (bytes, size) = spaces_newlines_0(self.cur_bytes);
    if bytes.is_empty() {
      Some((self.forward(size), self.index + 1))
    } else {
      None
    }
  }

  pub fn finish_js(&mut self) -> Option<usize> {
    let (bytes, size) = take_while(self.cur_bytes, |c| {
      if c == b';' || c == b' ' {
        true
      } else {
        false
      }
    });
    let eol_size = eol(bytes)?.1;
    Some(self.forward(size + eol_size))
  }

  pub fn read_separator(&mut self) -> Option<usize> {
    let mut flag = false;
    let size = take_while(self.cur_bytes, |c| {
      if c == b'\r' || c == b'\n' || c == b';' {
        flag = true;
        return false;
      } else if c != b' ' {
        return false;
      }
      true
    })
    .1;
    if flag || size == self.cur_bytes.len() {
      Some(self.forward(size))
    } else {
      self.forward(size);
      None
    }
  }
}

fn is_reserved_word(word: &[u8]) -> bool {
  let reserved_words: Vec<&[u8]> = vec![
    b"abstract",
    b"else",
    b"instanceof",
    b"super",
    b"boolean",
    b"enum",
    b"int",
    b"switch",
    b"break",
    b"export",
    b"interface",
    b"synchronized",
    b"byte",
    b"extends",
    b"let",
    b"this",
    b"case",
    b"false",
    b"long",
    b"throw",
    b"catch",
    b"final",
    b"native",
    b"throws",
    b"char",
    b"finally",
    b"new",
    b"transient",
    b"class",
    b"float",
    b"null",
    b"true",
    b"const",
    b"for",
    b"package",
    b"try",
    b"continue",
    b"function",
    b"private",
    b"typeof",
    b"debugger",
    b"goto",
    b"protected",
    b"var",
    b"default",
    b"if",
    b"public",
    b"void",
    b"delete",
    b"implements",
    b"return",
    b"volatile",
    b"do",
    b"import",
    b"short",
    b"while",
    b"double",
    b"in",
    b"static",
  ];
  reserved_words.contains(&word)
}

fn identifier(bytes: &[u8]) -> Option<(&[u8], usize)> {
  let first_byte = bytes.first()?;
  if first_byte.is_ascii_alphabetic() || *first_byte == b'_' || *first_byte == b'$' {
    let (bytes, mut size) = take_while(&bytes[1..], |c| {
      c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
    });
    size += 1;
    return Some((bytes, size));
  }
  None
}

fn number(bytes: &[u8]) -> Option<usize> {
  let (_, size) = take_while(bytes, |c| c.is_ascii_digit());
  if size > 0 {
    return Some(size);
  }
  None
}

fn string_literal(bytes: &[u8]) -> Option<usize> {
  let len = bytes.len();
  if len == 0 {
    return None;
  }
  let ch = bytes[0];
  if ch == b'\'' || ch == b'"' {
    let mut escaped = false;
    let (bytes, size) = take_while(&bytes[1..], |c| {
      if c == b'\r' || c == b'\n' {
        return false;
      }
      if escaped {
        escaped = false;
        return true;
      }
      if c == ch {
        return false;
      }
      true
    });
    let bytes = single_char(bytes, ch)?;
    return Some(len - bytes.len());
  }
  None
}

fn spaces_newlines_0(bytes: &[u8]) -> (&[u8], usize) {
  take_while(bytes, |c| c == b' ' || c == b'\r' || c == b'\n')
}
