extern crate wasm_bindgen;
#[macro_use]
extern crate lazy_static;

use wasm_bindgen::prelude::*;

mod block;
mod codegen;
mod document;
mod inline;
mod input;
mod jsx;
#[cfg(test)]
mod jsx_test;
mod lexer;
mod module;
mod token;
use crate::block::*;
use crate::codegen::*;

#[wasm_bindgen]
pub fn parse(source: &str) -> String {
  let mut blockParser = BlockParser::new(source);
  let mut ast = blockParser.parse();
  let mut codegen = Codegen::new(source, source.as_bytes());
  codegen.gen(&mut ast);
  codegen.code
}

#[test]
fn test_parse() {
  let source = r#"
# ti`tle`
this is a ~~paragraph~~
> 123321

- list item 1
- list item2
"#;
  println!("{}", parse(source));
}
