extern crate wasm_bindgen;
use compiler_core;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse(source: &str) -> String {
	compiler_core::parse(source)
}
