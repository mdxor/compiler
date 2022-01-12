use crate::jsx_lexer::*;
use crate::token::*;

pub struct JSXParser<'a> {
  source: &'a str,
  lexer: JSXLexer<'a>,
}

impl<'a> JSXParser<'a> {
  pub fn new(source: &'a str, bytes: &'a [u8], spans: &'a Vec<Span>) -> Self {
    let lexer = JSXLexer::new(bytes, spans);
    Self { source, lexer }
  }

  pub fn jsx_element(&mut self, opened: bool) -> Option<JSXElement> {
    if !opened {
      self.lexer.read_target_punctuator(b"<")?;
    }
    if self.lexer.read_target_punctuator(b"/").is_some() {
      return None;
    }
    let open_tag = self.jsx_tag()?;
    let attributes = self.jsx_attributes()?;
    if self.lexer.read_target_punctuator(b"/").is_some() {
      self.lexer.read_target_punctuator(b">")?;
      return Some(JSXElement {
        tag: open_tag,
        attributes,
        children: vec![],
      });
    }
    self.lexer.read_target_punctuator(b">")?;
    let children = self.jsx_children()?;
    // "</" eat by children
    let close_tag = self.jsx_tag()?;
    if close_tag != open_tag {
      return None;
    }
    self.lexer.read_target_punctuator(b">")?;
    return Some(JSXElement {
      tag: open_tag,
      attributes,
      children,
    });
    None
  }

  fn jsx_tag(&mut self) -> Option<String> {
    let mut tag = String::new();
    loop {
      if let Some(JSToken::Identifier(Span { start, end })) = self.lexer.read_identifier() {
        tag.push_str(&self.source[start..end]);
        if let Some(JSToken::Punctuator(Span { start, end })) =
          self.lexer.read_target_punctuator(b".")
        {
          tag.push_str(&self.source[start..end]);
        } else {
          break;
        }
      } else {
        if !tag.is_empty() {
          return None;
        }
        break;
      }
    }
    Some(tag)
  }

  fn jsx_attributes(&mut self) -> Option<Vec<JSXAttr>> {
    let mut attributes: Vec<JSXAttr> = vec![];
    loop {
      if let Some(attr) = self.jsx_spread_attr() {
        attributes.push(attr);
      } else if let Some(JSToken::Identifier(id_span)) = self.lexer.read_identifier() {
        if self.lexer.read_target_punctuator(b"=").is_none() {
          attributes.push(JSXAttr::KeyTrueValue { key: id_span });
        } else if let Some(JSToken::String(string_span)) = self.lexer.read_string_literal() {
          attributes.push(JSXAttr::KeyLiteralValue {
            key: id_span,
            value: string_span,
          })
        } else {
          let segments = self.jsx_expression()?;
          attributes.push(JSXAttr::KeyValue {
            key: id_span,
            value: segments,
          })
        }
      } else {
        break;
      }
    }
    Some(attributes)
  }

  fn jsx_spread_attr(&mut self) -> Option<JSXAttr> {
    self.lexer.read_target_punctuator(b"{")?;
    self.lexer.read_target_punctuator(b"...")?;
    let expression_segments = self.js_expression()?;
    self.lexer.read_target_punctuator(b"}")?;
    return Some(JSXAttr::Spread(expression_segments));
  }

  fn jsx_children(&mut self) -> Option<Vec<JSX>> {
    let mut children = vec![];
    loop {
      if let Some(JSToken::Text(span)) = self.lexer.read_jsx_text() {
        children.push(JSX::Text(span));
      } else if let Some(segments) = self.jsx_expression() {
        children.push(JSX::Expression(segments));
      } else if self.lexer.read_target_punctuator(b"<").is_some() {
        if let Some(jsx_element) = self.jsx_element(true) {
          children.push(JSX::Element(jsx_element));
        } else {
          break;
        }
      } else {
        break;
      }
    }
    Some(children)
  }

  fn jsx_expression(&mut self) -> Option<Vec<JSXExpressionSegment>> {
    self.lexer.read_target_punctuator(b"{")?;
    let expression_segments = self.js_expression()?;
    self.lexer.read_target_punctuator(b"}")?;
    return Some(expression_segments);
  }

  fn js_array(&mut self) -> Option<Vec<JSXExpressionSegment>> {
    let mut segments = vec![];
    loop {
      if let Some(mut spread_segments) = self.js_spread_expression() {
        segments.append(&mut spread_segments);
      } else {
        segments.append(&mut self.js_expression()?);
        if let Some(JSToken::Punctuator(span)) = self.lexer.read_target_punctuator(b",") {
          segments.push(JSXExpressionSegment::JS(span));
        } else {
          break;
        }
      }
    }
    if let JSToken::Punctuator(span) = self.lexer.read_target_punctuator(b"]")? {
      segments.push(JSXExpressionSegment::JS(span));
    }
    Some(segments)
  }

  fn js_object_member(&mut self) -> Option<Vec<JSXExpressionSegment>> {
    let mut segments = vec![];
    if let Some(spread_segments) = self.js_spread_expression() {
      return Some(spread_segments);
    } else if let Some(JSToken::Identifier(span)) = self.lexer.read_identifier() {
      segments.push(JSXExpressionSegment::JS(span));
      if let Some(JSToken::Punctuator(span)) = self.lexer.read_target_punctuator(b":") {
        segments.push(JSXExpressionSegment::JS(span));
        segments.append(&mut self.js_expression()?);
      }
    } else if let Some(JSToken::Identifier(span)) = self.lexer.read_string_literal() {
      segments.push(JSXExpressionSegment::JS(span));
      if let JSToken::Punctuator(span) = self.lexer.read_target_punctuator(b":")? {
        segments.push(JSXExpressionSegment::JS(span));
        segments.append(&mut self.js_expression()?);
      }
    }
    Some(segments)
  }

