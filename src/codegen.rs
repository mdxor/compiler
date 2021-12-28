use crate::block::*;
use crate::document::*;
use crate::inline::*;
use crate::token::*;
use crate::tree::*;
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

  pub fn gen<'source>(
    &mut self,
    tree: &mut Tree<Item<Token<'source>>>,
    document: &mut Document<'source>,
  ) {
    self.visitTree(tree, document, 0);
  }
  fn gen_inline<'source>(
    &mut self,
    tree: &mut Tree<Item<Token<'source>>>,
    document: &mut Document<'source>,
    block_id: usize,
  ) {
    let tokens = parse_block_to_inlines(tree, document, block_id);
    let source = document.source;
    for item in tokens {
      let start = item.start;
      let end = item.end;
      match item.value {
        InlineToken::Text(text) => {
          self.write("\"");
          self.write(text);
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
        InlineToken::EmphasisStart(ch, repeat) => match ch {
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
        InlineToken::EmphasisEnd(..) => {
          self.write("]})");
        }
        _ => {}
      }
    }
    self.write("]})");
  }

  fn visitTree<'source>(
    &mut self,
    tree: &mut Tree<Item<Token<'source>>>,
    document: &mut Document<'source>,
    id: usize,
  ) {
    match &tree[id].item.value {
      Token::ATXHeading(level) => {
        self.write("jsxs(\"");
        self.write(level.to_str());
        self.write("\", {children: [");
        return self.gen_inline(tree, document, id);
      }
      Token::SetextHeading(level) => {
        self.write("jsxs(\"");
        self.write(level.to_str());
        self.write("\", {children: [");
        return self.gen_inline(tree, document, id);
      }
      Token::Paragraph => {
        self.write("jsxs(\"p\", {children: [");
        return self.gen_inline(tree, document, id);
      }
      Token::BlockQuote(level) => {
        self.write("jsxs(\"blockquote\", {children: [");
      }
      Token::List(..) => {
        self.write("jsxs(\"ul\", {children: [");
      }
      Token::ListItem(..) => {
        self.write("jsxs(\"li\", {children: [");
      }
      _ => {
        self.write("jsxs(\"div\", {children: [");
      }
    }
    let mut child = tree[id].child;
    while child.is_some() {
      self.visitTree(tree, document, child.unwrap());
      child = tree[child.unwrap()].next;
      if child.is_some() {
        self.write(",");
      }
    }
    self.write("]})");
  }
}
