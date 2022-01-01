use swc_common::source_map::Pos;
use swc_common::{input::Input, BytePos, Span, Spanned};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{EsConfig, Parser, StringInput, Syntax, Tokens};

fn module(input: &str) {
  let lexer = swc_ecma_parser::lexer::Lexer::new(
    Syntax::Es(EsConfig {
      jsx: true,
      ..Default::default()
    }),
    EsVersion::latest(),
    StringInput::new(input, BytePos::from_usize(0), BytePos::from_usize(0)),
    None,
  );
  let mut es_parser = Parser::new_from(lexer);
  let ret = es_parser.parse_stmt(true);
  if let Ok(module) = ret {
    println!("{:?}", module);
  } else {
    println!("error");
  }
}

#[test]
fn test_module() {
  let source = "import x from 'react'";
  module(source);
}
