use super::document::*;
use crate::byte::*;
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

  let (spaces_size, spaces) = scan_spaces(bytes);
  let starting_spaces = remaining_spaces + spaces;
  let mut first_line = true;
  tree.append(Token {
    start,
    end: start,
    body: TokenBody::IndentedCode,
  });
  tree.lower();
  let mut index = spaces_size;
  while index < bytes.len() {
    if first_line {
      first_line = false;
      let raw_line_size = scan_raw_line(bytes);
      let raw_line = &source[index..index + raw_line_size];
      index += raw_line_size;
      tree.append(Token {
        start: offset,
        end: index + offset,
        body: TokenBody::Code(raw_line),
      });
      continue;
    }
    if let Some((spaces_size, spaces)) = scan_spaces_by_range(&bytes[index..], 4, starting_spaces) {
      index += spaces_size;
      if spaces > 0 {
        tree.append(Token {
          // meaningful?
          start: offset + index,
          end: offset + index,
          body: TokenBody::Code(&"   "[..spaces]),
        });
      }
      let raw_line_size = scan_raw_line(&bytes[index..]);
      let raw_line = &source[offset + index..offset + index + raw_line_size];
      tree.append(Token {
        start: index,
        end: index + raw_line_size,
        body: TokenBody::Code(raw_line),
      });
      index += raw_line_size;
    } else {
      break;
    }
  }
}
