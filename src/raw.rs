use crate::token::*;
pub enum LoopInstruction {
  None,
  Text(usize),
  Move(usize),
}
pub fn iterate_raws<'source, F>(
  raws: &Vec<Span>,
  bytes: &'source [u8],
  special_bytes: &'source [bool; 256],
  mut callback: F,
) where
  // raw bytes, raw start, start, end
  F: FnMut(&'source [u8], usize, usize, usize) -> LoopInstruction,
{
  for raw in raws {
    let Span { start, end } = *raw;
    let bytes = &bytes[start..end];
    let raw_str = callback(bytes, start, 0, 0);
    let mut text_start = 0;
    let mut index = 0;
    let len = bytes.len();
    loop {
      if index >= len {
        break;
      }
      let byte = bytes[index];
      if byte == b'\\' && index < len - 1 {
        if bytes[index + 1].is_ascii_punctuation() {
          if index > text_start {
            callback(bytes, start, text_start, index);
          }
          callback(bytes, start, index, index + 2);
          text_start = index + 2;
          index += 2;
          continue;
        }
      }
      if special_bytes[byte as usize] {
        if index > text_start {
          callback(bytes, start, text_start, index);
        }
        match callback(bytes, start, index, index + 1) {
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
    if text_start < len - 1 {
      callback(bytes, start, text_start, len);
    }
  }
}

pub fn inverse_iterate_bytes() {}
