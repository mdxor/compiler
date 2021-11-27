// Reference: https://github.com/markedjs/marked/blob/master/src/rules.js
pub const ATX_HEADING_RULE: &str = "^(#{1,6})(?:| (.*))(?:\n|$)";
pub const ATX_HEADING_CLOSING_RULE: &str = "(^| )#+ *$";
pub const THEMATIC_BREAK_RULE: &str = r"((?:- *){3,}|(?:_ *){3,}|(?:\* *){3,})(?:\n+|$)";
pub const SETEXT_HEADING_RULE: &str = "^(=+|-+) *(?:\n+|$)";
pub const INDENTED_CODE_RULE: &str = "^( {4}[^\n]+(?:\n(?: *(?:\n|$))*)?)+";
pub const FENCED_CODE_HEAD_RULE: &str = r"^(`{3,})([^`\n]*)\n|^(~{3,})([^\n]*)\n";
pub const FENCED_CODE_BODY_RULE: &str = r"^(?:|([\s\S]*?)\n) {0,3}`{3,} *(?:\n|$)|$";

pub const LABEL_RULE: &str = r"(?:\\[\[\]]|[^\[\]])+";
pub const TITLE_RULE: &str = r#"(?:"(?:\\"?|[^"\\])*"|'[^'\n]*(?:\n[^'\n]+)*\n?'|\([^()]*\))"#;
pub const LINK_DEFINITION_RULE: &str =
  r"\[(label)\]: *\n? *<?([^\s>]+)>?(?:(?: +\n? *| *\n *)(title))? *(?:\n+|$)";

pub const BLOCK_QUOTE: &str = "^> ?";
pub const LIST_ITEM_RULE: &str = r"^([*+-]) |(\d{1,9})([.)]) ";
pub const TABLE_RULE: &str =
  r"^([^\n ].*\|.*)\n {0,3}((?:\| *)?:?-+:? *(?:\| *:?-+:? *)*(?:\| *)?)(?:\n+|$)";
pub const CODE_SPAN_RULE: &str = r"^(`+)([^`]|[^`][\s\S]*?[^`])(`+)";

// emStrong
pub const LDELIM_RULE: &str = r"^(?:\*+(?:([punct_])|[^\s*]))|^_+(?:([punct*])|([^\s_]))";
pub const RDELIMAST_RULE: &str = r"";
