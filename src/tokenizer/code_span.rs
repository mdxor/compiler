use crate::tokenizer::rule;
use crate::tokenizer::token::*;
use lazy_static::lazy_static;
use regex::Regex;
pub fn code_span(source: &str) -> Option<TokenResult> {
  lazy_static! {
    static ref RULE: Regex = Regex::new(rule::CODE_SPAN_RULE).unwrap();
  }
  if let Some(caps) = RULE.captures(source) {
    let starting = caps.get(1).unwrap().as_str();
    let code = caps.get(2).unwrap().as_str();
    let ending = caps.get(3).unwrap().as_str();
    if code.is_empty() || starting.len() != ending.len() {
      return None;
    }
    let size = caps.get(0).unwrap().as_str().len();
    Some(TokenResult {
      size,
      token: Token::CodeSpan(code),
    })
  } else {
    None
  }
}

#[test]
fn test_code_span() {
  let cases = vec!["`12`", "``", "` `", "``123``", "``123```"];
  let mut results = vec![];
  for case in cases {
    results.push(code_span(case));
  }
  insta::assert_yaml_snapshot!(results);
}
