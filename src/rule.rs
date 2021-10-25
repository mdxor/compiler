use regex::Regex;

pub struct Rule {
  pub atx_heading: Regex,
  pub empty_atx_heading: Regex,
  pub indented_code: Regex,
  pub closing_atx_heading: Regex,
  pub thematic_break: Regex,
  pub fenced_code: Regex,
  pub fenced_code_end: Regex,
  pub whitespace: Regex,
}

impl Rule {
  pub fn new() -> Self {
    Rule {
      indented_code: Regex::new("^ {4}").unwrap(),
      atx_heading: Regex::new("^(#{1,6}) ").unwrap(),
      empty_atx_heading: Regex::new("^(#{1,6})(?:\n|$)").unwrap(),
      closing_atx_heading: Regex::new("(^| )#+ *$").unwrap(),
      thematic_break: Regex::new("^(?:(?:\\* *){3,}|(?:_ *){3,}|(?:\\- *){3,})(?:\n|$)").unwrap(),
      fenced_code: Regex::new("^```").unwrap(),
      fenced_code_end: Regex::new("(^ {0,3}|\n {0,3})``` *\n?").unwrap(),
      whitespace: Regex::new(" +").unwrap(),
    }
  }
}
