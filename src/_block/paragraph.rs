use super::document::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_paragraph<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<TokenValue<'source>>>,
) {
  let start = document.offset();
  let bytes = document.bytes();
  let source = document.source();
  let cur = tree.cur().unwrap();
  let (raw_size, raw) = scan_raw_line(bytes, source);
  if tree[cur].item.value == TokenValue::Paragraph {
    let cur = tree.cur().unwrap();
    tree.lower_to_last();
    if let Some(_) = tree.cur() {
      tree.append(Token {
        start,
        value: TokenValue::Raw(raw),
      });
      document.forward(raw_size);
    }
    tree.raise();
  }
  tree.append(Token {
    start,
    value: TokenValue::Paragraph,
  });
  tree.lower();
  tree.append(Token {
    start,
    value: TokenValue::Raw(raw),
  });
  document.forward(raw_size);
  tree.raise();
}

fn scan_hr<'source>(bytes: &'source [u8]) -> bool {
  if bytes.len() < 3 {
    return false;
  }
  let c = bytes[0];
  if c != b'-' && c != b'*' && c != b'_' {
    return false;
  }
  let size = scan_ch_repeat(bytes, c);
  if size >= 3 {
    if scan_eol(&bytes[size..]).is_some() {
      return true;
    }
  }
  false
}

fn scan_atx<'source>(bytes: &'source [u8]) -> bool {
  let size = scan_ch_repeat(bytes, b'#');
  if size <= 6 && size >= 1 {
    if scan_eol(&bytes[size..]).is_some() || scan_ch(&bytes[size..], b' ') {
      return true;
    }
  }
  false
}

fn scan_code<'source>(bytes: &'source [u8]) -> bool {
  scan_ch_repeat(bytes, b'`') >= 3 || scan_ch_repeat(bytes, b'~') >= 3
}

fn scan_quote<'source>(bytes: &'source [u8]) -> bool {
  scan_ch(bytes, b'>')
}

fn match_quote<'source>(bytes: &'source [u8], target: usize, spaces: usize) -> Option<usize> {
  let mut result = false;
  let mut spaces = spaces;
  let mut level = 0;
  let size = scan_while(bytes, |ch| match ch {
    b'>' => {
      level += 1;
      spaces = 0;
      if level == target {
        result = true;
        return false;
      }
      true
    }
    b' ' => {
      spaces += 1;
      if spaces >= 4 {
        false
      } else {
        true
      }
    }
    _ => false,
  });
  if result {
    Some(size)
  } else {
    None
  }
}

// for ref def, table
pub(crate) fn continue_paragraph<'source>(
  bytes: &'source [u8],
  tree: &mut Tree<Token<TokenValue<'source>>>,
) -> Option<usize> {
  let mut size = 0;
  let spine = tree.spine();
  let len = spine.len();
  let mut spaces = scan_spaces(&bytes[size..]);
  if len > 2 {
    let mut index = 1;
    size += spaces;
    while index < len - 1 {
      let id = spine[index];
      if let TokenValue::BlockQuote(level) = tree[id].item.value {
        if index == len - 2 {
          // do nothing
        } else if spaces >= 4 {
          return None;
        } else if let Some(quote_size) = match_quote(&bytes[size..], level, spaces) {
          spaces = 0;
          size += quote_size;
          spaces = scan_spaces(&bytes[size..]);
          size += spaces;
        } else {
          return None;
        }
      } else if let TokenValue::List(_, __, ___) = tree[id].item.value {
        let last_child = tree[id].last_child.unwrap();
        if let TokenValue::ListItem(indent) = tree[last_child].item.value {
          if spaces >= indent {
            spaces - indent;
            index += 1;
          } else {
            return None;
          }
        }
      }
      index += 1;
    }
  }
  // TODO: JSX
  if spaces >= 4 {
    return Some(size);
  }
  let bytes = &bytes[size + spaces..];
  if scan_eol(bytes).is_none() && !scan_atx(bytes)
    || !scan_quote(bytes)
    || !scan_hr(bytes)
    || !scan_code(bytes)
  {
    Some(size)
  } else {
    None
  }
}
