extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

mod jsx;
#[cfg(test)]
mod jsx_test;
mod tokenizer;
// mod parse;

#[wasm_bindgen]
pub fn transform(code: &str) -> String {
    // let mut lex = lexer::lexer(code);
    // lex.next();
    // lex.slice().to_owned()
    "".to_owned()
}
