use crate::input::*;
use std::collections::HashSet;
#[cfg(test)]
use std::str;

pub struct JsLexer<'a> {
  reserved_words_set: HashSet<&'a [u8]>,
}
impl<'a> JsLexer<'a> {
  pub fn new() -> Self {
    let mut reserved_words_set = HashSet::new();
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
    reserved_words.iter().for_each(|word| {
      reserved_words_set.insert(*word);
    });
    Self { reserved_words_set }
  }

  fn is_reserved_word(&self, word: &[u8]) -> bool {
    self.reserved_words_set.contains(word)
  }

  fn import_export<'b>(&self, bytes: &'b [u8]) -> Option<(&'b [u8], usize)> {
    None
  }

  fn identifier<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    let first_byte = bytes.first()?;
    if first_byte.is_ascii_alphabetic() || *first_byte == b'_' || *first_byte == b'$' {
      let (rest_bytes, mut size) = take_while(&bytes[1..], |c| {
        c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
      });
      size += 1;
      let identifier = &bytes[..size];
      if !self.is_reserved_word(identifier) {
        return Some(rest_bytes);
      }
    }
    None
  }

  fn variable_declaration<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    None
  }

  // fn new_expression<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
  //   None
  // }

  fn arguments<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    let mut bytes = single_char(bytes, b'(')?;
    loop {
      bytes = spaces_newlines_0(bytes);
      if single_char(bytes, b']').is_some() {
        break;
      }
      bytes = self.array_member(bytes)?;
      bytes = spaces_newlines_0(bytes);
      if let Some(rest) = single_char(bytes, b',') {
        bytes = rest;
      } else {
        break;
      }
    }
    bytes = single_char(bytes, b')')?;
    Some(bytes)
  }

  // a.b, a[b]
  fn member_expression<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    if let Some(bytes) = single_char(bytes, b'.') {
      self.identifier(bytes)
    } else if let Some(bytes) = single_char(bytes, b'[') {
      let bytes = spaces_newlines_0(bytes);
      let bytes = self.expression(bytes)?;
      let bytes = spaces_newlines_0(bytes);
      let bytes = single_char(bytes, b']')?;
      Some(bytes)
    } else {
      None
    }
  }

  // a ? b : c
  fn conditional_expression<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    let mut bytes = single_char(bytes, b'?')?;
    bytes = spaces_newlines_0(bytes);
    bytes = self.expression(bytes)?;
    bytes = spaces_newlines_0(bytes);
    bytes = single_char(bytes, b'?')?;
    bytes = spaces_newlines_0(bytes);
    bytes = self.expression(bytes)?;
    Some(bytes)
  }

  // fn parenthesis_expression<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {

  // }

  fn call_expression<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    self.arguments(bytes)
  }

  fn binary_expression<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    let len = bytes.len();
    let rest = take_while(bytes, |c| c != b' ' && c != b'\r' && c != b'\n').0;
    let size = bytes.len() - rest.len();
    match &bytes[..size] {
      b"+" | b"-" | b"*" | b"/" | b"&&" | b"||" => {
        let bytes = self.expression(rest)?;
        Some(bytes)
      }
      _ => None,
    }
  }

  fn unary<'b>(&self, bytes: &'b [u8]) -> &'b [u8] {
    if let Some(bytes) = single_char(bytes, b'!') {
      let bytes = spaces_newlines_0(bytes);
      if let Some(bytes) = single_char(bytes, b'!') {
        let bytes = spaces_newlines_0(bytes);
        return bytes;
      }
      return bytes;
    }
    bytes
  }

  fn common_expression<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    let mut bytes = bytes;
    loop {
      if let Some(rest) = self.call_expression(bytes) {
        bytes = rest;
      } else if let Some(rest) = self.call_expression(bytes) {
        bytes = rest;
      } else if let Some(rest) = self.binary_expression(bytes) {
        bytes = rest;
      } else if let Some(rest) = self.conditional_expression(bytes) {
        bytes = rest;
      } else if let Some(rest) = self.member_expression(bytes) {
        bytes = rest;
      } else {
        break;
      }
    }
    Some(bytes)
  }

  pub fn expression<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    let mut bytes = spaces_newlines_0(bytes);
    bytes = self.unary(bytes);
    if let Some(rest) = single_char(bytes, b'(') {
      bytes = rest;
      bytes = self.expression(bytes)?;
      bytes = spaces_newlines_0(bytes);
      bytes = single_char(bytes, b')')?;
      bytes = spaces_newlines_0(bytes);
      bytes = self.common_expression(bytes)?;
      return Some(bytes);
    }
    bytes = self.pattern_like(bytes)?;
    bytes = spaces_newlines_0(bytes);
    bytes = self.common_expression(bytes)?;
    return Some(bytes);
  }

  fn object_key<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    let len = bytes.len();
    // string key
    if let Some((bytes, size)) = string_literal(bytes) {
      return Some(bytes);
    }
    if let Some(bytes) = self.identifier(bytes) {
      return Some(bytes);
    }
    let bytes = single_char(bytes, b'[')?;
    let bytes = self.expression(bytes)?;
    let bytes = single_char(bytes, b']')?;
    Some(bytes)
  }

  // string, number, boolean, null, undefined, object, array, variable
  fn pattern_like<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    if let Some(re) = literal(bytes) {
      return Some(re);
    } else if let Some(re) = number(bytes) {
      return Some(re);
    } else if let Some(re) = boolean(bytes) {
      return Some(re);
    } else if let Some(re) = null_undefined(bytes) {
      return Some(re);
    } else if let Some(re) = self.object(bytes) {
      return Some(re);
    } else if let Some(re) = self.array(bytes) {
      return Some(re);
    } else if let Some(re) = self.identifier(bytes) {
      return Some(re);
    }
    None
  }

  fn object<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    let mut bytes = single_char(bytes, b'{')?;
    loop {
      bytes = spaces_newlines_0(bytes);
      if let Some(_) = single_char(bytes, b'}') {
        break;
      }
      bytes = self.object_member(bytes)?;
      bytes = spaces_newlines_0(bytes);
      if let Some(rest) = single_char(bytes, b',') {
        bytes = rest;
      } else {
        break;
      }
    }
    bytes = single_char(bytes, b'}')?;
    Some(bytes)
  }

  fn object_member<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    let len = bytes.len();
    let mut bytes = bytes;
    if let Some(rest) = tag(bytes, b"...") {
      bytes = rest;
      bytes = spaces_newlines_0(bytes);
      bytes = self.expression(bytes)?;
      return Some(bytes);
    }
    bytes = self.object_key(bytes)?;
    bytes = spaces_newlines_0(bytes);
    if let Some(rest) = single_char(bytes, b':') {
      bytes = rest;
      return Some(bytes);
    }
    bytes = spaces_newlines_0(bytes);
    bytes = self.expression(bytes)?;
    Some(bytes)
  }

  fn array_member<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    let mut bytes = bytes;
    if let Some(rest) = tag(bytes, b"...") {
      bytes = rest;
      bytes = spaces_newlines_0(bytes);
      bytes = self.expression(bytes)?;
      return Some(bytes);
    }
    bytes = self.expression(bytes)?;
    Some(bytes)
  }

  fn array<'b>(&self, bytes: &'b [u8]) -> Option<&'b [u8]> {
    let mut bytes = single_char(bytes, b'[')?;
    loop {
      bytes = spaces_newlines_0(bytes);
      if single_char(bytes, b']').is_some() {
        break;
      }
      bytes = self.array_member(bytes)?;
      bytes = spaces_newlines_0(bytes);
      if let Some(rest) = single_char(bytes, b',') {
        bytes = rest;
      } else {
        break;
      }
    }
    bytes = single_char(bytes, b']')?;
    Some(bytes)
  }
}

