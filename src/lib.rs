extern crate wasm_bindgen;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate pest_derive;

use wasm_bindgen::prelude::*;

mod block;
mod byte;
mod document;
mod inline;
mod jsx;
#[cfg(test)]
mod jsx_test;
mod parser;
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
