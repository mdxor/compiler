use crate::input::*;
// size, repeat
pub fn atx_heading_start(bytes: &[u8]) -> Option<(usize, usize)> {
  let (bytes, repeat) = ch_repeat_min_max(bytes, b'#', 1, 6)?;
  let size = repeat + eol_or_space(bytes)?.1;
  Some((size, repeat))
}

// size, repeat size, meta size
pub fn open_fenced_code(bytes: &[u8]) -> Option<(usize, usize, usize)> {
  if let Some((bytes, repeat)) = ch_repeat_min(bytes, b'`', 3) {
    let (bytes, meta_size) = take_while(bytes, |c| c != b'`');
    let (_, eol_size) = eol(bytes)?;
    return Some((repeat + meta_size + eol_size, repeat, meta_size));
  } else {
    let (bytes, repeat) = ch_repeat_min(bytes, b'~', 3)?;
    let (size, meta_size) = one_line(bytes);
    return Some((repeat + size, repeat, meta_size));
  }
}

pub fn close_fenced_code(bytes: &[u8]) -> Option<(usize, usize)> {
  if let Some((bytes, repeat)) = ch_repeat_min(bytes, b'`', 3) {
    Some((repeat + spaces_eol(bytes)?.1, repeat))
  } else {
    let (bytes, repeat) = ch_repeat_min(bytes, b'~', 3)?;
    Some((repeat + spaces_eol(bytes)?.1, repeat))
  }
}
// size, level
pub fn block_quote(bytes: &[u8]) -> Option<(usize, usize)> {
  let mut level = 0;
  let mut input = bytes;
  let mut spaces_size = 0;
  loop {
    if let Some((bytes)) = tag(input, b">") {
      level += 1;
      input = bytes;
      if let Some((bytes, size)) = ch_repeat_max(input, b' ', 4) {
        spaces_size = size;
        input = bytes;
      } else {
        if level > 0 {
          return Some((bytes.len() - input.len() + 1, level));
        }
        break;
      }
    } else {
      break;
    }
  }
  if level > 0 {
    Some((
      bytes.len() - input.len() - if spaces_size > 0 { spaces_size - 1 } else { 0 },
      level,
    ))
  } else {
    None
  }
}

// size, marker size, ending indent
pub fn list_item_start(bytes: &[u8]) -> Option<(usize, usize, usize)> {
  if bytes.len() > 0 {
    let ch = bytes[0];
    if ch == b'-' || ch == b'*' || ch == b'+' {
      eol_or_space(&bytes[1..])?;
      let (_, spaces) = spaces0(&bytes[1..]);
      if spaces >= 5 {
        return Some((1 + 1, 1, 1));
      } else {
        return Some((1 + spaces, 1, spaces));
      }
    } else {
      let (bytes, size) = take_while(bytes, |ch| ch > b'0' && ch < b'9');
      if (size > 0 && size < 10) {
        if single_char(bytes, b'.').is_some() || single_char(bytes, b')').is_some() {
          eol_or_space(&bytes[1..])?;
          let (_, spaces) = spaces0(&bytes[1..]);
          if spaces >= 5 {
            return Some((size + 1 + 1, size + 1, 1));
          } else {
            return Some((size + 1 + spaces, size + 1, spaces));
          }
        }
      }
    }
  }
  None
}

pub fn setext_heading(bytes: &[u8]) -> Option<usize> {
  if let Some(c) = bytes.get(0) {
    if *c == b'-' || *c == b'=' {
      let (bytes, size) = take_while(bytes, |ch| ch == *c);
      if size > 0 {
        return Some(size + spaces_eol(bytes)?.1);
      }
    }
  }
  None
}

pub fn thematic_break(bytes: &[u8]) -> Option<usize> {
  let len = bytes.len();
  if len > 0 {
    let ch = bytes[0];
    if ch == b'*' || ch == b'-' || ch == b'_' {
      let mut input = bytes;
      let mut repeat = 0;
      loop {
        let bytes = single_char(input, ch)?;
        repeat += 1;
        let (bytes, _) = spaces0(bytes);
        if let Some((bytes, _)) = eol(bytes) {
          if repeat >= 3 {
            return Some(len - bytes.len());
          }
          break;
        }
        input = bytes;
      }
    } else {
      return None;
    }
  }
  None
}

pub fn blank_line(bytes: &[u8]) -> Option<usize> {
  let (_, size) = spaces_eol(bytes)?;
  Some(size)
}

// fn import_declaration(input: &[u8]) -> IResult<&[u8], ()> {
//   let (input, _) = preceded(space0, tag("import"))(input)?;
//   Ok((input, ()))
// }
// fn import_declaration_specifier(input: &[u8]) -> IResult<&[u8], ()> {
//   let mut input = input;
//   if let Ok((rest, _)) = variable(input) {
//     input = rest;
//     if let Ok((rest, _)) = comma(input) {
//       input = rest;
//     } else {
//       return Ok((input, ()));
//     }
//   }
//   let (rest, _) = import_declaration_object_specifier(input)?;
//   Ok((rest, ()))
// }
// // TODO
// fn import_declaration_object_specifier(input: &[u8]) -> IResult<&[u8], ()> {
//   let (input, _) = char('{')(input)?;
//   let (input, _) = char('}')(input)?;
//   Ok((input, ()))
// }
// fn comma(input: &[u8]) -> IResult<&[u8], ()> {
//   let (input, _) = spaces_newlines(input)?;
//   let (input, _) = char(',')(input)?;
//   let (input, _) = spaces_newlines(input)?;
//   Ok((input, ()))
// }
// fn variable(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
//   pair(
//     take_while_m_n(1, 1, |c| is_alphabetic(c) || c == b'_' || c == b'$'),
//     take_while(|c| is_alphanumeric(c) || c == b'_' || c == b'$'),
//   )(input)
// }
// fn spaces_newlines(input: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
//   many1(alt((space0, line_ending)))(input)
// }
// fn spaces_newline(input: &[u8]) -> IResult<&[u8], (&[u8], Vec<&[u8]>, &[u8])> {
//   tuple((space0, many_m_n(0, 1, line_ending), space0))(input)
// }
// fn import_declaration_source(input: &[u8]) -> IResult<&[u8], &[u8]> {
//   alt((
//     delimited(
//       char('\''),
//       take_till(|c| c == b' ' || c == b'\r' || c == b'\n' || c == b'\''),
//       char('\''),
//     ),
//     (delimited(
//       char('"'),
//       take_till(|c| c == b' ' || c == b'\r' || c == b'\n' || c == b'"'),
//       char('"'),
//     )),
//   ))(input)
// }
#[test]
fn test_scan_atx_heading_start() {
  // println!("{:?}", scan_atx_heading_start("# 123"));
  // println!("{:?}", scan_open_fenced_code("123```\n"));
  // println!("{:?}", scan_list_item_start("- 123"));
  // println!("{:?}", scan_close_fenced_code("```   "));
  // println!("{:?}", scan_blank_line(""));
  // println!("{:?}", scan_setext_heading("===== "));
}
