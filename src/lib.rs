extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

mod block;
mod codegen;
mod document;
mod inline;
mod input;
mod jsx;
mod jsx_lexer;
mod jsx_parser;
mod lexer;
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
import React from "react";
# ti`tle`
this is a ~~paragraph~~
> 123321

<div test={{a:{b:[2]}}}>222</div>
231<><div test={true}></div></>

- list item 1
- list item2

```
22
```

    let a = 11;
"#;
  println!("{}", parse(source));
}
