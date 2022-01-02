use crate::document::*;
use crate::input::*;
use crate::interrupt::*;
use crate::lexer::*;
use crate::token::*;
use crate::tree::*;

pub fn parse_source_to_blocks<'source>(
  source: &'source str,
) -> (Tree<Item<Token<'source>>>, Document<'source>) {
  let mut document = Document::new(source);
  let mut tree: Tree<Item<Token<'source>>> = Tree::new();
  tree.lower();
  while document.start() < document.bytes.len() {
    let bytes = document.bytes();
    let source = document.source();
    let (size, level) = continue_container(bytes, source, &mut tree);
    while level < tree.spine().len() {
      let cur = tree.cur().unwrap();
      let parent = tree.peek_parent().unwrap();
      tree[parent].item.end = tree[cur].item.end;
      tree.raise();
    }
    document.forward(size);
    scan_block(&mut tree, &mut document);
  }
  (tree, document)
}

// check tree[cur] is a paragraph
fn is_cur_paragraph<'source>(tree: &mut Tree<Item<Token<'source>>>) -> bool {
  if let Some(cur) = tree.cur() {
    if let Token::Paragraph = tree[cur].item.value {
      return true;
    }
  }
  false
}

fn scan_block<'source>(tree: &mut Tree<Item<Token<'source>>>, document: &mut Document<'source>) {
  let bytes = document.bytes();
  let source = document.source();
  let (_, spaces_size) = spaces(bytes);
  document.forward_offset(spaces_size);
  if scan_container_block(tree, document) {
    tree.lower();
    scan_block(tree, document);
  } else if !scan_leaf_block(tree, document) {
    let (mut line_size, _) = one_line(&bytes);
    let start = document.start();
    let offset = document.offset();
    if is_cur_paragraph(tree) {
      tree.lower_to_last();
    } else {
      tree.append(Item {
        start,
        end: start,
        value: Token::Paragraph,
      });
      tree.lower();
    }
    let end = offset + line_size;
    tree.append(Item {
      start: start,
      end,
      value: Token::Raw(&source[..line_size]),
    });
    tree.raise();
    let cur = tree.cur().unwrap();
    tree[cur].item.end = end;
    document.forward_to(end);
  }
}

fn scan_leaf_block<'source>(
  tree: &mut Tree<Item<Token<'source>>>,
  document: &mut Document<'source>,
) -> bool {
  let mut block: Option<Token> = None;
  let source = document.source();
  let bytes = document.bytes();
  let start = document.start();
  let offset = document.offset();
  let spaces = offset - start;
  let mut size = 0;

  if let Some(block_size) = blank_line(bytes) {
    size += block_size;
    block = Some(Token::BlankLine);
  } else if let Some(block_size) = thematic_break(bytes) {
    size += block_size;
    block = Some(Token::ThematicBreak);
  }
  if let Some(token) = block {
    let end = offset + size;
    tree.append(Item {
      start,
      end: offset + size,
      value: token,
    });
    document.forward_to(end);
    return true;
  }

  if let Some((start_size, level)) = atx_heading_start(bytes) {
    size += start_size;
    let (line_size, raw_size) = one_line(&bytes[size..]);
    let end = offset + size + line_size;
    tree.append(Item {
      start,
      end: offset + size + line_size,
      value: Token::ATXHeading(HeadingLevel::new(level).unwrap()),
    });
    tree.lower();
    let raw_start = offset + size;
    let raw_end = offset + size + raw_size;
    tree.append(Item {
      start: raw_start,
      end: raw_end,
      value: Token::Raw(&source[size..size + raw_size]),
    });
    tree.raise();
    document.forward_to(end);
    return true;
  }
  if let Some(size) = setext_heading(bytes) {
    if let Some(cur) = tree.cur() {
      if let Token::Paragraph = tree[cur].item.value {
        let level = if bytes[0] == b'=' { 1 } else { 2 };
        tree[cur].item.value = Token::SetextHeading(HeadingLevel::new(level).unwrap());
        let end = offset + size;
        tree[cur].item.end += end;
        document.forward_to(end);
        return true;
      }
    }
    return false;
  }
  if let Some((open_size, repeat, meta)) = open_fenced_code(bytes) {
    size += open_size;
    let token = Token::FencedCode;
    let open_ch = bytes[0];

    tree.append(Item {
      start,
      // set later
      end: start,
      value: Token::FencedCode,
    });
    loop {
      if size >= bytes.len() {
        break;
      }
      if let Some(container_size) = interrupt_container(&bytes[size..], &source[size..], tree) {
        size += container_size;
        if let Some((close_size, close_repeat)) = close_fenced_code(&bytes[size..]) {
          if bytes[size] == open_ch && repeat == close_repeat {
            size += close_size;
            break;
          }
        }
        let (line_size, _) = one_line(&bytes[size..]);
        tree.lower();
        let code_start = offset + size;
        size += line_size;
        let code_end = offset + size;
        tree.append(Item {
          start: code_start,
          end: code_end,
          value: Token::Code(&source[size - line_size..size]),
        });
        tree.raise();
      } else {
        break;
      }
    }
    let end = offset + size;
    let cur = tree.cur().unwrap();
    tree[cur].item.end = end;
    document.forward_to(end);
  }
  false
}

fn scan_container_block<'source>(
  tree: &mut Tree<Item<Token<'source>>>,
  document: &mut Document<'source>,
) -> bool {
  let start = document.start();
  let offset = document.offset();
  let bytes = document.bytes();
  let source = document.source();
  let spaces_size = offset - start;
  if spaces_size > 3 {
    return false;
  }

  if let Some((size, level)) = block_quote(bytes) {
    tree.append(Item {
      start,
      end: size + offset,
      value: Token::BlockQuote(level),
    });
    document.forward_to(size + offset);
    return true;
  }

  if let Some((size, marker_size)) = list_item_start(bytes) {
    let order_index = if marker_size > 1 {
      &source[..marker_size - 1]
    } else {
      ""
    };
    // TODO
    let (_, ending_indent) = spaces(&bytes[size..]);
    let ch = bytes[marker_size - 1];
    if let Some(cur) = tree.cur() {
      if let Token::List(list_ch, _, __) = tree[cur].item.value {
        if list_ch == ch {
          tree.lower();
          tree.append(Item {
            start,
            end: start + size + ending_indent,
            value: Token::ListItem(spaces_size + size + ending_indent),
          });
          document.forward_to(start + size + ending_indent);
          return true;
        }
      }
    }
    // TODO: is_tight
    tree.append(Item {
      start,
      end: start + size + ending_indent,
      value: Token::List(ch, false, order_index),
    });
    tree.lower();
    tree.append(Item {
      start,
      end: start + size + ending_indent,
      value: Token::ListItem(spaces_size + size + ending_indent),
    });
    document.forward_to(start + size + ending_indent);
    return true;
  }
  false
}

// #[test]
// fn test_parse_block() {
//   let source = r#"
// # ti`tle`
// this is a ~~paragraph~~
// > 123
// "#;
//   insta::assert_yaml_snapshot!(parse_source_to_blocks(source).0);
// }
