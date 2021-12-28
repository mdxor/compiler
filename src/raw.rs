use crate::byte::*;
use crate::document::*;
use crate::token::*;
use crate::tree::*;
pub enum LoopInstruction {
  None,
  Text(usize),
  Move(usize),
}
pub fn iterate_raw<'source, F>(
  tree: &Tree<Item<Token<'source>>>,
  block_id: usize,
  bytes: &'source [u8],
  special_bytes: &'source [bool; 256],
  mut callback: F,
) where
  // raw, raw offset, start, end
  F: FnMut(&'source str, usize, usize, usize) -> LoopInstruction,
{
  if let Some(child) = tree[block_id].child {
    let mut cur = child;
    loop {
      let start = tree[cur].item.start;
      if let Token::Raw(raw) = tree[cur].item.value {
        callback(raw, start, 0, 0);
        let mut text_start = 0;
        let mut index = 0;
        let len = raw.len();
        loop {
          if index >= len {
            break;
          }
          let byte = bytes[start + index];
          if byte == b'\\' && index < len - 1 {
            if is_ascii_punctuation(bytes[start + index + 1]) {
              if index > text_start {
                callback(raw, start, text_start, index);
              }
              callback(raw, start, index, index + 2);
              text_start = index + 2;
              index += 2;
              continue;
            }
          }
          if special_bytes[byte as usize] {
            if index > text_start {
              callback(raw, start, text_start, index);
            }
            match callback(raw, start, index, index + 1) {
              LoopInstruction::None => text_start = index + 1,
              LoopInstruction::Text(size) => {
                text_start = index;
                index += size;
              }
              LoopInstruction::Move(size) => {
                index += size;
                text_start = index;
                continue;
              }
            }
          }
          index += 1;
        }
        if text_start < raw.len() - 1 {
          callback(raw, start, text_start, raw.len());
        }
      }
      if let Some(next) = tree[cur].next {
        cur = next;
      } else {
        break;
      }
    }
  }
}

pub fn inverse_iterate_bytes() {}
