use crate::byte::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_atx_heading<'source>(
  d_source: &'source str,
  d_bytes: &'source [u8],
  offset: usize,
  tree: &mut Tree<Token<'source>>,
) -> Option<()> {
  let bytes = &d_bytes[offset..];
  let source = &d_source[offset..];

  let level = scan_ch_repeat(bytes, b'#');
  if bytes.get(level).copied().map_or(true, is_ascii_whitespace) {
    if let Some(heading_level) = HeadingLevel::new(level) {
      let start = offset;
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
        println!("{}", raw_line_start);
        let raw_line_size = scan_raw_line(&d_bytes[raw_line_start..]);
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
      return Some(());
    }
  }
  None
}

#[test]
fn test_atx_heading() {
  let source = "# 123";
  let mut tree = Tree::new();
  scan_atx_heading(source, source.as_bytes(), 0, &mut tree);
  insta::assert_yaml_snapshot!(tree);
}