fn literal<'a>(bytes: &'a [u8]) -> Option<&'a [u8]> {
  if let Some(re) = string_literal(bytes) {
    Some(re.0)
  } else if let Some(re) = template_literal(bytes) {
    Some(re)
  } else {
    None
  }
}

fn string_literal<'a>(bytes: &'a [u8]) -> Option<(&'a [u8], usize)> {
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
    return Some((bytes, bytes.len() - len));
  }
  None
}

fn template_literal<'a>(bytes: &'a [u8]) -> Option<&'a [u8]> {
  let len = bytes.len();
  let bytes = single_char(bytes, b'`')?;
  let mut escaped = false;
  let (bytes, size) = take_while(bytes, |c| {
    if escaped {
      escaped = false;
      return true;
    }
    if c == b'`' {
      return false;
    }
    true
  });
  let bytes = single_char(bytes, b'`')?;
  Some(bytes)
}

fn number(bytes: &[u8]) -> Option<&[u8]> {
  let (bytes, size) = take_while(bytes, |c| c.is_ascii_digit());
  if size > 0 {
    return Some(bytes);
  }
  None
}

fn boolean(bytes: &[u8]) -> Option<&[u8]> {
  if let Some(bytes) = tag(bytes, b"true") {
    Some(bytes)
  } else if let Some(bytes) = tag(bytes, b"false") {
    Some(bytes)
  } else {
    None
  }
}

fn null_undefined(bytes: &[u8]) -> Option<&[u8]> {
  if let Some(bytes) = tag(bytes, b"null") {
    Some(bytes)
  } else if let Some(bytes) = tag(bytes, b"undefined") {
    Some(bytes)
  } else {
    None
  }
}

fn spaces_newlines_0(bytes: &[u8]) -> &[u8] {
  take_while(bytes, |c| c == b' ' || c == b'\r' || c == b'\n').0
}

fn spaces_newlines_1(bytes: &[u8]) -> Option<&[u8]> {
  let rest = spaces_newlines_0(bytes);
  if bytes.len() - rest.len() > 0 {
    return Some(rest);
  }
  None
}

#[test]
fn test_expression_parse() {
  let cases: Vec<&[u8]> = vec![
    // b"10",
    // b"abc",
    // b"import",
    // b"10 + a",
    // b"a.b(10)",
    b"{a:b,c:[10, true]}",
  ];
  // let cases: Vec<&[u8]> = vec![b"10+ a"];
  let mut js_lexer = JsLexer::new();
  let mut result = vec![];
  for case in &cases {
    if let Some(rest) = js_lexer.expression(case) {
      result.push(Some(
        str::from_utf8(&case[..case.len() - rest.len()]).unwrap(),
      ));
    } else {
      result.push(None);
    }
  }
  insta::assert_yaml_snapshot!(result);
}
