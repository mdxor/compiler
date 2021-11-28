use crate::byte::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
use memchr::memchr;

struct BlockParser<'source> {
  source: &'source str,
  bytes: &'source [u8],
  index: usize,
  tree: Tree<Token<'source>>,
}

impl<'source> BlockParser<'source> {
  fn new(source: &'source str) -> Self {
    Self {
      source,
      bytes: source.as_bytes(),
      index: 0,
      tree: Tree::new(),
    }
  }

  fn bytes(&self) -> &'source [u8] {
    &self.bytes[self.index..]
  }

  fn source(&self) -> &'source str {
    &self.source[self.index..]
  }

  pub fn parse(mut self) -> Tree<Token<'source>> {
    self.tree
  }

  fn scan_atx_heading(&mut self) -> Option<()> {
    let bytes = self.bytes();
    let level = scan_ch_repeat(bytes, b'#');
    if bytes.get(level).copied().map_or(true, is_ascii_whitespace) {
      if let Some(heading_level) = HeadingLevel::new(level) {
        let start = self.index;
        let mut end = self.index + level;
        let mut raw_line_start = end;
        let mut raw_line = "";
        if bytes
          .get(level)
          .copied()
          .map_or(false, is_ascii_whitespace_no_nl)
        {
          end += 1;
          raw_line_start = end;
          raw_line = self.scan_raw_line();
          end += raw_line.len();
        }
        self.index = end;
        self.tree.append(Token {
          start,
          end,
          body: TokenBody::ATXHeading(heading_level),
        });
        self.tree.lower();
        self.tree.append(Token {
          start: raw_line_start,
          end,
          body: TokenBody::Raw(raw_line),
        });
        self.tree.raise();
        return Some(());
      }
    }
    None
  }

  fn scan_setext_heading(&mut self) -> Option<()> {
    let cur = self.tree.cur().unwrap();
    let node = &self.tree[cur];
    if node.item.body != TokenBody::Paragraph {
      return None;
    }
    let bytes = self.bytes();
    let c = *bytes.get(0)?;
    if !(c == b'-' || c == b'=') {
      return None;
    }
    let mut i = scan_ch_repeat(&bytes[1..], c);
    i += scan_blank_line(&bytes[i..])?;
    let level = if c == b'=' {
      HeadingLevel::H1
    } else {
      HeadingLevel::H2
    };
    self.index += i;
    self.tree[cur].item.end = self.index;
    self.tree[cur].item.body = TokenBody::SetextHeading(level);
    Some(())
  }

  fn scan_paragraph(&mut self) -> Option<()> {
    let cur = self.tree.cur().unwrap();
    if let TokenBody::Raw(raw) = self.tree[cur].item.body {
      let raw_line = self.scan_raw_line();
      let start = self.index;
      let end = self.index + raw_line.len();
      self.index = end;
      self.tree.append(Token {
        start,
        end,
        body: TokenBody::Paragraph,
      });
      self.tree.lower();
      self.tree.append(Token {
        start,
        end,
        body: TokenBody::Raw(raw_line),
      });
    } else {
      let raw_line = self.scan_raw_line();
      let end = self.index + raw_line.len();
      self.index = end;
      let start = self.tree[cur].item.start;
      self.tree[cur].item.end = end;
      self.tree[cur].item.body = TokenBody::Raw(&self.source[start..end]);
    }
    Some(())
  }

  fn scan_raw_line(&mut self) -> &'source str {
    let size = scan_raw_line(self.bytes());
    &self.source()[..size]
  }
}