  fn js_object(&mut self) -> Option<Vec<JSXExpressionSegment>> {
    let mut segments = vec![];
    loop {
      segments.append(&mut self.js_object_member()?);
      if let Some(JSToken::Punctuator(span)) = self.lexer.read_target_punctuator(b",") {
        segments.push(JSXExpressionSegment::JS(span));
      } else {
        break;
      }
    }
    if let JSToken::Punctuator(span) = self.lexer.read_target_punctuator(b"}")? {
      segments.push(JSXExpressionSegment::JS(span));
      return Some(segments);
    }
    None
  }

  fn js_spread_expression(&mut self) -> Option<Vec<JSXExpressionSegment>> {
    let mut segments = vec![];
    if let JSToken::Punctuator(span) = self.lexer.read_target_punctuator(b"...")? {
      segments.push(JSXExpressionSegment::JS(span));
      segments.append(&mut self.js_expression()?);
    }
    Some(segments)
  }

  fn js_expression(&mut self) -> Option<Vec<JSXExpressionSegment>> {
    let mut segments = vec![];
    let token = self.lexer.read_token()?;
    match token {
      JSToken::Identifier(span)
      | JSToken::Keyword(span)
      | JSToken::Number(span)
      | JSToken::String(span) => {
        segments.push(JSXExpressionSegment::JS(span));
      }
      JSToken::Punctuator(span) => {
        let punctuator = &self.lexer.bytes[span.start..span.end];
        segments.push(JSXExpressionSegment::JS(span));
        match punctuator {
          b"<" => {
            let jsx_element = self.jsx_element(true)?;
            segments.push(JSXExpressionSegment::Element(jsx_element));
            return Some(segments);
          }
          b"(" => {
            segments.append(&mut self.js_expression()?);
            self.lexer.read_target_punctuator(b")")?;
          }
          b"[" => {
            segments.append(&mut self.js_array()?);
          }
          b"{" => {
            segments.append(&mut self.js_object()?);
          }
          b"!" => {
            segments.append(&mut self.js_expression()?);
          }
          b"`" => {
            segments.append(&mut self.js_template()?);
          }
          _ => {
            return None;
          }
        }
      }
      _ => return None,
    };
    loop {
      if let Some(mut child_segments) = self.js_child_expression() {
        segments.append(&mut child_segments);
      } else {
        break;
      }
    }
    Some(segments)
  }

  fn js_template(&mut self) -> Option<Vec<JSXExpressionSegment>> {
    let mut segments = vec![];
    loop {
      match self.lexer.read_template()? {
        JSToken::Punctuator(span) => {
          let punctuator = &self.lexer.bytes[span.start..span.end];
          if punctuator == b"${" {
            segments.append(&mut self.js_expression()?);
            self.lexer.read_target_punctuator(b"}")?;
          } else if punctuator == b"`" {
            return Some(segments);
          } else {
            return None;
          }
        }
        JSToken::Template(span) => {
          segments.push(JSXExpressionSegment::JS(span));
        }
        _ => return None,
      }
    }
  }

  fn js_arguments(&mut self) -> Option<Vec<JSXExpressionSegment>> {
    let mut segments = vec![];
    loop {
      if let Some(mut spread_segments) = self.js_spread_expression() {
        segments.append(&mut spread_segments);
      } else if let Some(mut expression_segments) = self.js_expression() {
        segments.append(&mut expression_segments);
      } else {
        break;
      }

      if let Some(JSToken::Punctuator(span)) = self.lexer.read_target_punctuator(b",") {
        segments.push(JSXExpressionSegment::JS(span));
      } else {
        break;
      }
    }
    Some(segments)
  }

  fn js_child_expression(&mut self) -> Option<Vec<JSXExpressionSegment>> {
    let mut segments = vec![];
    if let Some(JSToken::Punctuator(span)) = self.lexer.read_mid_punctuator() {
      let word = &self.lexer.bytes[span.start..span.end];
      match word {
        b"?" => {
          segments.append(&mut self.js_expression()?);
          self.lexer.read_target_punctuator(b":")?;
          segments.append(&mut self.js_expression()?);
        }
        b"(" => {
          segments.append(&mut self.js_arguments()?);
          self.lexer.read_target_punctuator(b")")?;
        }
        b"[" => {
          segments.append(&mut self.js_expression()?);
          self.lexer.read_target_punctuator(b"]")?;
        }
        _ => {
          segments.push(JSXExpressionSegment::JS(span));
        }
      }
    } else {
      return None;
    }
    Some(segments)
  }
}

#[test]
fn test_parse_jsx_element() {
  let cases = vec!["<div a={{b: true ? '1' : 3}}></div>"];
  let mut results = vec![];
  for case in &cases {
    let spans = vec![Span {
      start: 0,
      end: case.len(),
    }];
    let mut parser = JSXParser::new(case, case.as_bytes(), &spans);
    results.push(parser.jsx_element(false));
  }
  insta::assert_yaml_snapshot!(results);
}
