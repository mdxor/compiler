use crate::input::*;
use crate::lexer::*;
use crate::token::*;

pub(crate) fn continue_paragraph(ast: &mut AST, bytes: &[u8]) -> Option<usize> {
  if let Some(size) = interrupt_container(ast, bytes) {
    let (bytes, spaces) = spaces0(&bytes[size..]);
    if spaces >= 4 {
      return None;
    }
    if atx_heading_start(bytes).is_none()
      && eol(bytes).is_none()
      && bytes[0] != b'>'
      && thematic_break(bytes).is_none()
      && setext_heading(bytes).is_none()
      && open_fenced_code(bytes).is_none()
    {
      return Some(size);
    }
  }
  None
}

pub(crate) fn continue_container<'a>(
  ast: &'a mut AST,
  bytes: &[u8],
) -> (usize, &'a mut Vec<Token<BlockToken>>, usize) {
  let (mut size, mut blocks, mut nested) = inner_continue_container(ast, bytes);
  let list = blocks.last_mut().unwrap();
  if let Token {
    span: list_span,
    value: BlockToken::List {
      ch: list_ch,
      blocks: ref mut list_blocks,
      ..
    },
  } = list
  {
    let (bytes, start_indent) = spaces0(&bytes[size..]);
    if let Some((item_start_size, marker_size, end_indent)) = list_item_start(bytes) {
      let ch = &bytes[marker_size - 1];
      if list_ch == ch {
        let start = size + list_span.end;
        let end = start + start_indent + item_start_size;
        list_blocks.push(Token {
          value: BlockToken::ListItem {
            blocks: vec![],
            indent: start_indent + marker_size + end_indent,
          },
          span: Span { start, end },
        });
        let list_item = list_blocks.last_mut().unwrap();
        nested += 2;
        if let Token {
          value:
            BlockToken::ListItem {
              blocks: ref mut list_item_blocks,
              ..
            },
          ..
        } = list_item
        {
          blocks = list_item_blocks;
        }
      }
    }
  }
  (size, blocks, nested)
}

pub(crate) fn inner_continue_container<'a>(
  ast: &'a mut AST,
  bytes: &[u8],
) -> (usize, &'a mut Vec<Token<BlockToken>>, usize) {
  let mut nested = 0;
  if ast.blocks.is_empty() {
    return (0, &mut ast.blocks, nested);
  }
  let mut size = 0;
  let mut offset = 0;
  let (_, mut spaces) = spaces0(&bytes[offset..]);
  offset = spaces;
  let mut parent_blocks = &mut ast.blocks;
  loop {
    let last_block = parent_blocks.last_mut().unwrap();
    if let BlockToken::BlockQuote {
      level,
      ref mut blocks,
      ..
    } = last_block.value
    {
      if spaces < 4 {
        if let Some((quote_size, quote_level)) = block_quote(&bytes[size..]) {
          if level == quote_level {
            spaces = 0;
            offset += quote_size;
            size = offset;
            spaces = spaces0(&bytes[size..]).1;
            offset += spaces;
            parent_blocks = blocks;
            nested += 1;
            continue;
          }
        }
      }
    } else if let Token {
      value: BlockToken::List {
        blocks: list_blocks,
        ch: list_ch,
        ..
      },
      span: list_span,
    } = last_block
    {
      let list_item = list_blocks.last_mut().unwrap();
      if let BlockToken::ListItem {
        ref mut blocks,
        indent,
        ..
      } = list_item.value
      {
        if spaces >= indent {
          size = offset - spaces;
          parent_blocks = blocks;
          nested += 2;
          continue;
        }
      }
    }
    return (size, parent_blocks, nested);
  }
}

pub(crate) fn interrupt_container(ast: &mut AST, bytes: &[u8]) -> Option<usize> {
  let (size, blocks, _) = inner_continue_container(ast, bytes);
  let last_block = blocks.last();
  if let Some(last_block) = last_block {
    if let BlockToken::BlockQuote { .. } = &last_block.value {
      return None;
    } else if let BlockToken::List { .. } = &last_block.value {
      return None;
    }
  }
  Some(size)
}
