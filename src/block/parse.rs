use super::{atx_heading::*, document::*, indented_code::*, paragraph::*, setext_heading::*};
use crate::scan::*;
use crate::token::*;
use crate::tree::*;

fn scan_block<'source>(document: &mut Document<'source>, tree: &mut Tree<Token<'source>>) {
  let bytes = &document.bytes[document.offset..];
  if let Some((size, remaining)) = scan_spaces(bytes, 4) {
    document.offset += size;
    document.remaining = remaining;
    scan_indented_code(document, tree);
  } else {
    let spaces = scan_while(bytes, |v| v == b' ');
    document.offset += spaces;
    document.remaining = spaces;
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
  }
  tree
}
