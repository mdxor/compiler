use lazy_static::lazy_static;
use regex::Regex;
pub fn is_match_jsx(source: &str) -> bool {
  lazy_static! {
    static ref RE: Regex =
      Regex::new(r"^<( *)([\p{L}\p{Nl}$_][\p{L}\p{Nl}$\p{Mn}\p{Mc}\p{Nd}\p{Pc}]*)").unwrap();
  }
  RE.is_match(source)
}

#[derive(Debug, PartialEq)]
enum JSXAttrValue<'a> {
  String(&'a str),
  Expression(JSXExpression<'a>),
}

#[derive(Debug, PartialEq)]
struct JSXKeyValueAttr<'a> {
  key: &'a str,
  value: JSXAttrValue<'a>,
}

#[derive(Debug, PartialEq)]
enum JSXAttr<'a> {
  KeyValue(JSXKeyValueAttr<'a>),
  SpreadExpression(&'a str),
}

#[derive(Debug, PartialEq)]
struct JSXNode<'a> {
  name: &'a str,
  attrs: Vec<JSXAttr<'a>>,
  children: Vec<JSXChild<'a>>,
}

#[derive(Debug, PartialEq)]
enum JSXExpressionChild<'a> {
  Code(&'a str),
  Node(JSXNode<'a>),
}

#[derive(Debug, PartialEq)]
struct JSXExpression<'a> {
  children: Vec<JSXExpressionChild<'a>>,
}

