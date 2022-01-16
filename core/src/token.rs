#[cfg(test)]
use serde::Serialize;

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub struct Token<T> {
  pub span: Span,
  pub value: T,
}

#[derive(Eq, PartialEq, Debug, Clone)]
#[cfg_attr(test, derive(Serialize))]
pub struct Span {
  pub start: usize,
  pub end: usize,
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub enum HeadingLevel {
  H1,
  H2,
  H3,
  H4,
  H5,
  H6,
}

impl HeadingLevel {
  pub fn new(level: usize) -> Option<HeadingLevel> {
    match level {
      1 => Some(HeadingLevel::H1),
      2 => Some(HeadingLevel::H2),
      3 => Some(HeadingLevel::H3),
      4 => Some(HeadingLevel::H4),
      5 => Some(HeadingLevel::H5),
      6 => Some(HeadingLevel::H6),
      _ => None,
    }
  }

  pub fn to_str(&self) -> &str {
    match self {
      HeadingLevel::H1 => "h1",
      HeadingLevel::H2 => "h2",
      HeadingLevel::H3 => "h3",
      HeadingLevel::H4 => "h4",
      HeadingLevel::H5 => "h5",
      HeadingLevel::H6 => "h6",
    }
  }
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub struct LinkDefinition<'source> {
  pub label: &'source str,
  pub url: &'source str,
  pub title: String,
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub enum Align {
  Left,
  Center,
  Right,
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub enum BlockToken {
  Paragraph {
    raws: Vec<Span>,
  },
  JSX(JSXElement),
  ATXHeading {
    raws: Vec<Span>,
    level: HeadingLevel,
  },
  SetextHeading {
    raws: Vec<Span>,
    level: HeadingLevel,
  },
  IndentedCode(Vec<Span>),
  BlankLine,
  ThematicBreak,
  BlockQuote {
    blocks: Vec<Token<BlockToken>>,
    level: usize,
  },
  FencedCode {
    meta_span: Span,
    code_spans: Vec<Span>,
  },
  List {
    ch: u8,
    is_tight: bool,
    order_span: Span,
    blocks: Vec<Token<BlockToken>>,
  },
  ListItem {
    indent: usize,
    blocks: Vec<Token<BlockToken>>,
  },
  // LinkDefinition,
  // Table,
  // TableHead,
  // TableCell(&'source str, bool, bool),
  // TableAlignment,
  // TableAlign(Align),
  // TableRow
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub enum InlineToken {
  TextSegment,
  MaybeLinkStart,
  MaybeEmphasis {
    ch: u8,
    repeat: usize,
    can_open: bool,
    can_close: bool,
  },
  EmphasisStart,
  EmphasisEnd,
  LinkStart {
    url: Span,
    title: Vec<Span>,
  },
  LinkEnd,
  //
  Emphasis(Vec<Token<InlineToken>>),
  Link {
    url: Span,
    title: Vec<Span>,
    text_children: Vec<Token<InlineToken>>,
  },
  Text(Vec<Span>),
  Code(Vec<Span>),
  CodeSegment,
  SoftBreak,
  HardBreak,
  // is email
  AutoLink(bool),
  JSX(JSXElement),
}

#[derive(Eq, PartialEq, Debug)]
pub struct AST<T> {
  pub span: Span,
  pub children: Vec<T>,
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub enum ContainerBlock {
  BlockQuote(usize),
  List(u8),
  ListItem(usize),
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub enum JSToken {
  Keyword(Span),
  Punctuator(Span),
  String(Span),
  Template(Span),
  Identifier(Span),
  Number(Span),
  Text(Span),
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub enum JSXAttr {
  Spread(Vec<JSXExpressionSegment>),
  KeyLiteralValue {
    key: Span,
    value: Span,
  },
  KeyValue {
    key: Span,
    value: Vec<JSXExpressionSegment>,
  },
  KeyTrueValue {
    key: Span,
  },
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub enum JSXExpressionSegment {
  JS(Span),
  Element(JSXElement),
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub enum JSX {
  Element(JSXElement),
  Text(Span),
  Expression(Vec<JSXExpressionSegment>),
}

#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(test, derive(Serialize))]
pub struct JSXElement {
  pub tag: String,
  pub attributes: Vec<JSXAttr>,
  pub children: Vec<JSX>,
}
