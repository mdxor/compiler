use crate::byte::*;
use crate::document::*;
use crate::raw::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
lazy_static! {
  static ref SPECIAL_BYTES: [bool; 256] = {
    let mut bytes = [false; 256];
    let special_bytes = [b'*', b'_', b'~', b'[', b']', b'`', b'|'];
    for &byte in &special_bytes {
      bytes[byte as usize] = true;
    }
    bytes
  };
}

fn is_left_flanking_delimiter(raw: &str, start: usize, end: usize) -> bool {
  let bytes = raw.as_bytes();
  let len = bytes.len();
  if end >= len - 1 || is_ascii_whitespace(bytes[end + 1]) {
    return false;
  }
  let next_char = if let Some(c) = raw.chars().nth(end + 1) {
    c
  } else {
    return true;
  };
  if is_punctuation(next_char)
    && (start == 0
      || is_ascii_whitespace(bytes[start - 1])
      || is_punctuation(raw[..start].chars().last().unwrap()))
  {
    return true;
  }
  return false;
}

fn is_right_flanking_delimiter(raw: &str, start: usize, end: usize) -> bool {
  let bytes = raw.as_bytes();
  let len = bytes.len();
  if start == 0 || is_ascii_whitespace(bytes[start - 1]) {
    return false;
  }
  let prev_char = if let Some(c) = raw.chars().nth(start - 1) {
    c
  } else {
    return true;
  };
  if is_punctuation(prev_char)
    && (end >= len - 1
      || is_ascii_whitespace(bytes[end + 1])
      || is_punctuation(raw[end..].chars().next().unwrap()))
  {
    return true;
  }
  return false;
}

pub(crate) fn parse_block_to_inlines<'source>(
  block_tree: &mut Tree<Item<Token<'source>>>,
  document: &mut Document<'source>,
  block_id: usize,
) {
  let bytes = document.bytes;
  let mut raw = Raw::new(block_tree, bytes, block_id);
  let mut tokens: Vec<Item<InlineToken>> = vec![];
  raw.iterate_bytes(&*SPECIAL_BYTES, |raw, raw_start, start, end| {
    let offset = raw_start + start;
    if start == end {
      let byte = bytes[raw_start + start];
      match byte {
        b'*' | b'_' | b'~' => {
          let repeat = scan_ch_repeat(&bytes[offset..], b'*');
          let can_open = is_left_flanking_delimiter(raw, start, start + repeat - 1);
          let can_close = is_right_flanking_delimiter(raw, start, start + repeat - 1);
          if !can_open && !can_close {
            return LoopInstruction::Text(repeat);
          }
          if byte == b'~' {
            if repeat == 2 {
              tokens.push(Item {
                start: offset,
                end: offset + repeat,
                value: InlineToken::MaybeEmphasis(byte, repeat, can_open, can_close),
              });
              return LoopInstruction::Move(repeat);
            } else {
              return LoopInstruction::Move(repeat);
            }
          }
          tokens.push(Item {
            start: offset,
            end: offset + repeat,
            value: InlineToken::MaybeEmphasis(byte, repeat, can_open, can_close),
          });
          return LoopInstruction::Move(repeat);
        }
        b'[' => {
          tokens.push(Item {
            start: offset,
            end: offset + 1,
            value: InlineToken::MaybeLinkStart,
          });
          return LoopInstruction::None;
        }
        b']' => {}
        b'`' => {}
        _ => (),
      }
    } else if end - start == 1 {
      // escaped
      if bytes[offset] == b'\\' {
        if bytes[offset + 1] == b'`' {}
      }
    }
    tokens.push(Item {
      start: offset,
      end: offset + end - start,
      value: InlineToken::Text(&raw[start..end + 1]),
    });
    return LoopInstruction::None;
  });
}
