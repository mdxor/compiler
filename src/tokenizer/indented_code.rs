use crate::tokenizer::rule;
use crate::tokenizer::token::*;
use lazy_static::lazy_static;
use regex::Regex;
pub fn indented_code(source: &str) -> Option<TokenResult> {
  lazy_static! {
    static ref RULE: Regex = Regex::new(rule::INDENTED_CODE_RULE).unwrap();
  }
  if let Some(caps) = RULE.captures(source) {
    let raw = caps.get(0).unwrap().as_str();
    let size = raw.len();
    let mut codes = vec![];
    let lines = raw.lines();
    for line in lines {
      codes.push(&line[4..]);
    }
    Some(TokenResult {
      size,
      token: Token::IndentedCode(IndentedCode { codes }),
    })
  } else {
    None
  }
}

#[test]
fn test_indented_code() {
  let cases = vec![
    "    a simple\n      indented code block",
    "    chunk1\n      \n      chunk2",
  ];
  let mut results = vec![];
  for case in cases {
    results.push(indented_code(case));
  }
  insta::assert_yaml_snapshot!(results);
}
