use crate::document::*;
use crate::input::*;
use crate::raw::*;
use crate::token::*;
use std::collections::VecDeque;
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

fn is_left_flanking_delimiter(bytes: &[u8], start: usize, end: usize) -> bool {
  let len = bytes.len();
  if end >= len || bytes[end].is_ascii_whitespace() {
    return false;
  }
  let next_byte = if let Some(c) = bytes.get(end + 1) {
    c
  } else {
    return true;
  };
  // TODO
  if !next_byte.is_ascii_punctuation()
    && (start == 0
      || bytes[start - 1].is_ascii_whitespace()
      || bytes[start - 1].is_ascii_punctuation())
  {
    return true;
  }
  return false;
}

fn is_right_flanking_delimiter(bytes: &[u8], start: usize, end: usize) -> bool {
  let len = bytes.len();
  if start == 0 || bytes[start - 1].is_ascii_whitespace() {
    return false;
  }
  let prev_byte = if let Some(c) = bytes.get(start - 1) {
    c
  } else {
    return true;
  };
  // TODO
  if !prev_byte.is_ascii_punctuation()
    && (end >= len || bytes[end].is_ascii_whitespace() || bytes[end].is_ascii_punctuation())
  {
    return true;
  }
  return false;
}

pub(crate) fn parse_raws<'a>(raws: &Vec<Span>, bytes: &'a [u8]) -> Vec<Token<InlineToken>> {
  let mut tokens: Vec<Token<InlineToken>> = vec![];
  // (start, repeat)
  let mut inline_code_deque: VecDeque<(usize, usize)> = vec![];
  iterate_raws(
    raws,
    bytes,
    &*SPECIAL_BYTES,
    |callback_type| match callback_type {
      CallbackType::Text(raw_bytes, raw_start, start, end) => CallbackReturn::None,
      _ => CallbackReturn::None,
    },
  );

  // iterate_raws(
  //   raws,
  //   bytes,
  //   &*SPECIAL_BYTES,
  //   |raw_bytes, raw_start, start, end| {
  //     let offset = raw_start + start;
  //     if end == start && start == 0 {
  //       if let Some(code_end) = code_end {
  //         if code_end >= raw_start + raw_bytes.len() {
  //           tokens.push(Token {
  //             value: InlineToken::Code,
  //             span: Span {
  //               start: offset,
  //               end: offset + raw_bytes.len(),
  //             },
  //           });
  //           return LoopInstruction::Move(raw_bytes.len() - start);
  //         } else {
  //           tokens.push(Token {
  //             value: InlineToken::Code,
  //             span: Span {
  //               start: offset,
  //               end: code_end,
  //             },
  //           });
  //           return LoopInstruction::Move(code_end - offset);
  //         }
  //       }
  //       return LoopInstruction::None;
  //     } else if end - start == 1 {
  //       let byte = bytes[raw_start + start];
  //       match byte {
  //         b'*' | b'_' | b'~' => {
  //           let repeat = ch_repeat(&bytes[offset..], byte).1;
  //           let can_open = is_left_flanking_delimiter(raw_bytes, start, start + repeat);
  //           let can_close = is_right_flanking_delimiter(raw_bytes, start, start + repeat);
  //           if !can_open && !can_close {
  //             return LoopInstruction::Text(repeat);
  //           }
  //           if byte == b'~' {
  //             if repeat == 2 {
  //               tokens.push(Token {
  //                 value: InlineToken::MaybeEmphasis {
  //                   keyword: byte,
  //                   repeat,
  //                   can_open,
  //                   can_close,
  //                 },
  //                 span: Span {
  //                   start: offset,
  //                   end: offset + repeat,
  //                 },
  //               });
  //               return LoopInstruction::Move(repeat);
  //             } else {
  //               return LoopInstruction::Text(repeat);
  //             }
  //           }
  //           tokens.push(Token {
  //             value: InlineToken::MaybeEmphasis {
  //               keyword: byte,
  //               repeat,
  //               can_open,
  //               can_close,
  //             },
  //             span: Span {
  //               start: offset,
  //               end: offset + repeat,
  //             },
  //           });
  //           return LoopInstruction::Move(repeat);
  //         }
  //         // b'[' => {
  //         //   tokens.push(Item {
  //         //     start: offset,
  //         //     end: offset + 1,
  //         //     value: InlineToken::MaybeLinkStart,
  //         //   });
  //         //   return LoopInstruction::None;
  //         // }
  //         // b']' => {}
  //         // b'`' => {
  //         //   let repeat = scan_ch_repeat(&bytes[offset..], b'`');
  //         //   if code_end.is_some() {
  //         //     tokens.push(Item {
  //         //       start: offset,
  //         //       end: offset + repeat,
  //         //       value: InlineToken::InlineCodeEnd,
  //         //     });
  //         //     code_end = None;
  //         //     return LoopInstruction::Move(repeat);
  //         //   }
  //         //   if let Some(index) = inline_code_stack
  //         //     .iter()
  //         //     .position(|(end, re)| *end > offset && *re == repeat)
  //         //   {
  //         //     // TODO:
  //         //     while inline_code_stack.len() > index + 1 {
  //         //       inline_code_stack.pop();
  //         //     }
  //         //     let (end, _) = inline_code_stack.pop().unwrap();
  //         //     tokens.push(Item {
  //         //       start: offset,
  //         //       end: offset + repeat,
  //         //       value: InlineToken::InlineCodeStart,
  //         //     });
  //         //     code_end = Some(end);
  //         //     if end >= raw_start + raw.len() {
  //         //       tokens.push(Item {
  //         //         start: offset + repeat,
  //         //         end: raw.len() - start,
  //         //         value: InlineToken::Code,
  //         //       });
  //         //       return LoopInstruction::Move(raw.len() - start);
  //         //     } else {
  //         //       tokens.push(Item {
  //         //         start: offset + repeat,
  //         //         end: end,
  //         //         value: InlineToken::Code,
  //         //       });
  //         //       return LoopInstruction::Move(end - offset);
  //         //     }
  //         //   }
  //         // }
  //         _ => (),
  //       }
  //     } else if end - start == 2 {
  //       // escaped
  //       if bytes[offset] == b'\\' {
  //         tokens.push(Token {
  //           value: InlineToken::Text,
  //           span: Span {
  //             start: offset,
  //             end: offset + end - start,
  //           },
  //         });
  //         return LoopInstruction::None;
  //       }
  //     }
  //     tokens.push(Token {
  //       value: InlineToken::Text,
  //       span: Span {
  //         start: offset,
  //         end: offset + end - start,
  //       },
  //     });
  //     return LoopInstruction::None;
  //   },
  // );

  process_tokens(&mut tokens);
  tokens
}

fn process_tokens(tokens: &mut Vec<Token<InlineToken>>) {
  let mut emphasis_stack: Vec<(u8, usize, usize)> = vec![];
  let mut index = 0;
  let len = tokens.len();
  for index in 0..len {
    if let InlineToken::MaybeEmphasis {
      keyword,
      repeat,
      can_open,
      can_close,
      ..
    } = tokens[index].value
    {
      if can_close {
        let em_index = emphasis_stack
          .iter()
          .position(|(em_ch, em_repeat, _)| keyword == *em_ch && repeat == *em_repeat);
        if let Some(em_index) = em_index {
          while em_index + 1 < emphasis_stack.len() {
            let (.., id) = emphasis_stack.pop().unwrap();
            tokens[id].value = InlineToken::Text;
          }
          let (.., id) = emphasis_stack.pop().unwrap();
          tokens[id].value = InlineToken::EmphasisStart { keyword, repeat };
          tokens[index].value = InlineToken::EmphasisEnd { keyword, repeat };
          continue;
        }
      }
      if can_open {
        emphasis_stack.push((keyword, repeat, index));
      } else {
        tokens[index].value = InlineToken::Text;
      }
    }
  }
}
