use super::document::*;
use crate::byte::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_list<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<'source>>,
) -> bool {
  if continue_list(document, tree) {
    return true;
  }
  false
}

pub(crate) fn continue_list<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<'source>>,
) -> bool {
  if let Some(parent) = tree.peek_up() {
    if let TokenValue::List(ref mut is_tight, ch, indent) = tree[parent].item.value {
      let bytes = document.bytes;
      let (spaces_size, spaces) = scan_spaces(bytes);
      // blank line
      if let Some(_) = scan_eol(&bytes[spaces_size..]) {
        tree.lower();
        return true;
      }
    }
  }
  false
}

// return ch, ending indent, size, ordered index
fn scan_list_marker<'source>(bytes: &'source [u8]) -> Option<(u8, usize, usize, u64)> {
  let mut ch: Option<u8> = None;
  let mut size = 0;
  let mut ordered_index = 0;
  if let Some(c) = bytes.get(0) {
    if [b'-', b'+', b'*'].contains(c) {
      ch = Some(*c);
      size = 1;
    } else {
      scan_while(bytes, |v| {
        if b'0' <= v && v <= b'9' {
          ordered_index = ordered_index * 10 + u64::from(v - b'0');
          size += 1;
          if size >= 9 {
            return false;
          }
          true
        } else {
          false
        }
      });
      if let Some(c) = bytes.get(size) {
        if [b'.', b')'].contains(c) {
          ch = Some(*c);
          size += 1;
        }
      }
    }
  }
  if let Some(ch) = ch {
    let (spaces_size, spaces) = scan_spaces(&bytes[size..]);
    size += spaces_size;
    if let Some(_) = scan_eol(&bytes[size..]) {
      return Some((ch, spaces, size, ordered_index));
    }
  }
  None
}
