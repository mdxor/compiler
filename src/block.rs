use crate::byte::*;
use crate::scan::*;
use crate::token::*;
pub fn scan_block(bytes: &[u8], offset: usize) -> Result<Option<Token>, ()> {
  if let Some(block) = scan_atx_heading(bytes, offset) {
    Ok(Some(block))
  } else {
    Err(())
  }
}

pub fn scan_atx_heading(bytes: &[u8], offset: usize) -> Option<Token> {
  let level = scan_ch_repeat(bytes, b'#');
  if bytes.get(level).copied().map_or(true, is_ascii_whitespace) {
    if let Some(heading_level) = HeadingLevel::new(level) {
      let mut end = offset + level;
      if bytes
        .get(level)
        .copied()
        .map_or(false, is_ascii_whitespace_no_nl)
      {
        end += 1;
      }
      return Some(Token {
        start: offset,
        end,
        body: TokenBody::ATXHeading(heading_level),
      });
    }
  }
  return None;
}

#[test]
fn test_atx_heading() {
  assert_eq!(
    scan_atx_heading(b"#### ", 0),
    Some(Token {
      start: 0,
      end: 5,
      body: TokenBody::ATXHeading(HeadingLevel::H4),
    })
  );
}
