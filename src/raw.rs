use crate::block::document::*;
use crate::token::*;
use crate::tree::*;
// lazy_static! {
//   static ref SPECIAL_BYTES: [bool; 256] = {
//     let mut bytes = [false; 256];
//     let special_bytes = [
//       b'\n', b'\r', b'*', b'_', b'&', b'\\', b'[', b']', b'<', b'!', b'`', b'|', b'~',
//     ];
//     for &byte in &special_bytes {
//       bytes[byte as usize] = true;
//     }
//     bytes
//   };
// }
struct Raw<'source> {
  block_tree: &'source Tree<Token<'source>>,
  block_id: usize,
  bytes: &'source [u8],
}

impl<'source> Raw<'source> {
  pub fn new(
    block_tree: &'source Tree<Token<'source>>,
    bytes: &'source [u8],
    block_id: usize,
  ) -> Self {
    Raw {
      block_tree,
      bytes,
      block_id,
    }
  }
  pub fn iterate_bytes<F1, F2>(
    &mut self,
    special_bytes: &'source [bool; 256],
    mut special_byte_callback: F1,
    mut token_callback: F2,
  ) where
    F1: FnMut(u8, usize),
    F2: FnMut(InlineToken<'source>, usize),
  {
    if let Some(child) = self.block_tree[self.block_id].child {
      let mut cur = child;
      loop {
        let start = self.block_tree[cur].item.start;
        if let TokenValue::Raw(raw) = self.block_tree[cur].item.value {
          let mut text_start = 0;
          for index in 0..raw.len() {
            let byte = self.bytes[index];
            if special_bytes[byte as usize] {
              if index > text_start {
                token_callback(
                  InlineToken::Text(&raw[text_start..index]),
                  start + text_start,
                );
              }
              special_byte_callback(byte, start + index);
              text_start = index + 1;
            }
          }
          if text_start < raw.len() - 1 {
            token_callback(
              InlineToken::Text(&raw[text_start..raw.len()]),
              start + text_start,
            );
          }
        }
        if let Some(next) = self.block_tree[cur].next {
          cur = next;
        } else {
          break;
        }
      }
    }
  }
}
