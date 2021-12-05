use super::{atx_heading::*, document::*, indented_code::*, paragraph::*, setext_heading::*};
use crate::scan::*;
use crate::token::*;
use crate::tree::*;

fn scan_block<'source>(document: &mut Document<'source>, tree: &mut Tree<Token<'source>>) {
  let bytes = document.bytes();
  if let Some(size) = scan_blank_line(bytes) {
    let start = document.block_start;
    tree.append(Token {
      start,
      end: start + size,
      body: TokenBody::BlankLine,
    });
  } else if let Some((size, remaining_spaces)) = scan_matched_spaces(document.bytes(), 4) {
    document.offset += size;
    document.remaining_spaces = remaining_spaces;
    scan_indented_code(document, tree);
  } else {
    let spaces_size = scan_while(bytes, |v| v == b' ');
    document.offset += spaces_size;
    if !scan_atx_heading(document, tree) && !scan_setext_heading(document, tree) {
      scan_paragraph(document, tree);
    }
  }
}

pub fn parse_document_to_blocks<'source>(source: &'source str) -> Tree<Token<'source>> {
  let mut document = Document::new(source);
  let mut tree = Tree::new();
  while document.offset < document.bytes.len() {
    scan_block(&mut document, &mut tree);
    let cur = tree.cur().unwrap();
    document.offset = tree[cur].item.end;
    document.block_start = document.offset;
    document.remaining_spaces = 0;
  }
  tree
}
