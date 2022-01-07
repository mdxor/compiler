use crate::input::*;
use crate::token::*;
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

pub fn uri(bytes: &[u8]) -> Option<usize> {
  let bytes = single_char(bytes, b'<')?;
  let mut size = 1;
  if bytes.is_empty() {
    return None;
  }
  if !bytes[0].is_ascii_alphabetic() {
    return None;
  }
  let (bytes, scheme_size) = take_while(bytes, |ch| {
    ch != b':' && (ch.is_ascii_alphanumeric() || ch == b'+' || ch == b'-' || ch == b'.')
  });
  size += scheme_size;
  if scheme_size < 2 || scheme_size > 32 {
    return None;
  }
  let bytes = single_char(bytes, b':')?;
  size += 1;
  let (bytes, follow_size) = take_while(bytes, |ch| match ch {
    b'<' | b'>' => false,
    b'\0'..=b' ' => false,
    _ => true,
  });
  single_char(bytes, b'>')?;
  size += follow_size + 1;
  Some(size)
}

// size, url Span
pub fn link_url(bytes: &[u8], offset: usize) -> Option<(usize, Span)> {
  let bytes = single_char(bytes, b'(')?;
  let (bytes, spaces) = spaces0(bytes);
  if let Some(bytes) = single_char(bytes, b'<') {
    let mut size = 2 + spaces;
    let mut escaped = false;
    let (bytes, url_size) = take_while(bytes, |ch| {
      if ch == b' ' || ch == b'\r' || ch == b'\n' {
        return false;
      }
      if escaped {
        escaped = false;
        return true;
      }
      if ch == b'\\' {
        escaped = true;
        return true;
      }
      if ch == b'>' || ch == b'<' {
        return false;
      }
      true
    });
    single_char(bytes, b'>')?;
    return Some((
      size + url_size + 1,
      Span {
        start: offset + size,
        end: offset + size + url_size,
      },
    ));
  } else {
    let mut nested = 0;
    let mut escaped = false;
    let (bytes, url_size) = take_while(bytes, |ch| {
      if ch == b' ' || ch == b'\r' || ch == b'\n' {
        return false;
      }
      if escaped {
        escaped = false;
        return true;
      }
      if ch == b'\\' {
        escaped = true;
        return true;
      }
      if ch == b'(' {
        nested += 1;
      }
      if ch == b')' {
        if nested == 0 {
          return false;
        }
        nested -= 1;
      }
      true
    });
    if url_size > 0 && nested == 0 {
      return Some((
        url_size + 1,
        Span {
          start: offset,
          end: offset + url_size,
        },
      ));
    }
  }
  None
}

// return (end pos, url span, title spans)
pub fn link_url_title<'a, F>(mut get_raw_span: F) -> Option<(usize, Span, Vec<Span>)>
where
  F: FnMut() -> Option<(&'a [u8], usize)>,
{
  let (bytes, mut raw_start) = get_raw_span()?;
  single_char(bytes, b'(')?;
  let (url_size, url_span) = link_url(bytes, raw_start)?;
  let mut bytes = &bytes[url_size..];
  let mut title: Vec<Span> = vec![];
  let mut title_ch: Option<u8> = None;
  let mut title_end = false;
  let mut raw_start = raw_start + url_size;
  loop {
    let mut index = 0;
    let mut escaped = false;
    let mut title_start: Option<usize> = if title_ch.is_some() && !title_end {
      Some(0)
    } else {
      None
    };
    while index < bytes.len() {
      let byte = bytes[index];
      if title_end {
        if byte == b')' {
          return Some((raw_start + index, url_span, title));
        }
        if byte != b' ' && byte != b'\r' && byte != b'\n' {
          return None;
        }
      }
      if title_ch.is_none() {
        if byte == b')' {
          return Some((raw_start + index, url_span, title));
        }
        if byte == b'"' || byte == b'\'' {
          title_start = Some(index + 1);
          title_ch = Some(byte);
        } else if byte != b' ' && byte != b'\r' && byte != b'\n' {
          return None;
        }
      } else {
        if escaped {
          escaped = false;
        } else if byte == b'\\' {
          escaped = true;
        } else if byte == title_ch.unwrap() {
          if let Some(title_start) = title_start {
            title.push(Span {
              start: title_start + raw_start,
              end: raw_start + index,
            });
          }
          title_end = true;
        }
      }
      index += 1;
    }
    if let Some(title_start) = title_start {
      title.push(Span {
        start: title_start + raw_start,
        end: raw_start + bytes.len(),
      });
    }
    if let Some((next_bytes, next_raw_start)) = get_raw_span() {
      bytes = next_bytes;
      raw_start = next_raw_start;
    } else {
      break;
    }
  }
  None
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
