use crate::lexer::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;

pub(crate) fn continue_paragraph<'source>(
  bytes: &'source [u8],
  source: &'source str,
  tree: &mut Tree<Item<Token<'source>>>,
) -> Option<usize> {
  if let Some(size) = interrupt_container(bytes, source, tree) {
    let spaces = scan_spaces(bytes);
    if spaces >= 4 {
      return None;
    }
    let source = &source[size + spaces..];
    let bytes = &bytes[size + spaces..];
    if scan_atx_heading_start(source).is_none()
      && scan_blank_line(source).is_none()
      && scan_block_quote(bytes).is_none()
      && scan_thematic_break(source).is_none()
      && scan_open_fenced_code(source).is_none()
    {
      return Some(size);
    }
  }
  None
}

// size, spine's level
pub(crate) fn continue_container<'source>(
  bytes: &'source [u8],
  source: &'source str,
  tree: &mut Tree<Item<Token<'source>>>,
) -> (usize, usize) {
  let mut size = 0;
  let mut offset = 0;
  let spine = tree.spine();
  let len = spine.len();
  let mut level = 1;
  if len > level {
    let mut spaces = scan_spaces(&bytes[offset..]);
    offset += spaces;
    while level < len {
      let id = spine[level];
      if let Token::BlockQuote(level) = tree[id].item.value {
        if spaces >= 4 {
          return (size, level);
        } else if let Some((quote_size, quote_level)) = scan_block_quote(&bytes[size..]) {
          if level == quote_level {
            spaces = 0;
            offset += quote_size;
            size = offset;
            spaces = scan_spaces(&bytes[size..]);
            offset += spaces;
          }
        }
        return (size, level);
      } else if let Token::List(_, __, ___) = tree[id].item.value {
        let last_child = tree[id].last_child.unwrap();
        if let Token::ListItem(indent) = tree[last_child].item.value {
          if spaces >= indent {
            spaces -= indent;
            size = offset - spaces;
            level += 1;
          } else {
            return (size - spaces, level);
          }
        }
      }
      level += 1;
    }
  }
  (size, level)
}

pub(crate) fn interrupt_container<'source>(
  bytes: &'source [u8],
  source: &'source str,
  tree: &mut Tree<Item<Token<'source>>>,
) -> Option<usize> {
  let (size, level) = continue_container(bytes, source, tree);
  if level == tree.spine().len() {
    Some(size)
  } else {
    None
  }
}
