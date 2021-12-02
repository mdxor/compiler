use crate::block::atx_heading::*;
use crate::block::indented_code::*;
use crate::block::paragraph::*;
use crate::block::setext_heading::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;

fn scan_block<'source>(
  d_bytes: &'source [u8],
  d_source: &'source str,
  offset: usize,
  tree: &mut Tree<Token<'source>>,
) {
  let bytes = &d_bytes[offset..];
  if let Some((size, remaining)) = scan_spaces(bytes, 4) {
    scan_indented_code(d_source, d_bytes, offset + remaining, tree, remaining);
  } else {
    let spaces = scan_while(bytes, |v| v == b' ');
    if !scan_atx_heading(d_source, d_bytes, offset + spaces, tree, spaces)
      && !scan_setext_heading(d_bytes, offset + spaces, tree)
    {
      scan_paragraph(d_source, d_bytes, offset, tree, spaces);
    }
  }
}

pub fn parse_document_to_blocks<'source>(d_source: &'source str) -> Tree<Token<'source>> {
  let mut tree = Tree::new();
  let d_bytes = d_source.as_bytes();
  let mut offset = 0;
  while offset < d_bytes.len() {
    scan_block(d_bytes, d_source, offset, &mut tree);
    let cur = tree.cur().unwrap();
    offset = tree[cur].item.end;
  }
  tree
}
