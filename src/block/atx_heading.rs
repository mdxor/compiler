use crate::ast;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

pub fn atx_heading(code: &str) -> Option<ast::MDXNode> {
  lazy_static! {
    static ref REGEX: Regex = Regex::new(r"^ {0,3}(#{1,6})(?:\s|$)(.*)(?:\n+|$)").unwrap();
  }

  lazy_static! {
    static ref HEADING_MAP: HashMap<&'static str, &'static str> = {
      let mut map = HashMap::new();
      map.insert("#", "h1");
      map.insert("##", "h2");
      map.insert("###", "h3");
      map.insert("####", "h4");
      map.insert("#####", "h5");
      map.insert("######", "h6");
      map
    };
  }
  if let Some(cap) = REGEX.captures(code) {
    let mut node = ast::MDXNode::new(HEADING_MAP.get(&cap[1]).unwrap(), "");
    let text = ast::MDXNode::new("", &cap[2]);
    node.push(text);
    return Some(node);
  }
  None
}

#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  #[test]
  fn parse_heading_1() {
    assert_eq!(
      atx_heading("## 123").unwrap(),
      ast::MDXNode {
        node_type: "h2".to_string(),
        children: vec![ast::MDXNode {
          node_type: "".to_string(),
          children: vec![],
          attrs: HashMap::new(),
          text: "123".to_string(),
        }],
        attrs: HashMap::new(),
        text: "".to_string(),
      }
    );
  }
}
