use crate::tokenizer::rule;
use crate::tokenizer::token::*;
use lazy_static::lazy_static;
use regex::Regex;
pub fn list_item(source: &str) -> Option<TokenResult> {
  lazy_static! {
    static ref RULE: Regex = Regex::new(rule::LIST_ITEM_RULE).unwrap();
  }
  if let Some(caps) = RULE.captures(source) {
    let size = caps.get(0).unwrap().as_str().len();
    if let Some(order) = caps.get(2) {
      let kind = caps.get(3).unwrap().as_str();
      Some(TokenResult {
        size,
        token: Token::OrderedListItem(OrderedListItem {
          order: order.as_str().parse::<u32>().unwrap(),
          kind,
        }),
      })
    } else {
      let kind = caps.get(1).unwrap().as_str();
      Some(TokenResult {
        size,
        token: Token::BulletListItem(BulletListItem { kind }),
      })
    }
  } else {
    None
  }
}

#[test]
fn test_list_item() {
  let cases = vec!["- ", "* ", "+ ", "1. ", "123. ", "123456789) "];
  let mut results = vec![];
  for case in cases {
    results.push(list_item(case));
  }
  insta::assert_yaml_snapshot!(results);
}
