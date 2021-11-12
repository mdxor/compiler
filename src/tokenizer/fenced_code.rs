use crate::tokenizer::rule;
use crate::tokenizer::token::*;
use lazy_static::lazy_static;
use regex::Regex;

fn split_once(target: &str) -> (&str, &str) {
  let mut slice1 = target;
  let mut slice2 = "";
  if let Some(index) = target.find(" ") {
    slice1 = &target[..index];
    slice2 = &target[index + 1..]
  }
  (slice1, slice2)
}

pub fn fenced_code(source: &str) -> Option<TokenResult> {
  lazy_static! {
    static ref HEAD_RULE: Regex = Regex::new(rule::FENCED_CODE_HEAD_RULE).unwrap();
    static ref NORMAL_RULE: Regex = Regex::new(rule::FENCED_CODE_BODY_RULE).unwrap();
  }
  if let Some(caps) = HEAD_RULE.captures(source) {
    let head = caps.get(0).unwrap().as_str();
    let head_size = head.len();
    let backtick_size = caps.get(1).map_or(0, |v| v.as_str().len());
    let tilde_size = caps.get(3).map_or(0, |v| v.as_str().len());
    let mut lang_meta = "";
    if backtick_size > 0 {
      lang_meta = caps.get(2).unwrap().as_str();
    } else {
      lang_meta = caps.get(4).unwrap().as_str();
    }
    lang_meta = lang_meta.trim();
    let (language, meta_string) = split_once(lang_meta);

    let caps_option = if backtick_size == 3 {
      NORMAL_RULE.captures(&source[head_size..])
    } else {
      let mut regex_string = String::from(r"(?:|([\s\S]*?)\n) {0,3}");
      if backtick_size > 0 {
        regex_string.push_str("`{");
        regex_string.push_str(&backtick_size.to_string());
      } else {
        regex_string.push_str(&"~");
        regex_string.push_str(&tilde_size.to_string());
      }
      regex_string.push_str(",} *(?:\n|$)|$");
      let regex = Regex::new(&regex_string).unwrap();
      regex.captures(&source[head_size..])
    };
    if let Some(caps) = caps_option {
      let size = caps.get(0).unwrap().as_str().len();
      let code = caps.get(1).map_or("", |v| v.as_str());
      Some(TokenResult {
        size: head_size + size,
        token: Token::FencedCode(FencedCode {
          code,
          language,
          meta_string,
        }),
      })
    } else {
      Some(TokenResult {
        size: source.len(),
        token: Token::FencedCode(FencedCode {
          code: &source[head_size..],
          language,
          meta_string,
        }),
      })
    }
  } else {
    None
  }
}

#[test]
fn test_fenced_code() {
  let cases = vec!["```ruby meta\ndef foo(x)\n  return 3\nend\n```"];
  let mut results = vec![];
  for case in cases {
    results.push(fenced_code(case));
  }
  insta::assert_yaml_snapshot!(results);
}
