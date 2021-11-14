use crate::tokenizer::rule;
use crate::tokenizer::token::*;
use lazy_static::lazy_static;
use regex::Regex;
pub fn block_quote(source: &str) -> Option<TokenResult> {
  lazy_static! {
    static ref RULE: Regex = Regex::new(rule::BLOCK_QUOTE).unwrap();
  }
  if let Some(caps) = RULE.captures(source) {
    let size = caps.get(0).unwrap().as_str().len();
    Some(TokenResult {
      size,
      token: Token::BlockQuote,
    })
  } else {
    None
  }
}

#[test]
fn test_block_quote() {
  let cases = vec![">", "> "];
  let mut results = vec![];
  for case in cases {
    results.push(block_quote(case));
  }
  insta::assert_yaml_snapshot!(results);
}
