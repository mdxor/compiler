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

pub(crate) fn scan_raw_line<'source>(bytes: &[u8], source: &'source str) -> (usize, &'source str) {
  let size = memchr(b'\n', bytes).map_or(bytes.len(), |x| x + 1);
  let raw = &source[..size];
  (size, raw)
}

pub(crate) fn scan_matched_spaces(bytes: &[u8], spaces: usize) -> bool {
  let mut index = 0;
  while index < bytes.len() {
    if bytes[index] == b' ' {
      index += 1;
    } else {
      break;
    }
    if index == spaces {
      return true;
    }
  }
  return false;
}

pub(crate) fn scan_spaces_up_to(bytes: &[u8], max: usize) -> usize {
  let mut index = 0;
  while index < bytes.len() {
    if bytes[index] == b' ' {
      index += 1;
    } else {
      break;
    }
    if index == max {
      return index;
    }
  }
  index
}

pub(crate) fn scan_spaces(bytes: &[u8]) -> usize {
  let mut index = 0;
  while index < bytes.len() {
    if bytes[index] == b' ' {
      index += 1;
    } else {
      break;
    }
  }
  index
}

pub(crate) fn scan_ends_with(bytes: &[u8], ch: u8, with_escaped: bool) -> Option<usize> {
  let mut escaped = false;
  let mut size = 0;
  let mut flag = false;
  scan_while(bytes, |c| {
    if c == b'\n' || c == b'\r' {
      return flag;
    }
    size += 1;
    if c == ch {
      size += 1;
      if !escaped {
        flag = true;
      } else {
        escaped = false;
      }
      return flag;
    }
    if with_escaped && !escaped && c == b'\\' {
      escaped = true;
    }
    escaped = false;
    return flag;
  });
  if flag {
    Some(size)
  } else {
    None
  }
}
