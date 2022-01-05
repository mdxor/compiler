use crate::input::*;
use crate::token::*;
pub enum CallbackType<'a> {
  // raw bytes, raw start, start, end
  Text(&'a [u8], usize, usize, usize),
  // raw bytes, raw start, index in raw
  SpecialByte(&'a [u8], usize, usize),
  // raw bytes, raw start, index
  EscapedByte(&'a [u8], usize, usize),
  // raw start, start, end
  SoftBreak(usize, usize, usize),
  // raw start, start, end
  HardBreak(usize, usize, usize),
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

    let mut ending_callback: Option<CallbackType> = None;
    if i == raws_len - 1 {
      let ending = bytes
        .iter()
        .rev()
        .take_while(|&&c| c == b'\n' || c == b'\r')
        .count();
      raw_end -= ending;
    } else {
      let ending = bytes
        .iter()
        .rev()
        .take_while(|&&c| c == b' ' || c == b'\n' || c == b'\r')
        .count();

      if ending > 0 {
        let bytes = &bytes[raw_end - ending..];
        let (bytes, spaces) = spaces0(bytes);
        if let Some((_, eol_size)) = eol(bytes) {
          if eol_size > 0 {
            if spaces >= 2 {
              ending_callback = Some(CallbackType::HardBreak(start, raw_end - ending, raw_end));
              raw_end -= ending;
            } else {
              ending_callback = Some(CallbackType::SoftBreak(start, raw_end - eol_size, raw_end));
              raw_end -= eol_size;
            }
          }
        }
      }
    }
    let len = bytes.len();
    loop {
      if index >= raw_end {
        if raw_end > text_start {
          callback(CallbackType::Text(bytes, start, text_start, raw_end));
        }
        if let Some(callbackType) = ending_callback {
          callback(callbackType);
          ending_callback = None;
        }
        break;
      }
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
