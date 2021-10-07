use crate::jsx;
use crate::token;
use lazy_static::lazy_static;
use regex::Regex;

struct Lexer<'a> {
  source: &'a str,
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str) -> Self {
    Lexer { source }
  }

  fn skip_whitespace(&mut self) -> usize {
    lazy_static! {
      static ref WHITESPACE_REGEX: Regex = Regex::new("^ +").unwrap();
    }
    if let Some(caps) = WHITESPACE_REGEX.captures(self.source) {
      let size = caps.get(0).unwrap().as_str().len();
      self.move_by(size);
      size
    } else {
      0
    }
  }

  fn move_by(&mut self, size: usize) -> &'a str {
    let result = &self.source[..size];
    self.source = &self.source[size..];
    result
  }

  fn tokenize(&mut self) -> Result<token::AST<'a>, &'a str> {
    let mut blocks = vec![];
    loop {
      if self.source.is_empty() {
        break;
      } else {
        let token = self.scan_block_token()?;
        blocks.push(token);
      }
    }
    let ast = token::AST { blocks };
    Ok(ast)
  }

  fn scan_block_token(&mut self) -> Result<token::BlockToken<'a>, &'a str> {
    lazy_static! {
      static ref HEADING_START_REGEX: Regex = Regex::new("^(#{1,6}) ").unwrap();
    }
    if self.source.starts_with("    ") {
      self.move_by(4);
      return Ok(self.scan_single_line_code());
    } else {
      self.skip_whitespace();
      if let Some(caps) = HEADING_START_REGEX.captures(self.source) {
        let size = caps.get(1).unwrap().as_str().len();
        return self.scan_heading(size);
      } else if self.source.starts_with("```") {
        return Ok(self.scan_multiple_line_code());
      }
      return self.scan_paragraph();
    }
    Err("")
  }

  fn scan_paragraph(&mut self) -> Result<token::BlockToken<'a>, &'a str> {
    let content = self.scan_inline_blocks()?;
    Ok(token::BlockToken::Paragraph(token::Paragraph { content }))
  }

  fn scan_heading(&mut self, level: usize) -> Result<token::BlockToken<'a>, &'a str> {
    self.move_by(level + 1);
    let content = self.scan_inline_blocks()?;
    let heading = token::Heading {
      level: level as u8,
      content,
    };
    Ok(token::BlockToken::Heading(heading))
  }

  fn scan_inline_blocks(&mut self) -> Result<Vec<token::InlineBlock<'a>>, &'a str> {
    self.skip_whitespace();
    let mut inline_blocks = vec![];
    loop {
      if self.source.is_empty() || self.source.starts_with("\n") {
        if self.source.starts_with("\n") {
          self.move_by(1);
        }
        return Ok(inline_blocks);
      } else if self.source.starts_with("`") {
      } else {
        inline_blocks.push(self.scan_inline_text());
      }
    }
  }

  fn scan_inline_text(&mut self) -> token::InlineBlock<'a> {
    let mut chars = self.source.chars();
    let mut text_size = 0;
    let mut text = "";
    loop {
      if let Some(char) = chars.next() {
        match char {
          '\n' | '`' => {
            text = self.move_by(text_size);
            break;
          }
          _ => {
            text_size += char.len_utf8();
          }
        }
      } else {
        text = self.move_by(text_size);
        break;
      }
    }
    token::InlineBlock::Text(text)
  }

  fn scan_single_line_by_end_char(&mut self, end_char: char) -> &'a str {
    let mut size = 0;
    let mut chars = self.source.chars();
    loop {
      if let Some(char) = chars.next() {
        if char == end_char || char == '\n' {
          break;
        } else {
          size += char.len_utf8();
        }
      } else {
        return self.move_by(size);
      }
    }
    self.move_by(size + 1)
  }

  fn scan_single_line_code(&mut self) -> token::BlockToken<'a> {
    let code = self.scan_single_line_by_end_char('\n');
    token::BlockToken::SCode(code)
  }

  fn scan_multiple_line_code(&mut self) -> token::BlockToken<'a> {
    lazy_static! {
      static ref CODE_END_REGEX: Regex = Regex::new(r"(^ {0,3}|\n {0,3})``` *\n?").unwrap();
    }
    self.move_by(3);
    let mut language = "";
    let mut metastring = "";
    let mut code = "";

    self.skip_whitespace();
    let mut code_info = self.scan_single_line_by_end_char('\n');
    if code_info.ends_with("\n") {
      code_info = &code_info[..code_info.len() - 1];
    }
    if !code_info.is_empty() {
      if let Some(index) = code_info.find(" ") {
        language = &code_info[0..index].trim();
        metastring = &code_info[index..].trim_start();
      } else {
        language = code_info.trim();
      }
    }

    if let Some(captures) = CODE_END_REGEX.captures(self.source) {
      let end_token = captures.get(0).unwrap().as_str();
      let code_size = self.source.find(end_token).unwrap();
      code = self.move_by(code_size);
      self.move_by(end_token.len());
    } else {
      code = self.move_by(self.source.len());
    }
    token::BlockToken::MCode(token::MCode {
      code,
      language,
      metastring,
    })
  }

  fn scan_jsx(&mut self, is_inline: bool) -> Result<jsx::JSXNode<'a>, &'a str> {
    let mut jsx_parser = jsx::JSXParser::new(self.source, 0, is_inline);
    let jsx_node = jsx_parser.parse();
    self.move_by(jsx_parser.size);
    jsx_node
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn parse(source: &str) -> Result<token::AST, &str> {
    let mut lex = Lexer::new(source);
    let ast = lex.tokenize();
    ast
  }
  #[test]
  fn test_lexer_parse() {
    let cases = vec![
      "    abc",
      "    <div></div>",
      "```\ncode\n```",
      "```jsx\nlet a = 11;\n```",
      "```jsx meta\nlet a = 11;\n```",
      "# 123",
      "###### 123",
      "#123",
      "####### 123",
      "#### 123\n```\ncode\n```",
    ];
    let mut results = vec![];
    for case in &cases {
      let result = parse(case);
      results.push(result)
    }
    insta::assert_yaml_snapshot!(results);
  }
}
