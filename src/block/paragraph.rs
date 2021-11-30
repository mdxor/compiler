use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_paragraph<'source>(
  d_source: &'source str,
  d_bytes: &'source [u8],
  offset: usize,
  tree: &mut Tree<Token<'source>>,
) {
  let bytes = &d_bytes[offset..];
  let source = &d_source[offset..];
  let cur = tree.cur().unwrap();
  let raw_line_size = scan_raw_line(bytes, offset);
  let raw_line = &source[..raw_line_size];
  if tree[cur].item.body == TokenBody::Paragraph {
    tree.lower();
    let cur = tree.cur().unwrap();
    if let TokenBody::Raw(last_raw) = tree[cur].item.body {
      let raw = &d_source[offset - last_raw.len()..offset + raw_line_size];
      tree[cur].item.end = offset + raw_line_size;
      tree[cur].item.body = TokenBody::Raw(raw);
      tree.raise();
      return;
    }
    tree.raise();
  }
  tree.append(Token {
    start: offset,
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
