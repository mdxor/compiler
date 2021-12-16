use super::document::*;
use crate::byte::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_thematic_break<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<'source>>,
) -> bool {
  let bytes = document.bytes();
  let chars = vec![b'-', b'_', b'*'];
  let mut count = 0;
  let mut c = b' ';
  let mut size = scan_while(document.bytes(), |x| {
    if count == 0 {
      if chars.contains(&x) {
        count += 1;
        c = x;
        return true;
      }
    } else {
      if c == x {
        count += 1;
      }
    }
    if is_ascii_whitespace_no_nl(x) {
      return true;
    }
    false
  });
  if count >= 3 {
    if let Some(eol_size) = scan_eol(bytes) {
      size += eol_size;
      tree.append(Token {
        start: document.offset(),
        value: TokenValue::ThematicBreak,
      });
      document.forward(size);
      return true;
    }
  }
  false
}
