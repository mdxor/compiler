use lazy_static::lazy_static;
use regex::Regex;
pub fn is_match_jsx(source: &str) -> bool {
  lazy_static! {
    static ref RE: Regex =
      Regex::new(r"^<( *)([\p{L}\p{Nl}$_][\p{L}\p{Nl}$\p{Mn}\p{Mc}\p{Nd}\p{Pc}]*)").unwrap();
  }
  RE.is_match(source)
}

struct JSXExpressionCode<'a> {
  code: &'a str,
}

enum JSXAttrValue<'a> {
  String(&'a str),
  Expression(JSXExpression<'a>),
}

struct JSXKeyValueAttr<'a> {
  key: &'a str,
  value: JSXAttrValue<'a>,
}

enum JSXAttr<'a> {
  KeyValue(JSXKeyValueAttr<'a>),
  SpreadExpression(&'a str),
}

struct JSXNode<'a> {
  name: &'a str,
  attrs: Vec<JSXAttr<'a>>,
  children: Vec<JSXChild<'a>>,
}

enum JSXExpressionChild<'a> {
  Code(JSXExpressionCode<'a>),
  Node(JSXNode<'a>),
}

struct JSXExpression<'a> {
  children: Vec<JSXExpressionChild<'a>>,
}

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
  tag_stack: Vec<&'a str>,
}

impl<'a> JSXParser<'a> {
  pub fn new(source: &'a str, offset: usize, inline: bool) -> Self {
    JSXParser {
      source,
      inline,
      offset,
      size: 0,
      tag_stack: vec![],
    }
  }

  pub fn parse(&mut self) -> Result<JSXNode, &'a str> {
    self.skip();
    self.scan_jsx_node()
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
    Ok(())
  }

  fn scan_jsx_tag(&mut self) -> Result<&'a str, &'a str> {
    lazy_static! {
      static ref JSX_TAG_REGEX: Regex =
        Regex::new(r"^[\p{L}\p{Nl}$_][\p{L}\p{Nl}$\p{Mn}\p{Mc}\p{Nd}\p{Pc}]*").unwrap();
    }
    if let Some(caps) = JSX_TAG_REGEX.captures(&self.source[self.size..]) {
      let whole_size = caps.get(0).unwrap().as_str().len();
      let whitespace_len = caps.get(1).unwrap().as_str().len();
      let tag_start = self.size + whitespace_len;
      let tag_end = self.size + whole_size;
      let tag_name = &self.source[tag_start..tag_end];
      Ok(self.move_by_size(whole_size))
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
        let expression = self.scan_jsx_expression()?;
        Ok(JSXAttr::KeyValue(JSXKeyValueAttr {
          key: attr_key,
          value: JSXAttrValue::Expression(expression),
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
      self.move_by_size(1);
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
    let mut escaped = false;
    let mut chars = self.cur_source().chars();

    loop {
      if let Some(char) = chars.next() {
        match char {
          '\n' => {
            return Err("");
          }
          '\\' => {
            size += 1;
            escaped = !escaped;
          }
          endChar => {
            if escaped {
              size += 1;
              escaped = false
            }
            break;
          }
          _ => {
            size += 1;
            if escaped {
              escaped = false;
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

  fn scan_jsx_expression(&mut self) -> Result<JSXExpression<'a>, &'a str> {
    Err("")
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
