use super::document::*;
use super::paragraph::*;
use crate::byte::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_link_definition<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<TokenValue<'source>>>,
) -> Option<()> {
  let bytes = document.bytes();
  let source = document.source();
  let start = document.offset();
  let mut size = 0;
  let mut link_label = "";
  let mut link_url = "";
  let mut title = String::default();

  let cur = tree.cur()?;
  if let TokenValue::Paragraph = tree[cur].item.value {
    return None;
  }
  let (label_size, label) = scan_link_label(bytes, source)?;
  link_label = label;
  size += label_size;
  let (spaces_size, _) = scan_link_spaces(&bytes[size..])?;
  size += spaces_size;
  let (url_size, url) = scan_link_url(&bytes[size..], &source[size..])?;
  link_url = url;
  size += url_size;
  let (spaces_size, line_break) = scan_link_spaces(&bytes[size..])?;
  size += spaces_size;
  let label_url_size = size;

  if let Some((title_size, ch, closing)) = scan_link_starting_title(&bytes[size..]) {
    title.push_str(&source[size..size + title_size]);
    size += title_size;
    if !closing {
      loop {
        if let Some(starting_size) = continue_paragraph(&bytes[size..], tree) {
          size += starting_size;
          if let Some((title_size, closing)) = scan_link_title(&bytes[size..], ch) {
            title.push_str(&source[size..size + title_size]);
            size += title_size;
            if closing {
              break;
            }
            continue;
          } else if line_break {
            return None;
          }
        } else if line_break {
          return None;
        }
        title = String::default();
        size = label_url_size;
        break;
      }
    }
  } else if line_break {
    return None;
  }
  tree.append(Token {
    start,
    value: TokenValue::LinkDefinition,
  });
  document.forward(size);
  document.link_definitions.push(LinkDefinition {
    label: link_label,
    url: link_url,
    title,
  });
  Some(())
}

fn scan_link_label<'source>(
  bytes: &'source [u8],
  source: &'source str,
) -> Option<(usize, &'source str)> {
  if bytes.get(0).map_or(false, |v| *v == b'[') {
    if let Some(size) = scan_ends_with(&bytes[1..], b']', true) {
      let label = &source[1..size];
      if !label.is_empty() {
        if bytes.get(size + 1).map_or(false, |v| *v == b':') {
          return Some((size + 2, label));
        }
      }
    }
  }
  None
}

// return (size, line_break)
fn scan_link_spaces<'source>(bytes: &'source [u8]) -> Option<(usize, bool)> {
  let mut size = scan_spaces(bytes);
  if let Some(eol_size) = scan_eol(&bytes[size..]) {
    size += eol_size;
    Some((size, true))
  } else if size > 0 {
    Some((size, false))
  } else {
    None
  }
}

fn scan_link_url<'source>(
  bytes: &'source [u8],
  source: &'source str,
) -> Option<(usize, &'source str)> {
  let mut size = 0;
  let mut url = "";
  let mut title = String::default();
  let mut title_ch: Option<u8> = None;
  let mut title_closing = false;
  if bytes.get(0).map_or(false, |v| *v == b'<') {
    if let Some(url_size) = scan_ends_with(&bytes[1..], b'>', true) {
      size += url_size + 1;
      url = &source[1..size];
    } else {
      return None;
    }
  } else {
    let url_size = scan_while(bytes, is_ascii_whitespace);
    if url_size > 0 {
      size = url_size;
      url = &source[..size];
    } else {
      return None;
    }
  }
  return Some((size, url));
}

fn scan_link_starting_title<'source>(bytes: &'source [u8]) -> Option<(usize, u8, bool)> {
  let mut size = scan_spaces(bytes);
  let mut ch: Option<u8> = None;
  let mut closing = false;
  if scan_ch(&bytes[size..], b'\'') {
    size += 1;
    ch = Some(b'\'');
  } else if scan_ch(&bytes[size..], b'"') {
    size += 1;
    ch = Some(b'\'');
  }
  if let Some(ch) = ch {
    if let Some(title_size) = scan_ends_with(&bytes[size..], ch, false) {
      size += title_size;
      closing = true;
    } else {
      let raw_size = scan_raw_line_without_source(&bytes[size..]);
      size += raw_size;
    }
    return Some((size, ch, closing));
  }
  None
}

fn scan_link_title<'source>(bytes: &'source [u8], ch: u8) -> Option<(usize, bool)> {
  let mut size = 0;
  let mut closing = false;
  if let Some(title_size) = scan_ends_with(bytes, ch, false) {
    size = title_size;
    if let Some(eol_size) = scan_eol(&bytes[size..]) {
      size += eol_size;
      closing = true;
    } else {
      return None;
    }
  } else {
    size = scan_raw_line_without_source(bytes);
  }
  return Some((size, closing));
}
