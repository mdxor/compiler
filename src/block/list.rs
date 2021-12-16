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
  if continue_list(document, tree) {
    return true;
  }
  let bytes = document.bytes();
  if let Some(list_marker) = scan_list_marker(bytes) {
    // indented code
    if list_marker.starting_indent >= 4 {
      return false;
    }
    append_list(list_marker, document, tree);
    return true;
  }
  false
}

pub(crate) fn continue_list<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<'source>>,
) -> bool {
  if let Some(parent) = tree.peek_parent() {
    if let TokenValue::List(list_ch, ref mut is_tight, list_indent, _) = tree[parent].item.value {
      let bytes = document.bytes();
      if let Some(list_marker) = scan_list_marker(bytes) {
        let ListMarker {
          ch,
          starting_indent,
          ending_indent,
          size,
          ordered_index,
        } = list_marker;
        let cur = tree.cur().unwrap();
        let source = document.source();
        let start = document.offset();
        if let TokenValue::ListItem(prev_indent) = tree[cur].item.value {
          if starting_indent >= prev_indent {
            if starting_indent - prev_indent >= 4 {
              scan_paragraph(document, tree);
              return false;
            } else {
              tree.lower_to_last();
              append_list(list_marker, document, tree);
              return true;
            }
          }
        }
        if starting_indent < list_indent {}
      }
    }
  }
  false
}

fn append_list<'source>(
  list_marker: ListMarker,
  document: &Document<'source>,
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
  let end = start + size;
  tree.append(Token {
    start,
    value: TokenValue::List(ch, true, starting_indent, ordered_index),
  });
  tree.lower();
  tree.append(Token {
    start,
    value: TokenValue::ListItem(starting_indent + ending_indent),
  });
}

// return ch, starting_indent, ending indent, size, ordered index
// TODO: remaining spaces
fn scan_list_marker<'source>(bytes: &'source [u8]) -> Option<ListMarker> {
  let (starting_size, starting_indent) = scan_spaces(bytes);
  let mut ch: Option<u8> = None;
  let mut size = starting_size;
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
          if size >= 9 + starting_size {
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
    let (ending_size, ending_indent) = scan_spaces(&bytes[size..]);
    if ending_indent >= 1 {
      size += ending_size;
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
