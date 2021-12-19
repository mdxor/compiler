use super::{
  atx_heading::*, block_quote::*, document::*, fenced_code::*, indented_code::*, list::*,
  paragraph::*, setext_heading::*,
};
use crate::scan::*;
use crate::token::*;
use crate::tree::*;

fn scan_container_block<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<'source>>,
) -> bool {
  if scan_block_quote(document, tree) | scan_list(document, tree) {
    scan_block(document, tree);
    tree.to_root_last_child();
    true
  } else {
    false
  }
}

fn scan_block<'source>(document: &mut Document<'source>, tree: &mut Tree<Token<'source>>) {
  let bytes = document.bytes();
  if scan_container_block(document, tree) | scan_inner_fenced_code(document, tree) {
    // do nothing
  } else if let Some(size) = scan_blank_line(bytes) {
    tree.append(Token {
      start: document.offset(),
      value: TokenValue::BlankLine,
    });
    document.forward(size);
  } else if scan_matched_spaces(document.bytes(), 4) {
    if let Some(cur) = tree.cur() {
      // An indented code block cannot interrupt a paragraph
      // https://github.github.com/gfm/#example-83
      if let TokenValue::Paragraph = tree[cur].item.value {
        scan_paragraph(document, tree);
        return;
      }
    }
    document.forward_for_next(4);
    scan_indented_code(document, tree);
  } else {
    let spaces_size = scan_while(bytes, |v| v == b' ');
    document.forward_for_next(spaces_size);
    if !scan_atx_heading(document, tree)
      && !scan_setext_heading(document, tree)
      && !scan_fenced_code(document, tree)
    {
      scan_paragraph(document, tree);
    }
  }
}

pub fn parse_source_to_blocks<'source>(source: &'source str) -> Tree<Token<'source>> {
  let mut document = Document::new(source);
  let mut tree = Tree::new();
  while document.bytes().len() > 0 {
    scan_block(&mut document, &mut tree);
  }
  tree
}
