extern crate wasm_bindgen;
#[macro_use]
extern crate lazy_static;

use wasm_bindgen::prelude::*;

mod block;
mod byte;
mod codegen;
mod document;
mod inline;
mod input;
mod interrupt;
mod jsx;
#[cfg(test)]
mod jsx_test;
mod lexer;
mod module;
mod parse;
mod punctuation;
mod raw;
mod scan;
mod token;
mod tree;
use crate::block::*;
use crate::codegen::*;

#[wasm_bindgen]
pub fn parse(source: &str) -> String {
  let (mut tree, mut document) = parse_source_to_blocks(source);
  let mut codegen = Codegen::new();
  codegen.gen(&mut tree, &mut document);
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
