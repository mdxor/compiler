#[derive(Debug, PartialEq)]
pub enum Token<'a> {
  Heading1,
  Heading2,
  Heading3,
  Heading4,
  Heading5,
  Heading6,
  Text(&'a str),
  InlineCode,
}

pub fn get_token(token: &str) -> Token {
  match &*token {
    "#" => Token::Heading1,
    "##" => Token::Heading2,
    "###" => Token::Heading3,
    "####" => Token::Heading4,
    "#####" => Token::Heading5,
    "######" => Token::Heading6,
    _ => Token::Text(token),
  }
}
