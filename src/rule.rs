use regex::Regex;
struct Rule {
  atx_heading: Regex,
}

impl Rule {
  pub fn new() -> Self {
    Rule {
      atx_heading: Regex::new("^(#{1,6}) ").unwrap(),
    }
  }
}
