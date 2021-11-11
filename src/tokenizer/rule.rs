pub const ATX_HEADING_RULE: &str = "^(#{1,6})(?:| (.*))(?:\n|$)";
pub const ATX_HEADING_CLOSING_RULE: &str = "(^| )#+ *$";
pub const THEMATIC_BREAK_RULE: &str = r"((?:- *){3,}|(?:_ *){3,}|(?:\* *){3,})(?:\n+|$)";
