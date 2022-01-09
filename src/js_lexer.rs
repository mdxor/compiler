use crate::input::*;
pub fn js_variable(bytes: &[u8]) -> Option<(&[u8], usize)> {
  None
}

pub fn js_identifier(bytes: &[u8]) -> Option<(&[u8], usize)> {
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

// '' "" ``
pub fn js_string(bytes: &[u8]) -> Option<(&[u8], usize)> {
  let first_byte = bytes.first()?;
  if *first_byte == b'\'' || *first_byte == b'"' || *first_byte == b'`' {
    let mut escaped = false;
    let (bytes, size) = take_while(&bytes[1..], |c| {
      if *first_byte != b'`' {
        if c == b'\r' || c == b'\n' {
          return false;
        }
      }
      if escaped {
        escaped = false;
        return true;
      }
      if c == *first_byte {
        return false;
      }
      true
    });
    let bytes = single_char(bytes, *first_byte)?;
    return Some((bytes, size + 2));
  }

  None
}

pub fn js_number(bytes: &[u8]) -> Option<(&[u8], usize)> {
  let (bytes, size) = take_while(bytes, |c| c.is_ascii_digit());
  if size > 0 {
    return Some((bytes, size));
  }
  None
}

pub fn js_boolean(bytes: &[u8]) -> Option<(&[u8], usize)> {
  if let Some(bytes) = tag(bytes, b"true") {
    Some((bytes, 4))
  } else if let Some(bytes) = tag(bytes, b"false") {
    Some((bytes, 5))
  } else {
    None
  }
}

pub fn js_null_undefined(bytes: &[u8]) -> Option<(&[u8], usize)> {
  if let Some(bytes) = tag(bytes, b"null") {
    Some((bytes, 4))
  } else if let Some(bytes) = tag(bytes, b"undefined") {
    Some((bytes, 9))
  } else {
    None
  }
}

pub fn spaces_newlines_1(bytes: &[u8]) -> Option<(&[u8], usize)> {
  let (bytes, size) = take_while(bytes, |c| c == b' ' || c == b'\r' || c == b'\n');
  if size > 0 {
    return Some((bytes, size));
  }
  None
}
