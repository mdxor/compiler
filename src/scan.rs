use crate::byte::*;
use memchr::memchr;

// scan a single character
pub(crate) fn scan_ch(data: &[u8], c: u8) -> usize {
  if !data.is_empty() && data[0] == c {
    1
  } else {
    0
  }
}

pub(crate) fn scan_while<F>(data: &[u8], mut f: F) -> usize
where
  F: FnMut(u8) -> bool,
{
  data.iter().take_while(|&&c| f(c)).count()
}

pub(crate) fn scan_rev_while<F>(data: &[u8], mut f: F) -> usize
where
  F: FnMut(u8) -> bool,
{
  data.iter().rev().take_while(|&&c| f(c)).count()
}

pub(crate) fn scan_ch_repeat(data: &[u8], c: u8) -> usize {
  scan_while(data, |x| x == c)
}

// Note: this scans ASCII whitespace only, for Unicode whitespace use
// a different function.
pub(crate) fn scan_whitespace_no_nl(data: &[u8]) -> usize {
  scan_while(data, is_ascii_whitespace_no_nl)
}

fn scan_attr_value_chars(data: &[u8]) -> usize {
  scan_while(data, is_valid_unquoted_attr_value_char)
}

pub(crate) fn scan_eol(bytes: &[u8]) -> Option<usize> {
  if bytes.is_empty() {
    return Some(0);
  }
  match bytes[0] {
    b'\n' => Some(1),
    b'\r' => Some(if bytes.get(1) == Some(&b'\n') { 2 } else { 1 }),
    _ => None,
  }
}

pub(crate) fn scan_blank_line(bytes: &[u8]) -> Option<usize> {
  let i = scan_whitespace_no_nl(bytes);
  scan_eol(&bytes[i..]).map(|n| i + n)
}

pub(crate) fn scan_raw_line(bytes: &[u8]) -> usize {
  memchr(b'\n', bytes).map_or(bytes.len(), |x| x + 1)
}

// return the matched bytes size, and the remaining spaces
pub(crate) fn scan_matched_spaces(bytes: &[u8], spaces: usize) -> Option<(usize, usize)> {
  let mut _spaces = 0;
  let mut index = 0;
  while index < bytes.len() {
    match bytes[index] {
      b' ' => {
        index += 1;
        _spaces += 1;
      }
      b'\t' => {
        index += 1;
        _spaces += 4;
      }
      _ => break,
    }
    if _spaces >= spaces {
      return Some((index, _spaces - spaces));
    }
  }
  return None;
}

pub(crate) fn scan_spaces_by_range(bytes: &[u8], min: usize, max: usize) -> Option<(usize, usize)> {
  let mut spaces = 0;
  let mut index = 0;
  while index < bytes.len() {
    match bytes[index] {
      b' ' => {
        index += 1;
        spaces += 1;
      }
      b'\t' => {
        index += 1;
        spaces += 4;
      }
      _ => break,
    }
    if spaces >= max {
      break;
    }
  }
  if spaces >= min && spaces <= max {
    Some((index, spaces - max))
  } else {
    None
  }
}

pub(crate) fn scan_spaces(bytes: &[u8]) -> (usize, usize) {
  let mut spaces = 0;
  let mut index = 0;
  while index < bytes.len() {
    match bytes[index] {
      b' ' => {
        index += 1;
        spaces += 1;
      }
      b'\t' => {
        index += 1;
        spaces += 4;
      }
      _ => break,
    }
  }
  (index, spaces)
}
