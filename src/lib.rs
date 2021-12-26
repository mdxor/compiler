extern crate wasm_bindgen;
#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use wasm_bindgen::prelude::*;

mod block;
mod byte;
mod codegen;
mod document;
mod inline;
mod interrupt;
mod jsx;
#[cfg(test)]
mod jsx_test;
mod lexer;
mod parse;
mod punctuation;
mod raw;
mod scan;
mod token;
mod tree;

#[wasm_bindgen]
pub fn transform(code: &str) -> String {
    // let mut lex = lexer::lexer(code);
    // lex.next();
    // lex.slice().to_owned()
    "".to_owned()
}
