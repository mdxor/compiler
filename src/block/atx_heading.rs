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
  if bytes.get(level).copied().map_or(true, is_ascii_whitespace) {
    if let Some(heading_level) = HeadingLevel::new(level) {
      let mut end = offset + level;
      let mut raw_line_start = end;
      let mut raw_line = "";
      if bytes
        .get(level)
        .copied()
        .map_or(false, is_ascii_whitespace_no_nl)
      {
        end += 1;
        raw_line_start = end;
        let raw_line_size = scan_raw_line(&document.bytes[raw_line_start..]);
        raw_line = &source[raw_line_start..raw_line_start + raw_line_size];
        end += raw_line_size;
      }
      tree.append(Token {
        start,
        end,
        body: TokenBody::ATXHeading(heading_level),
      });
      tree.lower();
      tree.append(Token {
        start: raw_line_start,
        end,
        body: TokenBody::Raw(raw_line),
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
