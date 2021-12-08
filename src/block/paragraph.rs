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
  let raw_size = scan_raw_line(bytes);
  let raw = &source[..raw_size];
  if tree[cur].item.body == TokenBody::Paragraph {
    let cur = tree.cur().unwrap();
    tree[cur].item.end = offset + raw_size;
    if let TokenBody::Raws(raws) = &mut tree[cur].item.body {
      raws.push(raw);
    }
  }
  tree.append(Token {
    start,
    end: offset + raw_size,
    body: TokenBody::Paragraph,
  });
  tree.lower();
  tree.append(Token {
    start: offset,
    end: offset + raw_size,
    body: TokenBody::Raws(vec![raw]),
  });
  tree.raise();
}

pub(crate) fn interrupt_paragraph<'source>(tree: &mut Tree<Token<'source>>) {
  if let Some(cur) = tree.cur() {
    if let TokenBody::Paragraph = tree[cur].item.body {}
  }
}
