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
  block_tree: &'source mut Tree<Item<Token<'source>>>,
  document: &mut Document<'source>,
  block_id: usize,
) -> Vec<Item<InlineToken<'source>>> {
  let bytes = document.bytes;
  let mut raw = Raw::new(block_tree, bytes, block_id);
  let mut tokens: Vec<Item<InlineToken>> = vec![];
  // (start, repeat)
  let mut inline_code_stack: Vec<(usize, usize)> = vec![];
  raw.iterate_bytes(&*SPECIAL_BYTES, |raw, raw_start, start, end| {
    let offset = raw_start + start;
    if end - start == 1 {
      let byte = bytes[offset];
      match byte {
        b'`' => {
          let repeat = scan_ch_repeat(&bytes[offset..], b'`');
          inline_code_stack.push((offset, repeat));
          return LoopInstruction::Move(repeat);
        }
        _ => (),
      }
    } else if end - start == 2 {
      // escaped
      if bytes[offset] == b'\\' {
        if bytes[offset + 1] == b'`' {
          let repeat = scan_ch_repeat(&bytes[offset + 1..], b'`');
          inline_code_stack.push((offset + 1, repeat));
          return LoopInstruction::Move(repeat + 1);
        }
      }
    }
    return LoopInstruction::None;
  });

  let mut code_end: Option<usize> = None;

  raw.iterate_bytes(&*SPECIAL_BYTES, |raw, raw_start, start, end| {
    let offset = raw_start + start;
    if end == start && start == 0 {
      if let Some(code_end) = code_end {
        if code_end >= raw_start + raw.len() {
          tokens.push(Item {
            start: offset,
            end: offset + raw.len(),
            value: InlineToken::Code,
          });
          return LoopInstruction::Move(raw.len() - start);
        } else {
          tokens.push(Item {
            start: offset,
            end: code_end,
            value: InlineToken::Code,
          });
          return LoopInstruction::Move(code_end - offset);
        }
      }
    } else if end - start == 1 {
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
        // b'[' => {
        //   tokens.push(Item {
        //     start: offset,
        //     end: offset + 1,
        //     value: InlineToken::MaybeLinkStart,
        //   });
        //   return LoopInstruction::None;
        // }
        // b']' => {}
        b'`' => {
          let repeat = scan_ch_repeat(&bytes[offset..], b'`');
          if code_end.is_some() {
            tokens.push(Item {
              start: offset,
              end: offset + repeat,
              value: InlineToken::InlineCodeEnd,
            });
            code_end = None;
            return LoopInstruction::Move(repeat);
          }
          if let Some((end, _)) = inline_code_stack.iter().find(|(_, re)| *re == repeat) {
            tokens.push(Item {
              start: offset,
              end: offset + repeat,
              value: InlineToken::InlineCodeStart,
            });
            code_end = Some(*end);
            if *end >= raw_start + raw.len() {
              tokens.push(Item {
                start: offset + repeat,
                end: raw.len() - start,
                value: InlineToken::Code,
              });
              return LoopInstruction::Move(raw.len() - start);
            } else {
              tokens.push(Item {
                start: offset + repeat,
                end: *end,
                value: InlineToken::Code,
              });
              return LoopInstruction::Move(*end - offset);
            }
          }
        }
        _ => (),
      }
    } else if end - start == 2 {
      // escaped
      if bytes[offset] == b'\\' {
        tokens.push(Item {
          start: offset,
          end: offset + end - start,
          value: InlineToken::Text(&raw[start + 1..end]),
        });
        return LoopInstruction::None;
      }
    }
    tokens.push(Item {
      start: offset,
      end: offset + end - start,
      value: InlineToken::Text(&raw[start..end]),
    });
    return LoopInstruction::None;
  });
  process_tokens(&mut tokens, document);
  tokens
}

fn process_tokens<'source>(
  tokens: &mut Vec<Item<InlineToken<'source>>>,
  document: &mut Document<'source>,
) {
  let source = document.source();
  let mut emphasis_stack: Vec<(u8, usize, usize)> = vec![];
  let mut index = 0;
  let len = tokens.len();
  for index in 0..len {
    if let InlineToken::MaybeEmphasis(ch, repeat, can_open, can_close) = tokens[index].value {
      if can_close {
        let em_index = emphasis_stack
          .iter()
          .position(|(em_ch, em_repeat, _)| ch == *em_ch && repeat == *em_repeat);
        if let Some(em_index) = em_index {
          while em_index < emphasis_stack.len() {
            let (.., id) = emphasis_stack.pop().unwrap();
            let start = tokens[id].start;
            let end = tokens[id].end;
            tokens[id].value = InlineToken::Text(&source[start..end]);
          }
          let (.., id) = emphasis_stack.pop().unwrap();
          tokens[id].value = InlineToken::EmphasisStart(ch, repeat);
          tokens[index].value = InlineToken::EmphasisEnd(ch, repeat);
          continue;
        }
      }
      if can_open {
        emphasis_stack.push((ch, repeat, index));
      } else {
        let start = tokens[index].start;
        let end = tokens[index].end;
        tokens[index].value = InlineToken::Text(&source[start..end]);
      }
    }
  }
}
