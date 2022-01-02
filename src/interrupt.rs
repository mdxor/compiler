use crate::input::*;
use crate::lexer::*;
use crate::token::*;
use crate::tree::*;

pub(crate) fn continue_paragraph<'source>(
  bytes: &'source [u8],
  source: &'source str,
  tree: &mut Tree<Item<Token<'source>>>,
) -> Option<usize> {
  if let Some(size) = interrupt_container(bytes, source, tree) {
    let (bytes, spaces) = spaces(&bytes[size..]);
    if spaces >= 4 {
      return None;
    }
    if atx_heading_start(bytes).is_none()
      && eol(bytes).is_none()
      && bytes[0] != b'>'
      && thematic_break(bytes).is_none()
      && setext_heading(bytes).is_none()
      && open_fenced_code(bytes).is_none()
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
    let (_, mut spaces_size) = spaces(&bytes[offset..]);
    offset += spaces_size;
    while level < len {
      let id = spine[level];
      if let Token::BlockQuote(level) = tree[id].item.value {
        if spaces_size >= 4 {
          return (size, level);
        } else if let Some((quote_size, quote_level)) = block_quote(&bytes[size..]) {
          if level == quote_level {
            spaces_size = 0;
            offset += quote_size;
            size = offset;
            spaces_size = spaces(&bytes[size..]).1;
            offset += spaces_size;
          }
        }
        return (size, level);
      } else if let Token::List(_, __, ___) = tree[id].item.value {
        let last_child = tree[id].last_child.unwrap();
        if let Token::ListItem(indent) = tree[last_child].item.value {
          if spaces_size >= indent {
            spaces_size -= indent;
            size = offset - spaces_size;
            level += 1;
          } else {
            return (size - spaces_size, level);
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
