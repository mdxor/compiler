use super::document::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_paragraph<'source>(
  document: &Document<'source>,
  tree: &mut Tree<Token<'source>>,
) {
  let offset = document.offset;
  let start = document.block_start;
  let bytes = document.bytes();
  let source = document.source();
  let cur = tree.cur().unwrap();
  let raw_line_size = scan_raw_line(bytes);
  let raw_line = &source[..raw_line_size];
  if tree[cur].item.body == TokenBody::Paragraph {
    let cur = tree.cur().unwrap();
    if let Some(last_child) = tree[cur].last_child {
      if let TokenBody::Raw(last_raw) = tree[last_child].item.body {
        let raw = &document.source[offset - last_raw.len()..offset + raw_line_size];
        tree[last_child].item.end = offset + raw_line_size;
        tree[last_child].item.body = TokenBody::Raw(raw);
        return;
      }
    }
  }
  tree.append(Token {
    start,
    end: offset + raw_line_size,
    body: TokenBody::Paragraph,
  });
  tree.lower();
  tree.append(Token {
    start: offset,
    end: offset + raw_line_size,
    body: TokenBody::Raw(raw_line),
  });
  tree.raise();
}
