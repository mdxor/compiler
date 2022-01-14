use crate::jsx_lexer::*;
use crate::token::*;
use std::collections::VecDeque;

pub struct JSXParser<'a> {
  source: &'a str,
  lexer: JSXLexer<'a>,
}

impl<'a> JSXParser<'a> {
  pub fn new(source: &'a str, bytes: &'a [u8], spans: &'a VecDeque<Span>) -> Self {
    let lexer = JSXLexer::new(bytes, spans);
    Self { source, lexer }
  }

  pub fn js_import_export(&mut self) -> usize {
    let mut size: usize = 0;
    loop {
      if let Some(span) = self.lexer.read_keyword() {
        let word = &self.lexer.bytes[span.start..span.end];
        if word == b"import" {
          if self.js_import().is_none() {
            break;
          }
        } else if word == b"export" {
          if self.js_export().is_none() {
            break;
          }
        } else {
          break;
        }
        if self.lexer.read_separator().is_some() {
          if let Some(pos) = self.lexer.finish_js() {
            size = pos;
          }
          continue;
        }
      }
      break;
    }
    size
  }

  pub fn js_import(&mut self) -> Option<()> {
    self.js_import_specifier()?;
    let Span { start, end } = self.lexer.read_identifier()?;
    let from = &self.lexer.bytes[start..end];
    if from != b"from" {
      return None;
    }
    self.lexer.read_string_literal()?;
    Some(())
  }

  pub fn js_import_specifier(&mut self) -> Option<()> {
    let mut default = false;
    if let Some(span) = self.lexer.read_identifier() {
      default = true;
      if self.lexer.read_target_punctuator(b",").is_none() {
        return Some(());
      }
    }
    if self.lexer.read_target_punctuator(b"{").is_some() {
      if self.lexer.read_target_punctuator(b"}").is_none() {
        loop {
          if self.js_import_specifier_member().is_none() {
            break;
          }
          if self.lexer.read_target_punctuator(b",").is_none() {
            break;
          }
        }
        self.lexer.read_target_punctuator(b"}")?;
      }
    }
    if default {
      Some(())
    } else {
      None
    }
  }

  pub fn js_import_specifier_member(&mut self) -> Option<()> {
    let mut default = false;
    if self.lexer.read_identifier().is_some() {
    } else if let Some(Span { start, end }) = self.lexer.read_keyword() {
      let word = &self.lexer.bytes[start..end];
      if word != b"default" {
        return None;
      }
      default = true;
    } else {
      return None;
    }
    if let Some(Span { start, end }) = self.lexer.read_identifier() {
      let word = &self.lexer.bytes[start..end];
      if word != b"as" {
        return None;
      }
      self.lexer.read_identifier()?;
    } else if default {
      return None;
    }
    Some(())
  }

  pub fn js_export(&mut self) -> Option<()> {
    if let Some(Span { start, end }) = self.lexer.read_keyword() {
      let word = &self.lexer.bytes[start..end];
      if word != b"var" && word != b"let" && word != b"const" {
        return None;
      }
      self.lexer.read_target_punctuator(b"=")?;
      self.js_expression()?;
      return Some(());
    }
    self.js_import()
  }

  pub fn jsx(&mut self) -> Option<(JSXElement, usize, usize)> {
    let element = self.jsx_element(false)?;
    let (pos, index) = self.lexer.finish_jsx()?;
    Some((element, pos, index))
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
      if let Some(Span { start, end }) = self.lexer.read_identifier() {
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
      } else if let Some(id_span) = self.lexer.read_identifier() {
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
    } else if let Some(span) = self.lexer.read_identifier() {
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
  let cases = vec!["<><div test={true}></div></>\n"];
  let mut results = vec![];
  for case in &cases {
    let spans = VecDeque::from(vec![Span {
      start: 0,
      end: case.len(),
    }]);
    let mut parser = JSXParser::new(case, case.as_bytes(), &spans);
    results.push(parser.jsx());
  }
  insta::assert_yaml_snapshot!(results);
}

#[test]
fn test_parse_import_export() {
  let cases = vec!["import React from 'react';\nmport Vue from 'vue'"];
  let mut results = vec![];
  for case in &cases {
    let spans = VecDeque::from(vec![Span {
      start: 0,
      end: case.len(),
    }]);
    let mut parser = JSXParser::new(case, case.as_bytes(), &spans);
    results.push(parser.js_import_export());
  }
  insta::assert_yaml_snapshot!(results);
}
