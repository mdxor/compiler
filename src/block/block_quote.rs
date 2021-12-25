use super::document::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_block_quote<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<TokenValue<'source>>>,
) -> bool {
  let start = document.offset();
  let bytes = document.bytes();
  if bytes[0] != b'>' {
    if let Some(cur) = tree.cur() {
      if let TokenValue::BlockQuote(_) = tree[cur].item.value {
        tree.raise();
      }
    }
    return false;
  }

  let mut spaces = 0;
  let mut level = 1;
  let size = scan_while(&bytes[1..], |x| match x {
    b'>' => {
      level += 1;
      spaces = 0;
      true
    }
    b' ' => {
      spaces += 1;
      if spaces > 3 {
        false
      } else {
        true
      }
    }
    _ => false,
  }) + 1;
  document.forward(size);
  if let Some(cur) = tree.cur() {
    if let TokenValue::BlockQuote(_level) = tree[cur].item.value {
      if level == _level {
        tree.lower();
        return true;
      }
    }
  }
  tree.append(Token {
    start,
    value: TokenValue::BlockQuote(level),
  });
  tree.lower();
  true
}
