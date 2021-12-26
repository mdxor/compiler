use crate::block::*;
use crate::inline::*;
use crate::token::*;
use crate::tree::*;
struct CodeGen {
  code: String,
}

impl CodeGen {
  pub fn new() -> Self {
    CodeGen {
      code: String::default(),
    }
  }

  fn write(&mut self, str: &str) {
    self.code.push_str(str);
  }

  pub fn gen<'source>(tree: &mut Tree<Item<Token<'source>>>) {}
}
