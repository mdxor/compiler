use crate::block::*;
use crate::document::*;
// use crate::inline::*;
use crate::token::*;
pub struct Codegen<'source> {
  pub code: String,
  source: &'source str,
}

impl<'source> Codegen<'source> {
  pub fn new(source: &'source str) -> Self {
    Codegen {
      code: String::new(),
      source,
    }
  }

  fn write(&mut self, str: &str) {
    self.code.push_str(str);
  }

  pub fn gen(&mut self, ast: &AST) {
    self.write("jsxs(\"div\", {children: [");
    self.gen_blocks(&ast.blocks);
    self.write("]})");
  }

  pub fn gen_blocks(&mut self, blocks: &Vec<Token<BlockToken>>) {
    for block in blocks {
      self.gen_block(block);
    }
  }

  pub fn gen_block(&mut self, block: &Token<BlockToken>) {
    let Span { start, end } = block.span;
    match &block.value {
      BlockToken::ATXHeading { level, raws } => {
        self.write("jsxs(\"");
        self.write(level.to_str());
        self.write("\", {children: [");
        self.gen_raws(raws);
      }
      BlockToken::SetextHeading { level, raws } => {
        self.write("jsxs(\"");
        self.write(level.to_str());
        self.write("\", {children: [");
        self.gen_raws(raws);
      }
      BlockToken::Paragraph { raws } => {
        self.write("jsxs(\"p\", {children: [");
        self.gen_raws(raws);
      }
      BlockToken::BlockQuote { level, blocks } => {
        self.write("jsxs(\"blockquote\", {children: [");
        self.gen_blocks(blocks);
      }
      BlockToken::List { blocks, .. } => {
        self.write("jsxs(\"ul\", {children: [");
        self.gen_blocks(blocks);
      }
      BlockToken::ListItem { blocks, .. } => {
        self.write("jsxs(\"li\", {children: [");
        self.gen_blocks(blocks);
      }
      _ => {
        self.write("jsxs(\"div\", {children: [");
      }
    }
    self.write("]})");
  }

  fn gen_raws(&mut self, raws: &Vec<Span>) {
    // let tokens = parse_raws_to_inlines(raws);
    // let source = document.source;
    // for item in tokens {
    //   let start = item.span.start;
    //   let end = item.span.end;
    //   match item.value {
    //     InlineToken::Text => {
    //       self.write("\"");
    //       self.write(&source[start..end]);
    //       self.write("\",");
    //     }
    //     InlineToken::Code => {
    //       self.write("\"");
    //       self.write(&source[start..end]);
    //       self.write("\",");
    //     }
    //     InlineToken::InlineCodeStart => {
    //       self.write("jsxs(\"code\", {children: [");
    //     }
    //     InlineToken::InlineCodeEnd => {
    //       self.write("]})");
    //     }
    //     InlineToken::EmphasisStart { keyword, repeat } => match keyword {
    //       b'*' | b'_' => {
    //         if repeat > 1 {
    //           self.write("jsxs(\"strong\", {children: [");
    //         } else {
    //           self.write("jsxs(\"em\", {children: [");
    //         }
    //       }
    //       b'~' => {
    //         self.write("jsxs(\"del\", {children: [");
    //       }
    //       _ => (),
    //     },
    //     InlineToken::EmphasisEnd { .. } => {
    //       self.write("]})");
    //     }
    //     _ => {}
    //   }
    // }
  }
}
