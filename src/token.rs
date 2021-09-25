use crate::jsx;
#[derive(Debug, PartialEq)]
pub enum Token<'a> {
  // block
  Heading1,
  Heading2,
  Heading3,
  Heading4,
  Heading5,
  Heading6,
  Newline,
  CodeBlock(&'a str),
  BulletList,
  OrderedList,
  TaskList,
  Hr,
  // inline
  Img(&'a str, &'a str),
  Link(&'a str, &'a str),
  Text(&'a str),
  CodeInline(&'a str),
  TableVerticalBar,
  // jsx
  JSX(jsx::JSXNode<'a>),
}
