use super::document::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
pub(crate) fn scan_fenced_code<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<'source>>,
) -> bool {
  let start = document.offset();
  let bytes = document.bytes();
  let source = document.source();
  let mut keyword = b'`';
  let mut keyword_size = scan_ch_repeat(bytes, keyword);
  if keyword_size < 3 {
    keyword = b'~';
    keyword_size = scan_ch_repeat(bytes, keyword);
    if keyword_size < 3 {
      return false;
    }
  }
  let (raw_size, raw) = scan_raw_line(&bytes[keyword_size..], &source[keyword_size..]);
  let mut splitter = raw.trim().splitn(2, ' ');
  let language = splitter.next().map_or("", |v| v);
  let meta = splitter.next().map_or("", |v| v);
  tree.append(Token {
    start,
    value: TokenValue::FencedCode(FencedCode {
      language,
      meta,
      keyword,
      keyword_size,
    }),
  });
  document.forward(keyword_size + raw_size);
  true
}

pub(crate) fn scan_inner_fenced_code<'source>(
  document: &mut Document<'source>,
  tree: &mut Tree<Token<'source>>,
) -> bool {
  if let Some(cur) = tree.cur() {
    if let TokenValue::FencedCode(fenced_code) = &tree[cur].item.value {
      let start = document.offset();
      let bytes = document.bytes();
      // try to end fenced code
      let (spaces_size, spaces) = scan_spaces(bytes);
      if spaces < 4 {
        let keyword = fenced_code.keyword;
        let keyword_size = fenced_code.keyword_size;
        let repeat = scan_ch_repeat(&bytes[spaces_size..], keyword);
        if repeat == keyword_size {
          if let Some(blank_line_size) = scan_blank_line(&bytes[spaces_size + repeat..]) {
            tree.append(Token {
              start,
              value: TokenValue::FencedCodeEnding,
            });
            document.forward(spaces_size + keyword_size + blank_line_size);
            return true;
          }
        }
      }

      tree.lower_to_last();
      let source = document.source();
      let (raw_size, raw) = scan_raw_line(bytes, source);
      tree.append(Token {
        start,
        value: TokenValue::Code(raw),
      });
      document.forward(raw_size);
      tree.raise();
      return true;
    }
  }
  false
}
