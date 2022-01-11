use crate::_js_lexer::*;
use crate::token::*;

pub struct JSParser<'a> {
  bytes: &'a [u8],
  lexer: JSLexer<'a>,
  token: Option<Token<JSToken>>,
}

impl<'a> JSParser<'a> {
  pub fn new(bytes: &'a [u8], spans: &'a Vec<Span>) -> Self {
    let lexer = JSLexer::new(bytes, spans);
    Self {
      bytes,
      lexer,
      token: None,
    }
  }

  fn next_token(&mut self) -> &Option<Token<JSToken>> {
    self.token = self.lexer.next_token();
    &self.token
  }
}
