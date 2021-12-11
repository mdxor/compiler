use super::{
  atx_heading::*, block_quote::*, document::*, fenced_code::*, indented_code::*, paragraph::*,
  setext_heading::*,
};
use crate::scan::*;
use crate::token::*;
use crate::tree::*;

fn scan_container_block<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<'source>>,
) -> bool {
  if scan_block_quote(document, tree) {
    prepare_next_block(document, tree);
    scan_block(document, tree);
    tree.raise();
    true
  } else {
    false
  }
}

fn prepare_next_block<'source>(document: &mut Document<'source>, tree: &mut Tree<Token<'source>>) {
  let cur = tree.cur().unwrap();
  document.offset = tree[cur].item.end;
  document.block_start = document.offset;
}

fn scan_block<'source>(document: &mut Document<'source>, tree: &mut Tree<Token<'source>>) {
  let bytes = document.bytes();
  if scan_inner_fenced_code(document, tree) {
    // do nothing
  } else if let Some(size) = scan_blank_line(bytes) {
    let start = document.block_start;
    tree.append(Token {
      start,
      end: start + size,
      value: TokenValue::BlankLine,
    });
  } else if let Some((size, remaining_spaces)) = scan_matched_spaces(document.bytes(), 4) {
    if let Some(cur) = tree.cur() {
      // An indented code block cannot interrupt a paragraph
      // https://github.github.com/gfm/#example-83
      if let TokenValue::Paragraph = tree[cur].item.value {
        document.offset += size;
        scan_paragraph(document, tree);
        prepare_next_block(document, tree);
        return;
      }
    }
    document.offset += size;
    document.remaining_spaces = remaining_spaces;
    scan_indented_code(document, tree);
  } else {
    let spaces_size = scan_while(bytes, |v| v == b' ');
    document.offset += spaces_size;
    if !scan_container_block(document, tree)
      && !scan_atx_heading(document, tree)
      && !scan_setext_heading(document, tree)
      && !scan_fenced_code(document, tree)
    {
      scan_paragraph(document, tree);
    }
  }
  prepare_next_block(document, tree);
}

pub fn parse_document_to_blocks<'source>(source: &'source str) -> Tree<Token<'source>> {
  let mut document = Document::new(source);
  let mut tree = Tree::new();
  while document.offset < document.bytes.len() {
    scan_block(&mut document, &mut tree);
  }
  tree
}