#[derive(Debug, PartialEq)]
enum JSXChild<'a> {
  Node(JSXNode<'a>),
  Expression(JSXExpression<'a>),
  Text(&'a str),
}

pub struct JSXParser<'a> {
  source: &'a str,
  inline: bool,
  offset: usize,
  size: usize,
}

impl<'a> JSXParser<'a> {
  pub fn new(source: &'a str, offset: usize, inline: bool) -> Self {
    JSXParser {
      source,
      inline,
      offset,
      size: 0,
    }
  }

  pub fn parse(&mut self) -> Result<(JSXNode<'a>, usize), &'a str> {
    self.skip();
    let node = self.scan_jsx_node()?;
    Ok((node, self.size))
  }

  fn scan_jsx_node(&mut self) -> Result<JSXNode<'a>, &'a str> {
    if self.cur_source().starts_with("<") {
      self.move_by_size(1);
      let tag_name = self.scan_jsx_tag()?;
      let attrs = self.scan_jsx_attributes()?;
      let children = self.scan_jsx_children()?;
      self.scan_jsx_end_tag(tag_name)?;
      Ok(JSXNode {
        name: tag_name,
        attrs,
        children,
      })
    } else {
      Err("")
    }
  }

  fn scan_jsx_end_tag(&mut self, expected_tag_name: &str) -> Result<(), &'a str> {
    let end_tag_name = self.scan_jsx_tag()?;
    if end_tag_name != expected_tag_name {
      return Err("");
    }
    self.skip();
    if !self.cur_source().starts_with(">") {
      return Err("");
    }
    self.move_by_size(1);
    Ok(())
  }

  fn scan_jsx_tag(&mut self) -> Result<&'a str, &'a str> {
    lazy_static! {
      static ref JSX_TAG_REGEX: Regex =
        Regex::new(r"^[\p{L}\p{Nl}$_][\p{L}\p{Nl}$\p{Mn}\p{Mc}\p{Nd}\p{Pc}]*").unwrap();
    }
    if let Some(caps) = JSX_TAG_REGEX.captures(&self.source[self.size..]) {
      let size = caps.get(0).unwrap().as_str().len();
      Ok(self.move_by_size(size))
    } else {
      // TODO
      Err("error")
    }
  }

  fn skip(&mut self) {
    // TODO
    lazy_static! {
      static ref WHITESPACE_REGEX: Regex = Regex::new(r"^ *").unwrap();
    }
    if let Some(caps) = WHITESPACE_REGEX.captures(&self.source[self.size..]) {
      let size = caps.get(0).unwrap().as_str().len();
      self.size += size;
    }
  }

  fn cur_source(&mut self) -> &str {
    &self.source[self.size..]
  }

  fn move_by_size(&mut self, size: usize) -> &'a str {
    let str = &self.source[self.size..self.size + size];
    self.size += size;
    str
  }

  fn scan_jsx_attributes(&mut self) -> Result<Vec<JSXAttr<'a>>, &'a str> {
    let mut attrs = vec![];
    loop {
      self.skip();
      if self.cur_source().starts_with(">") {
        self.move_by_size(1);
        return Ok(attrs);
      }
      let attr = self.scan_jsx_attribute()?;
      attrs.push(attr);
    }
    Err("")
  }

  fn scan_jsx_attribute(&mut self) -> Result<JSXAttr<'a>, &'a str> {
    // TODO: spread attr
    lazy_static! {
      static ref ATTR_KEY_REGEX: Regex =
        Regex::new(r"^[\p{L}\p{Nl}$_][\p{L}\p{Nl}$\p{Mn}\p{Mc}\p{Nd}\p{Pc}]*").unwrap();
    }

    // <H1 {...attrs}>
    if self.cur_source().starts_with("{") {
      self.move_by_size(1);
      let spread_expression = self.scan_jsx_attribute_spread_expression()?;
      Ok(spread_expression)
    } else if let Some(caps) = ATTR_KEY_REGEX.captures(self.cur_source()) {
      let attr_key_size = caps.get(0).unwrap().as_str().len();
      let attr_key = self.move_by_size(attr_key_size);
      self.skip();

      // <H1 title="title" />
      if self.cur_source().starts_with("=") {
        self.move_by_size(1);
        let value = self.scan_jsx_value()?;
        Ok(JSXAttr::KeyValue(JSXKeyValueAttr {
          key: attr_key,
          value,
        }))
      } else {
        // <H1 inline />
        return Ok(JSXAttr::KeyValue(JSXKeyValueAttr {
          key: attr_key,
          value: JSXAttrValue::String("true"),
        }));
      }
    } else {
      return Err("");
    }
  }

  fn scan_jsx_attribute_spread_expression(&mut self) -> Result<JSXAttr<'a>, &'a str> {
    Err("")
  }

  fn scan_jsx_value(&mut self) -> Result<JSXAttrValue<'a>, &'a str> {
    self.skip();
    if self.cur_source().starts_with("{") {
      let expression = self.scan_jsx_expression()?;
      return Ok(JSXAttrValue::Expression(expression));
    }
    let mut endCharOption: Option<char> = None;
    if self.cur_source().starts_with("\"") {
      endCharOption = Some('"');
    }
    if self.cur_source().starts_with("'") {
      endCharOption = Some('\'');
    }
    if let Some(endChar) = endCharOption {
      let value = self.scan_jsx_value_string(endChar)?;
      return Ok(value);
    }
    Err("")
  }

  fn scan_jsx_value_string(&mut self, endChar: char) -> Result<JSXAttrValue<'a>, &'a str> {
    let mut size = 0;
    self.move_by_size(endChar.len_utf8());
    let mut chars = self.cur_source().chars();

    loop {
      if let Some(char) = chars.next() {
        match char {
          '\n' => {
            return Err("");
          }
          _ => {
            if char == endChar {
              break;
            } else {
              size += 1;
            }
          }
        }
      } else {
        return Err("");
      }
    }
    let string = self.move_by_size(size);
    self.move_by_size(1);
    return Ok(JSXAttrValue::String(string));
  }

  // {"}"} {'}'} {`}`}
  // {{}}
  // {{'{': '}'}}
  // {() => <div></div>}
  fn scan_jsx_expression(&mut self) -> Result<JSXExpression<'a>, &'a str> {
    let mut parentheses_num = 0;
    let mut jsx_expression = JSXExpression { children: vec![] };
    loop {
      if self.cur_source().starts_with("{") {
        parentheses_num += 1;
        self.move_by_size(1);
      } else if self.cur_source().starts_with("}") {
        self.move_by_size(1);
        if parentheses_num > 0 {
          parentheses_num -= 1;
          if parentheses_num == 0 {
            break;
          }
        } else {
          return Err("");
        }
      } else if self.cur_source().starts_with("<") {
        let node = self.scan_jsx_node()?;
        jsx_expression.children.push(JSXExpressionChild::Node(node));
      } else {
        jsx_expression
          .children
          .push(self.scan_jsx_expression_code()?);
      }
    }
    Ok(jsx_expression)
  }

  fn scan_jsx_expression_code(&mut self) -> Result<JSXExpressionChild<'a>, &'a str> {
    let mut stack = vec![];
    let mut size = 0;
    let mut escaped = false;
    let mut chars = self.cur_source().chars();
    loop {
      if let Some(char) = chars.next() {
        match char {
          '<' | '{' => {
            if stack.is_empty() {
              break;
            } else {
              size += 1;
            }
          }
          '\\' => {
            size += 1;
            if escaped {
              escaped = false;
            } else if !stack.is_empty() {
              escaped = true;
            }
          }
          '\'' | '"' | '`' => {
            size += 1;
            if !escaped {
              if stack.is_empty() {
                stack.push(char);
              } else if stack.last().unwrap() == &char {
                stack.pop();
              }
            }
          }
          _ => {
            size += char.len_utf8();
            if escaped {
              escaped = false
            }
          }
        }
      } else {
        return Err("");
      }
    }
    Ok(JSXExpressionChild::Code(self.move_by_size(size)))
  }

  fn scan_jsx_children(&mut self) -> Result<Vec<JSXChild<'a>>, &'a str> {
    let mut children = vec![];
    loop {
      let cur_source = self.cur_source();
      if cur_source.starts_with("</") {
        self.move_by_size(2);
        return Ok(children);
      } else if cur_source.starts_with("<") {
        let child = self.scan_jsx_node()?;
        children.push(JSXChild::Node(child));
      } else if cur_source.starts_with("{") {
        let expression = self.scan_jsx_expression()?;
        children.push(JSXChild::Expression(expression));
      } else {
        children.push(self.scan_jsx_text_child()?);
      }
    }
    Err("")
  }

  fn scan_jsx_text_child(&mut self) -> Result<JSXChild<'a>, &'a str> {
    let mut chars = self.cur_source().chars();
    let mut size = 0;
    loop {
      if let Some(char) = chars.next() {
        match char {
          '<' | '{' => break,
          _ => {
            size += char.len_utf8();
          }
        }
      } else {
        return Err("");
      }
    }
    if size > 0 {
      return Ok(JSXChild::Text(self.move_by_size(size)));
    }
    Err("")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn parse(source: &str) -> Result<(JSXNode, usize), &str> {
    let mut jsx_parser = JSXParser::new(source, 0, true);
    jsx_parser.parse()
  }
  #[test]
  fn parse_empty_div() {
    assert_eq!(
      parse("<div></div>").unwrap(),
      (
        JSXNode {
          name: "div",
          attrs: vec![],
          children: vec![],
        },
        11
      )
    );
  }
  #[test]
  fn parse_nested_div() {
    assert_eq!(
      parse(r#"<div test="true">中文测试<div>en</div></div>"#).unwrap(),
      (
        JSXNode {
          name: "div",
          attrs: vec![JSXAttr::KeyValue(JSXKeyValueAttr {
            key: "test",
            value: JSXAttrValue::String("true")
          })],
          children: vec![
            JSXChild::Text("中文测试"),
            JSXChild::Node(JSXNode {
              name: "div",
              attrs: vec![],
              children: vec![JSXChild::Text("en"),],
            })
          ],
        },
        48
      )
    );
  }
  #[test]
  fn parse_complex() {
    assert_eq!(
      parse(r#"<div test="true" content={() => <span>content</span>}>中文测试<div>en</div></div>"#)
        .unwrap(),
      (
        JSXNode {
          name: "div",
          attrs: vec![
            JSXAttr::KeyValue(JSXKeyValueAttr {
              key: "test",
              value: JSXAttrValue::String("true")
            }),
            JSXAttr::KeyValue(JSXKeyValueAttr {
              key: "content",
              value: JSXAttrValue::Expression(JSXExpression {
                children: vec![
                  JSXExpressionChild::Code("() => "),
                  JSXExpressionChild::Node(JSXNode {
                    name: "span",
                    attrs: vec![],
                    children: vec![JSXChild::Text("content")]
                  })
                ]
              })
            })
          ],
          children: vec![
            JSXChild::Text("中文测试"),
            JSXChild::Node(JSXNode {
              name: "div",
              attrs: vec![],
              children: vec![JSXChild::Text("en"),],
            })
          ],
        },
        85
      )
    );
  }
}
