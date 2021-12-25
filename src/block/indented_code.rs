use super::document::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_indented_code<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<TokenValue<'source>>>,
) {
  let bytes = document.bytes();
  let source = document.source();
  let start = document.offset();
  if let Some(cur) = tree.cur() {
    if let TokenValue::IndentedCode(starting_spaces) = tree[cur].item.value {
      let spaces = scan_spaces_up_to(bytes, starting_spaces);
      let (raw_size, raw) = scan_raw_line(&bytes[spaces..], &source[spaces..]);
      tree.lower_to_last();
      if spaces > 0 {
        tree.append(Token {
          start,
          value: TokenValue::Code(&"   "[..spaces]),
        });
      }
      tree.append(Token {
        start,
        value: TokenValue::Code(raw),
      });
      document.forward(spaces + raw_size);
      return;
    }
  }
  let spaces = scan_spaces(bytes);
  let starting_spaces = spaces;
  let (raw_size, raw) = scan_raw_line(&bytes[spaces..], &source[spaces..]);
  tree.append(Token {
    start,
    value: TokenValue::IndentedCode(starting_spaces),
  });
  document.forward(spaces);
  tree.lower();
  tree.append(Token {
    start,
    value: TokenValue::Code(raw),
  });
  document.forward(raw_size);
  tree.raise();
}
