use super::document::*;
use crate::byte::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
// TODO: update document
pub(crate) fn scan_link_definition<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<'source>>,
) -> bool {
  let bytes = document.bytes();
  let source = document.source();
  let start = document.offset();
  let mut size = 0;
  let len = tree.len();
  let mut title_id: Option<usize> = None;
  if let Some(cur) = tree.cur() {
    if let TokenValue::LinkDefinition(ref mut link_definition) = tree[cur].item.value {
      let LinkDefinition {
        ref mut url,
        ref mut title,
        ref mut title_ch,
        ref mut closing,
        title_break,
        ..
      } = link_definition;
      if *closing {
        return false;
      }
      if url.is_empty() {
        if let Some((url_size, link_url)) = scan_link_url(bytes, source) {
          *url = link_url;
          size = url_size;
          return true;
        } else {
          tree[cur].item.value = TokenValue::Paragraph;
          return false;
        }
      }
      if let Some(_title_id) = title {
        if let Some((title_size, title_closing)) = scan_link_title(bytes, title_ch.unwrap()) {
          size = title_size;
          *closing = title_closing;
        } else if *title_break {
          tree[cur].item.value = TokenValue::Paragraph;
          return false;
        } else {
          *closing = true;
          title_id = Some(*_title_id);
          *title = None;
        }
      } else {
        if let Some((title_size, ch, title_closing)) = scan_link_starting_title(bytes) {
          size = title_size;
          *closing = title_closing;
          *title_ch = Some(ch);
          *title = Some(len);
        } else {
          return false;
        }
      }
    } else {
    }
  }
  if let Some(title_id) = title_id {
    let cur = tree.cur().unwrap();
    let start = tree[title_id].item.start;
    let paragraph = tree.append(Token {
      start,
      value: TokenValue::Paragraph,
    });
    let title_prev = tree[title_id].prev.unwrap();
    tree[cur].last_child = Some(title_prev);
    tree[title_prev].next = None;
    tree[paragraph].child = Some(title_id);
    tree[title_id].prev = None;
    return false;
  }
  if size > 0 {
    tree.lower_to_last();
    tree.append(Token {
      start,
      value: TokenValue::Raw(&source[..size]),
    });
    document.forward(size);
    tree.raise();
  }
  true
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

fn scan_link_spaces<'source>(bytes: &'source [u8]) -> Option<usize> {
  let mut size = scan_spaces(bytes);
  if let Some(eol_size) = scan_eol(&bytes[size..]) {
    size += eol_size;
    Some(size)
  } else if size > 0 {
    Some(size)
  } else {
    None
  }
}

fn scan_link_url<'source>(
  bytes: &'source [u8],
  source: &'source str,
) -> Option<(usize, &'source str)> {
  if bytes.get(0).map_or(false, |v| *v == b'<') {
    if let Some(size) = scan_ends_with(&bytes[1..], b'>', true) {
      let label = &source[1..size];
      return Some((size, label));
    }
  } else {
    let size = scan_while(bytes, is_ascii_whitespace);
    if size > 0 {
      return Some((size, &source[..size]));
    }
  }
  None
}
fn scan_link_starting_title<'source>(bytes: &'source [u8]) -> Option<(usize, u8, bool)> {
  None
}
fn scan_link_title<'source>(bytes: &'source [u8], ch: u8) -> Option<(usize, bool)> {
  None
}
