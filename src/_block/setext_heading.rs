use super::document::*;
use crate::byte::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_setext_heading<'source>(
  document: &Document<'source>,
  tree: &mut Tree<Token<TokenValue<'source>>>,
) -> bool {
  let cur = tree.cur().unwrap();
  if tree[cur].item.value != TokenValue::Paragraph {
    return false;
  }
  let bytes = &document.bytes();
  if let Some(c) = bytes.get(0) {
    if *c == b'-' || *c == b'=' {
      let mut i = scan_ch_repeat(&bytes[1..], *c);
      if let Some(_i) = scan_blank_line(&bytes[i..]) {
        i += _i;
        let level = if *c == b'=' {
          HeadingLevel::H1
        } else {
          HeadingLevel::H2
        };
        tree[cur].item.value = TokenValue::SetextHeading(level);
        return true;
      }
    }
  }
  return false;
}
