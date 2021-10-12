extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

mod jsx;
pub mod lexer;
#[cfg(test)]
mod lexer_test;
mod rule;
mod token;
// mod parse;

#[wasm_bindgen]
pub fn transform(code: &str) -> String {
    // let mut lex = lexer::lexer(code);
    // lex.next();
    // lex.slice().to_owned()
    "".to_owned()
}
