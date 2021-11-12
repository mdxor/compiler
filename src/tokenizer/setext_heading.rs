use crate::tokenizer::rule;
use crate::tokenizer::token::*;
use lazy_static::lazy_static;
use regex::Regex;
pub fn setext_heading(source: &str) -> Option<TokenResult> {
  lazy_static! {
    static ref RULE: Regex = Regex::new(rule::SETEXT_HEADING_RULE).unwrap();
  }
  if let Some(caps) = RULE.captures(source) {
    let raw = caps.get(0).unwrap().as_str();
    let level = if raw.starts_with("=") { 1 } else { 2 };
    let size = raw.len();
    Some(TokenResult {
      size,
      token: Token::SetextHeading(SetextHeading { raw, level }),
    })
  } else {
    None
  }
}

#[test]
fn test_setext_heading() {
  let cases = vec!["=========", "---------", "= =", "----      "];
  let mut results = vec![];
  for case in cases {
    results.push(setext_heading(case));
  }
  insta::assert_yaml_snapshot!(results);
}
