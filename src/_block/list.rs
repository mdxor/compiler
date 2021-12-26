use super::document::*;
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
  tree: &mut Tree<Token<TokenValue<'source>>>,
) -> bool {
  let bytes = document.bytes();
  if let Some(_) = scan_blank_line(bytes) {
    return false;
  }
  let starting_indent = scan_spaces(bytes);
  let mut pre_indent = 0;
  loop {
    if let Some(cur) = tree.cur() {
      if let TokenValue::List(..) = tree[cur].item.value {
        let last_item = tree[cur].last_child.unwrap();
        if let TokenValue::ListItem(last_item_indent) = tree[last_item].item.value {
          if starting_indent >= last_item_indent + 2 {
            pre_indent += last_item_indent;
            tree.lower_to_last();
            tree.lower_to_last();
          } else if starting_indent > pre_indent + 4 {
            return false;
          } else {
            break;
          }
        }
      } else {
        break;
      }
    }
  }
  document.forward(pre_indent);
  if starting_indent >= 4 {
    document.forward(pre_indent);
    return true;
  }
  if let Some(list_marker) = scan_list_marker(&bytes[starting_indent..], starting_indent) {
    if let Some(cur) = tree.cur() {
      if let TokenValue::List(list_ch, is_tight, _) = tree[cur].item.value {
        let ListMarker {
          ch,
          starting_indent,
          ending_indent,
          ..
        } = list_marker;
        if ch == list_ch {
          tree.lower();
          tree.append(Token {
            start: document.offset(),
            value: TokenValue::ListItem(starting_indent + ending_indent),
          });
        } else {
          append_list(list_marker, document, tree);
        }
      }
    }
  }

  if pre_indent == 0 {
    false
  } else {
    true
  }
}

fn append_list<'source>(
  list_marker: ListMarker,
  document: &mut Document<'source>,
  tree: &mut Tree<Token<TokenValue<'source>>>,
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
fn scan_list_marker<'source>(bytes: &'source [u8], starting_indent: usize) -> Option<ListMarker> {
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
