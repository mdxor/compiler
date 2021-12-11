use super::document::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_indented_code<'source>(
  document: &Document<'source>,
  tree: &mut Tree<Token<'source>>,
) {
  let bytes = document.bytes();
  let offset = document.offset;
  let source = document.source();
  let start = document.block_start;
  let remaining_spaces = document.remaining_spaces;
  if let Some(cur) = tree.cur() {
    if let TokenValue::IndentedCode(starting_spaces) = tree[cur].item.value {
      let (spaces_size, spaces) = scan_spaces_up_to(bytes, starting_spaces - remaining_spaces);
      let (raw_size, raw) = scan_raw_line(&bytes[spaces_size..], &source[spaces_size..]);
      let end = offset + spaces_size + raw_size;
      tree.lower_to_last();
      if spaces > 0 {
        tree.append(Token {
          start,
          end: start,
          value: TokenValue::Code(&"   "[..spaces]),
        });
      }
      tree.append(Token {
        start,
        end,
        value: TokenValue::Code(raw),
      });
      tree[cur].item.end = end;
      return;
    }
  }
  let (spaces_size, spaces) = scan_spaces(bytes);
  let starting_spaces = remaining_spaces + spaces;
  let (raw_size, raw) = scan_raw_line(&bytes[spaces_size..], &source[spaces_size..]);
  let end = offset + spaces_size + raw_size;
  tree.append(Token {
    start,
    end,
    value: TokenValue::IndentedCode(starting_spaces),
  });
  tree.lower();
  tree.append(Token {
    start,
    end,
    value: TokenValue::Code(raw),
  });
  tree.raise();
}
