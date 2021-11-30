use crate::byte::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_setext_heading<'source>(
  d_bytes: &'source [u8],
  offset: usize,
  tree: &mut Tree<Token<'source>>,
) -> Option<()> {
  let cur = tree.cur().unwrap();
  if tree[cur].item.body != TokenBody::Paragraph {
    return None;
  }
  let bytes = &d_bytes[offset..];
  let c = *bytes.get(0)?;
  if !(c == b'-' || c == b'=') {
    return None;
  }
  let mut i = scan_ch_repeat(&bytes[1..], c);
  i += scan_blank_line(&bytes[i..])?;
  let level = if c == b'=' {
    HeadingLevel::H1
  } else {
    HeadingLevel::H2
  };
  tree[cur].item.end = i;
  tree[cur].item.body = TokenBody::SetextHeading(level);
  Some(())
}
