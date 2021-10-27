use crate::lexer;
use crate::token;
fn tokenizes(cases: Vec<&str>) -> Vec<Result<token::AST, &str>> {
  let mut results = vec![];
  for case in &cases {
    let mut lex = lexer::Lexer::new(case);
    let ast = lex.tokenize();
    results.push(ast)
  }
  results
}

#[test]
fn test_indented_code() {
  let cases = vec!["    abc", "    <div></div>"];
  insta::assert_yaml_snapshot!(tokenizes(cases));
}
#[test]
fn test_fenced_code() {
  let cases = vec![
    "```\ncode\n```",
    "```jsx\nlet a = 11;\n```",
    "```jsx meta\nlet a = 11;\n```",
  ];
  insta::assert_yaml_snapshot!(tokenizes(cases));
}
#[test]
fn test_atx_heading() {
  let cases = vec![
    // "# foo",
    // "## foo",
    // "### foo",
    // "#### foo",
    // "##### foo",
    // "###### foo",
    // "####### foo",
    // "#5 bolt",
    // "#hashtag",
    // r"\## foo",
    // // TODO:
    // // r"# foo *bar* \*baz\*"
    // "#                  foo                     ",
    // "### foo",
    // " ## foo",
    // "  # foo",
    // "    # foo",
    // // "foo\n    # bar",
    // "## foo ##\n###   bar    ###",
    // "# foo ##################################\n##### foo ##",
    // "### foo ###     ",
    // "### foo ### b",
    // "# foo#",
    // "****\n## foo\n****"
    "Foo bar\n# baz\nBar foo",
    // "## \n#\n### ###",
  ];
  insta::assert_yaml_snapshot!(tokenizes(cases));
}
#[test]
fn test_thematic_break() {
  let cases = vec![
    "***\n---\n___",
    "+++",
    "===",
    "--\n**\n__",
    "***\n ***\n   ***",
    "    ***",
    // "Foo\n    ***",
    "_____________________________________",
    "- - -",
    " **  * ** * ** * **",
    "-     -      -      -",
    "- - - -    ",
    "_ _ _ _ a\na------\n---a---",
    // " *-*",
    // "- foo\n***\n- bar"
    "Foo\n***\nbar",
    // "Foo\n---\nbar",
    // "* Foo\n* * *\n* Bar",
    // "- Foo\n- * * *",
  ];
  insta::assert_yaml_snapshot!(tokenizes(cases));
}
