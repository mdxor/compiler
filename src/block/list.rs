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

// return indent, size, remaining spaces, ch, ordered list index
fn scan_list_item<'source>(
  bytes: &'source [u8],
  source: &'source str,
) -> Option<(usize, usize, usize, u8, Option<&'source str>)> {
  let mut ch: Option<u8> = None;
  let mut indent = 0;
  let mut starting_size = 0;
  let mut start_index: Option<&'source str> = None;
  if let Some(c) = bytes.get(0) {
    if [b'-', b'+', b'*'].contains(c) {
      ch = Some(*c);
      indent = 1;
    } else {
      let digit_size = scan_while(bytes, is_digit);
      if digit_size < 10 {
        if let Some(c) = bytes.get(digit_size) {
          if [b'.', b')'].contains(c) {
            ch = Some(*c);
            start_index = Some(&source[..digit_size]);
            indent = digit_size + 1;
          }
        }
      }
    }
  }
  if let Some(ch) = ch {
    starting_size = indent;
    let (spaces_size, spaces) = scan_spaces(&bytes[starting_size..]);
    if let Some(_) = scan_eol(&bytes[starting_size + spaces_size..]) {
      indent += 1;
      return Some((indent, starting_size, 0, ch, start_index));
    }
    if spaces_size >= 5 {
      indent += 1;
      starting_size += 1;
      let (_, remaining_spaces) = scan_matched_spaces(&bytes[starting_size..], 1).unwrap();
      return Some((indent, starting_size, remaining_spaces, ch, start_index));
    }
    indent += spaces;
    starting_size += spaces_size;
    return Some((indent, starting_size, 0, ch, start_index));
  }
  None
}
