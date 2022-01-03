use crate::block::*;
use crate::document::*;
use crate::inline::*;
use crate::token::*;
pub struct Codegen {
  pub code: String,
}

impl Codegen {
  pub fn new() -> Self {
    Codegen {
      code: String::new(),
    }
  }

  fn write(&mut self, str: &str) {
    self.code.push_str(str);
  }

  pub fn gen<'source>(&mut self, ast: &AST, document: &mut Document<'source>) {
    self.write("jsxs(\"div\", {children: [");
    self.gen_blocks(&ast.blocks, document);
    self.write("]})");
  }

  pub fn gen_blocks<'source>(
    &mut self,
    blocks: &Vec<Token<BlockToken>>,
    document: &mut Document<'source>,
  ) {
    for block in blocks {
      self.gen_block(block, document);
    }
  }

  pub fn gen_block<'source>(
    &mut self,
    block: &Token<BlockToken>,
    document: &mut Document<'source>,
  ) {
    let source = document.source;
    let Span { start, end } = block.span;
    match &block.value {
      BlockToken::ATXHeading { level, raws } => {
        self.write("jsxs(\"");
        self.write(level.to_str());
        self.write("\", {children: [");
        self.gen_raws(raws, document);
      }
      BlockToken::SetextHeading { level, raws } => {
        self.write("jsxs(\"");
        self.write(level.to_str());
        self.write("\", {children: [");
        self.gen_raws(raws, document);
      }
      BlockToken::Paragraph { raws } => {
        self.write("jsxs(\"p\", {children: [");
        self.gen_raws(raws, document);
      }
      BlockToken::BlockQuote { level, blocks } => {
        self.write("jsxs(\"blockquote\", {children: [");
        self.gen_blocks(blocks, document);
      }
      BlockToken::List { blocks, .. } => {
        self.write("jsxs(\"ul\", {children: [");
        self.gen_blocks(blocks, document);
      }
      BlockToken::ListItem { blocks, .. } => {
        self.write("jsxs(\"li\", {children: [");
        self.gen_blocks(blocks, document);
      }
      _ => {
        self.write("jsxs(\"div\", {children: [");
      }
    }
    self.write("]})");
  }

  fn gen_raws<'source>(&mut self, raws: &Vec<Span>, document: &mut Document<'source>) {
    let tokens = parse_raws_to_inlines(raws, document);
    let source = document.source;
    for item in tokens {
      let start = item.span.start;
      let end = item.span.end;
      match item.value {
        InlineToken::Text => {
          self.write("\"");
          self.write(&source[start..end]);
          self.write("\",");
        }
        InlineToken::Code => {
          self.write("\"");
          self.write(&source[start..end]);
          self.write("\",");
        }
        InlineToken::InlineCodeStart => {
          self.write("jsxs(\"code\", {children: [");
        }
        InlineToken::InlineCodeEnd => {
          self.write("]})");
        }
        InlineToken::EmphasisStart { keyword, repeat } => match keyword {
          b'*' | b'_' => {
            if repeat > 1 {
              self.write("jsxs(\"strong\", {children: [");
            } else {
              self.write("jsxs(\"em\", {children: [");
            }
          }
          b'~' => {
            self.write("jsxs(\"del\", {children: [");
          }
          _ => (),
        },
        InlineToken::EmphasisEnd { .. } => {
          self.write("]})");
        }
        _ => {}
      }
    }
  }
}
