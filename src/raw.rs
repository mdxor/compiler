use crate::input::*;
use crate::token::*;
pub enum CallbackType<'a> {
  // raw bytes, raw start, start, end
  Text(&'a [u8], usize, usize, usize),
  // raw bytes, raw start, index in raw
  SpecialByte(&'a [u8], usize, usize),
  // raw bytes, raw start, index
  EscapedByte(&'a [u8], usize, usize),
}
pub enum CallbackReturn {
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
  F: FnMut(CallbackType) -> CallbackReturn,
{
  let raws_len = raws.len();
  for (i, raw) in raws.iter().enumerate() {
    let Span { start, end } = *raw;
    let bytes = &bytes[start..end];
    let mut index = 0;
    let mut text_start = index;
    let mut raw_end = bytes.len();

    let len = bytes.len();
    while index < raw_end {
      let byte = bytes[index];
      if byte == b'\\' && index < len - 1 {
        if bytes[index + 1].is_ascii_punctuation() {
          if index > text_start {
            callback(CallbackType::Text(bytes, start, text_start, index));
          }
          match callback(CallbackType::EscapedByte(bytes, start, index)) {
            CallbackReturn::Text(size) => {
              text_start = index;
              index += size;
            }
            CallbackReturn::Move(size) => {
              index += size;
              text_start = index;
            }
            _ => {
              text_start = index + 2;
              index += 2;
            }
          }
          continue;
        }
      }

      if special_bytes[byte as usize] {
        if index > text_start {
          callback(CallbackType::Text(bytes, start, text_start, index));
        }

        match callback(CallbackType::SpecialByte(bytes, start, index)) {
          CallbackReturn::Text(size) => {
            text_start = index;
            index += size;
          }
          CallbackReturn::Move(size) => {
            index += size;
            text_start = index;
            continue;
          }
          _ => {
            text_start = index + 1;
          }
        }
      }
      index += 1;
    }
  }
}
