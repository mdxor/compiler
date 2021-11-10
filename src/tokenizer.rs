use crate::jsx;
use crate::token;
pub struct Tokenizer {}
impl Tokenizer {
  fn atx_heading(&mut self, source: &str) -> Option<token::ATXHeading> {
    Some(token::ATXHeading {
      level: 8,
      inlines: source,
    })
  }
}
