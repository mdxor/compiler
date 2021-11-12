use crate::tokenizer::rule;
use crate::tokenizer::token::*;
use lazy_static::lazy_static;
use regex::Regex;
pub fn thematic_break(source: &str) -> Option<TokenResult> {
  lazy_static! {
    static ref RULE: Regex = Regex::new(rule::THEMATIC_BREAK_RULE).unwrap();
  }
  if let Some(caps) = RULE.captures(source) {
    let size = caps.get(0).unwrap().as_str().len();
    Some(TokenResult {
      size,
      token: Token::ThematicBreak,
    })
  } else {
    None
  }
}

#[test]
fn test_thematic_break() {
  let cases = vec!["***", "---", "___\n", " **  * ** * ** * **"];
  let mut results = vec![];
  for case in cases {
    results.push(thematic_break(case));
  }
  insta::assert_yaml_snapshot!(results);
}
