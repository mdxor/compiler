// use crate::jsx;
// fn parse(source: &str) -> Result<jsx::JSXNode, &str> {
//   let mut jsx_parser = jsx::JSXParser::new(source, 0, true);
//   jsx_parser.parse()
// }
// #[test]
// fn test_jsx_parse() {
//   let cases = vec![
//     "<div></div>",
//     r#"<div test="true">中文测试<div>en</div></div>"#,
//     r#"<div test="true" content={() => <span>content</span>}>中文测试<div>en</div></div>"#,
//     "<React.Fragment></React.Fragment>",
//     "<></>",
//     "<SelfClosed />",
//   ];
//   let mut results = vec![];
//   for case in &cases {
//     let result = parse(case);
//     results.push(result)
//   }
//   insta::assert_yaml_snapshot!(results);
// }
