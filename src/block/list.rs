use super::document::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_list<'source>(
  document: &Document<'source>,
  tree: &mut Tree<Token<'source>>,
) -> bool {
  false
}

pub(crate) fn continue_list<'source>(
  document: &Document<'source>,
  tree: &mut Tree<Token<'source>>,
) -> bool {
  if let Some(parent) = tree.peek_up() {
    if let TokenValue::List(is_tight, ch, indent) = tree[parent].item.value {}
  }
  false
}
