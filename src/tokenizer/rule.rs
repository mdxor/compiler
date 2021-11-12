pub const ATX_HEADING_RULE: &str = "^(#{1,6})(?:| (.*))(?:\n|$)";
pub const ATX_HEADING_CLOSING_RULE: &str = "(^| )#+ *$";
pub const THEMATIC_BREAK_RULE: &str = r"((?:- *){3,}|(?:_ *){3,}|(?:\* *){3,})(?:\n+|$)";
pub const SETEXT_HEADING_RULE: &str = "^(=+|-+) *(?:\n+|$)";
pub const INDENTED_CODE_RULE: &str = "^( {4}[^\n]+(?:\n(?: *(?:\n|$))*)?)+";
pub const FENCED_CODE_HEAD_RULE: &str = r"^(`{3,})([^`\n]*)\n|^(~{3,})([^\n]*)\n";
pub const FENCED_CODE_BODY_RULE: &str = r"^(?:|([\s\S]*?)\n) {0,3}`{3,} *(?:\n|$)|$";
