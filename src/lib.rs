extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

mod jsx;
#[cfg(test)]
mod jsx_test;
// pub mod tokenizer;
// #[cfg(test)]
// mod lexer_test;
mod tokens;
// mod scanner;
// mod parse;

#[wasm_bindgen]
pub fn transform(code: &str) -> String {
    // let mut lex = lexer::lexer(code);
    // lex.next();
    // lex.slice().to_owned()
    "".to_owned()
}
