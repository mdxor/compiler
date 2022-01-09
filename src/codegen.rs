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

  fn write_jsx_start(&mut self, tag: &str, jsxs: bool) {
    if jsxs {
      self.code.push_str("_jsxRuntime.jsxs(\"");
    } else {
      self.code.push_str("_jsxRuntime.jsx(\"");
    }
    self.code.push_str(tag);
    if jsxs {
      self.code.push_str("\",{children:[");
    } else {
      self.code.push_str("\",{children:");
    }
  }
  fn write_jsx_end(&mut self, jsxs: bool) {
    if jsxs {
      self.code.push_str("]})");
    } else {
      self.code.push_str("})");
    }
  }

  pub fn gen(&mut self, ast: &AST<Token<BlockToken>>) {
    self.gen_blocks("_jsxRuntime.Fragment", &ast.children);
  }

  pub fn gen_blocks(&mut self, tag: &str, blocks: &Vec<Token<BlockToken>>) {
    let jsxs = blocks.len() > 1;
    self.write_jsx_start(tag, jsxs);
    for block in blocks {
      self.gen_block(block, jsxs);
    }
    self.write_jsx_end(jsxs);
  }

  fn gen_leaf_block(&mut self, tag: &str, raws: &Vec<Span>) {
    let mut inline_parser = InlineParser::new(self.bytes, raws);
    let inlines = inline_parser.parse();
    self.gen_inlines_with_tag(tag, &inlines.children);
  }

  fn gen_inlines_with_tag(&mut self, tag: &str, inlines: &Vec<Token<InlineToken>>) {
    let jsxs = inlines.len() > 1;
    self.write_jsx_start(tag, jsxs);
    self.gen_inlines(inlines);
    self.write_jsx_end(jsxs);
  }

  fn gen_inlines(&mut self, inlines: &Vec<Token<InlineToken>>) {
    for inline in inlines {
      self.gen_inline(inline, inlines.len() > 1);
    }
  }

  fn gen_span(&mut self, span: &Span) {
    self.write(&self.source[span.start..span.end]);
  }

  fn gen_spans(&mut self, spans: &Vec<Span>) {
    for span in spans {
      self.gen_span(span);
    }
  }

  fn gen_inline(&mut self, inline: &Token<InlineToken>, jsxs: bool) {
    let span = &inline.span;
    match &inline.value {
      InlineToken::Text(text_spans) => {
        self.write("\"");
        for text_span in text_spans {
          self.write(&self.source[text_span.start..text_span.end]);
        }
        self.write("\"");
      }
      InlineToken::HardBreak => {
        self.write("_jsxRuntime.jsx(\"br\")");
      }
      InlineToken::Emphasis(children) => {
        let tag = if self.bytes[span.start] == b'~' {
          "del"
        } else if span.end - span.start >= 2 {
          "strong"
        } else {
          "em"
        };
        self.gen_inlines_with_tag(tag, children);
      }
      InlineToken::Link {
        text_children,
        url,
        title,
      } => {
        // TODO: url title
        self.gen_inlines_with_tag("a", text_children);
      }
      InlineToken::Code(code_spans) => {
        self.write_jsx_start("code", false);
        self.write("\"");
        self.gen_spans(code_spans);
        self.write("\"");
        self.write_jsx_end(false);
      }
      _ => {
        return;
      }
    }
    if jsxs {
      self.write(",");
    }
  }

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
      BlockToken::FencedCode { meta_span, codes } => {
        self.write_jsx_start("pre", false);
        self.write_jsx_start("code", false);
        self.gen_spans(codes);
        self.write_jsx_end(false);
        self.write_jsx_end(false);
      }
      _ => {
        return;
      }
    }
    if jsxs {
      self.write(",");
    }
  }
}
