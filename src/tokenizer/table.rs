use crate::tokenizer::rule;
use crate::tokenizer::token::*;
use lazy_static::lazy_static;
use regex::Regex;
pub fn table_head(source: &str) -> Option<TokenResult> {
  lazy_static! {
    static ref RULE: Regex = Regex::new(rule::TABLE_RULE).unwrap();
  }
  if let Some(caps) = RULE.captures(source) {
    let size = caps.get(0).unwrap().as_str().len();
    let column_number = caps.get(2).unwrap().as_str().split("|").count();
    Some(TokenResult {
      size,
      token: Token::TableHead(TableHead { column_number }),
    })
  } else {
    None
  }
}

#[test]
fn test_table_head() {
  let cases = vec!["| foo | bar |\n| --- | --- |"];
  let mut results = vec![];
  for case in cases {
    results.push(table_head(case));
  }
  insta::assert_yaml_snapshot!(results);
}
