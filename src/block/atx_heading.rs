use super::document::*;
use crate::byte::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_atx_heading<'source>(
  document: &Document<'source>,
  tree: &mut Tree<Token<'source>>,
) -> bool {
  let offset = document.offset;
  let start = document.block_start;
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
      let end = offset + starting_size + raw_size;
      tree.append(Token {
        start,
        end,
        value: TokenValue::ATXHeading(heading_level),
      });
      tree.lower();
      tree.append(Token {
        start: offset + starting_size,
        end,
        value: TokenValue::Raw(raw),
      });
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
