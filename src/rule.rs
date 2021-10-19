use regex::Regex;

pub struct Rule {
  pub atx_heading: Regex,
  pub empty_atx_heading: Regex,
  pub indented_code: Regex,
  pub closing_atx_heading: Regex,
  pub thematic_break: Regex,
}

impl Rule {
  pub fn new() -> Self {
    let line_end = "(?:\n|$)";
    Rule {
      indented_code: Regex::new("^ {4}").unwrap(),
      atx_heading: Regex::new("^(#{1,6}) ").unwrap(),
      empty_atx_heading: Regex::new(&format!("{}{}", "^(#{1,6})", line_end)).unwrap(),
      closing_atx_heading: Regex::new("(^| )#+ *$").unwrap(),
      thematic_break: Regex::new(r"^(?:(?:\* *){3,}|(?:_ *){3,}|(?:\- *){3,})(?:\n|$)").unwrap(),
    }
  }
}
