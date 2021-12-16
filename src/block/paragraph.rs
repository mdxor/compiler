use super::document::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_paragraph<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<'source>>,
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
