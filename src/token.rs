#[derive(Debug, PartialEq)]
pub enum Token<'a> {
  // block
  Heading1,
  Heading2,
  Heading3,
  Heading4,
  Heading5,
  Heading6,
  Newline,
  CodeBlock,
  BulletList,
  OrderedList,
  TaskList,
  Hr,
  // inline
  Img(&'a str, &'a str),
  Link(&'a str, &'a str),
  Text(&'a str),
  CodeInline,
  JSXTag(&'a str),
  JSXAttribute(&'a str, &'a str),
  JSXText(&'a str),
  TableVerticalBar,
}

pub fn match_block_token(token: &str) -> Option<Token> {
  match &*token {
    "#" => Some(Token::Heading1),
    "##" => Some(Token::Heading2),
    "###" => Some(Token::Heading3),
    "####" => Some(Token::Heading4),
    "#####" => Some(Token::Heading5),
    "######" => Some(Token::Heading6),
    "```" => Some(Token::CodeBlock),
    _ => None,
  }
}
