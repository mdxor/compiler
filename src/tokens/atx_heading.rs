use crate::tokens::def::*;
use crate::tokens::rule;
use lazy_static::lazy_static;
use regex::Regex;
pub fn atx_heading(source: &str) -> Option<ATXHeading> {
  lazy_static! {
    static ref RULE: Regex = Regex::new(rule::ATX_HEADING_RULE).unwrap();
  }
  if let Some(caps) = RULE.captures(source) {
    let level = caps.get(1).unwrap().as_str().len();
    let raw_inlines = caps.get(2).map_or("", |v| v.as_str());
    Some(ATXHeading { level, raw_inlines })
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
  ];
  let mut results = vec![];
  for case in cases {
    results.push(atx_heading(case));
  }
  insta::assert_yaml_snapshot!(results);
}
