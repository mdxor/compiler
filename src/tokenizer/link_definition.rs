use crate::tokenizer::rule;
use crate::tokenizer::token::*;
use lazy_static::lazy_static;
use regex::Regex;
pub fn link_definition(source: &str) -> Option<TokenResult> {
  lazy_static! {
    static ref RULE: Regex = Regex::new(
      &rule::LINK_DEFINITION_RULE
        .replace("label", rule::LABEL_RULE)
        .replace("title", rule::TITLE_RULE)
    )
    .unwrap();
  }
  if let Some(caps) = RULE.captures(source) {
    let label = caps.get(1).unwrap().as_str().trim();
    if label == "" {
      return None;
    }
    let raw = caps.get(0).unwrap().as_str();
    let size = raw.len();

    let href = caps.get(2).unwrap().as_str();
    let mut title = caps.get(3).map_or("", |v| v.as_str());
    if !title.is_empty() {
      title = &title[1..title.len() - 1];
    }
    Some(TokenResult {
      size,
      token: Token::LinkDefinition(LinkDefinition { label, href, title }),
    })
  } else {
    None
  }
}

#[test]
fn test_link_definition() {
  let cases = vec![r#"[foo]: /url "title""#];
  let mut results = vec![];
  for case in cases {
    results.push(link_definition(case));
  }
  insta::assert_yaml_snapshot!(results);
}
