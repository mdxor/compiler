use crate::block::*;
use crate::scan::*;
use crate::token::*;
use crate::tree::*;
struct Parser<'source> {
  source: &'source str,
  bytes: &'source [u8],
  index: usize,
  ast: Tree<Token<'source>>,
}

impl<'source> Parser<'source> {
  fn new(source: &'source str) -> Self {
    Self {
      source,
      bytes: source.as_bytes(),
      index: 0,
      ast: Tree::new(),
    }
  }

  fn bytes(&self) -> &'source [u8] {
    &self.bytes[self.index..]
  }

  fn source(&self) -> &'source str {
    &self.source[self.index..]
  }

  fn parse(&self) {}

  fn parse_blocks(&self) {}

  // fn scan_block(&mut self) -> Result<(), ()> {
  //   let cur = self.ast.cur().unwrap();
  //   if
  //   if let Some(token) = scan_block(self.bytes(), self.index)? {
  //     self.ast.append(token);
  //     let cur = self.ast.cur().unwrap();
  //     let node = &self.ast[cur];

  //     let token_body = &node.item.body;
  //     let end = node.item.end;
  //     self.index = end;
  //     match token_body {
  //       TokenValue::ATXHeading(_) => {
  //         let raw_size = scan_raw_line(self.bytes());
  //         self.index += raw_size;

  //         let mut raw = &self.source()[..raw_size];
  //         let mut block_end = end + raw_size;
  //         if raw.ends_with("\n") {
  //           block_end -= 1;
  //         }
  //         self.ast[cur].item.end = block_end;

  //         self.ast.lower();
  //         self.ast.append(Token {
  //           start: end,
  //           end: end + raw_size,
  //           body: TokenValue::Raw(raw),
  //         });
  //         // TODO
  //         self.ast.raise();
  //       }
  //       _ => {}
  //     }
  //   }
  //   Ok(())
  // }
}
