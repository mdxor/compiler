use crate::input::*;
use crate::token::*;

pub struct JSLexer<'a> {
  bytes: &'a [u8],
  pos: usize,
  index: usize,
  spans: &'a Vec<Span>,
}

impl<'a> JSLexer<'a> {
  pub fn new(bytes: &'a [u8], spans: &'a Vec<Span>) -> Self {
    let pos = if spans.len() > 0 { spans[0].start } else { 0 };
    Self {
      bytes,
      index: 0,
      pos,
      spans,
    }
  }

  fn bytes(&mut self) -> Option<&'a [u8]> {
    if let Some(Span { end, .. }) = self.spans.get(self.index) {
      if self.pos == *end {
        self.index += 1;
        if let Some(Span { end, start }) = self.spans.get(self.index) {
          self.pos = *start;
          return Some(&self.bytes[self.pos..*end]);
        }
      } else {
        return Some(&self.bytes[self.pos..*end]);
      }
    }
    None
  }
  pub fn pos(&self) -> usize {
    self.pos
  }

  fn forward(&mut self, size: usize) -> usize {
    self.pos += size;
    self.pos
  }

  fn skip_spaces_newlines(&mut self) -> Option<&'a [u8]> {
    loop {
      let bytes = self.bytes()?;
      let (bytes, size) = take_while(bytes, |c| c == b' ' || c == b'\r' || c == b'\n');
      self.forward(size);
      if !bytes.is_empty() {
        return Some(bytes);
      }
    }
    None
  }

  pub fn next_token(&mut self) -> Option<Token<JSToken>> {
    let bytes = self.bytes()?;
    let bytes = self.skip_spaces_newlines()?;
    if let Some((_, size)) = identifier(bytes) {
      let word = &bytes[..size];
      let span = Span {
        start: self.pos,
        end: self.forward(size),
      };
      if is_reserved_word(word) {
        return Some(Token {
          value: JSToken::Keyword,
          span,
        });
      } else {
        return Some(Token {
          value: JSToken::Identifier,
          span,
        });
      }
    }
    if let Some(size) = number(bytes) {
      return Some(Token {
        value: JSToken::Number,
        span: Span {
          start: self.pos,
          end: self.forward(size),
        },
      });
    }
    let first_byte = bytes.first()?;
    if first_byte.is_ascii_punctuation() {
      return self.handle_punctuation(bytes, *first_byte);
    }
    None
  }

  pub fn next_template(&mut self) -> Option<Token<JSToken>> {
    let bytes = self.bytes()?;
    if let Some(_) = single_char(bytes, b'`') {
      return Some(Token {
        value: JSToken::Punctuator,
        span: Span {
          start: self.pos,
          end: self.forward(1),
        },
      });
    }
    let mut size = 0;
    let mut bytes_iter = bytes.iter();
    loop {
      if let Some(i) = bytes_iter.position(|c| *c == b'$') {
        size += i + 2;
        if let Some(b'{') = bytes_iter.next() {
          if size == 2 {
            return Some(Token {
              value: JSToken::Punctuator,
              span: Span {
                start: self.pos,
                end: self.forward(2),
              },
            });
          }
          return Some(Token {
            value: JSToken::Template,
            span: Span {
              start: self.pos,
              end: self.forward(size - 2),
            },
          });
        }
      } else {
        return Some(Token {
          value: JSToken::Template,
          span: Span {
            start: self.pos,
            end: self.forward(bytes.len()),
          },
        });
      }
    }
    None
  }

  fn handle_punctuation(&mut self, bytes: &[u8], byte: u8) -> Option<Token<JSToken>> {
    let mut size = 1;
    match byte {
      b'`' => {}
      b'"' => {}
      b'\'' => {}
      b'?' | b'{' | b'}' | b'(' | b')' | b':' | b',' | b'[' | b']' => {}
      b'&' => {
        if let Some(b'&') = bytes.get(1) {
          size += 1;
        }
      }
      b'|' => {}
      _ => {
        return None;
      }
    }
    Some(Token {
      value: JSToken::Punctuator,
      span: Span {
        start: self.pos,
        end: self.forward(size),
      },
    })
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

#[test]
fn test_jsx_lexer() {
  let case = b"{a:b,c:[10, true]}";
  let spans = vec![Span {
    start: 0,
    end: case.len(),
  }];
  let mut result = vec![];
  let mut js_lexer = JSLexer::new(case, &spans);
  loop {
    if let Some(token) = js_lexer.next_token() {
      result.push(token);
    } else {
      break;
    }
  }
  println!("{:?}", result);
}
