use crate::tokenizer::rule;
use crate::tokenizer::token::*;
use lazy_static::lazy_static;
use regex::Regex;

fn _table_cell(source: &str) -> Option<TableCell> {
  let mut bytes = source.bytes();
  let mut columns = vec![];
  let mut start = 0;
  let mut end = 0;
  let mut size = 0;
  let mut escaped = false;
  loop {
    if let Some(b) = bytes.next() {
      size += 1;
      if escaped {
        end = size;
        escaped = false;
        continue;
      }
      match b {
        b'\n' => {
          break;
        }
        b'\\' => {
          end = size;
          escaped = true;
        }
        b'|' => {
          if !source[start..end].is_empty() {
            columns.push(&source[start..end]);
          }
          start = size + 1;
          end = size + 1;
        }
        _ => {
          end = size;
        }
      }
    } else {
      break;
    }
  }
  if start != end {
    if !source[start..end].is_empty() {
      columns.push(&source[start..end]);
    }
  }
  if columns.len() > 0 {
    Some(TableCell {
      head: false,
      columns,
    })
  } else {
    None
  }
}

pub fn table_head(source: &str) -> Option<TokenResult> {
  lazy_static! {
    static ref RULE: Regex = Regex::new(rule::TABLE_RULE).unwrap();
  }
  if let Some(caps) = RULE.captures(source) {
    let size = caps.get(0).unwrap().as_str().len();
    let head = caps.get(1).unwrap().as_str();
    let align = caps.get(2).unwrap().as_str().trim();

    if align.starts_with("- ") {
      return None;
    }

    if let Some(align_cell) = _table_cell(align) {
      if align_cell.columns.len() == 0 && !align.starts_with("|") && !align.ends_with("|") {
        return None;
      }
      if let Some(head_cell) = _table_cell(head) {
        if align_cell.columns.len() != head_cell.columns.len() {
          return None;
        }
        return Some(TokenResult {
          size,
          token: Token::TableCell(TableCell {
            head: true,
            columns: head_cell.columns,
          }),
        });
      }
    }
    return None;
  } else {
    None
  }
}

#[test]
fn test_table_head() {
  let cases = vec!["| foo | bar |\n| --- | --- |"];
  let mut results = vec![];
  for case in cases {
    results.push(table_head(case));
  }
  insta::assert_yaml_snapshot!(results);
}
