use regex::Regex;

pub struct Rule {
  pub atx_heading: Regex,
  pub empty_atx_heading: Regex,
  pub indented_code: Regex,
  pub closing_atx_heading: Regex,
}

impl Rule {
  pub fn new() -> Self {
    Rule {
      indented_code: Regex::new("^ {4}").unwrap(),
      atx_heading: Regex::new("^(#{1,6}) ").unwrap(),
      empty_atx_heading: Regex::new("^(#{1,6})(?:\n|$)").unwrap(),
      closing_atx_heading: Regex::new("(^| )#+ *$").unwrap(),
    }
  }
}
