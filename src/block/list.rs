use super::document::*;
use super::paragraph::*;
use crate::byte::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
struct ListMarker {
  ch: u8,
  starting_indent: usize,
  ending_indent: usize,
  size: usize,
  ordered_index: u64,
}
// TODO: is_tight
pub(crate) fn scan_list<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<'source>>,
) -> bool {
  let bytes = document.bytes();
  if let Some(_) = scan_blank_line(bytes) {
    return false;
  }
  if let Some(list_marker) = scan_list_marker(document.bytes()) {
    let ListMarker {
      starting_indent,
      ending_indent,
      ..
    } = list_marker;

    loop {
      if let Some(cur) = tree.cur() {
        if let TokenValue::List(ch, is_tight, start_index) = tree[cur].item.value {
        } else if let TokenValue::ListItem(indent) = tree[cur].item.value {
        } else {
          break;
        }
      } else {
        break;
      }
    }
  }
  false
}

fn append_list<'source>(
  list_marker: ListMarker,
  document: &mut Document<'source>,
  tree: &mut Tree<Token<'source>>,
) {
  let start = document.offset();
  let ListMarker {
    starting_indent,
    ending_indent,
    ordered_index,
    size,
    ch,
  } = list_marker;
  tree.append(Token {
    start,
    value: TokenValue::List(ch, true, ordered_index),
  });
  tree.lower();
  tree.append(Token {
    start,
    value: TokenValue::ListItem(starting_indent + ending_indent),
  });
  document.forward(size);
}

// return ch, starting indent, ending indent, size, ordered index
fn scan_list_marker<'source>(bytes: &'source [u8]) -> Option<ListMarker> {
  let starting_indent = scan_spaces(bytes);
  let mut ch: Option<u8> = None;
  let mut size = starting_indent;
  let mut ordered_index = 0;
  if let Some(c) = bytes.get(0) {
    if [b'-', b'+', b'*'].contains(c) {
      ch = Some(*c);
      size += 1;
    } else {
      scan_while(bytes, |v| {
        if b'0' <= v && v <= b'9' {
          ordered_index = ordered_index * 10 + u64::from(v - b'0');
          size += 1;
          if size >= 9 + starting_indent {
            return false;
          }
          true
        } else {
          false
        }
      });
      if size == starting_indent {
        return None;
      }
      if let Some(c) = bytes.get(size) {
        if [b'.', b')'].contains(c) {
          ch = Some(*c);
          size += 1;
        }
      }
    }
  }
  if let Some(ch) = ch {
    if let Some(_) = scan_eol(&bytes[size..]) {
      return Some(ListMarker {
        ch,
        starting_indent,
        ending_indent: 1,
        size,
        ordered_index,
      });
    }
    let ending_indent = scan_spaces(&bytes[size..]);
    if ending_indent >= 1 {
      size += ending_indent;
      return Some(ListMarker {
        ch,
        starting_indent,
        ending_indent,
        size,
        ordered_index,
      });
    }
  }
  None
}
