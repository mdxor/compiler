use crate::document::*;
use crate::input::*;
use crate::interrupt::*;
use crate::lexer::*;
use crate::token::*;

pub fn parse_source_to_blocks(source: &str) -> (AST, Document) {
  let mut document = Document::new(source);
  let mut ast = AST {
    blocks: vec![],
    span: Span {
      start: 0,
      end: source.len(),
    },
  };
  while document.start() < document.bytes.len() {
    let bytes = document.bytes();
    let (size, mut blocks, mut spine) = continue_container(&mut ast, bytes);
    blocks.push(Token {
      value: BlockToken::ThematicBreak,
      span: Span { start: 0, end: 0 },
    })
    // document.forward(size);
    // scan_block(&mut document, &mut blocks, &mut spine);
  }
  (ast, document)
}

fn scan_block(
  document: &mut Document,
  // ast: &AST,
  blocks: &mut Vec<Token<BlockToken>>,
  spine: &mut Vec<&Token<BlockToken>>,
) {
  let bytes = document.bytes();
  let source = document.source();
  let (_, spaces_size) = spaces0(bytes);
  document.forward_offset(spaces_size);
  if let Some(container_block) = scan_container_block(document) {
    blocks.push(container_block);
    let container_block = blocks.last_mut().unwrap();
    if let Token {
      value: BlockToken::BlockQuote {
        blocks: ref mut quote_blocks,
        ..
      },
      ..
    } = container_block
    {
      // spine.push(container_block);
      scan_block(document, quote_blocks, spine);
    } else if let Token {
      value: BlockToken::List {
        blocks: list_blocks,
        ..
      },
      ..
    } = container_block
    {
      // spine.push(&container_block);
      let list_item = list_blocks.last_mut().unwrap();
      if let Token {
        value:
          BlockToken::ListItem {
            blocks: ref mut list_item_blocks,
            ..
          },
        ..
      } = list_item
      {
        // spine.push(list_item);
        scan_block(document, list_item_blocks, spine);
      }
    }
  } else if let Some(leaf_block) = scan_leaf_block(document, blocks) {
    blocks.push(leaf_block);
    // TODO
  } else {
    let (mut line_size, _) = one_line(&bytes);
    let start = document.start();
    let offset = document.offset();
    let end = offset + line_size;
    let mut paragraph = blocks.last_mut().unwrap();
    if let Token {
      value: BlockToken::Paragraph { ref mut raws },
      ref mut span,
    } = paragraph
    {
      raws.push(Span { start, end });
      span.end = end;
    } else {
      blocks.push(Token {
        value: BlockToken::Paragraph {
          raws: vec![Span { start, end }],
        },
        span: Span { start, end },
      });
    }
    document.forward_to(end);
  }
}

fn scan_leaf_block<'source>(
  document: &mut Document<'source>,
  // ast: &AST,
  blocks: &mut Vec<Token<BlockToken>>,
) -> Option<Token<BlockToken>> {
  let bytes = document.bytes();
  let start = document.start();
  let offset = document.offset();
  let spaces = offset - start;

  if let Some(size) = blank_line(bytes) {
    let end = offset + size;
    document.forward_to(end);
    return Some(Token {
      value: BlockToken::BlankLine,
      span: Span { start, end },
    });
  } else if let Some(size) = thematic_break(bytes) {
    let end = offset + size;
    document.forward_to(end);
    return Some(Token {
      value: BlockToken::ThematicBreak,
      span: Span { start, end },
    });
  }

  if let Some((start_size, level)) = atx_heading_start(bytes) {
    let (line_size, raw_size) = one_line(bytes);
    let end = offset + start_size + line_size;
    document.forward_to(end);
    return Some(Token {
      value: BlockToken::ATXHeading {
        level: HeadingLevel::new(level).unwrap(),
        raws: vec![Span {
          start: offset + start_size,
          end: offset + start_size + raw_size,
        }],
      },
      span: Span { start, end },
    });
  }
  if let Some(size) = setext_heading(bytes) {
    if let Some(Token {
      value: BlockToken::Paragraph { .. },
      ..
    }) = blocks.last()
    {
      if let Token {
        value: BlockToken::Paragraph { raws },
        span,
      } = blocks.pop().unwrap()
      {
        let level = if bytes[0] == b'=' { 1 } else { 2 };
        let end = offset + size;
        document.forward_to(end);
        return Some(Token {
          value: BlockToken::SetextHeading {
            level: HeadingLevel::new(level).unwrap(),
            raws,
          },
          span: Span {
            start: span.start,
            end,
          },
        });
      }
    }
    return None;
  }
  if let Some((open_size, repeat, meta)) = open_fenced_code(bytes) {
    let mut size = open_size;
    let open_ch = bytes[0];

    let meta_span = Span {
      start: offset + repeat,
      end: offset + repeat + meta,
    };
    let mut codes: Vec<Span> = vec![];
    loop {
      if size >= bytes.len() {
        break;
      }
      // if let Some(container_size) = interrupt_container(ast, &bytes[size..]) {
      //   size += container_size;
      //   if let Some((close_size, close_repeat)) = close_fenced_code(&bytes[size..]) {
      //     if bytes[size] == open_ch && repeat == close_repeat {
      //       size += close_size;
      //       break;
      //     }
      //   }
      //   let (line_size, _) = one_line(&bytes[size..]);
      //   let code_start = offset + size;
      //   size += line_size;
      //   let code_end = offset + size;
      //   codes.push(Span {
      //     start: code_start,
      //     end: code_end,
      //   });
      // } else {
      //   break;
      // }
    }
    let end = offset + size;
    return Some(Token {
      value: BlockToken::FencedCode { meta_span, codes },
      span: Span { start, end },
    });
  }
  None
}

fn scan_container_block<'source>(document: &mut Document<'source>) -> Option<Token<BlockToken>> {
  let start = document.start();
  let offset = document.offset();
  let bytes = document.bytes();
  let source = document.source();
  let spaces = offset - start;
  if spaces >= 4 {
    return None;
  }

  if let Some((size, level)) = block_quote(bytes) {
    document.forward_to(size + offset);
    return Some(Token {
      value: BlockToken::BlockQuote {
        blocks: vec![],
        level,
      },
      span: Span {
        start,
        end: offset + size,
      },
    });
  }

  if let Some((size, marker_size, end_indent)) = list_item_start(bytes) {
    let order_span = if marker_size > 1 {
      Span {
        start: offset,
        end: offset,
      }
    } else {
      Span {
        start: offset,
        end: offset + marker_size - 1,
      }
    };
    let ch = bytes[marker_size - 1];
    let end = start + size + end_indent;
    document.forward_to(end);
    return Some(Token {
      value: BlockToken::List {
        ch,
        is_tight: false,
        order_span,
        blocks: vec![Token {
          value: BlockToken::ListItem {
            indent: spaces + marker_size + end_indent,
            blocks: vec![],
          },
          span: Span { start, end },
        }],
      },
      span: Span { start, end },
    });
  }
  None
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
