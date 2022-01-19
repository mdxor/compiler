mod block;
mod codegen;
mod document;
mod inline;
mod jsx_lexer;
mod jsx_parser;
mod lexer;
mod md_lexer;
mod token;
use crate::block::*;
use crate::codegen::*;

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
  export const date = Date.now();
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
  [link](url)
  "#;
  println!("{}", parse(source));
}
