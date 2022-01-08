use crate::block::*;
use crate::document::*;
use crate::inline::*;
use crate::token::*;
pub struct Codegen<'a> {
  pub code: String,
  source: &'a str,
  bytes: &'a [u8],
}

impl<'a> Codegen<'a> {
  pub fn new(source: &'a str, bytes: &'a [u8]) -> Self {
    Codegen {
      code: String::new(),
      source,
      bytes,
    }
  }

  fn write(&mut self, str: &str) {
    self.code.push_str(str);
  }

  fn write_jsx_start(&mut self, tag: &str) {
    self.code.push_str("_jsxRuntime.jsxs)(\"");
    self.code.push_str(tag);
    self.code.push_str("\",{children:[");
  }
  fn write_jsx_end(&mut self) {
    self.code.push_str("]})");
  }
  fn write_jsxs_start(&mut self, tag: &str) {
    self.code.push_str("_jsxRuntime.jsx)(\"");
    self.code.push_str(tag);
    self.code.push_str("\",{children:");
  }
  fn write_jsxs_end(&mut self) {
    self.code.push_str("})");
  }

  pub fn gen(&mut self, ast: &AST<Token<BlockToken>>) {
    self.gen_blocks("_jsxRuntime.Fragment", &ast.children);
  }

  pub fn gen_blocks(&mut self, tag: &str, blocks: &Vec<Token<BlockToken>>) {
    let jsxs = blocks.len() > 1;
    if jsxs {
      self.write_jsxs_start(tag);
    } else {
      self.write_jsx_start(tag);
    }
    for block in blocks {
      self.gen_block(block, jsxs);
    }
    if jsxs {
      self.write_jsxs_end();
    } else {
      self.write_jsx_end();
    }
  }

  fn gen_leaf_block(&mut self, tag: &str, raws: &Vec<Span>) {
    let mut inline_parser = InlineParser::new(self.bytes, raws);
    let inlines = inline_parser.parse();
    self.gen_inlines(tag, &inlines.children);
  }

  fn gen_inlines(&mut self, tag: &str, inlines: &Vec<Token<InlineToken>>) {}

  fn gen_block(&mut self, block: &Token<BlockToken>, jsxs: bool) {
    match &block.value {
      BlockToken::ATXHeading { level, raws } => {
        self.gen_leaf_block(level.to_str(), raws);
      }
      BlockToken::SetextHeading { level, raws } => {
        self.gen_leaf_block(level.to_str(), raws);
      }
      BlockToken::Paragraph { raws } => {
        self.gen_leaf_block("p", raws);
      }
      BlockToken::BlockQuote { level, blocks } => {
        self.gen_blocks("blockquote", blocks);
      }
      BlockToken::List { blocks, .. } => {
        self.gen_blocks("ul", blocks);
      }
      BlockToken::ListItem { blocks, .. } => {
        self.gen_blocks("li", blocks);
      }
      _ => {}
    }
    if jsxs {
      self.write(",");
    }
  }
}
