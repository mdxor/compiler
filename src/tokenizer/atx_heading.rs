use crate::tokenizer::rule;
use crate::tokenizer::token::*;
use lazy_static::lazy_static;
use regex::Regex;
pub fn atx_heading(source: &str) -> Option<TokenResult> {
  lazy_static! {
    static ref RULE: Regex = Regex::new(rule::ATX_HEADING_RULE).unwrap();
    static ref CLOSING_RULE: Regex = Regex::new(rule::ATX_HEADING_CLOSING_RULE).unwrap();
  }
  if let Some(caps) = RULE.captures(source) {
    let size = caps.get(0).unwrap().as_str().len();
    let level = caps.get(1).unwrap().as_str().len();
    let mut raw_inlines = caps.get(2).map_or("", |v| v.as_str());
    if let Some(caps) = CLOSING_RULE.captures(raw_inlines) {
      let closing_size = caps.get(0).map_or(0, |v| v.as_str().len());
      raw_inlines = &raw_inlines[..raw_inlines.len() - closing_size];
    }
    Some(TokenResult {
      size,
      token: Token::ATXHeading(ATXHeading { level, raw_inlines }),
    })
  } else {
    None
  }
}

#[test]
fn test_atx_heading() {
  let cases = vec![
    "# foo",
    "## foo",
    "### foo",
    "#### foo",
    "##### foo",
    "###### foo",
    "####### foo",
    "#5 bolt",
    "#hashtag",
    r"\## foo",
    r"# foo *bar* \*baz\*",
    "###",
    "# foo#",
    "# foo #",
    "# #",
  ];
  let mut results = vec![];
  for case in cases {
    results.push(atx_heading(case));
  }
  insta::assert_yaml_snapshot!(results);
}
