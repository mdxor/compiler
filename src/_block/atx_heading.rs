use super::document::*;
use crate::byte::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_atx_heading<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<TokenValue<'source>>>,
) -> bool {
  let start = document.offset();
  let bytes = document.bytes();
  let source = document.source();

  let level = scan_ch_repeat(bytes, b'#');
  let mut starting_size = level;
  let c = bytes.get(level).copied();
  if c.map_or(true, is_ascii_whitespace) {
    if let Some(heading_level) = HeadingLevel::new(level) {
      let mut raw = "";
      let mut raw_size = 0;

      if c.map_or(false, is_ascii_whitespace_no_nl) {
        starting_size += 1;

        let result = scan_raw_line(&bytes[starting_size..], &source[starting_size..]);
        raw_size = result.0;
        raw = result.1;
      }
      tree.append(Token {
        start,
        value: TokenValue::ATXHeading(heading_level),
      });
      document.forward(starting_size);
      tree.lower();
      tree.append(Token {
        start: document.offset(),
        value: TokenValue::Raw(raw),
      });
      document.forward(raw_size);
      tree.raise();
      return true;
    }
  }
  false
}

// #[test]
// fn test_atx_heading() {
//   let source = "# 123";
//   let mut tree = Tree::new();
//   scan_atx_heading(source, source.as_bytes(), 0, &mut tree, 0);
//   insta::assert_yaml_snapshot!(tree);
// }
